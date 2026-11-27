// SPDX-License-Identifier: GPL-3.0-or-later
//! Decode actual client entities before taking the actor lock.
use liquidbounce_behaviors::targeting::{EntityKind, TargetEntity};
use serde_json::Value;

pub(crate) struct Snapshot {
    pub entity: TargetEntity,
    pub player_name: Option<String>,
    pub abstract_client_player: bool,
    pub murder_mystery_uuid:Option<String>,
    pub anti_bot:Option<crate::anti_bot_runtime::Query>,
    pub teams: Option<liquidbounce_behaviors::teams::Facts>,
}

pub(crate) fn snapshot(json: &str) -> Option<Snapshot> {
    let value: Value = serde_json::from_str(json).ok()?;
    Some(Snapshot {
        entity: TargetEntity {
            kind: EntityKind::for_type(value["kind"].as_str()?),
            alive: value["alive"].as_bool()?,
            invisible: value["invisible"].as_bool()?,
            sleeping: value["sleeping"].as_bool()?,
            is_local: value["is_local"].as_bool()?,
            has_local_passenger: value["has_local_passenger"].as_bool()?,
            within_border: value["within_border"].as_bool()?,
        },
        player_name: value["player_name"].as_str().map(str::to_owned),
        abstract_client_player: value["abstract_client_player"].as_bool().unwrap_or(false),
        murder_mystery_uuid:value["murderMysteryUuid"].as_str().map(str::to_owned),
        anti_bot:crate::anti_bot_runtime::query(&value["antiBot"]),
        teams: crate::teams_runtime::facts(&value["teams"]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_malformed_host_entities_cannot_become_targets() {
        for json in [
            "null",
            "{}",
            "invalid",
            r#"{"kind":"minecraft:player","alive":true}"#,
        ] {
            assert!(snapshot(json).is_none());
        }
        let entity = snapshot(r#"{"kind":"minecraft:player","alive":true,"invisible":false,"sleeping":false,"is_local":false,"has_local_passenger":false,"within_border":true,"player_name":"PortTest"}"#).unwrap();
        assert!(entity.entity.kind.living);
        assert_eq!(entity.player_name.as_deref(), Some("PortTest"));
    }
}
