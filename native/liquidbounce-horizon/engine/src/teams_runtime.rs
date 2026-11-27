// SPDX-License-Identifier: GPL-3.0-or-later
use liquidbounce_behaviors::{
    targeting::TargetInfo,
    teams::{EntityFacts, Facts, Teams},
};
use serde_json::Value;
pub(crate) fn facts(v: &Value) -> Option<Facts> {
    fn entity(v: &Value) -> Option<EntityFacts> {
        let armor = v["armor"].as_array()?;
        Some(EntityFacts {
            team: v["team"].as_str().map(str::to_owned),
            team_color: v["teamColor"].as_u64().map(|v| v as u32),
            name: v["name"].as_str()?.to_owned(),
            name_color: v["nameColor"].as_u64().map(|v| v as u32),
            armor: std::array::from_fn(|i| armor.get(i).and_then(Value::as_u64).map(|v| v as u32)),
            player: v["player"].as_bool()?,
        })
    }
    Some(Facts {
        own: entity(&v["own"])?,
        other: entity(&v["other"])?,
    })
}
pub(crate) fn apply_tag(
    module: &Teams,
    running: bool,
    living: bool,
    facts: Option<&Facts>,
    info: &mut TargetInfo,
) {
    let Some(facts) = facts else {
        return;
    };
    let decision = module.decide(running, living, facts);
    if decision.teammate {
        info.dont_target();
    }
    if let Some(color) = decision.color {
        info.color(color, 20);
    }
}
