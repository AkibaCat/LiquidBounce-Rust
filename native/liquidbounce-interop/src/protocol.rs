// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ModuleFunctions.kt, copyright 2015-2026 CCBlueX.
//! Original REST module semantics over the actual game-thread registry.
use crate::{ApiRequest, ApiResponse, Method, StatusCode};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Implement on the game actor. JSON comes from its actual registered modules.
/// Mutations must execute lifecycle callbacks and persist their resulting state.
/// Broadcast events from those callbacks, including changes caused by key binds;
/// the HTTP dispatcher deliberately does not emit duplicate synthetic events.
pub trait ModuleAccess {
    fn modules(&self) -> Vec<Value>;
    fn settings(&self, name: &str) -> Option<Value>;
    fn set_enabled(&mut self, name: &str, enabled: bool) -> Result<(), String>;
    fn set_settings(&mut self, name: &str, settings: &Value) -> Result<(), String>;
}

fn find_module(access: &impl ModuleAccess, name: &str) -> Option<Value> {
    access.modules().into_iter().find(|module| {
        module["name"]
            .as_str()
            .is_some_and(|v| v.eq_ignore_ascii_case(name))
    })
}

fn failure(error: String) -> ApiResponse {
    ApiResponse::error(StatusCode::INTERNAL_SERVER_ERROR, error)
}

fn invalid_body() -> ApiResponse {
    ApiResponse::error(StatusCode::BAD_REQUEST, "Invalid request body")
}

/// `None` means this is not a module route and another real backend may handle it.
pub fn module_request(access: &mut impl ModuleAccess, request: &ApiRequest) -> Option<ApiResponse> {
    let path = request.path.strip_prefix("/api/v1/client/")?;
    if request.method == Method::GET && path == "modules" {
        return Some(ApiResponse::json(Value::Array(access.modules())));
    }
    if request.method == Method::GET && path == "modules/categories" {
        let categories: BTreeSet<_> = access
            .modules()
            .iter()
            .filter_map(|v| v["category"].as_str().map(str::to_owned))
            .collect();
        // The resource endpoint is not assumed to exist. A null icon is part of
        // the upstream schema and preserves the original theme's text fallback.
        return Some(ApiResponse::json(json!(
            categories
                .into_iter()
                .map(|name| json!({"name":name,"icon":null}))
                .collect::<Vec<_>>()
        )));
    }
    if request.method == Method::GET && path.starts_with("module/") {
        return Some(
            find_module(access, &path[7..])
                .map(ApiResponse::json)
                .unwrap_or_else(|| ApiResponse::error(StatusCode::FORBIDDEN, "Module not found")),
        );
    }
    if path == "modules/settings"
        && (request.method == Method::GET || request.method == Method::PUT)
    {
        let Some(name) = request.query.get("name") else {
            return Some(ApiResponse::error(
                StatusCode::BAD_REQUEST,
                "Missing parameter 'name'",
            ));
        };
        // ClickGUI's real UI settings may be supplied without registering it as
        // a completed gameplay module. The owning UI actor handles that route.
        let Some(module) = find_module(access, name) else {
            return Some(ApiResponse::error(
                StatusCode::FORBIDDEN,
                format!("Module '{name}' not found"),
            ));
        };
        let name = module["name"].as_str().expect("registered module name");
        if request.method == Method::GET {
            return Some(
                access
                    .settings(name)
                    .map(ApiResponse::json)
                    .unwrap_or_else(|| {
                        ApiResponse::error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Registered module settings missing",
                        )
                    }),
            );
        }
        let settings = match serde_json::from_slice(&request.body) {
            Ok(settings) => settings,
            Err(_) => return Some(invalid_body()),
        };
        return Some(
            access
                .set_settings(name, &settings)
                .map(|()| ApiResponse::no_content())
                .unwrap_or_else(failure),
        );
    }
    if path == "modules/toggle"
        && matches!(request.method, Method::POST | Method::PUT | Method::DELETE)
    {
        let payload: Value = match serde_json::from_slice(&request.body) {
            Ok(payload) => payload,
            Err(_) => return Some(invalid_body()),
        };
        let Some(name) = payload["name"].as_str() else {
            return Some(invalid_body());
        };
        let Some(module) = find_module(access, name) else {
            return Some(ApiResponse::error(
                StatusCode::FORBIDDEN,
                format!("Module '{name}' not found"),
            ));
        };
        let current = module["enabled"]
            .as_bool()
            .expect("registered module state");
        // Upstream POST always toggles; the browser's `enabled` field is ignored.
        let next = request.method == Method::PUT || (request.method == Method::POST && !current);
        if current == next {
            return Some(ApiResponse::error(
                StatusCode::FORBIDDEN,
                format!(
                    "{name} already {}",
                    if next { "enabled" } else { "disabled" }
                ),
            ));
        }
        let canonical = module["name"].as_str().expect("registered module name");
        return Some(
            access
                .set_enabled(canonical, next)
                .map(|()| ApiResponse::no_content())
                .unwrap_or_else(failure),
        );
    }
    if path == "modules/panic" && request.method == Method::POST {
        for module in access.modules() {
            if module["category"] != "Render"
                && module["enabled"] == true
                && let Some(name) = module["name"].as_str()
                && let Err(error) = access.set_enabled(name, false)
            {
                return Some(failure(error));
            }
        }
        return Some(ApiResponse::no_content());
    }
    None
}
