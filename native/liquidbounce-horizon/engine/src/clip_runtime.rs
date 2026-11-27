// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    coroutine_ticker::WaitOwner,
    runtime::{LiquidBounceHooks, raw_options},
};
use horizon_client::mod_api::{Host, HudBatch};
use liquidbounce_behaviors::clip::{Clip, Input, movement_direction};
use serde_json::Value;
impl LiquidBounceHooks {
    pub(crate) fn clip_old(&self, host: &mut dyn Host, s: Clip) {
        if let Some(c) = host
            .client_clip_context()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        {
            let yaw = c["yaw"].as_f64().unwrap_or(0.) as f32;
            let delta = s.old_offset(yaw);
            let position =
                std::array::from_fn(|i| c["position"][i].as_f64().unwrap_or(0.) + delta[i]);
            host.client_clip_snap(position, s.reset_velocity);
        }
        let effects = {
            let mut state = self.state();
            let id = state.behaviors.borrow().query_ids["Clip"];
            if let Err(e) =
                state.with_registry(raw_options(host), |r, c| r.set_enabled_sync(id, false, c))
            {
                eprintln!("[LiquidBounce] Clip: {e:?}");
            }
            state.take_effects()
        };
        crate::runtime::apply_effects(self, host, effects);
    }
    pub(crate) fn clip_tick(&self, host: &mut dyn Host) {
        let s = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            if !b.query_running(&state.registry, "Clip") {
                b.clip.cancel();
                return;
            }
            if b.clip.mode != 0 || b.clip.busy() {
                return;
            }
            b.clip.possible = [false; 2];
            if b.query_running(&state.registry, "Fly") {
                return;
            }
            b.clip
        };
        let Some(c) = host
            .client_clip_context()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return;
        };
        let mut targets = [None; 2];
        for direction in 0..2 {
            targets[direction] = host.client_clip_target(
                direction as i32,
                s.vertical,
                s.liquids,
                s.requires_stand_on,
            );
        }
        self.state().behaviors.borrow_mut().clip.possible = targets.map(|t| t.is_some());
        let input = Input {
            yaw: c["yaw"].as_f64().unwrap_or(0.) as f32,
            horizontal_collision: c["collision"].as_bool().unwrap_or(false),
            keys: std::array::from_fn(|i| c["keys"][i].as_bool().unwrap_or(false)),
        };
        let Some(direction) = movement_direction(input) else {
            return;
        };
        let target = if direction < 2 {
            targets[direction]
        } else {
            host.client_clip_target(
                direction as i32,
                s.horizontal,
                s.liquids,
                s.requires_stand_on,
            )
        };
        if let Some(position) = target {
            host.client_clip_snap(position, false);
            #[cfg(feature = "webview")]
            {
                let text = liquidbounce_interop::localization::describe(
                    &horizon_client::lang::code(),
                    "liquidbounce.module.clip.messages.whoosh",
                );
                self.state().events.borrow_mut().push((
                    "notification".into(),
                    serde_json::json!({"title":"Clip","message":text,"severity":"SUCCESS"}),
                ));
            }
        }
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let id = b.clip.register_wait();
        b.coroutine_ticker.register(WaitOwner::Clip, id);
    }
    pub(crate) fn clip_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let (keep, ready) = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let running = in_game && b.query_running(&state.registry, "Clip");
            b.clip.advance(id, running)
        };
        if ready {
            self.clip_tick(host);
        }
        keep
    }
    pub(crate) fn clip_render(&self, host: &dyn Host, batch: &mut HudBatch) {
        let possible = {
            let state = self.state();
            let b = state.behaviors.borrow();
            if !b.query_running(&state.registry, "Clip") || b.clip.mode != 0 {
                return;
            }
            b.clip.possible
        };
        if !possible[0] && !possible[1] {
            return;
        }
        let text = format!(
            "[ {}{} ]",
            if possible[0] { "▲" } else { "" },
            if possible[1] { "▼" } else { "" }
        );
        let position = [
            (batch.width as i32 / 2 + 10) as f32,
            (batch.height as i32 / 2 - 4 + 1) as f32,
        ];
        host.text(batch, &text, position, 0xffffffff, false);
    }
}
