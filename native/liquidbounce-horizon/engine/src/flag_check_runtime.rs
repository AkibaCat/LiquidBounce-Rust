// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::{ClientboundPlayPacket as C, Host};
use liquidbounce_behaviors::flag_check::{Alert, Player, Settings, State};
use serde_json::{Value, json};
use std::sync::Mutex;
#[derive(Default)]
struct Data {
    model: State,
    player: Player,
    alerts: Vec<Alert>,
}
#[derive(Default)]
pub struct FlagCheckService(Mutex<Data>);
fn now() -> i64 {
    crate::behaviors::current_time_millis()
}
impl FlagCheckService {
    pub fn configure(&self, running: bool) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .model
            .sync(running, now());
    }
    pub fn settings(&self) -> Settings {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .model
            .settings
            .clone()
    }
    pub fn edit(&self, f: impl FnOnce(&mut Settings)) {
        let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut d.model.settings);
    }
    fn tick(&self, player: Player) -> Vec<Alert> {
        let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
        d.player = player;
        d.model.clock(now());
        if let Some(a) = d.model.tick(player) {
            d.alerts.push(a);
        }
        std::mem::take(&mut d.alerts)
    }
}
impl PacketListener for FlagCheckService {
    fn incoming(&self, event: &mut PacketEvent<'_, C>) {
        if !event.context.original() {
            return;
        }
        let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let p = d.player;
        d.model.clock(now());
        match event.packet {
            C::PlayerPosition(v) => {
                if let Some(a) = d.model.position(p, v.position, [v.yaw, v.pitch], now()) {
                    d.alerts.push(a);
                }
            }
            C::Disconnect(_) => d.model.disconnect(p),
            _ => {}
        }
    }
}
fn player(v: &Value) -> Option<Player> {
    Some(Player {
        ticks: v["ticks"].as_i64()? as i32,
        yaw: v["yaw"].as_f64()? as f32,
        pitch: v["pitch"].as_f64()? as f32,
        health: v["health"].as_f64()? as f32,
        alive: v["alive"].as_bool()?,
        food: v["food"].as_i64()? as i32,
    })
}
fn text(a: &Alert, language: &str) -> String {
    let language = language.to_ascii_lowercase().replace('-', "_");
    let index = match language.as_str() {
        "zh_cn" | "zh_sg" | "zh_hans" => 1,
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => 2,
        _ => 0,
    };
    let reason = match a.reason {
        liquidbounce_behaviors::flag_check::Reason::Invalid => ["Invalid", "无效", "無效"],
        liquidbounce_behaviors::flag_check::Reason::ForceRotate => {
            ["Force-Rotate", "强制转头", "強制轉頭"]
        }
        _ => ["LagBack", "回弹", "回彈"],
    }[index];
    let extra = a.extra.as_deref().map(|s| {
        if index == 0 {
            s.to_owned()
        } else {
            s.replace("Health", if index == 1 { "生命值" } else { "生命值" })
                .replace("Hunger", if index == 1 { "饥饿值" } else { "飢餓值" })
        }
    });
    match (index, extra) {
        (0, Some(e)) => format!("Detected {reason} {e} ({}x)", a.count),
        (0, None) => format!("Detected {reason} ({}x)", a.count),
        (1, Some(e)) => format!("检测到{reason} {e}({}x)", a.count),
        (1, None) => format!("检测到{reason}({}x)", a.count),
        (_, Some(e)) => format!("檢測到{reason} {e}({}x)", a.count),
        (_, None) => format!("檢測到{reason}({}x)", a.count),
    }
}
/// Reuse the existing frozen six-part WireframePlayer mesh and renderer.
pub(crate) fn append_geometry(service: &FlagCheckService, first: bool, body: &mut Value) {
    let mut d = service.0.lock().unwrap_or_else(|e| e.into_inner());
    let Some([color, outline]) = d.model.render_colors(now(), first) else {
        return;
    };
    let wire = liquidbounce_behaviors::blink_esp::BlinkEsp {
        mode: 2,
        wire_color: color,
        wire_outline: outline,
        line: 0,
        ..Default::default()
    };
    let p = liquidbounce_behaviors::blink_esp::Player {
        first_person: false,
        width: 0.6,
        height: 1.8,
        pose: 0,
        swim: 0.,
    };
    let mesh = wire.visual(&[d.model.position], d.model.rotation, p).body;
    let point = |p: liquidbounce_behaviors::breadcrumbs::ColoredPoint| json!({"position":p.position,"color":p.color});
    if let Some(q) = body["quads"].as_array_mut() {
        q.extend(mesh.quads.into_iter().map(|q| json!(q.map(point))));
    }
    if let Some(q) = body["lines"].as_array_mut() {
        q.extend(mesh.lines.into_iter().map(|q| json!(q.map(point))));
    }
}
impl LiquidBounceHooks {
    pub(crate) fn flag_check_tick(&self, host: &mut dyn Host) {
        let Some(p) = host.client_flag_check_snapshot().as_ref().and_then(player) else {
            return;
        };
        let service = self.state().behaviors.borrow().flag_check.clone();
        let alerts = service.tick(p);
        let settings = service.settings();
        for alert in alerts {
            let message = text(&alert, &horizon_client::lang::code());
            if settings.chat {
                host.client_flag_check_chat(&message, &format!("FlagCheck#{}", alert.reason.key()));
            }
            if settings.notification {
                #[cfg(feature = "webview")]
                self.state().events.borrow_mut().push((
                    "notification".into(),
                    json!({"title":"FlagCheck","message":message,"severity":"INFO"}),
                ));
            }
        }
    }
}
