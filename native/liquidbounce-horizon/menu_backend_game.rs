// SPDX-License-Identifier: GPL-3.0-or-later
//! Menu data and native actions. Every request runs on the actual game thread.
use super::{Game, Screen};
use horizon_client::{
    multiplayer,
    title::{self, Action, Mode},
};
use horizon_protocol::servers::{self, ServerEntry};
use serde_json::{Value, json};
use std::{path::PathBuf, time::UNIX_EPOCH};

#[derive(Default)]
pub(super) struct MenuState {
    servers: Option<multiplayer::Screen>,
}

fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value[name]
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| format!("Missing {name}"))
}

fn base64(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for bytes in bytes.chunks(3) {
        let a = bytes[0];
        let b = bytes.get(1).copied().unwrap_or(0);
        let c = bytes.get(2).copied().unwrap_or(0);
        text.push(DIGITS[(a >> 2) as usize] as char);
        text.push(DIGITS[(((a & 3) << 4) | (b >> 4)) as usize] as char);
        text.push(if bytes.len() > 1 {
            DIGITS[(((b & 15) << 2) | (c >> 6)) as usize] as char
        } else {
            '='
        });
        text.push(if bytes.len() > 2 {
            DIGITS[(c & 63) as usize] as char
        } else {
            '='
        });
    }
    text
}

impl Game<'_> {
    pub(super) fn liquidbounce_menu_screen(&self) -> Option<String> {
        if !self.native_window()
            || self.pending_save.is_some()
            || self.pending_entry.is_some()
            || self.screen != Screen::Playing
        {
            return None;
        }
        self.title
            .as_ref()
            .filter(|title| !title.preparing())
            .map(|title| title.mode.id().to_owned())
    }

    fn liquidbounce_menu_saves(&self) -> Result<PathBuf, String> {
        if !self.native_window() {
            return Err("Menu requests need the client window".into());
        }
        std::env::current_dir()
            .map(|directory| directory.join("saves"))
            .map_err(|error| error.to_string())
    }

    pub(super) fn liquidbounce_menu_request(
        &mut self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, String> {
        let saves = self.liquidbounce_menu_saves()?;
        if self.pending_save.is_some() && action != "exit" {
            return Err("A world save is in progress".into());
        }
        if self.pending_entry.is_some()
            && matches!(
                action,
                "screen" | "world_join" | "world_edit" | "world_delete" | "server_connect"
            )
        {
            return Err("A world is already opening".into());
        }
        match action {
            "session" | "accounts" => {
                let authenticated = horizon_client::auth::current();
                let name = authenticated
                    .as_ref()
                    .map(|session| session.name.clone())
                    .unwrap_or_else(|| self.options.player_name());
                let uuid = authenticated
                    .as_ref()
                    .map(|session| session.uuid)
                    .unwrap_or_else(|| horizon_protocol::crypto::offline_uuid(&name));
                let online = authenticated.is_some();
                let kind = if online { "Microsoft" } else { "Cracked" };
                let avatar = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='64' height='64' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' fill='%23202b35'/%3E%3Crect x='12' y='10' width='40' height='44' rx='5' fill='%236fbed1'/%3E%3Cpath d='M21 27h7v7h-7zm15 0h7v7h-7zM24 42h16v5H24z' fill='%23202b35'/%3E%3C/svg%3E";
                let session = json!({"username":name,"uuid":horizon_protocol::crypto::uuid_string(uuid),
                    "type":kind,"service":if online {"Microsoft"} else {"Offline"},"avatar":avatar,"online":online});
                if action == "session" {
                    Ok(session)
                } else {
                    Ok(
                        json!([{"id":0,"username":session["username"],"uuid":session["uuid"],
                        "type":kind,"avatar":avatar,"favorite":false}]),
                    )
                }
            }
            "screen" => {
                let name = string(payload, "name")?;
                if name == "title" && self.title.is_none() {
                    self.close_world().map_err(|error| error.to_string())?;
                    return Ok(Value::Null);
                }
                if self.title.is_none() {
                    return Err("This menu needs the title screen".into());
                }
                match name {
                    "options" => self.open_options(true),
                    "language" => {
                        self.open_options_at(true, horizon_client::options_ui::Page::Language)
                    }
                    "accessibility" => {
                        self.open_options_at(true, horizon_client::options_ui::Page::Accessibility)
                    }
                    "credits" => {
                        self.open_options_at(true, horizon_client::options_ui::Page::Credits)
                    }
                    _ => {
                        self.title
                            .as_mut()
                            .unwrap()
                            .liquidbounce_menu_screen(name)?;
                        self.screen = Screen::Playing;
                    }
                }
                self.keys.clear();
                self.capture(false);
                Ok(Value::Null)
            }
            "close_screen" => {
                if let Some(title) = &mut self.title {
                    title.escape();
                } else {
                    return Err("There is no title menu to close".into());
                }
                self.screen = Screen::Playing;
                Ok(Value::Null)
            }
            "exit" => {
                if self.options.split_guest {
                    return Err("Only the main window can exit the game".into());
                }
                self.begin_closing_save(true)
                    .map_err(|error| error.to_string())?;
                Ok(Value::Null)
            }
            "worlds" => {
                let worlds = title::list_worlds(&saves).into_iter().enumerate().map(|(id,world)| {
                    let path = saves.join(&world.folder);
                    let difficulty = horizon_client::save::Level::read_summary(&path).map(|level| level.difficulty)
                        .unwrap_or_else(|| "normal".to_owned());
                    let icon_path = path.join("icon.png");
                    let icon = std::fs::metadata(&icon_path).ok().filter(|metadata| metadata.len() <= 1024*1024)
                        .and_then(|_|std::fs::read(icon_path).ok()).map(|bytes|base64(&bytes));
                    json!({"id":id,"name":world.folder,"displayName":world.name,
                        "lastPlayed":world.modified.duration_since(UNIX_EPOCH).map_or(0,|duration|duration.as_millis() as u64),
                        "gameMode":match world.game_mode {1=>"creative",2=>"adventure",3=>"spectator",_=>"survival"},
                        "difficulty":difficulty.to_lowercase(),"icon":icon,"hardcore":world.hardcore,
                        "commandsAllowed":world.commands,"version":world.version,
                        "locked":world.locked,"corrupted":world.corrupted})
                }).collect::<Vec<_>>();
                Ok(json!(worlds))
            }
            "world_join" | "world_edit" | "world_delete" => {
                let folder = string(payload, "name")?;
                let action = self
                    .title
                    .as_mut()
                    .ok_or("World actions need the title screen")?
                    .liquidbounce_world_action(folder, action)?;
                if let Action::Play { folder } = action {
                    self.open_world(&folder, None, None, false, None)
                        .map_err(|error| error.to_string())?;
                }
                Ok(Value::Null)
            }
            "servers" => {
                self.liquidbounce_menu.servers = Some(multiplayer::Screen::list(&saves));
                Ok(self
                    .liquidbounce_menu
                    .servers
                    .as_ref()
                    .unwrap()
                    .liquidbounce_snapshot(&saves, false))
            }
            "server_updates" | "lan_servers" => {
                if self.liquidbounce_menu.servers.is_none() && action == "lan_servers" {
                    self.liquidbounce_menu.servers = Some(multiplayer::Screen::list(&saves));
                }
                Ok(self
                    .liquidbounce_menu
                    .servers
                    .as_ref()
                    .map(|screen| screen.liquidbounce_snapshot(&saves, action == "lan_servers"))
                    .unwrap_or_else(|| json!([])))
            }
            "server_connect" => {
                if self
                    .title
                    .as_ref()
                    .is_none_or(|title| !title.allows_multiplayer)
                {
                    return Err("Joining a server requires the title screen and an account that allows multiplayer".into());
                }
                let address = string(payload, "address")?.trim();
                if servers::parse_address(address).is_none() {
                    return Err("Invalid server address".into());
                }
                multiplayer::remember_direct_join(&saves, address);
                let lan = self
                    .liquidbounce_menu
                    .servers
                    .as_ref()
                    .is_some_and(|screen| {
                        screen
                            .liquidbounce_snapshot(&saves, true)
                            .as_array()
                            .is_some_and(|rows| {
                                rows.iter()
                                    .any(|row| row["address"].as_str() == Some(address))
                            })
                    });
                self.connect_server_with_options(
                    address,
                    &self.options.player_name(),
                    None,
                    lan,
                    false,
                )
                .map_err(|error| error.to_string())?;
                Ok(Value::Null)
            }
            "server_add" | "server_edit" | "server_remove" | "server_order" => {
                let file = multiplayer::servers_file(&saves);
                {
                    let _guard = servers::lock();
                    let (mut shown, hidden) = servers::load_all(&file);
                    let id = || {
                        payload["id"]
                            .as_u64()
                            .and_then(|id| usize::try_from(id).ok())
                            .filter(|id| *id < shown.len())
                            .ok_or_else(|| "Invalid server id".to_owned())
                    };
                    match action {
                        "server_remove" => {
                            let index = id()?;
                            shown.remove(index);
                        }
                        "server_order" => {
                            let order =
                                payload["order"].as_array().ok_or("Missing server order")?;
                            let mut indices = Vec::new();
                            for index in order {
                                let index = index
                                    .as_u64()
                                    .and_then(|id| usize::try_from(id).ok())
                                    .filter(|id| *id < shown.len())
                                    .ok_or("Invalid server order")?;
                                if indices.contains(&index) {
                                    return Err("Server order contains duplicates".into());
                                }
                                indices.push(index);
                            }
                            if indices.len() != shown.len() {
                                return Err("Server order must include every server".into());
                            }
                            shown = indices
                                .into_iter()
                                .map(|index| shown[index].clone())
                                .collect();
                        }
                        _ => {
                            let name = string(payload, "name")?.trim();
                            let address = string(payload, "address")?.trim();
                            if servers::parse_address(address).is_none() {
                                return Err("Invalid server address".into());
                            }
                            let policy =
                                match payload["resourcePackPolicy"].as_str().unwrap_or("Prompt") {
                                    "Enabled" | "ENABLED" => Some(true),
                                    "Disabled" | "DISABLED" => Some(false),
                                    "Prompt" | "PROMPT" | "" => None,
                                    _ => return Err("Invalid resource pack policy".into()),
                                };
                            if action == "server_add" {
                                let mut entry = ServerEntry::new(name, address);
                                entry.accept_textures = policy;
                                shown.push(entry);
                            } else {
                                let index = id()?;
                                let entry = &mut shown[index];
                                if entry.ip != address {
                                    entry.icon = None;
                                    entry.accepted_code_of_conduct = 0;
                                }
                                entry.name = name.to_owned();
                                entry.ip = address.to_owned();
                                entry.accept_textures = policy;
                            }
                        }
                    }
                    servers::save_all(&file, &shown, &hidden).map_err(|error| error.to_string())?;
                }
                self.liquidbounce_menu.servers = Some(multiplayer::Screen::list(&saves));
                if let Some(title) = self.title.as_mut() {
                    if matches!(
                        &title.mode,
                        Mode::Multiplayer(multiplayer::Screen::List { .. })
                    ) {
                        title.mode = Mode::Multiplayer(multiplayer::Screen::list(&saves));
                    }
                }
                Ok(Value::Null)
            }
            "protocols" => Ok(
                json!([{"name":horizon_core::VANILLA_VERSION,"version":horizon_protocol::PROTOCOL_VERSION}]),
            ),
            "protocol" => Ok(
                json!({"name":horizon_core::VANILLA_VERSION,"version":horizon_protocol::PROTOCOL_VERSION}),
            ),
            "set_protocol"
                if payload["version"].as_i64()
                    == Some(i64::from(horizon_protocol::PROTOCOL_VERSION)) =>
            {
                Ok(Value::Null)
            }
            "set_protocol" => Err("This Horizon build only supports its compiled protocol".into()),
            _ => Err(format!("Unknown menu action: {action}")),
        }
    }
}
