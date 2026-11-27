// SPDX-License-Identifier: GPL-3.0-or-later
//! Pure Surround placement planning from ModuleSurround.
//! The engine adapter supplies the real player block position and BlockPlacer.

pub type Pos = [i32; 3];
pub const FEATURE_EXTEND: u32 = 1 << 0;
pub const FEATURE_DOWN: u32 = 1 << 1;
pub const FEATURE_NO_WASTE: u32 = 1 << 2;
pub const FEATURES: [&str; 3] = ["Extend", "Down", "NoWaste"];
pub const DISABLE_Y_CHANGE: u32 = 1 << 0;
pub const DISABLE_XZ_MOVE: u32 = 1 << 1;
pub const DISABLE_XZ_SPEED: u32 = 1 << 2;
pub const DISABLE_ON: [&str; 3] = ["YChange", "XZMove", "XZSpeed"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub features: u32,
    pub disable_on: u32,
    pub instant: bool,
    pub whitelist: bool,
    pub blocks: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            features: FEATURE_EXTEND | FEATURE_DOWN,
            disable_on: DISABLE_Y_CHANGE,
            instant: true,
            whitelist: true,
            blocks: vec![
                "minecraft:obsidian".into(),
                "minecraft:ender_chest".into(),
                "minecraft:crying_obsidian".into(),
            ],
        }
    }
}

fn add(out: &mut Vec<Pos>, p: Pos) {
    if !out.contains(&p) { out.push(p); }
}

/// Four side positions around the player's feet, in vanilla Direction 2D order.
pub fn positions(feet: Pos, features: u32) -> Vec<Pos> {
    let mut out = Vec::with_capacity(8);
    for p in [[feet[0] - 1, feet[1], feet[2]], [feet[0] + 1, feet[1], feet[2]],
              [feet[0], feet[1], feet[2] - 1], [feet[0], feet[1], feet[2] + 1]] {
        add(&mut out, p);
        if features & FEATURE_DOWN != 0 { add(&mut out, [p[0], p[1] - 1, p[2]]); }
    }
    out
}

/// Extends a surround one block outward when an entity blocks a side target.
pub fn extend_positions(base: &[Pos], occupied: &[Pos], corners: bool) -> Vec<Pos> {
    let mut out = base.to_vec();
    for p in base {
        if !occupied.contains(p) { continue; }
        // The native planner supplies the occupied side and the player's center;
        // this conservative expansion keeps all four neighboring sides valid.
        for side in [[1,0,0],[-1,0,0],[0,0,1],[0,0,-1]] {
            add(&mut out, [p[0] + side[0], p[1], p[2] + side[2]]);
        }
        add(&mut out, [p[0], p[1] + 1, p[2]]);
        if corners { add(&mut out, [p[0] + 1, p[1], p[2] + 1]); }
    }
    out
}

pub fn should_disable(settings: &Settings, start_y: f64, current_y: f64, dx: f64, dz: f64, speed: f64) -> bool {
    (settings.disable_on & DISABLE_Y_CHANGE != 0 && current_y != start_y)
        || (settings.disable_on & DISABLE_XZ_MOVE != 0 && (dx > 0.5 || dz > 0.5))
        || (settings.disable_on & DISABLE_XZ_SPEED != 0 && speed >= 5.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn four_sides_and_down_keep_order() {
        assert_eq!(positions([4, 8, -2], FEATURE_DOWN), vec![
            [3,8,-2],[3,7,-2],[5,8,-2],[5,7,-2],[4,8,-3],[4,7,-3],[4,8,-1],[4,7,-1]
        ]);
    }
    #[test] fn disable_rules_match_source_thresholds() {
        let s=Settings{disable_on:DISABLE_Y_CHANGE|DISABLE_XZ_MOVE|DISABLE_XZ_SPEED,..Default::default()};
        assert!(should_disable(&s,1.,2.,0.,0.,0.));
        assert!(should_disable(&s,1.,1.,.51,0.,0.));
        assert!(should_disable(&s,1.,1.,0.,0.,5.));
    }
}
