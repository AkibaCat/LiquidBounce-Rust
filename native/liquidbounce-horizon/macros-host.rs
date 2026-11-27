    pub(super) fn native_client_macros_snapshot(&self) -> Option<serde_json::Value> {
        if !self.client_auto_tool_player() {
            return None;
        }
        let slots=std::iter::once((-1,self.offhand.as_ref())).chain((0..9).map(|i|(i as i32,self.inventory.get(i).and_then(Option::as_ref)))).map(|(slot,stack)|{
            let stack=stack.filter(|s|s.count>0&&&*s.item!="minecraft:air");
            serde_json::json!({"slot":slot,"item":stack.map_or("minecraft:air",|s|&*s.item),"name":stack.map_or_else(||self.pack.names.display_item("minecraft:air"),|s|self.stack_hover_name(s))})
        }).collect::<Vec<_>>();
        Some(
            serde_json::json!({"now":self.native_client_auto_tool_time(),"screen":self.screen!=Screen::Playing,"slots":slots}),
        )
    }
    pub(super) fn native_client_macros_send(&mut self, message: &str) {
        if !self.client_auto_tool_player() {
            return;
        }
        if let Some(command) = message.strip_prefix('/') {
            self.send_command(command);
        } else {
            self.send_chat(message);
        }
    }
    pub(super) fn native_client_commands_say(&mut self, message: &str) {
        if self.client_auto_tool_player() {
            self.send_chat(message);
        }
    }
    pub(super) fn native_client_macros_schedule(&self, deadline: Option<i64>) {
        self.client_macros_wake_at.set(deadline);
    }
    pub(super) fn native_client_macros_wake(&mut self) {
        if self.client_auto_tool_player() {
            mod_api::dispatch_client_macros_wake(self, mod_api::mods());
        }
    }
    pub(super) fn client_macros_poll_wake(&mut self) {
        if self
            .client_macros_wake_at
            .get()
            .is_some_and(|at| at <= self.native_client_auto_tool_time())
        {
            self.client_macros_wake_at.set(None);
            self.native_client_macros_wake();
        }
    }
    pub(super) fn client_macros_control_flow(
        &self,
        flow: winit::event_loop::ControlFlow,
    ) -> winit::event_loop::ControlFlow {
        use winit::event_loop::ControlFlow;
        let Some(at) = self.client_macros_wake_at.get() else {
            return flow;
        };
        let until = std::time::Instant::now()
            + std::time::Duration::from_millis(
                at.saturating_sub(self.native_client_auto_tool_time())
                    .max(0) as u64,
            );
        match flow {
            ControlFlow::Poll => flow,
            ControlFlow::Wait => ControlFlow::WaitUntil(until),
            ControlFlow::WaitUntil(old) => ControlFlow::WaitUntil(old.min(until)),
        }
    }
    pub(super) fn native_client_commands_chat(&mut self, message: &str, handled: &mut bool) {
        mod_api::dispatch_client_commands_chat(self, mod_api::mods(), message, handled);
    }
    /// The scope only changes the utility's locally predicted interaction
    /// swing. No Punch packet is emitted, including HideForClient.
    pub(super) fn native_client_macros_item(
        &mut self,
        hand: i32,
        place: bool,
        hide: bool,
        yaw: f32,
        pitch: f32,
    ) {
        if !self.client_auto_tool_player() {
            return;
        }
        let hand_enum = match hand {
            0 => Hand::Main,
            1 => Hand::Off,
            _ => return,
        };
        let previous = self.client_macros_swing_scope;
        let previous_swung = self.client_macros_swung;
        self.client_macros_swing_scope = Some(hide);
        self.client_macros_swung = false;
        if place {
            // mc.hitResult is shared with normal entity/block picking. A miss
            // is also a BlockHitResult, so retain its endpoint for useItemOn.
            if self.pick_entity_hit().is_none() {
                let hit = self.pick().or_else(|| {
                    let (eye, dir) = self.client_block_aim()?;
                    let point = std::array::from_fn(|i| eye[i] + dir[i] * self.block_reach);
                    let mut face = [0, -1, 0];
                    let mut dot = f64::NEG_INFINITY;
                    for candidate in [
                        [0, -1, 0],
                        [0, 1, 0],
                        [0, 0, -1],
                        [0, 0, 1],
                        [-1, 0, 0],
                        [1, 0, 0],
                    ] {
                        let next = (0..3)
                            .map(|i| f64::from(candidate[i]) * dir[i])
                            .sum::<f64>();
                        if next > dot {
                            dot = next;
                            face = candidate;
                        }
                    }
                    Some(Hit {
                        pos: point.map(|x| x.floor() as i32),
                        face,
                        point,
                    })
                });
                if let Some(hit) = hit {
                    let [min_x, min_z, max_x, max_z] = self.border.bounds();
                    if f64::from(hit.pos[0]) + 1.0 > min_x
                        && f64::from(hit.pos[0]) < max_x
                        && f64::from(hit.pos[2]) + 1.0 > min_z
                        && f64::from(hit.pos[2]) < max_z
                    {
                        let before = self
                            .with_hand(hand_enum, |g| g.held().map(|s| (s.item.clone(), s.count)));
                        let consumed = if self.remote.is_some() {
                            let old = self.forced_hit;
                            self.forced_hit = Some(hit);
                            let used = self.remote_use_item_on(hand_enum);
                            self.forced_hit = old;
                            used
                        } else if self.spectator() {
                            self.spectator_open_menu(hit.pos);
                            true
                        } else {
                            self.with_hand(hand_enum, |g| {
                                let old = g.guest_use;
                                g.guest_use = Some(super::GuestUse::OnBlock(hand_enum));
                                let used = g.use_hand(Some(&hit)).unwrap_or(false) || g.use_failed;
                                g.guest_use = old;
                                used
                            })
                        };
                        if !consumed {
                            if self.with_hand(hand_enum, |g| g.held().is_some()) {
                                self.native_client_use_item(hand, yaw, pitch);
                            }
                        } else if self.client_macros_swung && before.is_some() {
                            let after = self.with_hand(hand_enum, |g| {
                                g.held().map(|s| (s.item.clone(), s.count))
                            });
                            if self.creative || before != after {
                                self.heights.item_used(hand_enum);
                            }
                        }
                    }
                }
            }
        } else if self.with_hand(hand_enum, |g| g.held().is_none()) {
            // useItem still predicts and sends an empty stack's PASS packet.
            self.native_with_item_rotation(yaw, pitch, |g| g.native_client_auto_queue_use(hand));
        } else {
            self.native_client_use_item(hand, yaw, pitch);
        }
        self.client_macros_swing_scope = previous;
        self.client_macros_swung = previous_swung;
    }
