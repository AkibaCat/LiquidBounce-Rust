// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{scoreboard::comp_text, tab_list::TabPlayer, text::Text};
use std::{cell::RefCell, cmp::Ordering, collections::HashMap};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    hidden: bool,
    color: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    sorting: u8,
    visibility: u8,
    show_game_mode: bool,
    limit: i32,
    height: i32,
    accurate: bool,
    suffix: bool,
    rows: HashMap<String, Row>,
    error: bool,
}
thread_local! {static POLICY:RefCell<Option<Policy>>=const{RefCell::new(None)};}
pub struct Scope(Option<Policy>);
impl Drop for Scope {
    fn drop(&mut self) {
        POLICY.with(|p| *p.borrow_mut() = self.0.take());
    }
}
pub fn scope(json: &str) -> Scope {
    let policy = serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .filter(|v| v.is_object())
        .map(|v| Policy {
            sorting: v["sorting"].as_u64().unwrap_or(0) as u8,
            visibility: v["visibility"].as_u64().unwrap_or(3) as u8,
            show_game_mode: v["showGameMode"].as_bool().unwrap_or(true),
            limit: v["limit"].as_i64().unwrap_or(80) as i32,
            height: v["height"].as_i64().unwrap_or(20) as i32,
            accurate: v["accurate"].as_bool().unwrap_or(true),
            suffix: v["suffix"].as_bool().unwrap_or(true),
            rows: v["rows"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(k, r)| {
                    (
                        k.clone(),
                        Row {
                            hidden: r["hidden"].as_bool().unwrap_or(false),
                            color: r["color"].as_i64().map(|v| v as u32),
                        },
                    )
                })
                .collect(),
            error: !v["error"].is_null(),
        });
    Scope(POLICY.with(|p| p.replace(policy)))
}
pub fn current() -> Option<Policy> {
    POLICY.with(|p| p.borrow().clone())
}
pub fn ordered<'a>(raw: &'a [TabPlayer], vanilla: Vec<&'a TabPlayer>) -> Vec<&'a TabPlayer> {
    let Some(p) = current() else {
        return vanilla.into_iter().take(80).collect();
    };
    if p.error || p.limit < 0 {
        return vec![];
    }
    let mut rows = if p.sorting == 0 {
        vanilla
    } else {
        raw.iter().collect()
    };
    rows.retain(|r| !p.rows.get(&r.uuid.to_string()).is_some_and(|r| r.hidden));
    if p.sorting != 0 {
        rows.sort_by(|a, b| match p.sorting {
            1 => a.latency.cmp(&b.latency),
            2 => a
                .name
                .encode_utf16()
                .count()
                .cmp(&b.name.encode_utf16().count()),
            3 => {
                let len = |p: &TabPlayer| {
                    p.display_name
                        .as_ref()
                        .map_or(0, |d| comp_text(d).plain().encode_utf16().count())
                };
                len(a).cmp(&len(b))
            }
            4 => a.name.encode_utf16().cmp(b.name.encode_utf16()),
            5 => b.name.encode_utf16().cmp(a.name.encode_utf16()),
            _ => Ordering::Equal,
        });
    }
    rows.truncate(p.limit as usize);
    rows
}
pub fn name(player: &TabPlayer, mut original: Text) -> Text {
    let Some(p) = current() else {
        return original;
    };
    if p.visibility & 4 != 0 {
        return Text::literal(&player.name);
    }
    if p.show_game_mode {
        original.push(Text::literal(format!(
            " [{}]",
            match player.game_mode {
                1 => "C",
                2 => "A",
                _ => "S",
            }
        )));
    }
    original
}
pub fn visible(bit: u8) -> bool {
    current().is_none_or(|p| p.visibility & bit != 0)
}
pub fn dimensions(slots: i32, vanilla: (i32, i32)) -> (i32, i32) {
    let Some(p) = current() else {
        return vanilla;
    };
    let cols = ((f64::from(slots) / f64::from(p.height.max(1))).ceil() as i32).max(1);
    (cols, (f64::from(slots) / f64::from(cols)).ceil() as i32)
}
pub fn width_extra() -> i32 {
    if current().is_some_and(|p| p.accurate) {
        30
    } else {
        0
    }
}
pub fn color(id: u128, vanilla: u32) -> u32 {
    current()
        .and_then(|p| p.rows.get(&id.to_string()).and_then(|r| r.color))
        .unwrap_or(vanilla)
}
pub fn latency(v: i32) -> Option<(Text, u32)> {
    let p = current().filter(|p| p.accurate)?;
    let v = v.clamp(0, 9999);
    Some((
        Text::literal(format!("{v}{}", if p.suffix { "ms" } else { "" })),
        if v < 150 {
            0xff00e970
        } else if v < 300 {
            0xffe7d020
        } else {
            0xffd74238
        },
    ))
}
