// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::{ClientboundPlayPacket as C, Host, NativeScreenKind};
use liquidbounce_behaviors::anti_staff::{State, staff_lines};
use serde_json::{Value, json};
use std::sync::{Mutex, mpsc};
struct Completion {
    generation: u64,
    domain: String,
    result: Result<String, horizon_client::anti_staff::FetchError>,
}
struct Data {
    model: State,
    tx: mpsc::Sender<Completion>,
    rx: mpsc::Receiver<Completion>,
    alerts: Vec<String>,
}
pub struct AntiStaffService(Mutex<Data>);
impl Default for AntiStaffService {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self(Mutex::new(Data {
            model: State::default(),
            tx,
            rx,
            alerts: vec![],
        }))
    }
}
impl AntiStaffService {
    pub fn configure(&self, enabled: bool, running: bool) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .model
            .sync(enabled, running);
    }
    pub fn show(&self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .model
            .show_in_tab_list
    }
    pub fn set_show(&self, show: bool) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .model
            .show_in_tab_list = show;
    }
    fn tab(&self) -> Vec<String> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .model
            .tab_staff()
    }
    fn frame(
        &self,
        server: Option<&str>,
        in_game: bool,
        screen_open: bool,
    ) -> (Vec<Completion>, Vec<String>) {
        let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(domain) = d.model.frame(server, in_game, screen_open) {
            let tx = d.tx.clone();
            let generation = d.model.generation;
            let failed_domain = domain.clone();
            if let Err(e) = std::thread::Builder::new()
                .name("AntiStaff-CDN".into())
                .spawn(move || {
                    let result = horizon_client::anti_staff::load(&domain);
                    let _ = tx.send(Completion {
                        generation,
                        domain,
                        result,
                    });
                })
            {
                let _ = d.tx.send(Completion {
                    generation,
                    domain: failed_domain,
                    result: Err(horizon_client::anti_staff::FetchError::Transport(
                        e.to_string(),
                    )),
                });
            }
        }
        let mut done = vec![];
        while let Ok(c) = d.rx.try_recv() {
            let staffs = c
                .result
                .as_ref()
                .map(|s| staff_lines(s))
                .unwrap_or_default();
            if d.model.accept(c.generation, &c.domain, staffs) {
                done.push(c);
            }
        }
        (done, std::mem::take(&mut d.alerts))
    }
}
impl PacketListener for AntiStaffService {
    fn incoming(&self, event: &mut PacketEvent<'_, C>) {
        if !event.context.original() {
            return;
        }
        if let C::PlayerInfoUpdate(p) = event.packet {
            if p.actions & 1 == 0 {
                return;
            }
            let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
            for name in p.entries.iter().filter_map(|p| p.name.as_ref()) {
                if d.model.is_staff(name) {
                    d.alerts.push(name.clone());
                }
            }
        }
    }
}
fn message(key: &str, args: &[String]) -> String {
    let mut text = liquidbounce_interop::localization::describe(
        &horizon_client::lang::code(),
        &format!("liquidbounce.module.antiStaff.messages.{key}"),
    );
    for arg in args {
        text = text.replacen("%s", arg, 1);
    }
    text
}
impl LiquidBounceHooks {
    pub(crate) fn anti_staff_frame(&self, host: &mut dyn Host) {
        let Some(client) = host.client_state() else {
            return;
        };
        let service = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.anti_staff.configure(
                b.query_ids
                    .get("AntiStaff")
                    .is_some_and(|id| s.registry.enabled_without_world(*id)),
                b.query_running(&s.registry, "AntiStaff"),
            );
            b.anti_staff.clone()
        };
        let (done, alerts) = service.frame(
            client.server_address.as_deref(),
            client.logged_in && client.world.is_some(),
            client.screen != NativeScreenKind::Playing,
        );
        for c in done {
            let (key, args, severity) = match c.result {
                Ok(body) => (
                    "staffsLoaded",
                    vec![staff_lines(&body).len().to_string(), c.domain],
                    "SUCCESS",
                ),
                Err(horizon_client::anti_staff::FetchError::Http(404)) => {
                    ("noStaffs", vec![c.domain], "ERROR")
                }
                Err(e) => ("staffsFailed", vec![c.domain, e.code()], "ERROR"),
            };
            let text = message(key, &args);
            eprintln!("[AntiStaff] {text}");
            #[cfg(feature="webview")]
   self.state().events.borrow_mut().push(("notification".into(),json!({"title":liquidbounce_interop::localization::display_module_name(&horizon_client::lang::code(),"AntiStaff"),"message":text,"severity":severity})));
        }
        for name in alerts {
            let text = message("staffAlert", &[name.clone()]);
            host.client_anti_staff_chat(&text, &format!("AntiStaff#{name}"));
            #[cfg(feature = "webview")]
            self.state().events.borrow_mut().push((
                "notification".into(),
                json!({"title":message("staffDetected",&[]),"message":text,"severity":"INFO"}),
            ));
        }
    }
    pub(crate) fn anti_staff_tab(
        &self,
        event: &mut horizon_client::mod_api::ClientBetterTabEvent<'_>,
    ) {
        let staffs = self.state().behaviors.borrow().anti_staff.tab();
        if staffs.is_empty() {
            return;
        }
        let mut policy=serde_json::from_str::<Value>(event.policy).ok().filter(Value::is_object).unwrap_or_else(||json!({"sorting":0,"visibility":3,"showGameMode":false,"limit":80,"height":20,"accurate":false,"suffix":false,"rows":{},"error":null}));
        policy["staffs"] = json!(staffs);
        policy["staffSuffix"] = json!(message("staffSuffix", &[]));
        *event.policy = policy.to_string();
    }
}
