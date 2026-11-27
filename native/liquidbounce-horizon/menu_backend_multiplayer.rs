// SPDX-License-Identifier: GPL-3.0-or-later
// Exact-source extension appended to Horizon's multiplayer module.
impl Screen {
    pub fn liquidbounce_snapshot(&self, saves: &Path, lan_only: bool) -> serde_json::Value {
        use serde_json::json;
        let Self::List {
            servers,
            pings,
            lan,
            ..
        } = self
        else {
            return json!([]);
        };
        let protocol = horizon_protocol::PROTOCOL_VERSION;
        if lan_only {
            return json!(
                lan.games()
                    .iter()
                    .enumerate()
                    .map(|(index, game)| json!({
                        "id":index,"name":game.motd,"address":game.address,"icon":null,
                        "label":game.motd,"players":{"online":0,"max":0},"online":true,
                        "playerCountLabel":"","protocolVersion":protocol,"version":"",
                        "ping":0,"resourcePackPolicy":"Prompt","lan":true
                    }))
                    .collect::<Vec<_>>()
            );
        }
        let saved = servers::load(&servers_file(saves));
        let pings = pings
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        json!(servers.iter().enumerate().map(|(index, server)| {
            let icon = saved.iter().find(|entry| entry.ip == server.ip)
                .and_then(|entry| entry.icon.clone()).or_else(|| server.icon.clone());
            let mut value = json!({
                "id":index,"name":server.name,"address":server.ip,"icon":icon,
                "label":crate::lang::get_or("multiplayer.status.pinging", "Pinging..."),
                "players":{"online":0,"max":0},"online":false,"playerCountLabel":"",
                "protocolVersion":protocol,"version":"","ping":-2,
                "resourcePackPolicy":match server.accept_textures { Some(true)=>"Enabled",Some(false)=>"Disabled",None=>"Prompt" },
                "lan":false
            });
            match pings.get(&server.ip) {
                Some(Ping::Ok { motd, players, ms, version, .. }) => {
                    let component = motd.component();
                    value["label"] = if component.is_string() || component.get("text").is_some() {
                        component
                    } else {
                        json!(motd.plain())
                    };
                    value["ping"] = json!((*ms).min(i64::MAX as u128) as i64);
                    value["online"] = json!(true);
                    value["version"] = json!(version);
                    if let Some((online,max)) = players {
                        value["players"] = json!({"online":online,"max":max});
                        value["playerCountLabel"] = json!(format!("{online}/{max}"));
                    }
                }
                Some(Ping::Incompatible { motd, players, version, .. }) => {
                    let component = motd.component();
                    value["label"] = if component.is_string() || component.get("text").is_some() {
                        component
                    } else {
                        json!(motd.plain())
                    };
                    value["ping"] = json!(-1);
                    value["online"] = json!(true);
                    value["version"] = json!(version.plain());
                    value["protocolVersion"] = json!(-1);
                    if let Some((online,max)) = players { value["players"] = json!({"online":online,"max":max}); }
                }
                Some(Ping::Failed(reason)) => {
                    value["label"] = json!(reason);
                    value["ping"] = json!(-1);
                }
                _ => {}
            }
            value
        }).collect::<Vec<_>>())
    }
}
