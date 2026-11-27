// SPDX-License-Identifier: GPL-3.0-or-later
//! Draw protocol shared by the actual three rendering hooks. Floats travel as
//! bits so arbitrary source configuration values never disappear through JSON.
use liquidbounce_behaviors::better_inventory::{BetterInventory, DrawError, Highlight};
use serde_json::{json, Value};

pub(crate) fn render(
    state: &BetterInventory,
    running: bool,
    last_clicked: i32,
    query: &str,
) -> Result<Value, String> {
    let value: Value = serde_json::from_str(query).map_err(|error| error.to_string())?;
    let int = |name: &str| -> Result<i32, String> {
        value[name]
            .as_i64()
            .and_then(|n| i32::try_from(n).ok())
            .ok_or_else(|| format!("missing i32 {name}"))
    };
    let boolean = |name: &str| -> Result<bool, String> {
        value[name]
            .as_bool()
            .ok_or_else(|| format!("missing bool {name}"))
    };
    match value["kind"].as_str() {
        Some("slot") => Ok(
            match state.highlight(running, last_clicked, int("index")?, int("x")?, int("y")?) {
                None => Value::Null,
                Some(Highlight::Border { rects, color }) => {
                    json!({"kind":"border","rects":rects,"color":color})
                }
                Some(Highlight::Texture { rect }) => json!({"kind":"texture","rect":rect}),
            },
        ),
        Some("cooldown") => {
            let entry = if value["end"].is_null() {
                None
            } else {
                Some((int("end")?, int("tick")?))
            };
            match state.cooldown_with_locale(
                running,
                boolean("empty")?,
                boolean("player")?,
                f32::from_bits(int("progress")? as u32),
                entry,
                int("x")?,
                int("y")?,
                value["locale"].as_str().unwrap_or(""),
            ) {
                Ok(None) => Ok(Value::Null),
                Ok(Some(draw)) => Ok(
                    json!({"kind":"text","text":draw.text,"centerX":draw.center_x,"y":draw.y,"scale":draw.scale.to_bits() as i32,"color":draw.color}),
                ),
                Err(DrawError::MissingCooldownEntry) => {
                    Err("BetterInventory cooldown entry is absent".into())
                }
                Err(error) => Err(format!("{error:?}")),
            }
        }
        Some("container") => {
            if !BetterInventory::screen_preview_eligible(
                boolean("carriedEmpty")?,
                boolean("hovered")?,
            ) {
                return Ok(Value::Null);
            }
            let stacks = value["stacks"]
                .as_array()
                .map(|stacks| {
                    stacks
                        .iter()
                        .map(|value| {
                            value
                                .as_bool()
                                .ok_or_else(|| "non-boolean container stack".to_owned())
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            match state.container(
                running,
                boolean("empty")?,
                stacks.as_deref(),
                int("left")?,
                int("top")?,
                int("mouseX")?,
                int("mouseY")?,
            ) {
                Ok(None) => Ok(Value::Null),
                Ok(Some(draw)) => Ok(
                    json!({"kind":"container","indices":draw.indices,"center":draw.center.map(|v|v.to_bits() as i32),"scale":draw.scale.to_bits() as i32,"dimensions":draw.dimensions}),
                ),
                Err(error) => Err(format!("{error:?}")),
            }
        }
        _ => Err("unknown BetterInventory render point".into()),
    }
}
