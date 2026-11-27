// SPDX-License-Identifier: GPL-3.0-or-later
//! Decode the native client's actual component snapshot before registry locks.
use liquidbounce_behaviors::smart_eat::{Context, Food, Slot, Stack};
use serde_json::Value;

fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value, String> {
    v.get(key)
        .ok_or_else(|| format!("SmartEat snapshot missing {key}"))
}
fn int(v: &Value, key: &str) -> Result<i32, String> {
    field(v, key)?
        .as_i64()
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| format!("SmartEat snapshot invalid {key}"))
}
fn boolean(v: &Value, key: &str) -> Result<bool, String> {
    field(v, key)?
        .as_bool()
        .ok_or_else(|| format!("SmartEat snapshot invalid {key}"))
}
pub fn parse_snapshot(payload: &str) -> Result<Option<Context>, String> {
    let value: Value = serde_json::from_str(payload).map_err(|e| e.to_string())?;
    if value.is_null() {
        return Ok(None);
    }
    let server_slot = u8::try_from(int(&value, "serverSlot")?)
        .ok()
        .filter(|s| *s < 9)
        .ok_or("SmartEat server slot outside hotbar")?;
    let raw = field(&value, "stacks")?
        .as_array()
        .ok_or("SmartEat snapshot invalid stacks")?;
    let mut stacks = Vec::new();
    for (index, stack) in raw.iter().enumerate() {
        let slot = int(stack, "slot")?;
        if slot != index as i32 - 1 {
            return Err("SmartEat stacks must be offhand then hotbar 0..8".into());
        }
        let slot = if index == 0 {
            Slot::Offhand
        } else {
            Slot::Hotbar((index - 1) as u8)
        };
        let food = field(stack, "food")?;
        stacks.push(Stack {
            slot,
            item: field(stack, "item")?
                .as_str()
                .ok_or("SmartEat invalid item")?
                .to_owned(),
            instant_health: boolean(stack, "instantHealth")?,
            food: if food.is_null() {
                None
            } else {
                Some(Food {
                    nutrition: int(food, "nutrition")?,
                    can_always_eat: boolean(food, "canAlwaysEat")?,
                })
            },
        });
    }
    if stacks.len() != 10 {
        return Err("SmartEat snapshot must contain all ten slots".into());
    }
    Ok(Some(Context {
        server_slot,
        stacks,
        health: field(&value, "health")?
            .as_f64()
            .ok_or("SmartEat invalid health")? as f32,
        hunger: int(&value, "hunger")?,
        can_eat: boolean(&value, "canEat")?,
        main_mining_tool: boolean(&value, "mainMiningTool")?,
        using_food_or_drink: boolean(&value, "usingFoodOrDrink")?,
    }))
}
