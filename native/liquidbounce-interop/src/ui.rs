// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce's Theme/LocalStorage/ScreenFunctions and ModuleClickGui.
//! Persistent presentation state. This is not a simulated player or game backend.
use crate::{ApiRequest, ApiResponse, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Saved {
    #[serde(default)]
    storage: BTreeMap<String, String>,
    scale: f64,
    snapping: bool,
    grid_size: i32,
    search_auto_focus: bool,
    space_separated_names: bool,
}

impl Default for Saved {
    fn default() -> Self {
        Self {
            storage: BTreeMap::new(),
            scale: 1.0,
            snapping: true,
            grid_size: 10,
            search_auto_focus: true,
            space_separated_names: true,
        }
    }
}

/// The owning game actor sets its screen from real open/close callbacks, calls
/// `request` before `protocol::module_request`, and consumes typing/Escape state.
pub struct UiState {
    theme: Value,
    save_file: PathBuf,
    saved: Saved,
    screen: Option<String>,
    pub typing: bool,
    pub hud_editor_selected: bool,
    pub page_keeps_escape: bool,
    events: Vec<(String, Value)>,
}

fn group(name: &str, children: Vec<Value>) -> Value {
    json!({"name":name,"valueType":"CONFIGURABLE","value":children})
}

fn leaf(name: &str, kind: &str, value: Value) -> Value {
    json!({"name":name,"valueType":kind,"value":value})
}

fn color(value: &str) -> io::Result<i32> {
    let hex = value
        .strip_prefix('#')
        .ok_or_else(|| io::Error::other("Invalid theme color"))?;
    let bits = u32::from_str_radix(hex, 16).map_err(io::Error::other)?;
    let argb = match hex.len() {
        6 => 0xff00_0000 | bits,
        8 => (bits >> 8) | ((bits & 255) << 24),
        _ => return Err(io::Error::other("Unsupported theme color length")),
    };
    Ok(argb as i32)
}

impl UiState {
    /// Reads actual metadata defaults and this isolated game's saved UI state.
    /// Neither the source theme nor another client's profile is modified.
    pub fn load(theme_root: &Path, state_root: &Path) -> io::Result<Self> {
        let metadata: Value = serde_json::from_slice(&fs::read(theme_root.join("metadata.json"))?)
            .map_err(io::Error::other)?;
        let accent = color(
            metadata["colors"]["Accent"]
                .as_str()
                .ok_or_else(|| io::Error::other("Missing Accent color"))?,
        )?;
        let tint = color(
            metadata["colors"]["Tint"]
                .as_str()
                .ok_or_else(|| io::Error::other("Missing Tint color"))?,
        )?;
        fs::create_dir_all(state_root)?;
        let save_file = state_root.join("web-ui.json");
        let saved = if save_file.is_file() {
            serde_json::from_slice(&fs::read(&save_file)?).map_err(io::Error::other)?
        } else {
            Saved::default()
        };
        Ok(Self {
            theme: json!({"name":metadata["name"],"id":metadata["id"],"colors":{"accent":accent,"tint":tint},"settings":{"colors":{"accent":accent,"tint":tint}}}),
            save_file,
            saved,
            screen: None,
            typing: false,
            hud_editor_selected: false,
            page_keeps_escape: false,
            events: vec![],
        })
    }

    /// Call only after the native screen actually changes. `None` routes to none.
    pub fn set_screen(&mut self, name: Option<&str>) {
        self.screen = name.map(str::to_owned);
        self.page_keeps_escape = false;
        self.typing = false;
        self.hud_editor_selected = false;
    }

    pub fn screen(&self) -> Option<&str> {
        self.screen.as_deref()
    }

    /// Broadcast these original envelopes using `Server::broadcast` after the
    /// actual UI mutation. Screen events remain owned by the native adapter.
    pub fn take_events(&mut self) -> Vec<(String, Value)> {
        std::mem::take(&mut self.events)
    }

    pub fn clickgui_settings(&self) -> Value {
        let mut scale = leaf("Scale", "FLOAT", json!(self.saved.scale));
        scale["range"] = json!({"from":0.5,"to":2.0});
        scale["suffix"] = json!("");
        let mut grid = leaf("GridSize", "INT", json!(self.saved.grid_size));
        grid["range"] = json!({"from":1,"to":100});
        grid["suffix"] = json!("px");
        let mut snapping = group(
            "Snapping",
            vec![leaf("Enabled", "BOOLEAN", json!(self.saved.snapping)), grid],
        );
        snapping["valueType"] = json!("TOGGLEABLE");
        group(
            "ClickGUI",
            vec![
                scale,
                leaf(
                    "SearchBarAutoFocus",
                    "BOOLEAN",
                    json!(self.saved.search_auto_focus),
                ),
                snapping,
            ],
        )
    }

    pub fn hud_settings(&self) -> Value {
        // Only the presentation option used by the original theme is implemented.
        // No native HUD components or module registration are inferred from it.
        group(
            "HUD",
            vec![leaf(
                "SpaceSeperatedNames",
                "BOOLEAN",
                json!(self.saved.space_separated_names),
            )],
        )
    }

    fn save(&self) -> io::Result<()> {
        let pending = self.save_file.with_extension("json.tmp");
        fs::write(
            &pending,
            serde_json::to_vec_pretty(&self.saved).map_err(io::Error::other)?,
        )?;
        fs::rename(pending, &self.save_file)
    }

    fn saved(&self) -> ApiResponse {
        match self.save() {
            Ok(()) => ApiResponse::no_content(),
            Err(error) => ApiResponse::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("UI persistence failed: {error}"),
            ),
        }
    }

    /// Handles only presentation-owned routes. Gameplay/window/player data must
    /// be supplied separately by the actual Horizon actor.
    pub fn request(&mut self, request: &ApiRequest) -> Option<ApiResponse> {
        let path = request.path.strip_prefix("/api/v1/client/")?;
        if request.method == Method::GET
            && (path == "theme" || path == format!("theme/{}", self.theme["id"].as_str()?))
        {
            return Some(ApiResponse::json(self.theme.clone()));
        }
        if path == "virtualScreen" {
            if request.method == Method::GET {
                return Some(ApiResponse::json(
                    json!({"name":self.screen,"showingSplash":false}),
                ));
            }
            if request.method == Method::POST {
                let payload = match payload(request) {
                    Ok(v) => v,
                    Err(e) => return Some(e),
                };
                return Some(
                    if payload["name"].as_str() == Some(self.screen.as_deref().unwrap_or("none")) {
                        ApiResponse::no_content()
                    } else {
                        ApiResponse::error(StatusCode::FORBIDDEN, "Wrong virtual screen")
                    },
                );
            }
        }
        if path == "modules/settings" {
            let name = request.query.get("name")?;
            if name.eq_ignore_ascii_case("ClickGUI") || name.eq_ignore_ascii_case("HUD") {
                let name = if name.eq_ignore_ascii_case("ClickGUI") {
                    "ClickGUI"
                } else {
                    "HUD"
                };
                if request.method == Method::GET {
                    return Some(ApiResponse::json(if name == "ClickGUI" {
                        self.clickgui_settings()
                    } else {
                        self.hud_settings()
                    }));
                }
                if request.method == Method::PUT {
                    let before = (
                        self.saved.scale,
                        self.saved.snapping,
                        self.saved.grid_size,
                        self.saved.search_auto_focus,
                        self.saved.space_separated_names,
                    );
                    let tree = match payload(request) {
                        Ok(v) => v,
                        Err(e) => return Some(e),
                    };
                    let Some(values) = tree["value"].as_array() else {
                        return Some(bad("Invalid settings tree"));
                    };
                    for value in values {
                        match (name, value["name"].as_str()) {
                            ("ClickGUI", Some("Scale")) => {
                                if let Some(v) = value["value"].as_f64() {
                                    // The range is UI metadata, not a setter clamp.
                                    self.saved.scale = f64::from(v as f32);
                                }
                            }
                            ("ClickGUI", Some("SearchBarAutoFocus")) => {
                                if let Some(v) = value["value"].as_bool() {
                                    self.saved.search_auto_focus = v;
                                }
                            }
                            ("ClickGUI", Some("Snapping")) => {
                                if let Some(children) = value["value"].as_array() {
                                    for child in children {
                                        match child["name"].as_str() {
                                            Some("Enabled") => {
                                                if let Some(v) = child["value"].as_bool() {
                                                    self.saved.snapping = v;
                                                }
                                            }
                                            Some("GridSize") => {
                                                if let Some(v) = child["value"].as_i64() {
                                                    self.saved.grid_size = v as i32;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            ("HUD", Some("SpaceSeperatedNames")) => {
                                if let Some(v) = value["value"].as_bool() {
                                    self.saved.space_separated_names = v;
                                }
                            }
                            _ => {}
                        }
                    }
                    let after = (
                        self.saved.scale,
                        self.saved.snapping,
                        self.saved.grid_size,
                        self.saved.search_auto_focus,
                        self.saved.space_separated_names,
                    );
                    if before.0 != after.0 {
                        self.events
                            .push(("clickGuiScaleChange".into(), json!({"value":after.0})));
                    }
                    if (before.0, before.1, before.2, before.3)
                        != (after.0, after.1, after.2, after.3)
                    {
                        self.events.push((
                            "clickGuiValueChange".into(),
                            json!({"configurable":self.clickgui_settings()}),
                        ));
                    }
                    if before.4 != after.4 {
                        self.events
                            .push(("spaceSeperatedNamesChange".into(), json!({"value":after.4})));
                    }
                    return Some(self.saved());
                }
            }
        }
        if path == "localStorage/all" {
            if request.method == Method::GET {
                return Some(ApiResponse::json(
                    json!({"items":self.saved.storage.iter().map(|(key,value)|json!({"key":key,"value":value})).collect::<Vec<_>>()}),
                ));
            }
            if request.method == Method::PUT {
                let payload = match payload(request) {
                    Ok(v) => v,
                    Err(e) => return Some(e),
                };
                let Some(items) = payload["items"].as_array() else {
                    return Some(bad("Missing items"));
                };
                let mut storage = BTreeMap::new();
                for item in items {
                    let (Some(key), Some(value)) = (item["key"].as_str(), item["value"].as_str())
                    else {
                        return Some(bad("Invalid storage item"));
                    };
                    storage.insert(key.to_owned(), value.to_owned());
                }
                self.saved.storage = storage;
                return Some(self.saved());
            }
        }
        if path == "typing" && request.method == Method::POST {
            let payload = match payload(request) {
                Ok(v) => v,
                Err(e) => return Some(e),
            };
            let Some(value) = payload["typing"].as_bool() else {
                return Some(bad("Missing typing"));
            };
            self.typing = value;
            return Some(ApiResponse::no_content());
        }
        if path == "hud-editor" && request.method == Method::PUT {
            let payload = match payload(request) {
                Ok(v) => v,
                Err(e) => return Some(e),
            };
            let Some(value) = payload["selected"].as_bool() else {
                return Some(bad("Missing selected"));
            };
            self.hud_editor_selected = value;
            return Some(ApiResponse::no_content());
        }
        if path == "screen/escape" && request.method == Method::PUT {
            let payload = match payload(request) {
                Ok(v) => v,
                Err(e) => return Some(e),
            };
            if payload["name"].as_str() != self.screen.as_deref() || self.screen.is_none() {
                return Some(ApiResponse::error(
                    StatusCode::FORBIDDEN,
                    "Not the open screen",
                ));
            }
            let Some(keep) = payload["keep"].as_bool() else {
                return Some(bad("Missing keep"));
            };
            self.page_keeps_escape = keep;
            return Some(ApiResponse::no_content());
        }
        None
    }
}

fn bad(reason: &str) -> ApiResponse {
    ApiResponse::error(StatusCode::BAD_REQUEST, reason)
}
fn payload(request: &ApiRequest) -> Result<Value, ApiResponse> {
    serde_json::from_slice(&request.body).map_err(|_| bad("Invalid request body"))
}
