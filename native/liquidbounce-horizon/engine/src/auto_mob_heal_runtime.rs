// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::auto_mob_heal::{self as model, Entity, Kind, Slot};
use serde_json::{Value, json};
pub const REQUESTER: u64 = 0x4155_544f_4d48_4541;
#[derive(Default)]
pub struct AutoMobHealRuntime {
    pub settings: model::Settings,
    remaining: Option<i32>,
    callback: u64,
}
impl AutoMobHealRuntime {
    pub fn clear(&mut self) {
        self.remaining = None;
        self.callback = self.callback.wrapping_add(1);
    }
}
fn query(host: &mut dyn Host, value: Value) -> Value {
    serde_json::from_str(&host.client_auto_mob_heal_query(&value.to_string())).unwrap_or(Value::Null)
}
fn entity(v: &Value) -> Option<Entity> {
    Some(Entity {
        id: i32::try_from(v["id"].as_i64()?).ok()?,
        kind: Kind::from_tag(v["kind"].as_str()?)?,
        alive: v["alive"] == true,
        visible: v["visible"] == true,
        health: v["health"].as_f64()? as f32,
        max_health: v["maxHealth"].as_f64()? as f32,
        distance_squared: v["distanceSquared"].as_f64()?,
        baby: v["baby"] == true,
        tame: v["tame"] == true,
        owned: v["owned"] == true,
        sitting: v["sitting"] == true,
        wearing_body_armor: v["wearingBodyArmor"] == true,
        armor_empty: v["armorEmpty"] == true,
        armor_max_damage: v["armorMaxDamage"].as_i64()? as i32,
        armor_damage: v["armorDamage"].as_i64()? as i32,
    })
}
fn slot(v: &Value) -> Option<Slot> {
    Some(Slot {
        index: u8::try_from(v["index"].as_u64()?).ok()?,
        item: v["item"].as_str()?.into(),
        nutrition: v["nutrition"].as_i64().map(|n| n as i32),
        wolf_food: v["wolfFood"] == true,
        cat_food: v["catFood"] == true,
        repairs_armor: v["repairsArmor"] == true,
    })
}
impl LiquidBounceHooks {
    pub(crate) fn auto_mob_heal_tick(&self, host: &mut dyn Host) {
        let settings = {
            let s = self.state();
            let b = s.behaviors.borrow();
            if !b.query_running(&s.registry, "AutoMobHeal") || b.auto_mob_heal.remaining.is_some() {
                return;
            }
            b.auto_mob_heal.settings.clone()
        };
        // Host calls may dispatch other modules; retain no Behaviors borrow.
        let facts = query(host, json!({"op":"facts"}));
        if facts["usingItem"] == true || facts["screenOpen"] != false {
            return;
        }
        let rows = facts["entities"].as_array().cloned().unwrap_or_default();
        let entities = rows.iter().filter_map(entity).collect::<Vec<_>>();
        let Some(plan) = model::plan(
            &settings,
            &entities,
            facts["selected"].as_u64().unwrap_or(0) as u8,
            facts["entityRange"].as_f64().unwrap_or(0.),
            facts["secondaryUse"] == true,
            |e| rows.iter().find(|r| r["id"].as_i64() == Some(i64::from(e.id)))
                .into_iter().flat_map(|r| r["slots"].as_array().into_iter().flatten())
                .filter_map(slot).collect(),
        ) else { return; };
        if plan.slot != 40 {
            let _ = host.client_select_slot_silently(REQUESTER, i32::from(plan.slot), settings.slot_reset_delay);
        }
        if query(host, json!({"op":"interact","id":plan.id,"offhand":plan.slot==40,"swingMode":settings.swing_mode})) != true {
            return;
        }
        let delay = liquidbounce_behaviors::random::random_i32_inclusive(settings.delay[0], settings.delay[1]);
        if delay == 0 { return; }
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let r = &mut b.auto_mob_heal;
        r.remaining = Some(delay);
        r.callback = r.callback.wrapping_add(1);
        let callback = r.callback;
        b.coroutine_ticker.register(WaitOwner::AutoMobHeal, callback);
    }
    pub(crate) fn auto_mob_heal_callback(&self, in_game: bool, callback: u64) -> bool {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        if !in_game || !b.query_running(&s.registry, "AutoMobHeal") || b.auto_mob_heal.callback != callback {
            return false;
        }
        let r = &mut b.auto_mob_heal;
        let Some(ticks) = r.remaining else { return false; };
        let ticks = ticks.wrapping_sub(1);
        r.remaining = (ticks > 0).then_some(ticks);
        r.remaining.is_some()
    }
}
