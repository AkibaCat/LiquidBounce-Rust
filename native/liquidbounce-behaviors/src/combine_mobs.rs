// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleCombineMobs, LiquidBounce ae0eb396 (CCBlueX, 2015-2026).
use std::collections::HashMap;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub kind: String,
    pub baby: bool,
    pub position: i64,
}
impl Key {
    pub fn at(kind: &str, baby: bool, position: [f64; 3]) -> Self {
        let [x, y, z] = position.map(|v| v.floor() as i32 as i64);
        Self {
            kind: kind.into(),
            baby,
            position: ((x & 0x3ffffff) << 38) | ((z & 0x3ffffff) << 12) | (y & 0xfff),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CombineMobs {
    pub combine_armor_stands: bool,
    pub combine_minecarts: bool,
    render: HashMap<Key, i32>,
    nametags: HashMap<Key, i32>,
}
impl CombineMobs {
    pub fn clear_render(&mut self) {
        self.render.clear();
    }
    pub fn clear_nametags(&mut self) {
        self.nametags.clear();
    }
    pub fn disabled(&mut self) {
        self.clear_render();
        self.clear_nametags();
    }
    /// group: 0 actual Mob, 1 ArmorStand, 2 AbstractMinecart, 3 other entity.
    pub fn track(&mut self, running: bool, key: Key, group: u8, for_nametag: bool) -> bool {
        if !running
            || !match group {
                0 => true,
                1 => self.combine_armor_stands,
                2 => self.combine_minecarts,
                _ => false,
            }
        {
            return false;
        }
        let count = (if for_nametag {
            &mut self.nametags
        } else {
            &mut self.render
        })
        .entry(key)
        .or_default();
        let old = *count;
        *count = count.wrapping_add(1);
        old > 0
    }
    pub fn count(&self, running: bool, key: &Key) -> i32 {
        if !running {
            return 1;
        }
        let render = self.render.get(key).copied().unwrap_or(0);
        if render > 0 {
            render
        } else {
            self.nametags.get(key).copied().unwrap_or(1)
        }
    }
}
