// SPDX-License-Identifier: GPL-3.0-or-later
//! Original-theme menu routes, executed without the module actor or browser locks.
use horizon_client::mod_api::Host;
use liquidbounce_interop::{ApiRequest, ApiResponse, Method, StatusCode};
use serde_json::{Value, json};

/// These actions may synchronously publish native lifecycle hooks. The caller
/// must release both the actor State and Browser mutex before entering here.
pub(crate) fn request(host: &mut dyn Host, request: &ApiRequest) -> Option<ApiResponse> {
    let path = request.path.strip_prefix("/api/v1/client/")?;
    let (method, action, returns_json) = match path {
        "screen" if request.method == Method::DELETE => (Method::DELETE, "close_screen", false),
        "screen" => (Method::PUT, "screen", false),
        "exit" => (Method::POST, "exit", false),
        "worlds" => (Method::GET, "worlds", true),
        "worlds/join" => (Method::POST, "world_join", false),
        "worlds/edit" => (Method::POST, "world_edit", false),
        "worlds/delete" => (Method::POST, "world_delete", false),
        "session" => (Method::GET, "session", true),
        "accounts" => (Method::GET, "accounts", true),
        "servers" => (Method::GET, "servers", true),
        "servers/lan" => (Method::GET, "lan_servers", true),
        "servers/connect" => (Method::POST, "server_connect", false),
        "servers/add" => (Method::PUT, "server_add", false),
        "servers/edit" => (Method::PUT, "server_edit", false),
        "servers/remove" => (Method::DELETE, "server_remove", false),
        "servers/order" => (Method::POST, "server_order", false),
        "protocols" => (Method::GET, "protocols", true),
        "protocols/protocol" if request.method == Method::PUT => {
            (Method::PUT, "set_protocol", false)
        }
        "protocols/protocol" => (Method::GET, "protocol", true),
        // Horizon has no separate global SpooferManager. An empty configurable
        // keeps the original multiplayer page usable without inventing options.
        "spoofer" if request.method == Method::GET => {
            return Some(ApiResponse::json(json!({
                "name":"Spoofer", "valueType":"CONFIGURABLE", "value":[]
            })));
        }
        "update" if request.method == Method::GET => {
            return Some(ApiResponse::json(
                json!({"development":true,"commit":null,"update":null}),
            ));
        }
        _ => return None,
    };
    if request.method != method {
        return Some(ApiResponse::error(
            StatusCode::METHOD_NOT_ALLOWED,
            "Unsupported menu method",
        ));
    }
    let payload = if request.body.is_empty() {
        json!({})
    } else {
        match serde_json::from_slice::<Value>(&request.body) {
            Ok(value) if value.is_object() => value,
            _ => {
                return Some(ApiResponse::error(
                    StatusCode::BAD_REQUEST,
                    "Expected a JSON object",
                ));
            }
        }
    };
    Some(match host.client_menu_request(action, &payload) {
        Ok(value) if returns_json => ApiResponse::json(value),
        Ok(_) => ApiResponse::no_content(),
        Err(error) => ApiResponse::error(StatusCode::BAD_REQUEST, error),
    })
}

/// Poll with no State/Browser guard held. The caller compares this real ping
/// snapshot with the previous one and broadcasts changed rows as serverPinged.
pub(crate) fn server_updates(host: &mut dyn Host) -> Vec<Value> {
    host.client_menu_request("server_updates", &json!({}))
        .ok()
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
}
