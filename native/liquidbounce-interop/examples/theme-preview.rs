// SPDX-License-Identifier: GPL-3.0-or-later
//! Presentation-only transport/CEF regression harness: no game modules registered.
use liquidbounce_interop::{
    ApiRequest, ApiResponse, Backend, Method, Server, ServerConfig, ServerInfo, StatusCode,
    protocol::{ModuleAccess, module_request},
    ui::UiState,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

struct Preview {
    ui: Mutex<UiState>,
    width: u32,
    height: u32,
}

struct EmptyRegistry;
impl ModuleAccess for EmptyRegistry {
    fn modules(&self) -> Vec<Value> {
        vec![]
    }
    fn settings(&self, _: &str) -> Option<Value> {
        None
    }
    fn set_enabled(&mut self, _: &str, _: bool) -> Result<(), String> {
        Err("No game connected".into())
    }
    fn set_settings(&mut self, _: &str, _: &Value) -> Result<(), String> {
        Err("No game connected".into())
    }
}

impl Backend for Preview {
    fn request(&self, request: ApiRequest) -> ApiResponse {
        if let Some(reply) = self.ui.lock().expect("preview UI mutex").request(&request) {
            return reply;
        }
        if let Some(reply) = module_request(&mut EmptyRegistry, &request) {
            return reply;
        }
        if request.method == Method::GET {
            match request.path.as_str() {
                "/api/v1/client/info" => {
                    return ApiResponse::json(json!({
                        "os":if cfg!(windows) {"windows"} else if cfg!(target_os="macos") {"mac"} else {"linux"},
                        "clientName":"LiquidBounce theme preview", "clientVersion":env!("CARGO_PKG_VERSION"),
                        "development":true,"inGame":false,"viaFabricPlus":false,"hasProtocolHack":false
                    }));
                }
                "/api/v1/client/window" => {
                    return ApiResponse::json(json!({
                        "width":self.width,"height":self.height,"scaledWidth":self.width/2,"scaledHeight":self.height/2,"scaleFactor":2,"guiScale":2
                    }));
                }
                // This process has no registered global gameplay settings.
                "/api/v1/client/global" => {
                    return ApiResponse::json(
                        json!({"name":"Global","valueType":"CONFIGURABLE","value":[]}),
                    );
                }
                _ => {}
            }
        }
        ApiResponse::error(
            StatusCode::NOT_IMPLEMENTED,
            "Presentation preview: no game connected",
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let theme = PathBuf::from(args.next().ok_or("theme-preview <built-theme-root> <isolated-state-root> [width=1280] [height=720] [seconds=300]")?);
    let state = PathBuf::from(args.next().ok_or("isolated state root required")?);
    let width = args.next().map(|v| v.parse()).transpose()?.unwrap_or(1280);
    let height = args.next().map(|v| v.parse()).transpose()?.unwrap_or(720);
    let seconds = args.next().map(|v| v.parse()).transpose()?.unwrap_or(300);
    let mut ui = UiState::load(&theme, &state)?;
    // This is the actual preview's active page, not a claimed game screen.
    ui.set_screen(Some("clickgui"));
    let config = ServerConfig::original_theme(
        theme,
        ServerInfo {
            name: "LiquidBounce theme preview (no game connected)".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            author: "CCBlueX".into(),
        },
    )?;
    let server = Server::start(
        config,
        Arc::new(Preview {
            ui: Mutex::new(ui),
            width,
            height,
        }),
    )?;
    println!(
        "{}",
        json!({"theme_url":server.theme_url(),"mode":"presentation_only","registered_modules":0,"width":width,"height":height})
    );
    std::thread::sleep(Duration::from_secs(seconds));
    server.stop();
    Ok(())
}
