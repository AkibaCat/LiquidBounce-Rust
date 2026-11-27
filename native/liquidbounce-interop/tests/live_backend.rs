// SPDX-License-Identifier: GPL-3.0-or-later
//! Real TCP protocol tests. The small test registry is not a gameplay fixture.
use liquidbounce_interop::{
    ApiRequest, ApiResponse, Backend, Server, ServerConfig, ServerInfo, StatusCode,
    protocol::{ModuleAccess, module_request},
    ui::UiState,
};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

struct Registry {
    enabled: bool,
    settings: Value,
}
impl ModuleAccess for Registry {
    fn modules(&self) -> Vec<Value> {
        vec![
            json!({"name":"Zoom","category":"Render","enabled":self.enabled,"hidden":false,"aliases":["Magnify"],"keyBind":{"boundKey":"key.keyboard.unknown","action":"Toggle","modifiers":[]},"description":"Test registry","tag":null}),
        ]
    }
    fn settings(&self, _: &str) -> Option<Value> {
        Some(self.settings.clone())
    }
    fn set_enabled(&mut self, _: &str, enabled: bool) -> Result<(), String> {
        self.enabled = enabled;
        Ok(())
    }
    fn set_settings(&mut self, _: &str, settings: &Value) -> Result<(), String> {
        self.settings = settings.clone();
        Ok(())
    }
}
struct TestBackend {
    registry: Mutex<Registry>,
    ui: Mutex<UiState>,
}
impl Backend for TestBackend {
    fn request(&self, request: ApiRequest) -> ApiResponse {
        if let Some(reply) = self.ui.lock().unwrap().request(&request) {
            return reply;
        }
        module_request(&mut *self.registry.lock().unwrap(), &request)
            .unwrap_or_else(|| ApiResponse::error(StatusCode::NOT_IMPLEMENTED, "Not implemented"))
    }
}

fn http(server: &Server, method: &str, path: &str, body: &str) -> (u16, Option<Value>) {
    let mut stream = TcpStream::connect(server.address()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(stream,"{method} {path} HTTP/1.1\r\nHost: {}\r\nCookie: lb_auth=SecretToken123456\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",server.address(),body.len()).unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).unwrap();
    let (headers, body) = reply.split_once("\r\n\r\n").unwrap();
    let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
    (
        status,
        if body.is_empty() {
            None
        } else {
            Some(serde_json::from_str(body).unwrap())
        },
    )
}

#[test]
fn live_module_toggle_put_delete_and_settings_match_original_routes() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("index.html"), "<html></html>").unwrap();
    std::fs::write(root.path().join("metadata.json"),r##"{"id":"liquidbounce","name":"LiquidBounce","colors":{"Accent":"#4677ff","Tint":"#000000"}}"##).unwrap();
    let backend = Arc::new(TestBackend {
        registry: Mutex::new(Registry {
            enabled: false,
            settings: json!({"name":"Zoom","value":[]}),
        }),
        ui: Mutex::new(UiState::load(root.path(), &root.path().join("state")).unwrap()),
    });
    let server = Server::start(
        ServerConfig {
            port: 0,
            auth_code: "SecretToken123456".into(),
            theme_root: root.path().into(),
            info: ServerInfo {
                name: "test".into(),
                version: "test".into(),
                author: "test".into(),
            },
            external_theme: false,
        },
        backend.clone(),
    )
    .unwrap();
    // The original POST ignores enabled:false and toggles the actual state.
    assert_eq!(
        http(
            &server,
            "POST",
            "/api/v1/client/modules/toggle",
            r#"{"name":"zoom","enabled":false}"#
        )
        .0,
        204
    );
    assert!(backend.registry.lock().unwrap().enabled);
    assert_eq!(
        http(
            &server,
            "PUT",
            "/api/v1/client/modules/toggle",
            r#"{"name":"Zoom"}"#
        ),
        (403, Some(json!({"reason":"Zoom already enabled"})))
    );
    assert_eq!(
        http(
            &server,
            "DELETE",
            "/api/v1/client/modules/toggle",
            r#"{"name":"Zoom"}"#
        )
        .0,
        204
    );
    assert!(!backend.registry.lock().unwrap().enabled);
    // ModuleManager resolves names case-insensitively; descriptive aliases are
    // not REST lookup keys in the original implementation.
    assert_eq!(
        http(&server, "GET", "/api/v1/client/module/zOoM", "")
            .1
            .unwrap()["name"],
        "Zoom"
    );
    assert_eq!(
        http(&server, "GET", "/api/v1/client/module/Magnify", "").0,
        403
    );
    assert_eq!(
        http(&server, "GET", "/api/v1/client/module/KillAura", "").0,
        403
    );
    assert_eq!(
        http(&server, "GET", "/api/v1/client/modules/settings", "").0,
        400
    );
    let settings = json!({"name":"Zoom","value":[{"name":"Bind","value":{"boundKey":"key.keyboard.z","action":"Hold","modifiers":["Control"]}}]});
    assert_eq!(
        http(
            &server,
            "PUT",
            "/api/v1/client/modules/settings?name=Zoom",
            &settings.to_string()
        )
        .0,
        204
    );
    assert_eq!(
        http(
            &server,
            "GET",
            "/api/v1/client/modules/settings?name=zoom",
            ""
        )
        .1,
        Some(settings)
    );
    let modules = http(&server, "GET", "/api/v1/client/modules", "")
        .1
        .unwrap();
    assert_eq!(modules.as_array().unwrap().len(), 1);
    server.stop();
}

#[test]
fn ui_reads_original_colors_persists_storage_and_rejects_stale_screen_ack() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("metadata.json"),r##"{"id":"liquidbounce","name":"LiquidBounce","colors":{"Accent":"#4677ff","Tint":"#000000"}}"##).unwrap();
    let state = root.path().join("state");
    let request = |method: &str, path: &str, body: &str| ApiRequest {
        method: method.parse().unwrap(),
        path: format!("/api/v1/client/{path}"),
        query: Default::default(),
        body: body.to_owned().into(),
    };
    let mut ui = UiState::load(root.path(), &state).unwrap();
    let theme = ui
        .request(&request("GET", "theme/liquidbounce", ""))
        .unwrap()
        .body
        .unwrap();
    assert_eq!(theme["colors"]["accent"], json!(0xff46_77ffu32 as i32));
    assert_eq!(
        ui.request(&request(
            "PUT",
            "localStorage/all",
            r#"{"items":[{"key":"clickgui.Zoom","value":"true"}]}"#
        ))
        .unwrap()
        .status,
        StatusCode::NO_CONTENT
    );
    let mut reloaded = UiState::load(root.path(), &state).unwrap();
    assert_eq!(
        reloaded
            .request(&request("GET", "localStorage/all", ""))
            .unwrap()
            .body
            .unwrap(),
        json!({"items":[{"key":"clickgui.Zoom","value":"true"}]})
    );
    reloaded.set_screen(Some("clickgui"));
    assert_eq!(
        reloaded
            .request(&request("POST", "virtualScreen", r#"{"name":"title"}"#))
            .unwrap()
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        reloaded
            .request(&request(
                "PUT",
                "screen/escape",
                r#"{"name":"clickgui","keep":true}"#
            ))
            .unwrap()
            .status,
        StatusCode::NO_CONTENT
    );
    assert!(reloaded.page_keeps_escape);
    reloaded.set_screen(None);
    assert!(!reloaded.page_keeps_escape);
    assert_eq!(
        reloaded
            .request(&request(
                "PUT",
                "screen/escape",
                r#"{"name":"clickgui","keep":true}"#
            ))
            .unwrap()
            .status,
        StatusCode::FORBIDDEN
    );
    let mut settings = request(
        "PUT",
        "modules/settings",
        r#"{"name":"ClickGUI","value":[{"name":"Scale","value":3.25},{"name":"Snapping","value":[{"name":"GridSize","value":256}]}]}"#,
    );
    settings.query.insert("name".into(), "clickgui".into());
    assert_eq!(
        reloaded.request(&settings).unwrap().status,
        StatusCode::NO_CONTENT
    );
    let events = reloaded.take_events();
    assert_eq!(
        events
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["clickGuiScaleChange", "clickGuiValueChange"]
    );
    // Declared slider ranges do not clamp values loaded through ConfigSystem.
    assert_eq!(reloaded.clickgui_settings()["value"][0]["value"], 3.25);
    assert_eq!(
        reloaded.clickgui_settings()["value"][2]["value"][1]["value"],
        256
    );
    assert_eq!(
        reloaded.request(&settings).unwrap().status,
        StatusCode::NO_CONTENT
    );
    assert!(reloaded.take_events().is_empty());
    let again = UiState::load(root.path(), &state).unwrap();
    assert_eq!(again.clickgui_settings(), reloaded.clickgui_settings());
}
