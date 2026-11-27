    pub(super) fn native_client_auto_queue_message(&mut self, kind: u8, message: &str) {
        if self.client_auto_tool_player() {
            mod_api::dispatch_client_auto_queue_message(self, mod_api::mods(), kind, message);
        }
    }
    pub(super) fn native_client_auto_queue_wake(&mut self) {
        if self.client_auto_tool_player() {
            mod_api::dispatch_client_auto_queue_wake(self, mod_api::mods());
        }
    }
    pub(super) fn native_client_auto_queue_schedule(&self, deadline: Option<i64>) {
        self.client_auto_queue_wake_at.set(deadline);
    }
    pub(super) fn client_auto_queue_poll_wake(&mut self) {
        if self
            .client_auto_queue_wake_at
            .get()
            .is_some_and(|at| at <= self.native_client_auto_tool_time())
        {
            self.client_auto_queue_wake_at.set(None);
            self.native_client_auto_queue_wake();
        }
    }
    pub(super) fn client_auto_queue_control_flow(
        &self,
        flow: winit::event_loop::ControlFlow,
    ) -> winit::event_loop::ControlFlow {
        use winit::event_loop::ControlFlow;
        let Some(at) = self.client_auto_queue_wake_at.get() else {
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
    pub(super) fn native_client_auto_queue_snapshot(&self) -> Option<serde_json::Value> {
        use horizon_core::geometry::{Aabb, Axis};
        use serde_json::json;
        if !self.client_auto_tool_player() {
            return None;
        }
        let eye = self.eye_position()?;
        let near = |b: Aabb| -> bool {
            let axes = [Axis::X, Axis::Y, Axis::Z];
            (0..3)
                .map(|i| {
                    let d = eye[i] - eye[i].clamp(b.min(axes[i]), b.max(axes[i]));
                    d * d
                })
                .sum::<f64>()
                < 25.0
        };
        let stand = self.armor_stands.iter().find(|a| {
            if !near(a.bounds()) {
                return false;
            }
            let name = a.custom_name.clone().unwrap_or_else(|| {
                horizon_client::text::Text::tr("entity.minecraft.armor_stand", &[])
            });
            let name = self
                .mob_team(a.id)
                .map_or_else(|| name.clone(), |team| team.formatted_name(name.clone()));
            name.plain().contains("Duels")
        });
        let npc = stand.and_then(|a| {
            let target = [a.position.x, a.position.y - 2.0, a.position.z];
            // ReportHelper retains the real client world player insertion order.
            let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
            let players = self.client_report_helper_world_players();
            std::iter::once(local)
                .chain(
                    players
                        .iter()
                        .filter_map(|v| v["id"].as_i64().and_then(|n| i32::try_from(n).ok())),
                )
                .find(|&id| {
                    if id == local {
                        return self.player.as_ref().is_some_and(|p| {
                            [p.position.x, p.position.y, p.position.z] == target && near(p.bounds())
                        });
                    }
                    self.mobs
                        .iter()
                        .find(|m| m.id as i32 == id)
                        .is_some_and(|m| {
                            [m.position.x, m.position.y, m.position.z] == target && near(m.bounds())
                        })
                })
                .or_else(|| {
                    self.lan
                        .iter()
                        .flat_map(|h| h.hub.guests.iter())
                        .find(|g| {
                            g.placed
                                && g.dimension == self.dim.id()
                                && g.position == target
                                && near(g.bounds())
                        })
                        .map(|g| g.entity_id)
                })
        });
        let slots=std::iter::once((-1,self.offhand.as_ref())).chain((0..9).map(|i|(i as i32,self.inventory.get(i).and_then(Option::as_ref)))).map(|(slot,stack)|{
            let stack=stack.filter(|s|s.count>0&&&*s.item!="minecraft:air");
            json!({"slot":slot,"item":stack.map_or("minecraft:air",|s|&*s.item),"name":stack.map_or_else(||self.pack.names.display_item("minecraft:air"),|s|self.stack_hover_name(s))})
        }).collect::<Vec<_>>();
        let mut pause = 0u32;
        if self.screen == Screen::Chat {
            pause |= 2;
        }
        if matches!(self.screen, Screen::Inventory | Screen::Creative) {
            pause |= 4;
        }
        if self.screen == Screen::Paused {
            pause |= 8;
        }
        Some(
            json!({"now":self.native_client_auto_tool_time(),"pause":pause,"selected":self.selected,"header":self.tab_list.header.as_ref().map(|t|t.plain()),"footer":self.tab_list.footer.as_ref().map(|t|t.plain()),"slots":slots,"npc":npc}),
        )
    }
    pub(super) fn native_client_auto_queue_send(&mut self, message: &str, kind: u8) {
        if !self.client_auto_tool_player() {
            return;
        }
        match kind {
            0 => self.send_command(message),
            1 => self.send_chat(message),
            _ => {
                if let Some(command) = message.strip_prefix('/') {
                    self.send_command(command);
                } else {
                    self.send_chat(message);
                }
            }
        }
    }
    pub(super) fn native_client_auto_queue_raw_use(&mut self) {
        if !self.client_auto_tool_player() {
            return;
        }
        // Gomme explicitly starts prediction and sends the packet even for an
        // empty hand/spectator; it does not call Item.use or apply its result.
        let sequence = if self.remote.is_some() {
            self.next_sequence()
        } else {
            0
        };
        self.native_send_with_anti_hunger(horizon_protocol::packets::play::Serverbound::UseItem(
            horizon_protocol::packets::play::UseItem {
                hand: 0,
                sequence,
                yaw: self.yaw,
                pitch: self.pitch,
            },
        ));
    }
    pub(super) fn native_client_auto_queue_use(&mut self, hand: i32) {
        if !self.client_auto_tool_player() || self.spectator() {
            return;
        }
        let hand = match hand {
            0 => Hand::Main,
            1 => Hand::Off,
            _ => return,
        };
        if self.remote.is_some() && self.with_hand(hand, |g| g.held().is_none()) {
            let sequence = self.next_sequence();
            self.send(horizon_protocol::packets::play::Serverbound::UseItem(
                horizon_protocol::packets::play::UseItem {
                    hand: if matches!(hand, Hand::Off) { 1 } else { 0 },
                    sequence,
                    yaw: self.yaw,
                    pitch: self.pitch,
                },
            ));
        } else if self.remote.is_some() {
            self.remote_use_item(hand);
        } else {
            self.with_hand(hand, |g| g.use_hand(None).unwrap_or(false));
        }
        // This caller invokes MultiPlayerGameMode.useItem directly. Its return
        // value is ignored: Minecraft.startUseItem's equip-height reset and
        // swing do not belong to this path.
    }
    pub(super) fn native_client_auto_queue_interact(&mut self, id: i32) {
        if !self.client_auto_tool_player() {
            return;
        }
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
        let position = if id == local {
            self.player
                .as_ref()
                .map(|p| [p.position.x, p.position.y, p.position.z])
        } else {
            self.mobs
                .iter()
                .find(|m| m.id as i32 == id)
                .map(|m| [m.position.x, m.position.y, m.position.z])
                .or_else(|| {
                    self.lan
                        .iter()
                        .flat_map(|h| h.hub.guests.iter())
                        .find(|g| g.entity_id == id && g.placed && g.dimension == self.dim.id())
                        .map(|g| g.position)
                })
        };
        let Some(position) = position else {
            return;
        };
        let [min_x, min_z, max_x, max_z] = self.border.bounds();
        let x = position[0].floor();
        let z = position[2].floor();
        if x + 1.0 <= min_x || x >= max_x || z + 1.0 <= min_z || z >= max_z {
            return;
        }
        // EntityHitResult(entity) is its position, so relative location is zero.
        // Client Player.interactOn returns PASS; no fallback item use or swing.
        self.send(horizon_protocol::packets::play::Serverbound::Interact(
            horizon_protocol::packets::play::Interact {
                id,
                hand: 0,
                location: [0.0; 3],
                secondary: self.remote_shift_down(),
            },
        ));
    }
