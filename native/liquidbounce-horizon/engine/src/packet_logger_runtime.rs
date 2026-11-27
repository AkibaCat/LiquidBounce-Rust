// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{Delivery, PacketContext, PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::{ClientboundPlayPacket as C, Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::packet_logger::{Field, Record, Settings};
use serde_json::{Value, json};
use std::{fs::OpenOptions, io::Write, path::PathBuf, sync::Mutex};
#[derive(Default)]
struct Data {
    settings: Settings,
    running: bool,
    root: PathBuf,
    file: Option<PathBuf>,
    chat: Vec<(Record, bool)>,
    errors: Vec<String>,
}
#[derive(Default)]
pub struct PacketLoggerService(Mutex<Data>);
fn now() -> i64 {
    crate::behaviors::current_time_millis()
}
impl PacketLoggerService {
    pub fn settings(&self) -> Settings {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .settings
            .clone()
    }
    pub fn edit(&self, f: impl FnOnce(&mut Settings)) {
        let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut d.settings);
        if d.running && d.settings.file {
            Self::create(&mut d);
        }
    }
    fn create(d: &mut Data) {
        if d.file.is_none() {
            d.file = Some(d.root.join("packet-logger").join(format!("{}.csv", now())));
        }
    }
    pub fn configure(&self, running: bool) {
        let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if d.running != running {
            d.running = running;
            if running {
                Self::create(&mut d);
            } else {
                d.file = None;
            }
        }
    }
    pub fn root(&self, root: PathBuf) {
        let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if d.root != root {
            d.root = root;
            d.file = None;
            if d.running {
                Self::create(&mut d);
            }
        }
    }
    fn enabled(&self) -> bool {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).running
    }
    fn record(&self, incoming: bool, class: &str, id: &str, cancelled: bool, fields: Vec<Field>) {
        let mut d = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if !d.running || !d.settings.accepts(incoming, id) {
            return;
        }
        let r = Record {
            time: now(),
            incoming,
            class: class.into(),
            id: id.into(),
            cancelled,
            fields,
        };
        if d.settings.file {
            Self::create(&mut d);
            if let Some(path) = d.file.as_ref() {
                let result = (|| -> std::io::Result<()> {
                    std::fs::create_dir_all(path.parent().unwrap())?;
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)?
                        .write_all(r.csv().as_bytes())
                })();
                if let Err(e) = result {
                    d.errors.push(e.to_string());
                }
            }
        }
        if d.settings.chat {
            let show = d.settings.show_type;
            d.chat.push((r, show));
        }
    }
}
impl PacketListener for PacketLoggerService {
    fn incoming_final(&self, _context: PacketContext, p: &C, cancelled: bool) {
        if self.enabled() {
            let (id, fields) = crate::packet_logger_fields::clientbound(p);
            self.record(true, p.name(), id, cancelled, fields);
        }
    }
    fn outgoing_final(&self, _context: PacketContext, p: &S, cancelled: bool) {
        if self.enabled() {
            let (id, fields) = crate::packet_logger_fields::serverbound(p);
            self.record(false, p.name(), id, cancelled, fields);
        }
    }
    fn outgoing(&self, e: &mut PacketEvent<'_, S>) {
        if e.context.delivery == Delivery::Silent {
            self.outgoing_final(e.context, e.packet, e.is_cancelled());
        }
    }
}
fn lang() -> usize {
    match horizon_client::lang::code()
        .to_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "zh_cn" | "zh_sg" | "zh_hans" => 1,
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => 2,
        _ => 0,
    }
}
fn chat(r: &Record, show: bool) -> Value {
    let n = lang();
    let mut parts = vec![
        json!({"text":if r.incoming{["Receive","接收","接收"][n]}else{["Send","发送","傳送"][n]},"color":if r.incoming{0x5555ff}else{0xaaaaaa},"bold":true}),
        json!({"text":" "}),
        json!({"text":r.class,"color":0xaaaaaa,"copy":r.class}),
        json!({"text":" (ID: "}),
        json!({"text":r.id,"color":0x55ffff,"copy":r.id}),
        json!({"text":")"}),
    ];
    if r.cancelled {
        parts.push(
            json!({"text":format!(" ({})",["canceled","已取消","已取消"][n]),"color":0xff5555}),
        );
    }
    if !r.fields.is_empty() {
        parts.push(json!({"text":":"}));
    }
    for f in &r.fields {
        parts.extend([
            json!({"text":"\n- ","color":0xaaaaaa}),
            json!({"text":f.name,"copy":f.name,"color":0x55ffff}),
        ]);
        if show {
            parts.extend([
                json!({"text":": ","color":0xaaaaaa}),
                json!({"text":f.kind,"copy":f.kind,"color":0xffff55}),
            ]);
        }
        parts.extend([
            json!({"text":" = ","color":0xaaaaaa}),
            json!({"text":f.value,"copy":f.value,"color":0xffffff}),
        ]);
    }
    json!(parts)
}
impl LiquidBounceHooks {
    pub(crate) fn packet_logger_tick(&self, host: &mut dyn Host) {
        let service = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let s = b.packet_logger.clone();
            s.root(state.config_root.clone());
            s
        };
        let (records, errors) = {
            let mut d = service.0.lock().unwrap_or_else(|e| e.into_inner());
            (std::mem::take(&mut d.chat), std::mem::take(&mut d.errors))
        };
        for (r, show) in records {
            host.client_packet_logger_chat(&chat(&r, show).to_string());
        }
        for e in errors {
            host.chat(&format!(
                "{}: {e}",
                [
                    "Packet log file error",
                    "数据包日志文件错误",
                    "封包記錄檔案錯誤"
                ][lang()]
            ));
        }
    }
}
