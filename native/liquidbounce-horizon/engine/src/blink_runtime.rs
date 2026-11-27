// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{PacketEvent, PacketListener},
    runtime::{LiquidBounceHooks, raw_options},
};
use horizon_client::mod_api::{Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::blink::{Blink, Settings};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
pub struct BlinkService {
    pub state: Mutex<Blink>,
    running: AtomicBool,
    pending_disable: AtomicBool,
}
impl BlinkService {
    pub fn settings(&self) -> Settings {
        self.state.lock().expect("Blink").settings
    }
    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        f(&mut self.state.lock().expect("Blink").settings);
    }
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
    }
    pub fn enabled(&self) {
        self.pending_disable.store(false, Ordering::Release);
        self.state.lock().expect("Blink").enabled();
    }
    pub(crate) fn queues(&self, outgoing: bool) -> bool {
        self.state.lock().expect("Blink").queues(
            self.running.load(Ordering::Acquire) && !self.pending_disable.load(Ordering::Acquire),
            outgoing,
        )
    }
}
/// MODEL_STATE runs before the single shared BlinkManager FINAL_DECISION.
pub struct AmbushListener(
    pub Arc<BlinkService>,
    pub Arc<crate::criticals_runtime::CriticalsService>,
);
impl PacketListener for AmbushListener {
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        if event.is_cancelled() || !self.0.queues(true) || !self.0.settings().ambush {
            return;
        }
        if matches!(
            event.packet,
            S::Interact(_) | S::Attack(_) | S::SpectatorAction(_)
        ) {
            self.0.pending_disable.store(true, Ordering::Release);
            self.0.running.store(false, Ordering::Release);
            self.1.flush_origins(self.0.settings().directions);
            self.1.flush_outgoing_event(event);
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn blink_tick(&self, host: &mut dyn Host) {
        let (manager, active) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.criticals.clone(),
                b.query_running(&s.registry, "Blink")
                    && b.query_ids
                        .get("AutoDodge")
                        .is_some_and(|id| s.registry.enabled_without_world(*id)),
            )
        };
        if !active {
            return;
        }
        let positions = manager.positions();
        let Some(_) = positions.first().copied() else {
            return;
        };
        let query=serde_json::json!({"mode":"blink","positions":positions});
        let Ok(result)=serde_json::from_str::<serde_json::Value>(&host.client_auto_dodge_query(&query.to_string()))else{return;};
        if result["threatened"]!=true{return;}
        if let Some(index)=result["index"].as_i64(){
            let index=index as i32;let impact=result["impact"]==true;
            if impact {
                self.blink_notify(
                    "Trying to evade arrow...",
                    "正在尝试躲避箭矢……",
                    "正在嘗試躲避箭矢……",
                );
            } else {
                self.blink_notify("Arrow evaded.", "已躲避箭矢。", "已躲避箭矢。");
            }
            manager.flush_count(index.wrapping_add(1));
            self.criticals_flush_ready(host);
        } else {
            self.blink_notify(
                "Unable to evade arrow. Blinking.",
                "无法躲避箭矢，正在释放延迟数据包。",
                "無法躲避箭矢，正在釋放延遲封包。",
            );
            self.blink_disable_now(host);
        }
    }
    pub(crate) fn blink_enabled(&self, host: &mut dyn Host) {
        let service = self.state().behaviors.borrow().blink.clone();
        service.enabled();
        if service.settings().dummy {
            host.client_blink_control("create", &[]);
        }
    }
    pub(crate) fn blink_disabled(&self, host: &mut dyn Host) {
        let (service, manager) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (b.blink.clone(), b.criticals.clone())
        };
        manager.flush_origins(service.settings().directions);
        self.criticals_flush_ready(host);
        host.client_blink_control("remove", &[]);
    }
    pub(crate) fn blink_disable_now(&self, host: &mut dyn Host) {
        let options = raw_options(host);
        let effects = {
            let mut s = self.state();
            let id = s.behaviors.borrow().query_ids.get("Blink").copied();
            if let Some(id) = id {
                if let Err(e) = s.with_registry(options, |r, c| r.set_enabled_sync(id, false, c)) {
                    eprintln!("[LiquidBounce] Blink disable: {e:?}");
                }
            }
            s.take_effects()
        };
        crate::runtime::apply_effects(self, host, effects);
    }
    pub(crate) fn blink_pending(&self, host: &mut dyn Host) {
        let pending = self
            .state()
            .behaviors
            .borrow()
            .blink
            .pending_disable
            .swap(false, Ordering::AcqRel);
        if pending {
            self.blink_disable_now(host);
        }
    }
    pub(crate) fn blink_movement(&self, host: &mut dyn Host) {
        self.blink_pending(host);
        let (service, manager, running) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.blink.clone(),
                b.criticals.clone(),
                b.query_running(&s.registry, "Blink"),
            )
        };
        if !running {
            return;
        }
        let action = service.state.lock().expect("Blink").movement();
        if let Some(action) = action {
            if action == 0 {
                let at = manager.cancel_shared();
                if let Some(at) = at {
                    host.client_blink_control("reset", &at);
                }
            } else {
                manager.flush_origins(service.settings().directions);
                host.client_blink_control("move", &[]);
            }
            self.criticals_flush_ready(host);
            if service.settings().auto_disable {
                self.blink_notify("Auto reset", "自动重置", "自動重設");
                self.blink_disable_now(host);
            }
        }
    }
    pub(crate) fn blink_notify(&self, en: &str, cn: &str, tw: &str) {
        let lang = horizon_client::lang::code();
        let message = if lang.starts_with("zh_tw") || lang.starts_with("zh_hk") {
            tw
        } else if lang.starts_with("zh") {
            cn
        } else {
            en
        };
        #[cfg(feature = "webview")]
        let title = liquidbounce_interop::localization::display_module_name(&lang, "Blink");
        #[cfg(feature = "webview")]
        self.state().events.borrow_mut().push((
            "notification".into(),
            serde_json::json!({"title":title,"message":message,"severity":"INFO"}),
        ));
        #[cfg(not(feature = "webview"))]
        eprintln!("[LiquidBounce] Blink: {message}");
    }
}
