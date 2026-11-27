// SPDX-License-Identifier: GPL-3.0-or-later
// Appended inside auto_walk_runtime, retaining direct access to its real adapters.
struct FreeCamNavigatorRuntime {
    runtime: Runtime,
    enabled: bool,
}
impl FreeCamNavigatorRuntime {
    fn new(epoch: u64) -> Self {
        Self {
            runtime: Runtime::new(epoch),
            enabled: false,
        }
    }
    fn tick(&mut self, value: &Value, input: Input) -> Value {
        let runtime = &mut self.runtime;
        runtime.configure(value);
        for event in FREE_CAM_EVENTS.with(|s| std::mem::take(&mut *s.borrow_mut())) {
            match event {
                Event::Changed(epoch, p) if epoch == runtime.epoch => {
                    runtime.navigator.block_changed::<NativeHost>(p)
                }
                Event::Confirmed(epoch, p, id) if epoch == runtime.epoch => {
                    let state = runtime.level.live.registry.state(BlockStateId(id));
                    runtime
                        .navigator
                        .block_confirmed(&mut runtime.host, p, &state);
                }
                Event::Corrected(epoch) if epoch == runtime.epoch => {
                    runtime.navigator.position_corrected(&runtime.host)
                }
                _ => {}
            }
        }
        let running = boolean(value, "running", false);
        let enabled = boolean(value, "enabled", false);
        if self.enabled && !enabled {
            runtime.navigator.stop(&mut runtime.host);
            runtime.host.release_all();
        }
        self.enabled = enabled;
        let mut pressed = input;
        let mut plain = false;
        if enabled && running {
            let goal = value["goal"]
                .as_array()
                .filter(|a| a.len() == 3)
                .and_then(|a| Some([a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?]));
            if let Some(goal) = goal {
                plain = runtime.navigator.navigate(
                    &mut runtime.host,
                    &mut runtime.level,
                    NavVec::new(goal[0], goal[1], goal[2]),
                    1.5,
                );
                pressed = runtime.host.input.unwrap_or(input);
            } else {
                runtime.navigator.halt(&mut runtime.host);
                runtime.host.release_all();
            }
        } else {
            runtime.navigator.halt(&mut runtime.host);
            runtime.host.release_all();
        }
        let wants_sprint = runtime
            .host
            .input
            .is_some_and(|p| p.sprint && p.is_moving());
        pressed.sprint = input.sprint;
        let exact_keys = runtime
            .host
            .input
            .filter(|_| runtime.host.exact)
            .map(|mut keys_| {
                keys_.sprint = input.sprint;
                keys(keys_)
            });
        let request = runtime
            .host
            .requested
            .map(|(r, p)| json!({"rotation":[r.y_rot,r.x_rot],"purpose":format!("{p:?}")}));
        let movement = runtime.navigator.movement_rotation(&runtime.host);
        let route = runtime.host.route.borrow();
        let (head, ahead) = match route.progress {
            Some(nav::course::ProgressOwner::Walking) => (
                runtime.navigator.walking.follower.head(&runtime.host, 1.),
                runtime.navigator.walking.follower.ahead().to_vec(),
            ),
            Some(nav::course::ProgressOwner::Straight) => (
                runtime.host.player.position(),
                runtime.navigator.course.straight_goal.into_iter().collect(),
            ),
            Some(owner) => runtime
                .navigator
                .course
                .travel
                .progress(owner, &runtime.host, 1.),
            None => (runtime.host.player.position(), Vec::new()),
        };
        let previous_head = match route.progress {
            Some(nav::course::ProgressOwner::Walking) => {
                runtime.navigator.walking.follower.head(&runtime.host, 0.)
            }
            Some(nav::course::ProgressOwner::Straight) => runtime.host.player.interpolated(0.),
            Some(owner) => {
                runtime
                    .navigator
                    .course
                    .travel
                    .progress(owner, &runtime.host, 0.)
                    .0
            }
            None => runtime.host.player.interpolated(0.),
        };
        json!({"input":keys(pressed),"exactInput":exact_keys,"exact":runtime.host.exact,"enabled":enabled,"running":running,"plainWalk":plain,"acting":runtime.navigator.is_acting(),"movementRotation":[movement.y_rot,movement.x_rot],"request":request,"wantsSprint":wants_sprint,"usingItem":world::read(|g|g.using.is_some()),"route":{"points":route.points.iter().copied().map(point).collect::<Vec<_>>(),"head":point(head),"previousHead":point(previous_head),"ahead":ahead.into_iter().map(point).collect::<Vec<_>>(),"goal":route.destination.or(runtime.navigator.course.last_goal).map(point),"start":route.start.map(point),"breaks":route.breaks.iter().copied().map(pos).collect::<Vec<_>>(),"places":route.places.iter().copied().map(pos).collect::<Vec<_>>(),"preview":route.preview,"state":route.state,"since":route.since}})
    }
}
impl Game<'_> {
    pub(super) fn free_cam_navigation_config(&mut self, text: &str) {
        let Ok(value) = serde_json::from_str::<Value>(text) else {
            return;
        };
        let enabled = boolean(&value, "enabled", false);
        FREE_CAM_CONFIG.with(|s| *s.borrow_mut() = value);
        if enabled {
            return;
        }
        FREE_CAM_RESULT.with(|s| s.borrow_mut().take());
        FREE_CAM_EVENTS.with(|s| s.borrow_mut().clear());
        // onDisabled resets immediately when called at the canonical live host.
        // A callback made while navigation owns Game is left for the next input
        // instead of recursively borrowing Game or touching a retired player.
        if !self.native_window()
            || self.lan_acting.is_some()
            || self.lan_acting_uuid.is_some()
            || world::scope_is_active()
            || FREE_CAM_RETIRED.get()
            || !self
                .player
                .as_ref()
                .is_some_and(|p| p.native_client_queries_allowed())
        {
            return;
        }
        let epoch = world::current_epoch(self);
        let _owner = OwnerScope::enter(Owner::FreeCam);
        world::with_scope(self, || {
            FREE_CAM_RUNTIME.with(|slot| {
                let mut slot = slot.borrow_mut();
                if let Some(runtime) = slot
                    .as_mut()
                    .filter(|runtime| runtime.runtime.epoch == epoch && runtime.enabled)
                {
                    runtime.runtime.navigator.stop(&mut runtime.runtime.host);
                    runtime.runtime.host.release_all();
                    runtime.enabled = false;
                }
            });
        });
    }
    pub(super) fn free_cam_navigation_result(&self) -> Option<String> {
        FREE_CAM_RESULT.with(|s| s.borrow().clone())
    }
    pub(super) fn native_client_free_cam_navigation_input(
        &mut self,
        raw: [bool; 7],
    ) -> Option<([bool; 7], Option<[bool; 7]>)> {
        if !self.native_window()
            || self.lan_acting.is_some()
            || self.lan_acting_uuid.is_some()
            || !self
                .player
                .as_ref()
                .is_some_and(|p| p.native_client_queries_allowed())
        {
            return None;
        }
        let config = FREE_CAM_CONFIG.with(|s| s.borrow().clone());
        if config.is_null() {
            return None;
        }
        let epoch = world::current_epoch(self);
        let input = key_input(&json!(raw));
        let retired = FREE_CAM_RETIRED.replace(false);
        let _owner = OwnerScope::enter(Owner::FreeCam);
        let result = world::with_scope(self, || {
            FREE_CAM_RUNTIME.with(|slot| {
                let mut slot = slot.borrow_mut();
                if retired || slot.as_ref().is_none_or(|r| r.runtime.epoch != epoch) {
                    *slot = Some(FreeCamNavigatorRuntime::new(epoch));
                }
                slot.as_mut()
                    .expect("FreeCam navigator")
                    .tick(&config, input)
            })
        });
        let input = keys(key_input(&result["input"]));
        let exact = result["exactInput"]
            .as_array()
            .map(|_| keys(key_input(&result["exactInput"])));
        FREE_CAM_RESULT.with(|s| *s.borrow_mut() = Some(result.to_string()));
        Some((input, exact))
    }
}
