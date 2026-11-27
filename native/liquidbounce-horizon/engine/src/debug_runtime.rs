// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientDebugEvent;
use liquidbounce_behaviors::debug::{Debug, Priority};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};
#[derive(Default)]
pub struct DebugRuntime {
    pub model: Debug,
    file: Option<File>,
    pub pending_enable: bool,
}
impl DebugRuntime {
    pub fn enabled(&mut self) {
        self.pending_enable = true;
        self.model.enabled();
    }
    pub fn clear(&mut self) {
        self.model.clear();
        self.file = None;
        self.pending_enable = false;
    }
    fn logging(&mut self, root: &Path, now: i64) {
        if self.pending_enable {
            self.pending_enable = false;
            if self.model.logging {
                let dir = root.join("debug-log");
                let result = std::fs::create_dir_all(&dir).and_then(|_| {
                    OpenOptions::new()
                        .create_new(true)
                        .append(true)
                        .open(dir.join(format!("{now}.tsv")))
                });
                match result {
                    Ok(file) => self.file = Some(file),
                    Err(e) => eprintln!("[LiquidBounce] Debug log: {e}"),
                }
            }
        }
        let lines = std::mem::take(&mut self.model.log);
        if let Some(file) = self.file.as_mut() {
            for line in lines {
                if let Err(e) = file.write_all(line.as_bytes()) {
                    eprintln!("[LiquidBounce] Debug log: {e}");
                    break;
                }
            }
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn debug(&self, event: &mut ClientDebugEvent<'_>) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        if !b.query_running(&state.registry, "Debug") {
            b.debug.clear();
            return;
        }
        let now = crate::behaviors::current_time_millis();
        match input["action"].as_str().unwrap_or("") {
            "policy" => {
                *event.output=json!({"running":true,"simulation":b.debug.model.settings.simulated_player,"ticks":b.debug.model.settings.ticks_to_predict}).to_string();
            }
            "tick" => {
                let d = &mut b.debug.model;
                d.expire(now);
                for (name, value) in input["parameters"].as_object().into_iter().flatten() {
                    d.put(
                        "Debug/Client",
                        false,
                        name,
                        Priority::Normal,
                        value.clone(),
                        now,
                    );
                }
                if let Ok(predicted) = serde_json::from_value(input["predicted"].clone()) {
                    d.predicted = predicted;
                }
            }
            "parameter" => {
                b.debug.model.put(
                    input["owner"].as_str().unwrap_or("Debug"),
                    input["global"] == true,
                    input["name"].as_str().unwrap_or("Value"),
                    Priority::parse(input["priority"].as_str().unwrap_or("NORMAL")),
                    input["value"].clone(),
                    now,
                );
            }
            "geometry" => {
                b.debug.model.put_geometry(
                    input["owner"].as_str().unwrap_or("Debug"),
                    input["global"] == true,
                    input["name"].as_str().unwrap_or("Geometry"),
                    input["geometry"].clone(),
                );
            }
            "scroll" => {
                *event.output = json!(b.debug.model.scroll(
                    input["amount"].as_f64().unwrap_or(0.) as f32,
                    serde_json::from_value(input["cursor"].clone()).unwrap_or([0.; 2]),
                    input["chat"] == true
                ))
                .to_string();
            }
            "frame" => {
                let width = input["width"].as_f64().unwrap_or(0.) as f32;
                let height = input["height"].as_f64().unwrap_or(0.) as f32;
                let mut frame = b
                    .debug
                    .model
                    .frame(width, height, input["tab"] == true, now);
                localize_frame(&mut frame, &horizon_client::lang::code());
                *event.output = frame.to_string();
            }
            "worldChange" => {
                b.debug.model.predicted.clear();
            }
            _ => {}
        }
        b.debug.logging(&state.config_root, now);
    }
}

fn localize_frame(frame: &mut Value, language: &str) {
    use liquidbounce_interop::localization::{display_label, display_module_name};
    let status = display_label(
        language,
        "rows above: {above}, below: {below}, scroll with chat open",
    )
    .replace("{above}", &frame["above"].to_string())
    .replace("{below}", &frame["below"].to_string());
    frame["status"] = json!(status);
    frame["graphLabel"] = json!(display_label(language, "Graph"));
    for card in frame["cards"].as_array_mut().into_iter().flatten() {
        for row in card["rows"].as_array_mut().into_iter().flatten() {
            match row["type"].as_str() {
                Some("header") => {
                    if row["text"] == "Debug" {
                        row["text"] = json!(display_module_name(language, "Debug"));
                    }
                }
                Some("subheader") => {
                    if row["text"] == "Client" {
                        row["text"] = json!(display_label(language, "Client"));
                    }
                }
                Some("item") => {
                    if let Some(name) = row["name"].as_str() {
                        row["name"] = json!(display_label(language, name));
                    }
                }
                _ => {}
            }
        }
    }
}
