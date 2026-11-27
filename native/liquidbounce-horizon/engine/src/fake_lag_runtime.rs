// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::fake_lag::{Context, FakeLag, PacketKind, Settings};
use serde_json::{Value, json};
use std::sync::Mutex;
struct State {
    behavior: FakeLag,
    running: bool,
    context: Context,
}
pub struct FakeLagService(Mutex<State>);
fn random(r: [i32; 2]) -> i32 {
    liquidbounce_behaviors::random::random_i32_inclusive(r[0], r[1])
}
impl Default for FakeLagService {
    fn default() -> Self {
        Self(Mutex::new(State {
            behavior: FakeLag::new(random(Settings::default().delay)),
            running: false,
            context: Context::default(),
        }))
    }
}
impl FakeLagService {
    pub fn settings(&self) -> Settings {
        self.0.lock().expect("FakeLag").behavior.settings
    }
    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        f(&mut self.0.lock().expect("FakeLag").behavior.settings);
    }
    pub fn set_running(&self, running: bool) {
        let mut s = self.0.lock().expect("FakeLag");
        if s.running && !running {
            s.behavior.disabled();
        }
        s.running = running;
    }
    pub(crate) fn queues(&self, outgoing: bool, packet: PacketKind, oldest: Option<i64>) -> bool {
        let mut s = self.0.lock().expect("FakeLag");
        let running = s.running;
        let c = s.context;
        s.behavior.queues(
            running,
            outgoing,
            packet,
            crate::behaviors::current_time_millis(),
            oldest,
            c,
            random,
        )
    }
}
pub(crate) fn packet(p: &S) -> PacketKind {
    match p {
        S::Interact(_) | S::Attack(_) | S::SpectatorAction(_) | S::Punch(_) => {
            PacketKind::EntityInteract
        }
        S::UseItemOn(_) | S::SignUpdate(_) => PacketKind::BlockInteract,
        S::PlayerAction(_) => PacketKind::Action,
        S::ResourcePack(_) => PacketKind::ResourcePack,
        _ => PacketKind::Other,
    }
}
impl LiquidBounceHooks {
    pub(crate) fn fake_lag_refresh(&self, host: &dyn Host, tick: bool) {
        let (service, manager, running, shoot_constant, no_target) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.fake_lag.clone(),
                b.criticals.clone(),
                b.query_running(&s.registry, "FakeLag"),
                b.auto_shoot.behavior.settings.constant_lag,
                b.auto_shoot.behavior.target.is_none(),
            )
        };
        if !running {
            return;
        }
        let old = manager.positions().first().copied();
        let settings = service.settings();
        let Ok(v) = serde_json::from_str::<Value>(
            &host.client_fake_lag_query(&json!({"old":old,"range":settings.range}).to_string()),
        ) else {
            return;
        };
        if !v.is_object() {
            service.0.lock().expect("FakeLag").context = Context {
                dead: true,
                ..Default::default()
            };
            return;
        }
        let mut c = Context {
            dead: v["dead"] == true,
            in_water: v["water"] == true,
            screen: v["screen"] == true,
            using_consumable: v["usingConsumable"] == true,
            shoot_constant,
            no_shoot_target: no_target,
            old_position: old.is_some(),
            server_distance: f64::INFINITY,
            client_distance: f64::INFINITY,
            ..Default::default()
        };
        let mut nearby = false;
        let mut state = self.state();
        for e in v["entities"].as_array().into_iter().flatten() {
            let Some(id) = e["id"].as_i64() else {
                continue;
            };
            let Some(target) = crate::targeting::snapshot(&e["target"].to_string()) else {
                continue;
            };
            let info = state.target_info(id as i32, &target);
            if !state
                .global_targets
                .combat
                .should_be_attacked(&target.entity, info)
            {
                continue;
            }
            nearby |= e["nearby"] == true;
            if e["inRange"] != true {
                continue;
            }
            c.has_entities = true;
            c.intersects |= e["intersects"] == true;
            c.server_distance = c
                .server_distance
                .min(e["serverDistance"].as_f64().unwrap_or(f64::INFINITY));
            c.client_distance = c
                .client_distance
                .min(e["clientDistance"].as_f64().unwrap_or(f64::INFINITY));
        }
        drop(state);
        let mut s = service.0.lock().expect("FakeLag");
        s.context = c;
        if tick {
            s.behavior.enemy_nearby = nearby;
        }
    }
    pub(crate) fn fake_lag_tick(&self, host: &mut dyn Host) {
        self.fake_lag_refresh(host, true);
        let (manager, active) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.criticals.clone(),
                b.query_running(&s.registry, "FakeLag")
                    && b.query_ids
                        .get("AutoDodge")
                        .is_some_and(|id| s.registry.enabled_without_world(*id)),
            )
        };
        if !active {
            return;
        }
        let positions = manager.positions();
        if positions.is_empty() {
            return;
        }
        let Ok(v) =
            serde_json::from_str::<Value>(&host.client_auto_dodge_query(
                &json!({"mode":"blink","positions":positions}).to_string(),
            ))
        else {
            return;
        };
        if v["threatened"] != true {
            return;
        }
        let texts = if let Some(index) = v["index"].as_i64() {
            manager.flush_count((index as i32).wrapping_add(1));
            if v["impact"] == true {
                [
                    "Trying to evade arrow...",
                    "正在尝试躲避箭矢……",
                    "正在嘗試躲避箭矢……",
                ]
            } else {
                ["Arrow evaded.", "已躲避箭矢。", "已躲避箭矢。"]
            }
        } else {
            manager.flush_origins(2);
            [
                "Unable to evade arrow. Blinking.",
                "无法躲避箭矢，正在释放延迟数据包。",
                "無法躲避箭矢，正在釋放延遲封包。",
            ]
        };
        let lang = horizon_client::lang::code();
        let index = if lang.starts_with("zh_tw") || lang.starts_with("zh_hk") {
            2
        } else if lang.starts_with("zh") {
            1
        } else {
            0
        };
        #[cfg(feature="webview")]
        self.state().events.borrow_mut().push(("notification".into(),json!({"title":liquidbounce_interop::localization::display_module_name(&lang,"FakeLag"),"message":texts[index],"severity":"INFO"})));
        #[cfg(not(feature = "webview"))]
        eprintln!("[LiquidBounce] FakeLag: {}", texts[index]);
        self.criticals_flush_ready(host);
    }
}
