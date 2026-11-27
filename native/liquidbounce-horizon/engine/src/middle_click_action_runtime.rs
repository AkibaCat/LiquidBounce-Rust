// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::middle_click_action::{Action, Context, Mode, Slot};
pub(crate) const PEARL_REQUESTER: u64 = 0x4d43_415f_5045_4152;
fn slot(v: &serde_json::Value) -> Option<Slot> {
    v.as_i64().and_then(|v| match v {
        -1 => Some(Slot::Offhand),
        0..=8 => Some(Slot::Hotbar(v as u8)),
        _ => None,
    })
}
impl LiquidBounceHooks {
    pub(crate) fn middle_click_action_tick(&self, host: &mut dyn Host) {
        let (running, mode, range) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            (
                b.query_running(&state.registry, "MiddleClickAction"),
                b.middle_click_action.mode,
                b.middle_click_action.pick_up_range,
            )
        };
        if !running {
            return;
        }
        // Ray/shape hooks can read engine state, so the native snapshot precedes its lock.
        let Some(c) = host.client_middle_click_context(mode == Mode::FriendClicker, range) else {
            return;
        };
        let name = c["name"].as_str().map(str::to_owned);
        let action = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            b.middle_click_action.tick(
                running,
                Context {
                    screen_open: c["screenOpen"].as_bool().unwrap_or(true),
                    pitch: c["pitch"].as_f64().unwrap_or(0.0) as f32,
                    pickup: c["pickup"].as_bool().unwrap_or(false),
                    pearl: slot(&c["pearl"]),
                    candidate: name.is_some(),
                    visible: c["visible"].as_bool().unwrap_or(false),
                },
            )
        };
        match action {
            Some(Action::Select(slot, ticks)) => {
                if let Slot::Hotbar(index) = slot {
                    let _ =
                        host.client_select_slot_silently(PEARL_REQUESTER, i32::from(index), ticks);
                }
            }
            Some(Action::Use(slot, ticks)) => {
                if let Slot::Hotbar(index) = slot {
                    let _ =
                        host.client_select_slot_silently(PEARL_REQUESTER, i32::from(index), ticks);
                }
                let [yaw, pitch] = host.client_player_rotation();
                let rotation = {
                    let state = self.state();
                    let b = state.behaviors.borrow();
                    let r = b
                        .rotations
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    r.current
                };
                let (yaw, pitch) = rotation.map_or((yaw, pitch), |r| (r.yaw, r.pitch));
                host.client_use_item(slot.hand(), yaw, pitch);
            }
            Some(Action::ToggleFriend) => {
                if let Some(name) = name {
                    let mut state = self.state();
                    let removed = state.friends.contains(&name);
                    if removed {
                        state.friends.remove(&name);
                    } else {
                        state.friends.add(name.clone(), None);
                    }
                    // Target tags deliberately remain cached for the rest of GameTickEvent.
                    let _ = crate::friends::save(
                        &state.friends,
                        &state.config_root.join("friends.json"),
                    );
                    let language = horizon_client::lang::code()
                        .to_ascii_lowercase()
                        .replace('-', "_");
                    let message = if matches!(language.as_str(), "zh_cn" | "zh_sg" | "zh_hans") {
                        format!(
                            "{name} 已{}好友列表。",
                            if removed { "移除自" } else { "添加到" }
                        )
                    } else if matches!(language.as_str(), "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant") {
                        format!("{name} 已{}好友清單。", if removed { "移出" } else { "加入" })
                    } else {
                        format!(
                            "{name} was successfully {} the friends list.",
                            if removed { "removed from" } else { "added to" }
                        )
                    };
                    #[cfg(feature="webview")]state.events.borrow_mut().push(("notification".into(),serde_json::json!({"title":"FriendClicker","message":message,"severity":"INFO"})));
                    #[cfg(not(feature = "webview"))]
                    eprintln!("[LiquidBounce] FriendClicker: {message}");
                }
            }
            None => {}
        }
    }
    pub(crate) fn middle_click_action_cancel_pick(&self, host: &dyn Host) -> bool {
        let pearl = host.client_middle_click_pearl_slot();
        let state = self.state();
        let b = state.behaviors.borrow();
        b.middle_click_action.cancel_pick(
            b.query_running(&state.registry, "MiddleClickAction"),
            pearl.map(|v| {
                if v < 0 {
                    Slot::Offhand
                } else {
                    Slot::Hotbar(v as u8)
                }
            }),
        )
    }
}
