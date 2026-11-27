// SPDX-License-Identifier: GPL-3.0-or-later
//! Chromium and the original theme are attached to the real native client actor.
#[path="click_gui_browser.rs"]
mod click_gui;
#[path="hud_browser.rs"]
mod hud_browser;
#[path="menu_browser.rs"]
mod menu_browser;
use crate::{
    actor::Actor,
    behaviors::RawOptions,
    runtime::{self, LiquidBounceHooks, MOD_ID},
};
use horizon_client::mod_api::{
    Flow, Hooks, Host, InputKeyEvent, NativeClientState, NativeScreenKind, ScreenRenderPostEvent,
    winit::{
        event::ElementState,
        keyboard::{KeyCode, PhysicalKey},
    },
};
use liquidbounce_interop::{
    ApiRequest, ApiResponse, Method, Server, ServerConfig, ServerInfo, StatusCode,
    desktop::{DesktopBackend, SystemDesktop},
    localization,
    main_thread::{Inbox, QueuedBackend},
    protocol::module_request,
    ui::UiState,
};
use liquidbounce_webview::{
    horizon::Layer,
    input::{Button, Input},
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

pub struct BrowserConfig {
    pub theme_root: PathBuf,
    pub helper_executable: PathBuf,
    pub resources_directory: PathBuf,
    pub cache_directory: PathBuf,
    pub config_directory: PathBuf,
    pub initial_size: [u32; 2],
}
pub(crate) struct Browser {
    pub(crate) theme_root: PathBuf,
    server: Server,
    inbox: Inbox,
    ui: UiState,
    layer: &'static Layer,
    modules_file: PathBuf,
    global_file: PathBuf,
    friends_file: PathBuf,
    loaded: bool,
    saved: Option<Value>,
    saved_global: Option<Value>,
    saved_friends: Option<Value>,
    pub(crate) visible: bool,
    menu: menu_browser::MenuState,
    buttons: u32,
    last_click: Option<Instant>,
    last_click_count: u8,
    reload_held: bool,
    native_fonts:crate::theme_font_runtime::ThemeFontMonitor,
    requests: u64,
    language_generation: u64,
    desktop_language: Arc<RwLock<String>>,
}

/// Call on the window/render thread before entry::main_with; errors are fatal to
/// initialization instead of silently falling back to a replacement interface.
pub fn browser_hooks(config: BrowserConfig) -> Result<Box<dyn Hooks>, String> {
    std::fs::create_dir_all(&config.config_directory).map_err(|e| e.to_string())?;
    let ui =
        UiState::load(&config.theme_root, &config.config_directory).map_err(|e| e.to_string())?;
    let native_fonts=crate::theme_font_runtime::ThemeFontMonitor::new(config.theme_root.clone());
    let (backend, inbox) = QueuedBackend::channel(256, Duration::from_secs(8));
    let desktop_language = Arc::new(RwLock::new(horizon_client::lang::code()));
    let backend = Arc::new(DesktopBackend::new(
        backend,
        Arc::new(SystemDesktop),
        config.config_directory.clone(),
        desktop_language.clone(),
    ));
    let theme_root = config.theme_root.clone();
    let server = Server::start(
        ServerConfig::original_theme(
            config.theme_root,
            ServerInfo {
                name: "LiquidBounce".into(),
                version: "0.43.1 / Horizon port".into(),
                author: "CCBlueX".into(),
            },
        )
        .map_err(|e| e.to_string())?,
        backend,
    )
    .map_err(|e| e.to_string())?;
    let layer = Box::leak(Box::new(Layer::new(config.initial_size, 0)));
    let theme_url = server.theme_url();
    liquidbounce_webview::runtime::initialize_on_first_frame(
        liquidbounce_webview::runtime::Config {
            url: theme_url.clone(),
            click_gui_url:Some(format!("{theme_url}#/clickgui?static")),
            helper_executable: config.helper_executable,
            cache_directory: config.cache_directory,
            resources_directory: config.resources_directory,
        },
        layer.mailbox.clone(),
        layer.input.clone(),
    )?;
    let finish_initialization = (|| {
        layer
            .register()
            .map_err(|e| format!("GPU UI registration failed: {e:?}"))?;
        std::fs::write(config.config_directory.join("interop.json"),serde_json::to_vec_pretty(&json!({"themeUrl":theme_url,"address":server.address().to_string(),"statusPath":"/api/v1/client/integration-status"})).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
    })();
    if let Err(error) = finish_initialization {
        return match shutdown_browser_current_thread() {
            Ok(true) => Err(error),
            Ok(false) => Err(format!("{error}; CEF cleanup timed out")),
            Err(cleanup) => Err(format!("{error}; CEF cleanup failed: {cleanup}")),
        };
    }
    let mut hooks = LiquidBounceHooks::default();
    hooks.state().behaviors.borrow().packet_logger.root(config.config_directory.clone());
    hooks.state().config_root = config.config_directory.clone();
    hooks.browser = Some(Mutex::new(Browser {
        theme_root,
        server,
        inbox,
        ui,
        layer,
        modules_file: config.config_directory.join("modules.json"),
        global_file: config.config_directory.join("global.json"),
        friends_file: config.config_directory.join("friends.json"),
        loaded: false,
        saved: None,
        saved_global: None,
        saved_friends: None,
        visible: false,
        menu: menu_browser::MenuState::default(),
        buttons: 0,
        last_click: None,
        last_click_count: 1,
        reload_held: false,
        native_fonts,
        requests: 0,
        language_generation: horizon_client::lang::generation(),
        desktop_language,
    }));
    Ok(Box::new(hooks))
}

pub fn shutdown_browser_current_thread() -> Result<bool, String> {
    liquidbounce_webview::runtime::shutdown_current_thread(Duration::from_secs(5))
}

fn flags(bits: u32) -> u32 {
    ((bits & 1) << 1) | ((bits & 2) << 1) | ((bits & 4) << 1) | ((bits & 8) << 4)
}
fn clickgui_screen() -> Value {
    json!({"class":"net.ccbluex.liquidbounce.integration.screen.impl.CustomStandaloneMinecraftScreen","title":"ClickGUI"})
}
fn sdl_key_code(scan: i32) -> i32 {
    match scan {
        4..=29 => scan - 4 + 97,
        30..=38 => scan - 30 + 49,
        39 => 48,
        40 => 13,
        41 => 27,
        42 => 8,
        43 => 9,
        44 => 32,
        45 => 45,
        46 => 61,
        47 => 91,
        48 => 93,
        49 => 92,
        51 => 59,
        52 => 39,
        53 => 96,
        54 => 44,
        55 => 46,
        56 => 47,
        76 => 127,
        0 => 0,
        _ => scan | (1 << 30),
    }
}
fn position(browser: &Browser, client: &NativeClientState, mouse: [f32; 2]) -> (i32, i32) {
    let size = browser.layer.mailbox.desired_size();
    (
        (mouse[0] * size[0] as f32 / client.gui_size[0].max(1.0)).round() as i32,
        (mouse[1] * size[1] as f32 / client.gui_size[1].max(1.0)).round() as i32,
    )
}
fn push(browser: &Browser, event: Input) {
    if let Err(error) = browser.layer.input.push(event) {
        eprintln!("[LiquidBounce] {error}");
    }
}

impl Browser {
    fn screen(&mut self, visible: bool) {
        let changed = self.visible != visible;
        self.visible = visible;
        self.sync_route();
        if changed && !visible {
            push(self, Input::SelectPage(liquidbounce_webview::input::PageTarget::Shared));
        }
    }
    fn status(&self, actor: &Actor<'_>) -> Value {
        let page=liquidbounce_webview::runtime::page_state_current_thread().map(|p|json!({"loading":p.loading,"completedLoads":p.completed_loads,"httpStatus":p.http_status,"title":p.title,"loadErrors":p.load_errors,"console":p.console}));
        let pages=liquidbounce_webview::runtime::page_status_current_thread().map(|s|json!({"active":format!("{:?}",s.active),"sharedOpen":s.shared_open,"clickGuiOpen":s.click_gui_open,"sharedBrowserId":s.shared_browser_id,"clickGuiBrowserId":s.click_gui_browser_id,"retiredPages":s.retired_pages,"createdPages":s.created_pages,"closedPages":s.closed_pages,"fps":s.fps,"sharedFps":s.shared_fps,"clickGuiFps":s.click_gui_fps}));
        let modules = actor.state.behaviors.borrow();
        let slots = modules.slots.state().map(|slot| json!({"enforced":slot.enforced_slot.index(),
            "client":slot.clientside_slot.index(),"requester":slot.requester.raw(),"ticks":slot.ticks_until_reset}));
        let hand_derp_packets = modules.hand_derp.accepted_packets();
        json!({"world":{"worldPresent":actor.state.observed.world.world_present,"playerPresent":actor.state.observed.world.player_present},"observed":{"ticks":actor.state.observed.ticks,"keyboardEvents":actor.state.observed.keyboard_events,"mouseEvents":actor.state.observed.mouse_events,"disconnects":actor.state.observed.disconnects},"modules":actor.actual_modules(),"probes":actor.state.probes,"silentHotbar":slots,"handDerpPackets":hand_derp_packets,"combatTicks":actor.state.combat.remaining(),"browser":{"visible":self.visible,"menu":self.menu_status(),"paintCount":self.layer.mailbox.paint_count(),"viewPaintCount":self.layer.mailbox.view_paint_count(),"page":page,"pages":pages},"requests":self.requests})
    }
    fn request(
        &mut self,
        actor: &mut Actor<'_>,
        client: &NativeClientState,
        request: &ApiRequest,
    ) -> ApiResponse {
        self.requests = self.requests.saturating_add(1);
        if request.method == Method::GET && request.path == "/api/v1/client/integration-status" {
            return ApiResponse::json(self.status(actor));
        }
        if request.method == Method::GET && request.path == "/api/v1/client/localization" {
            return ApiResponse::json(localization::snapshot(
                &horizon_client::lang::code(),
                horizon_client::lang::generation(),
            ));
        }
        if request.method == Method::GET && request.path == "/api/v1/client/registry/client_module"
        {
            let language = horizon_client::lang::code();
            let entries: serde_json::Map<_, _> = actor.state.registry.modules().map(|module| {
                let name = &module.definition().name;
                (name.clone(), json!({"name":localization::display_module_name(&language,name),"icon":null}))
            }).collect();
            return ApiResponse::json(Value::Object(entries));
        }
        if request.method==Method::GET && matches!(request.path.as_str(),"/api/v1/client/registry/c2s_packet"|"/api/v1/client/registry/s2c_packet"){
            let ids=if request.path.ends_with("c2s_packet"){crate::packet_logger_fields::C2S_IDS}else{crate::packet_logger_fields::S2C_IDS};
            let entries:serde_json::Map<_,_>=ids.iter().map(|id|(id.to_string(),json!({"name":id,"icon":null}))).collect();
            return ApiResponse::json(Value::Object(entries));
        }
        if request.method==Method::GET && request.path=="/api/v1/client/registry/menu" {
            let entries:serde_json::Map<_,_>=liquidbounce_behaviors::chest_cleaner::MENUS.iter().map(|menu|{
                let path=match *menu {"generic_9x6"=>"chestDouble","generic_9x1"|"generic_9x2"|"generic_9x3"|"generic_9x4"|"generic_9x5"=>"chest","shulker_box"=>"shulkerBox","generic_3x3"=>"dispenser","brewing_stand"=>"brewing",other=>other};
                let key=format!("container.{path}");let name=horizon_client::lang::get(&key);
                (format!("minecraft:{menu}"),json!({"name":if name==key{menu.to_string()}else{name},"icon":null}))
            }).collect();
            return ApiResponse::json(Value::Object(entries));
        }
        if request.method == Method::GET
            && matches!(
                request.path.as_str(),
                "/api/v1/client/registry/items"
                    | "/api/v1/client/registry/item"
                    | "/api/v1/client/registry/entity_type"
                    | "/api/v1/client/registry/block"
            )
        {
            let entities = request.path.ends_with("entity_type");
            let blocks = request.path.ends_with("block");
            let registry = if entities {
                &actor.state.entity_type_registry
            } else if blocks {
                &actor.state.block_registry
            } else {
                &actor.state.item_registry
            };
            let items: serde_json::Map<_, _> = registry
                .iter()
                .map(|id| {
                    let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
                    let key = format!(
                        "{}.{namespace}.{path}",
                        if entities {
                            "entity"
                        } else if blocks {
                            "block"
                        } else {
                            "item"
                        }
                    );
                    let mut name = horizon_client::lang::get(&key);
                    if name == key && !entities && !blocks {
                        let block_key = format!("block.{namespace}.{path}");
                        name = horizon_client::lang::get(&block_key);
                        if name == block_key {
                            name = id.clone();
                        }
                    } else if name == key {
                        name = id.clone();
                    }
                    (id.clone(), json!({"name":name,"icon":null}))
                })
                .collect();
            return ApiResponse::json(Value::Object(items));
        }
        if request.method == Method::GET
            && matches!(
                request.path.as_str(),
                "/api/v1/client/registry/sounds" | "/api/v1/client/registry/sound_event"
            )
        {
            let items: serde_json::Map<_, _> = actor
                .state
                .sound_registry
                .iter()
                .map(|id| {
                    let path = id.split_once(':').map_or(id.as_str(), |(_, path)| path);
                    let name = path
                        .split(['_', '.'])
                        .map(|word| {
                            let mut characters = word.chars();
                            characters.next().map_or_else(String::new, |first| {
                                first.to_uppercase().to_string() + characters.as_str()
                            })
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    (id.clone(), json!({"name":name,"icon":null}))
                })
                .collect();
            return ApiResponse::json(Value::Object(items));
        }
        if request.method == Method::GET
            && matches!(
                request.path.as_str(),
                "/api/v1/client/registry/menu" | "/api/v1/client/registry/screen_handler"
            )
        {
            let items: serde_json::Map<_, _> = crate::config::MENU_IDS
                .iter()
                .map(|id| {
                    let name = id
                        .trim_start_matches("minecraft:")
                        .split('_')
                        .map(|word| {
                            let mut characters = word.chars();
                            characters.next().map_or_else(String::new, |first| {
                                first.to_uppercase().to_string() + characters.as_str()
                            })
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    ((*id).to_owned(), json!({"name":name,"icon":null}))
                })
                .collect();
            return ApiResponse::json(Value::Object(items));
        }
        if request.path=="/api/v1/client/modules/settings"&&request.query.get("name").is_some_and(|n|n=="ClickGUI"){
            if let Some(response)=module_request(actor,request){return response;}
        }
        if request.path=="/api/v1/client/modules/settings"&&request.query.get("name").is_some_and(|n|n.eq_ignore_ascii_case("HUD")){
            if let Some(response)=module_request(actor,request){return response;}
        }
        if let Some(response) = self.hud_request(actor, request) {
            return response;
        }
        if let Some(response) = self.ui.request(request) {
            self.menu_acknowledge(request, &response);
            horizon_client::inventory_move::SEARCH_FOCUSED.store(liquidbounce_behaviors::click_gui::ClickGui::in_search_bar(self.ui.typing,self.visible),std::sync::atomic::Ordering::Release);
            for (name, value) in self.ui.take_events() {
                self.server.broadcast(&name, &value);
            }
            return response;
        }
        if request.path == "/api/v1/client/global" {
            if request.method == Method::GET {
                return ApiResponse::json(actor.global_settings());
            }
            if request.method == Method::PUT {
                let value = match serde_json::from_slice(&request.body) {
                    Ok(value) => value,
                    Err(_) => {
                        return ApiResponse::error(StatusCode::BAD_REQUEST, "Invalid request body");
                    }
                };
                return match actor
                    .set_global_settings(&value)
                    .and_then(|()| actor.save_global(&self.global_file))
                {
                    Ok(()) => {
                        self.saved_global = Some(actor.global_file());
                        ApiResponse::no_content()
                    }
                    Err(error) => ApiResponse::error(StatusCode::INTERNAL_SERVER_ERROR, error),
                };
            }
        }
        if let Some(response) = module_request(actor, request) {
            return response;
        }
        if request.method == Method::GET {
            match request.path.as_str() {
                "/api/v1/client/window" => {
                    let size = self.layer.mailbox.desired_size();
                    let scale = size[0] as f32 / client.gui_size[0].max(1.0);
                    return ApiResponse::json(
                        json!({"width":size[0],"height":size[1],"scaledWidth":client.gui_size[0],"scaledHeight":client.gui_size[1],"scaleFactor":scale,"guiScale":scale}),
                    );
                }
                "/api/v1/client/info" => {
                    return ApiResponse::json(
                        json!({"os":std::env::consts::OS,"gameVersion":"26.3","clientVersion":"0.43.1 / Horizon port","clientName":"LiquidBounce","development":true,"fps":null,"gameDir":null,"clientDir":self.modules_file.parent().map(|p|p.to_string_lossy()),"inGame":client.logged_in&&client.world.is_some(),"viaFabricPlus":false,"hasProtocolHack":false}),
                    );
                }
                "/api/v1/client/input" => {
                    let key = request
                        .query
                        .get("key")
                        .map(String::as_str)
                        .unwrap_or("key.keyboard.unknown");
                    let name = horizon_client::lang::get(key);
                    return ApiResponse::json(json!({"translationKey":key,"localized":name}));
                }
                _ => {}
            }
        }
        ApiResponse::error(
            StatusCode::NOT_FOUND,
            "Endpoint is not implemented by this native adapter",
        )
    }
}

impl LiquidBounceHooks {
    pub(crate) fn browser_tick(
        &self,
        host: &mut dyn Host,
        client: &NativeClientState,
        options: RawOptions,
    ) {
        let Some(browser_mutex) = &self.browser else {
            return;
        };
        self.crosshair_initialize(host);
        let menu_screen = host.client_menu_screen();
        let click_gui = host.mod_screen() == Some(MOD_ID);
        let mut browser = browser_mutex.lock().unwrap_or_else(|e| e.into_inner());
        browser.native_fonts.refresh();
        let language_generation = horizon_client::lang::generation();
        if browser.language_generation != language_generation {
            browser.language_generation = language_generation;
            let game_language = horizon_client::lang::code();
            *browser
                .desktop_language
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = game_language.clone();
            browser.server.broadcast(
                "languageChange",
                &json!({
                    "gameLanguage": game_language,
                    "locale": localization::Locale::from_game_language(&game_language).code(),
                    "generation": language_generation,
                }),
            );
            browser
                .server
                .broadcast("clientLanguageChanged", &Value::Null);
        }
        browser.sync_screens(click_gui, menu_screen.as_deref());
        horizon_client::inventory_move::SEARCH_FOCUSED.store(liquidbounce_behaviors::click_gui::ClickGui::in_search_bar(browser.ui.typing,browser.visible),std::sync::atomic::Ordering::Release);
        let sound_registry = host.client_sound_event_names();
        let no_interact_registries =
            (!browser.loaded).then(|| host.client_no_interact_registries());
        let default_holding_items =
            (!browser.loaded).then(|| host.client_default_no_entity_holding_items());
        let block_registry = (!browser.loaded).then(|| host.client_auto_tool_block_registry());
        let entity_interaction_attribute =
            host.local_attribute("minecraft:entity_interaction_range");
        {
            let mut state = self.state();
            state.sound_registry = sound_registry;
            if let Some(block_registry) = block_registry {
                state.block_registry = block_registry;
                state.behaviors.borrow_mut().chest_stealer.initialize_storage(&state.block_registry);
            }
            if let Some((entity_types, items)) = no_interact_registries {
                state.entity_type_registry = entity_types;
                state.item_registry = items;
            }
            if let Some(defaults) = default_holding_items {
                state
                    .behaviors
                    .borrow_mut()
                    .no_entity_interact
                    .holding_items = defaults.into_iter().collect();
            }
            Self::observe_world(&mut state, client, options);
            let mut actor = Actor {
                state: &mut state,
                entity_interaction_attribute,
                options,
            };
            if !browser.loaded {
                if let Err(error) = actor
                    .state
                    .behaviors
                    .borrow_mut()
                    .hud
                    .components
                    .load(&browser.theme_root, browser.modules_file.parent().unwrap_or(std::path::Path::new(".")))
                {
                    eprintln!("[LiquidBounce] HUD theme load: {error}");
                }
                if let Err(error) =
                    crate::friends::load(&mut actor.state.friends, &browser.friends_file)
                {
                    eprintln!("[LiquidBounce] friend config load: {error}");
                }
                if let Err(error) = actor.load_global(&browser.global_file) {
                    eprintln!("[LiquidBounce] global config load: {error}");
                }
                browser.migrate_click_gui(&mut actor);
                if let Err(error) = actor.load(&browser.modules_file) {
                    eprintln!("[LiquidBounce] module config load: {error}");
                }
                browser.loaded = true;
            }
        }
        let mut pending_requests = Vec::new();
        while let Some(pending) = browser.inbox.try_next() {
            pending_requests.push(pending);
        }
        drop(browser);
        for pending in pending_requests {
            let local_response = browser_mutex.lock().unwrap_or_else(|e| e.into_inner())
                .menu_request(&pending.request);
            // Menu actions can fire native lifecycle hooks synchronously.
            // Neither the Browser mutex nor the actor State is held here.
            let menu_response = local_response.or_else(|| crate::menu_backend::request(host, &pending.request));
            if let Some(response) = menu_response {
                let menu_screen = host.client_menu_screen();
                let click_gui = host.mod_screen() == Some(MOD_ID);
                let mut browser = browser_mutex.lock().unwrap_or_else(|e| e.into_inner());
                browser.requests = browser.requests.saturating_add(1);
                browser.sync_screens(click_gui, menu_screen.as_deref());
                drop(browser);
                pending.reply(response);
                continue;
            }
            // Query Host before the actor lock, including menu/config loads.
            let options = runtime::raw_options(host);
            let sound_registry = host.client_sound_event_names();
            let (entity_types, items) = host.client_no_interact_registries();
            let entity_interaction_attribute =
                host.local_attribute("minecraft:entity_interaction_range");
            let (mut response, effects) = {
                let mut browser = browser_mutex.lock().unwrap_or_else(|e| e.into_inner());
                let mut state = self.state();
                state.sound_registry = sound_registry;
                state.entity_type_registry = entity_types;
                state.item_registry = items;
                let mut actor = Actor {
                    state: &mut state,
                    entity_interaction_attribute,
                    options,
                };
                let response = browser.request(&mut actor, client, &pending.request);
                (response, state.take_effects())
            };
            runtime::apply_effects(self, host, effects);
            if response.status.is_success()
                && pending.request.method != Method::GET
                && pending.request.path.starts_with("/api/v1/client/modules/")
            {
                let mut browser = browser_mutex.lock().unwrap_or_else(|e| e.into_inner());
                let mut state = self.state();
                let actor = Actor {
                    state: &mut state,
                    entity_interaction_attribute,
                    options,
                };
                match actor.save(&browser.modules_file) {
                    Ok(()) => {
                        browser.saved = actor.file().ok();
                    }
                    Err(error) => {
                        response = ApiResponse::error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Module persistence failed after applying state: {error}"),
                        );
                    }
                }
            }
            pending.reply(response);
        }
        let menu_screen = host.client_menu_screen();
        let click_gui = host.mod_screen() == Some(MOD_ID);
        let refresh = host.client_click_gui_refresh_rate();
        let poll_servers = browser_mutex.lock().unwrap_or_else(|e| e.into_inner()).menu_poll_servers();
        let server_updates = poll_servers.then(|| crate::menu_backend::server_updates(host));
        let mut browser = browser_mutex.lock().unwrap_or_else(|e| e.into_inner());
        browser.sync_screens(click_gui, menu_screen.as_deref());
        if let Some(updates) = server_updates {
            browser.menu_server_updates(updates);
        }
        browser.click_gui_frame(self,host,client);
        browser.menu_frame();
        let mut state = self.state();
        browser.hud_frame(&mut state, client, refresh);
        let actor = Actor {
            state: &mut state,
            entity_interaction_attribute,
            options,
        };
        match actor.file() {
            Ok(file) if browser.saved.as_ref() != Some(&file) => {
                match actor.save(&browser.modules_file) {
                    Ok(()) => browser.saved = Some(file),
                    Err(error) => eprintln!("[LiquidBounce] module config save: {error}"),
                }
            }
            Err(error) => eprintln!("[LiquidBounce] module config serialize: {error}"),
            _ => {}
        }
        let global = actor.global_file();
        if browser.saved_global.as_ref() != Some(&global) {
            match actor.save_global(&browser.global_file) {
                Ok(()) => browser.saved_global = Some(global),
                Err(error) => eprintln!("[LiquidBounce] global config save: {error}"),
            }
        }
        let friends = crate::friends::file(&actor.state.friends);
        if browser.saved_friends.as_ref() != Some(&friends) {
            match crate::friends::save(&actor.state.friends, &browser.friends_file) {
                Ok(()) => browser.saved_friends = Some(friends),
                Err(error) => eprintln!("[LiquidBounce] friend config save: {error}"),
            }
        }
        let notifications = actor
            .state
            .behaviors
            .borrow()
            .anti_hunger
            .notifications
            .drain(&|language, key| liquidbounce_interop::localization::describe(language, key));
        for (name, value) in &notifications {
            actor
                .state
                .probe("clientAntiHungerNotification", json!(name), value.clone());
        }
        actor.state.events.borrow_mut().extend(notifications);
        for (name, value) in actor.state.events.borrow_mut().drain(..) {
            browser.server.broadcast(&name, &value);
        }
    }

    pub(crate) fn browser_key(&self, host: &mut dyn Host, event: &InputKeyEvent<'_>) -> bool {
        let PhysicalKey::Code(code) = event.event.physical_key else {
            return false;
        };
        self.browser_physical_key(
            host,
            code,
            event.event.state,
            event.event.repeat,
            event.modifiers,
        )
    }

    pub(crate) fn browser_physical_key(
        &self,
        host: &mut dyn Host,
        code: KeyCode,
        state: ElementState,
        repeat: bool,
        modifiers: u32,
    ) -> bool {
        let Some(_) = &self.browser else {
            return false;
        };
        let own = host.mod_screen() == Some(MOD_ID);
        if own && state==ElementState::Pressed && !repeat && crate::input::keyboard_scan_code(code).is_some_and(|scan|self.click_gui_bound(liquidbounce_core::InputKey::Keyboard(scan),modifiers)) {
            host.mod_screen_close(MOD_ID);
            return true;
        }
        if !own && !self.browser_accepts_input(host) {
            return false;
        }
        // A screen's press hook precedes its separate character callback;
        // native_input_key is deliberately later. Releases have no text.
        if state == ElementState::Released {
            self.browser_raw_key(host, code, state, repeat, modifiers);
        }
        true
    }

    fn browser_raw_key(
        &self,
        host: &dyn Host,
        code: KeyCode,
        state: ElementState,
        repeat: bool,
        modifiers: u32,
    ) {
        if !self.browser_accepts_input(host) {
            return;
        }
        let Some(browser) = &self.browser else {
            return;
        };
        let mut browser = browser.lock().unwrap_or_else(|e| e.into_inner());
        use horizon_client::mod_api::winit::platform::scancode::PhysicalKeyExtScancode;
        let physical_scan = PhysicalKey::Code(code).to_scancode().unwrap_or(0);
        let sdl_scan = crate::input::keyboard_scan_code(code).unwrap_or(0);
        // Exact explicit non-extended entries of pinned JCEF MapScanCodeSdl.
        let scan = match sdl_scan {
            88 => 28,
            76 => 83,
            73 => 82,
            80 => 75,
            79 => 77,
            82 => 72,
            81 => 80,
            75 => 73,
            78 => 81,
            74 => 71,
            77 => 79,
            70 => 55,
            72 => 69,
            _ => physical_scan,
        };
        let up = state == ElementState::Released;
        #[cfg(windows)]
        let virtual_key = unsafe { MapVirtualKeyW(scan, 1) as i32 & 255 };
        #[cfg(not(windows))]
        let virtual_key = scan as i32;
        let reload = code == KeyCode::KeyR && !up && modifiers & 2 != 0;
        let reload_release = code == KeyCode::KeyR && up && browser.reload_held;
        if reload {
            browser.reload_held = true;
            browser.native_fonts.reload();
            browser.invalidate_menu();
            push(&browser, Input::Reload);
        } else if reload_release {
            browser.reload_held = false;
        } else {
            push(
                &browser,
                Input::Key {
                    virtual_key,
                    native_code: ((scan << 16) | 1 | if up { 0xc0000000 } else { 0 }) as i32,
                    modifiers: flags(modifiers) | browser.buttons,
                    up,
                    system: false,
                },
            );
        }
        let key = horizon_client::keybinds::key_name(code).unwrap_or("key.keyboard.unknown");
        browser.server.broadcast("keyboardKey",&json!({"keyCode":sdl_key_code(sdl_scan),"scanCode":sdl_scan,"action":if up{0}else if repeat{2}else{1},"mods":modifiers,"key":key,"screen":browser.screen_descriptor()}));
    }

    pub(crate) fn browser_screen_render(
        &self,
        host: &dyn Host,
        event: &mut ScreenRenderPostEvent<'_>,
    ) -> bool {
        let own = host.mod_screen() == Some(MOD_ID);
        if !own && menu_browser::route(Some(event.screen.id)).is_none() {
            return false;
        }
        let Some(browser) = &self.browser else {
            return false;
        };
        let Some(client) = host.client_state() else {
            return false;
        };
        let mut browser = browser.lock().unwrap_or_else(|e| e.into_inner());
        browser.menu_frame();
        if !own && !browser.accepts_menu(Some(event.screen.id)) {
            return false;
        }
        let (x, y) = position(&browser, &client, event.screen.mouse);
        push(
            &browser,
            Input::MouseMove {
                x,
                y,
                modifiers: browser.buttons,
                leave: false,
            },
        );
        if !own {
            // The CEF route has acknowledged its DOM and delivered a fresh
            // paint. Preserve glyph uploads; remove only the underlying UI.
            event.batch.vertices.clear();
            event.batch.blends.clear();
            event.batch.pictures.clear();
            event.batch.blur_at = 0;
        }
        true
    }

    pub(crate) fn browser_button(
        &self,
        host: &dyn Host,
        button: i32,
        modifiers: u32,
        up: bool,
    ) -> Flow {
        let own = host.mod_screen() == Some(MOD_ID);
        if !self.browser_accepts_input(host) {
            return Flow::Continue;
        }
        let Some(browser) = &self.browser else {
            return Flow::Continue;
        };
        let Some(client) = host.client_state() else {
            return Flow::Continue;
        };
        let mut browser = browser.lock().unwrap_or_else(|e| e.into_inner());
        let button = match button {
            0 => Button::Left,
            1 => Button::Right,
            2 => Button::Middle,
            _ => return Flow::Cancel,
        };
        if own && !up && self.click_gui_bound(liquidbounce_core::InputKey::Mouse(match button{Button::Left=>1,Button::Middle=>2,Button::Right=>3}),modifiers){
            self.state().behaviors.borrow_mut().click_gui.close_requested=true;
            return Flow::Cancel;
        }
        // Screen.Pre cancellation prevents Horizon's generic Mouse.Post hook.
        // Still deliver the real release to HOLD/SMART binds, as the upstream
        // module manager does while a screen is open. Its effects run at the
        // next mutable client callback.
        if own && up {
            let options = runtime::raw_options(host);
            let mut state = self.state();
            let sdl_button = match button {
                Button::Left => 1,
                Button::Middle => 2,
                Button::Right => 3,
            };
            state.observed.mouse_events = state.observed.mouse_events.saturating_add(1);
            let now_ms = self.started.elapsed().as_millis().min(u64::MAX as u128) as u64;
            if let Err(error) = state.with_registry(options, |registry, callbacks| {
                registry.handle_mouse_sync(
                    liquidbounce_core::MouseInput::new(
                        sdl_button,
                        liquidbounce_core::KeyAction::Release,
                        crate::input::modifiers(modifiers),
                    ),
                    liquidbounce_core::InputContext {
                        screen_open: true,
                        debug_modifier_down: host.client_debug_modifier_down(),
                    },
                    now_ms,
                    callbacks,
                )
            }) {
                eprintln!("[LiquidBounce] mouse release callback: {error:?}");
            }
        }
        let mask = match button {
            Button::Left => 16,
            Button::Middle => 32,
            Button::Right => 64,
        };
        if up {
            browser.buttons &= !mask;
        } else {
            browser.buttons |= mask;
        }
        let (x, y) = position(&browser, &client, host.cursor());
        if !up {
            browser.last_click_count = if browser
                .last_click
                .is_some_and(|last| last.elapsed() < Duration::from_millis(500))
            {
                2
            } else {
                1
            };
            browser.last_click = Some(Instant::now());
        }
        let clicks = browser.last_click_count;
        push(
            &browser,
            Input::MouseButton {
                x,
                y,
                modifiers: flags(modifiers) | browser.buttons,
                button,
                up,
                clicks,
            },
        );
        let name = match button {
            Button::Left => "key.mouse.left",
            Button::Middle => "key.mouse.middle",
            Button::Right => "key.mouse.right",
        };
        browser.server.broadcast("mouseButton",&json!({"key":name,"button":match button{Button::Left=>1,Button::Middle=>2,Button::Right=>3},"action":if up{0}else{1},"mods":modifiers,"screen":browser.screen_descriptor()}));
        Flow::Cancel
    }
    pub(crate) fn browser_wheel(&self, host: &dyn Host, scroll: [f64; 2]) -> Flow {
        if !self.browser_accepts_input(host) {
            return Flow::Continue;
        }
        let Some(browser) = &self.browser else {
            return Flow::Continue;
        };
        let browser = browser.lock().unwrap_or_else(|e| e.into_inner());
        let Some(client) = host.client_state() else {
            return Flow::Continue;
        };
        let (x, y) = position(&browser, &client, host.cursor());
        // Pinned addon: away-from-zero wheel rounding *3, JCEF amount32 =>96.
        push(
            &browser,
            Input::Wheel {
                x,
                y,
                modifiers: 0,
                delta_x: 0,
                delta_y: (if scroll[1] < 0.0 {
                    scroll[1].floor()
                } else {
                    scroll[1].ceil()
                }) as i32
                    * 96,
            },
        );
        Flow::Cancel
    }
    pub(crate) fn browser_char(&self, host: &dyn Host, c: char, modifiers: u32) -> Flow {
        if !self.browser_accepts_input(host) {
            return Flow::Continue;
        }
        let Some(browser) = &self.browser else {
            return Flow::Continue;
        };
        let browser = browser.lock().unwrap_or_else(|e| e.into_inner());
        push(
            &browser,
            liquidbounce_webview::input::text(&c.to_string(), flags(modifiers)),
        );
        Flow::Cancel
    }
    pub(crate) fn browser_screen_key(
        &self,
        host: &dyn Host,
        key: &str,
        repeat: bool,
        modifiers: u32,
        up: bool,
    ) -> Flow {
        if !self.browser_accepts_input(host) {
            return Flow::Continue;
        }
        let Some(browser) = &self.browser else {
            return Flow::Continue;
        };
        if !up && key == "key.keyboard.escape"
            && browser.lock().unwrap_or_else(|e| e.into_inner()).menu_escape() {
            return Flow::Cancel;
        }
        if !up {
            if let Some(code) = horizon_client::keybinds::key_code(key) {
                self.browser_raw_key(host, code, ElementState::Pressed, repeat, modifiers);
            }
        }
        let browser = browser.lock().unwrap_or_else(|e| e.into_inner());
        if key == "key.keyboard.escape" && !browser.ui.page_keeps_escape {
            Flow::Continue
        } else {
            Flow::Cancel
        }
    }
}

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn MapVirtualKeyW(code: u32, kind: u32) -> u32;
}
