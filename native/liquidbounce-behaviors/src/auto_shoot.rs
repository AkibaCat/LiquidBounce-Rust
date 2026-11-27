// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoShoot, pinned LiquidBounce ae0eb396.
use crate::auto_bow::{self, Candidate, Priority};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub custom: bool,
    pub whitelist: bool,
    pub items: BTreeSet<String>,
    /// Auto, Linear, Projectile.
    pub gravity: u8,
    pub range: [f32; 2],
    pub fov: f32,
    pub hurt_time: i32,
    pub priority: Vec<Priority>,
    pub aim_threshold: f32,
    pub hide_swing: bool,
    pub select_slot: bool,
    pub slot_reset: i32,
    pub consider_inventory: bool,
    pub requires_aura: bool,
    pub not_during_combat: bool,
    pub constant_lag: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            custom: false,
            whitelist: true,
            items: ["minecraft:egg".into(), "minecraft:snowball".into()].into(),
            gravity: 0,
            range: [3., 6.],
            fov: 180.,
            hurt_time: 10,
            priority: vec![Priority::Type, Priority::Distance],
            aim_threshold: 2.,
            hide_swing: false,
            select_slot: true,
            slot_reset: 1,
            consider_inventory: true,
            requires_aura: false,
            not_during_combat: false,
            constant_lag: false,
        }
    }
}
#[derive(Default)]
pub struct AutoShoot {
    pub settings: Settings,
    pub target: Option<i32>,
    pub closest_squared_enemy_distance: f64,
}
impl AutoShoot {
    pub fn accepts(&self, item: &str, count: i32, egg_or_snowball: bool) -> bool {
        if self.settings.custom {
            count > 0 && (self.settings.items.contains(item) == self.settings.whitelist)
        } else {
            count > 0 && egg_or_snowball
        }
    }
    pub fn projectile(&self, egg_or_snowball: bool) -> bool {
        match self.settings.gravity {
            0 => egg_or_snowball,
            1 => false,
            _ => true,
        }
    }
    pub fn choose_slot(&self, matching: [bool; 10], selected: u8) -> Option<u8> {
        (0..10usize)
            .filter(|i| matching[*i])
            .min_by_key(|i| {
                if *i == usize::from(selected) {
                    i32::MIN
                } else if *i == 9 {
                    i32::MIN + 1
                } else {
                    (*i as i32 - i32::from(selected)).abs()
                }
            })
            .map(|i| if i == 9 { 40 } else { i as u8 })
    }
    pub fn pause_for_aura(&mut self, aura_running: bool) -> bool {
        let pause = self.settings.requires_aura && !aura_running;
        if pause {
            self.target = None;
        }
        pause
    }
    pub fn select(&mut self, candidates: Vec<Candidate>, mut visible: impl FnMut(i32) -> bool) {
        let settings = auto_bow::Settings {
            fov: self.settings.fov,
            hurt_time: self.settings.hurt_time,
            priority: self.settings.priority.clone(),
            ..Default::default()
        };
        let low = f64::from(self.settings.range[0] * self.settings.range[0]);
        let high = f64::from(self.settings.range[1] * self.settings.range[1]);
        let targets = auto_bow::targets(
            &settings,
            candidates
                .into_iter()
                .filter(|c| c.distance >= low && c.distance <= high)
                .collect(),
        );
        if !targets.is_empty() {
            self.closest_squared_enemy_distance = targets
                .iter()
                .map(|c| c.distance)
                .fold(f64::INFINITY, f64::min);
        }
        self.target = targets.into_iter().find(|c| visible(c.id)).map(|c| c.id);
    }
    pub fn disabled(&mut self) {
        self.target = None;
    }
}
