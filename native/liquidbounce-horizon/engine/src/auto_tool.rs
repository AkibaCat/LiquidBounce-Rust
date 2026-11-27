// SPDX-License-Identifier: GPL-3.0-or-later
//! Actor-side decoding and suspended inventory work for ModuleAutoTool.
//! Native menu operations execute after releasing the module registry lock.
use liquidbounce_behaviors::auto_tool::{
    MiningContext, RequirementState, ToolStack,
    inventory::{Context, StackRecord},
};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub block: String,
    pub tick: i32,
    pub creative: bool,
    pub sneaking: bool,
    pub server_slot: u8,
    pub beds: Vec<f64>,
    pub stacks: Vec<ToolStack>,
    pub records: Vec<StackRecord>,
    pub context: Context,
}
fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("AutoTool snapshot missing {key}"))
}
fn integer(value: &Value, key: &str) -> Result<i32, String> {
    field(value, key)?
        .as_i64()
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| format!("AutoTool snapshot invalid {key}"))
}
fn number(value: &Value, key: &str) -> Result<f32, String> {
    field(value, key)?
        .as_f64()
        .map(|v| v as f32)
        .ok_or_else(|| format!("AutoTool snapshot invalid {key}"))
}
fn boolean(value: &Value, key: &str) -> Result<bool, String> {
    field(value, key)?
        .as_bool()
        .ok_or_else(|| format!("AutoTool snapshot invalid {key}"))
}
fn string(value: &Value, key: &str) -> Result<String, String> {
    field(value, key)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("AutoTool snapshot invalid {key}"))
}
fn pair(value: &Value) -> Result<[f32; 2], String> {
    let array = value
        .as_array()
        .filter(|v| v.len() == 2)
        .ok_or("AutoTool snapshot invalid pair")?;
    Ok([
        array[0].as_f64().ok_or("AutoTool snapshot invalid pair")? as f32,
        array[1].as_f64().ok_or("AutoTool snapshot invalid pair")? as f32,
    ])
}
fn record(value: &Value) -> Result<StackRecord, String> {
    Ok(StackRecord {
        identity: string(value, "identity")?,
        components: string(value, "components")?,
        count: integer(value, "count")?,
    })
}
impl Snapshot {
    pub fn parse(payload: &str) -> Result<Option<Self>, String> {
        let value: Value = serde_json::from_str(payload).map_err(|error| error.to_string())?;
        if value.is_null() {
            return Ok(None);
        }
        let mut stacks = Vec::new();
        let mut records = Vec::new();
        for stack in field(&value, "stacks")?
            .as_array()
            .ok_or("AutoTool snapshot invalid stacks")?
        {
            let slot = u8::try_from(integer(stack, "slot")?)
                .ok()
                .filter(|s| *s < 36)
                .ok_or("AutoTool snapshot invalid slot")?;
            stacks.push(ToolStack {
                slot,
                destroy_speed: number(stack, "destroySpeed")?,
                efficiency: integer(stack, "efficiency")?,
                silk_touch: integer(stack, "silkTouch")?,
                max_damage: integer(stack, "maxDamage")?,
                damage: integer(stack, "damage")?,
                empty: boolean(stack, "empty")?,
                identity: string(stack, "identity")?,
            });
            records.push(record(stack)?);
        }
        if stacks.len() != 36
            || stacks
                .iter()
                .enumerate()
                .any(|(i, s)| i != usize::from(s.slot))
        {
            return Err("AutoTool snapshot must contain inventory slots 0..35 in order".to_owned());
        }
        let beds = field(&value, "beds")?
            .as_array()
            .ok_or("AutoTool snapshot invalid beds")?
            .iter()
            .map(|v| {
                v.as_f64()
                    .ok_or_else(|| "AutoTool snapshot invalid bed distance".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let server_slot = u8::try_from(integer(&value, "serverSlot")?)
            .ok()
            .filter(|s| *s < 9)
            .ok_or("AutoTool snapshot invalid serverSlot")?;
        let cursor = field(&value, "visibleCursor")?;
        Ok(Some(Self {
            block: string(&value, "block")?,
            tick: integer(&value, "tick")?,
            creative: boolean(&value, "creative")?,
            sneaking: boolean(&value, "sneaking")?,
            server_slot,
            beds,
            stacks,
            records,
            context: Context {
                now: field(&value, "now")?
                    .as_i64()
                    .ok_or("AutoTool snapshot invalid now")?,
                player_inventory: boolean(&value, "playerInventory")?,
                server_controlled: boolean(&value, "serverControlled")?,
                cursor_empty: boolean(&value, "cursorEmpty")?,
                inventory_screen: boolean(&value, "inventoryScreen")?,
                inventory_open: boolean(&value, "inventoryOpen")?,
                requirements: RequirementState {
                    movement: pair(field(&value, "movement")?)?,
                    jumping: boolean(&value, "jumping")?,
                    rotation_matches: boolean(&value, "rotationMatches")?,
                    using_item: boolean(&value, "using")?,
                    breaking: boolean(&value, "breaking")?,
                    in_combat: false,
                },
                menu_identity: string(&value, "menuIdentity")?,
                visible_cursor: if cursor.is_null() {
                    None
                } else {
                    Some(pair(cursor)?)
                },
            },
        }))
    }
    pub fn mining_context(&self, position: [i32; 3], in_combat: bool) -> MiningContext<'_> {
        MiningContext {
            position,
            block: &self.block,
            tick: self.tick,
            creative: self.creative,
            sneaking: self.sneaking,
            in_combat,
            bed_head_distances_squared: &self.beds,
            serverside_slot: self.server_slot,
        }
    }
}
