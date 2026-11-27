//! HUD REST routes and the independent input-free CEF overlay frame.
use super::*;
use liquidbounce_behaviors::hud::{Overlay, Screen};
use liquidbounce_webview::input::Input;

fn push(browser: &Browser, event: Input) {
    if let Err(error) = browser.layer.input.push(event) {
        eprintln!("[LiquidBounce] {error}");
    }
}

impl Browser {
    pub(super) fn hud_request(
        &mut self,
        actor: &mut Actor<'_>,
        request: &ApiRequest,
    ) -> Option<ApiResponse> {
        let path = request.path.strip_prefix("/api/v1/client/components/")?;
        let mut parts = path.split('/');
        let id = parts.next().filter(|s| !s.is_empty())?;
        let tail = parts.next();
        let mut hud = actor.state.behaviors.borrow_mut();
        let components = &mut hud.hud.components;
        match (request.method.clone(), tail) {
            (Method::GET, None) => {
                if id == "native" || components.themes.iter().any(|t| t.id == id) {
                    return Some(ApiResponse::json(components.payloads(Some(id))));
                }
                components
                    .component(id)
                    .map(|c| ApiResponse::json(c.payload()))
            }
            (Method::GET, Some("catalog")) => Some(ApiResponse::json(components.catalog(id))),
            (Method::POST, None) => {
                let added = components.add(id);
                drop(hud);
                match added {
                    Some(new_id) => Some(ApiResponse::json(json!({"id":new_id}))),
                    None => Some(ApiResponse::error(
                        StatusCode::BAD_REQUEST,
                        "HUD component cannot be added",
                    )),
                }
            }
            (Method::POST, Some("z-index")) => {
                let z_index = components.bring_to_front(id);
                drop(hud);
                match z_index {
                    Some(z) => Some(ApiResponse::json(json!({"zIndex":z}))),
                    None => Some(ApiResponse::error(
                        StatusCode::NOT_FOUND,
                        "HUD component not found",
                    )),
                }
            }
            (Method::POST, Some("alignment")) => {
                let value: Value = match serde_json::from_slice(&request.body) {
                    Ok(value) => value,
                    Err(_) => {
                        return Some(ApiResponse::error(
                            StatusCode::BAD_REQUEST,
                            "Invalid alignment body",
                        ));
                    }
                };
                let Some(component) = components.component_mut(id) else {
                    return Some(ApiResponse::error(
                        StatusCode::NOT_FOUND,
                        "HUD component not found",
                    ));
                };
                let settings = &mut component.settings;
                for (name, key) in [
                    ("horizontalAlignment", "Horizontal"),
                    ("horizontalOffset", "HorizontalOffset"),
                    ("verticalAlignment", "Vertical"),
                    ("verticalOffset", "VerticalOffset"),
                ] {
                    if let Some(value) = value.get(name).cloned() {
                        if crate::hud_components::set(settings, &["Alignment", key], value).is_err()
                        {
                            return Some(ApiResponse::error(
                                StatusCode::BAD_REQUEST,
                                "Invalid HUD alignment",
                            ));
                        }
                    }
                }
                components.changed();
                drop(hud);
                Some(ApiResponse::no_content())
            }
            (Method::GET, Some("settings")) => components
                .component(id)
                .map(|c| ApiResponse::json(c.settings.clone()))
                .or_else(|| {
                    Some(ApiResponse::error(
                        StatusCode::NOT_FOUND,
                        "HUD component not found",
                    ))
                }),
            (Method::PUT, Some("settings")) => {
                let value: Value = match serde_json::from_slice(&request.body) {
                    Ok(value) => value,
                    Err(_) => {
                        return Some(ApiResponse::error(
                            StatusCode::BAD_REQUEST,
                            "Invalid request body",
                        ));
                    }
                };
                let response = {
                    let Some(component) = components.component_mut(id) else {
                        return Some(ApiResponse::error(
                            StatusCode::NOT_FOUND,
                            "HUD component not found",
                        ));
                    };
                    if let Err(error) = crate::hud_components::merge(&mut component.settings, &value) {
                        return Some(ApiResponse::error(StatusCode::BAD_REQUEST, error));
                    }
                    ApiResponse::json(component.settings.clone())
                };
                components.changed();
                drop(hud);
                Some(response)
            }
            (_, Some("file")) => Some(ApiResponse::error(
                StatusCode::NOT_FOUND,
                "HUD component files require the binary asset backend",
            )),
            _ => None,
        }
    }

    pub(super) fn hud_frame(
        &mut self,
        state: &mut crate::runtime::State,
        client: &NativeClientState,
        refresh_rate: u32,
    ) {
        let in_game = client.logged_in && client.world.is_some();
        let screen = if self.visible {
            Screen::ClickGui
        } else if in_game && matches!(client.screen, NativeScreenKind::Playing) {
            Screen::None
        } else {
            Screen::Other
        };
        let (generation, events) = {
            let mut behavior = state.behaviors.borrow_mut();
            behavior.hud.refresh_rate = refresh_rate.max(1);
            behavior.hud.behavior.hud_editor_selected = self.ui.hud_editor_selected;
            let running = behavior.query_running(&state.registry, "HUD");
            let overlay = behavior
                .hud
                .behavior
                .update(running, in_game, client.gui_hidden, screen);
            let generation =
                behavior.hud.behavior.reopen_generation ^ behavior.hud.components.generation;
            let events = (generation != behavior.hud.published_generation)
                .then(|| behavior.hud.components.events())
                .unwrap_or_default();
            let visible = overlay == Overlay::Visible;
            let close = overlay == Overlay::Closed;
            push(
                self,
                Input::HudOverlay {
                    url: format!("{}#/hud?static", self.server.theme_url()),
                    fps: behavior
                        .hud
                        .behavior
                        .current_fps(refresh_rate.max(1) as i32)
                        .max(1) as u32,
                    visible,
                    close,
                    generation,
                },
            );
            behavior.hud.published_generation = generation;
            (generation, events)
        };
        for (name, value) in events.into_iter() {
            self.server.broadcast(&name, &value);
        }
        let _ = generation;
    }
}
