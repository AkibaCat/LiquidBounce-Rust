// SPDX-License-Identifier: GPL-3.0-or-later
//! KillAura state and deterministic decisions, LiquidBounce ae0eb396.
use crate::{
    auto_bow::Priority,
    auto_bow_math::{self as math, B, V},
    rotations::Rotation,
};
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub increase: f32,
    pub walls: f32,
    pub scan: [f32; 2],
    pub priority: Vec<Priority>,
    pub fov: f32,
    pub hurt: i32,
    pub ignore_shield: bool,
    pub requirements: [bool; 6],
    pub raycast: u8,
    pub criticals: u8,
    pub keep_sprint: bool,
    pub ignore_inventory: bool,
    pub close_inventory: bool,
    pub timing: u8,
    pub through: bool,
    pub lazy: bool,
    pub cooldown_ignore: [bool; 3],
    pub block: BlockSettings,
    pub fail: FailSettings,
    pub fight: FightSettings,
    pub indicator: IndicatorSettings,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            increase: 1.,
            walls: 3.,
            scan: [2., 3.],
            priority: vec![Priority::Type, Priority::Health],
            fov: 180.,
            hurt: 10,
            ignore_shield: true,
            requirements: [false; 6],
            raycast: 2,
            criticals: 0,
            keep_sprint: true,
            ignore_inventory: true,
            close_inventory: true,
            timing: 0,
            through: false,
            lazy: false,
            cooldown_ignore: [true; 3],
            block: Default::default(),
            fail: Default::default(),
            fight: Default::default(),
            indicator: Default::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct BlockSettings {
    pub enabled: bool,
    pub mode: u8,
    pub vanilla: bool,
    pub unblock: u8,
    pub reblock: [i32; 2],
    pub pause: [i32; 2],
    pub chance: f32,
    pub blink: i32,
    pub prioritize: bool,
    pub scan: bool,
    pub danger: bool,
    pub tolerance: f32,
    pub force: [f32; 2],
    pub assume: bool,
}
impl Default for BlockSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: 1,
            vanilla: true,
            unblock: 0,
            reblock: [0, 0],
            pause: [0, 0],
            chance: 100.,
            blink: 0,
            prioritize: true,
            scan: true,
            danger: false,
            tolerance: 0.3,
            force: [0., 1.],
            assume: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct FailSettings {
    pub enabled: bool,
    pub additional: [f32; 2],
    pub mode: u8,
    pub fade: i32,
    pub color: u32,
    pub rainbow: bool,
    pub volume: f32,
    pub pitch: f32,
}
impl Default for FailSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            additional: [2.5, 3.],
            mode: 1,
            fade: 4,
            color: 0xffffb348,
            rainbow: false,
            volume: 50.,
            pitch: 0.8,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct FightSettings {
    pub enabled: bool,
    pub opponent: f32,
    pub yaw: f32,
    pub runaway: bool,
    pub range: f32,
    pub visible: bool,
    pub void: bool,
    pub leader: bool,
    pub username: String,
    pub radius: f32,
}
impl Default for FightSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            opponent: 3.,
            yaw: 55.,
            runaway: true,
            range: 50.,
            visible: true,
            void: true,
            leader: false,
            username: String::new(),
            radius: 5.,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct IndicatorSettings {
    pub enabled: bool,
    pub mode: u8,
    pub idle: u32,
    pub active: u32,
    pub outline: bool,
    pub outline_color: u32,
    pub pulse: bool,
    pub pulse_speed: f32,
    pub pulse_intensity: f32,
    pub fade: bool,
    pub fade_speed: f32,
    pub walls: u32,
    pub scan: u32,
    pub opponent: u32,
    pub hide_dead: bool,
    pub hide_spectator: bool,
    pub hide_vehicle: bool,
    pub inventory: bool,
    pub covered: bool,
}
impl Default for IndicatorSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: 0,
            idle: 0x50ff3232,
            active: 0x5032ff32,
            outline: true,
            outline_color: 0x78ffffff,
            pulse: false,
            pulse_speed: 2.,
            pulse_intensity: 0.15,
            fade: true,
            fade_speed: 0.1,
            walls: 0x00ffa500,
            scan: 0x006464ff,
            opponent: 0x00ff0000,
            hide_dead: true,
            hide_spectator: true,
            hide_vehicle: false,
            inventory: true,
            covered: false,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct AutoBlock {
    pub enforced: Option<u8>,
    pub wants_pose: bool,
    pub blocked_since_attack: bool,
    pub danger: bool,
    pub blocking_ticks: i32,
    pub flush_ticks: i32,
    pub reblock: i32,
    pub pause: i32,
}
impl AutoBlock {
    pub fn pose(&self, running: bool, animation: bool, sword_block: bool) -> bool {
        self.wants_pose && running && (animation || sword_block)
    }
    pub fn prioritize(&mut self, s: &BlockSettings, using: bool, hand: Option<u8>) -> bool {
        if using {
            self.blocked_since_attack = true;
        }
        s.enabled
            && s.prioritize
            && s.mode != 2
            && hand.is_some()
            && !self.blocked_since_attack
            && (!s.danger || self.danger)
    }
    /// BlinkPacketEvent only changes QUEUE when every flush condition is false.
    pub fn queues(&mut self, pose: bool, use_item_packet: bool, limit: i32) -> bool {
        if !pose || self.enforced.is_some() || use_item_packet || self.flush_ticks >= limit {
            self.flush_ticks = 0;
            false
        } else {
            true
        }
    }
    pub fn tick(&mut self, danger: bool) {
        self.flush_ticks = self.flush_ticks.wrapping_add(1);
        if self.enforced.is_some() {
            self.blocking_ticks = self.blocking_ticks.wrapping_add(1);
        }
        self.danger = danger;
    }
    pub fn outgoing(&mut self, carried: bool, swap: bool) {
        if swap || carried && self.enforced == Some(0) {
            self.wants_pose = false;
            self.enforced = None;
        }
    }
}
#[derive(Clone, Debug)]
pub struct KillAura {
    pub settings: Settings,
    pub target: Option<i32>,
    pub closest: f64,
    pub wait: i32,
    pub scan_addition: f32,
    pub fail_addition: f32,
    pub failed: Vec<(V, i64)>,
    pub failed_count: i32,
    pub color_factor: f32,
}
impl Default for KillAura {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            target: None,
            closest: 0.,
            wait: 0,
            scan_addition: 2.,
            fail_addition: 2.5,
            failed: vec![],
            failed_count: 0,
            color_factor: 0.,
        }
    }
}
impl KillAura {
    pub fn interaction(&self, attribute: f32) -> f32 {
        attribute + self.settings.increase
    }
    pub fn scan_range(&self, attribute: f32) -> f32 {
        self.interaction(attribute).max(self.settings.walls) + self.scan_addition
    }
    /// Uses the previous targets() call's closest distance. An empty target list
    /// leaves this value unchanged in the source TargetSelector.
    pub fn maximum_range(&self, attribute: f32) -> f32 {
        let r = self.interaction(attribute);
        if self.closest > f64::from(r * r) {
            self.scan_range(attribute)
        } else {
            r
        }
    }
    pub fn requirements(&self, actual: [bool; 6]) -> bool {
        self.settings
            .requirements
            .iter()
            .zip(actual)
            .all(|(&selected, met)| !selected || met)
    }
    pub fn prepare(
        &mut self,
        dead: bool,
        spectator: bool,
        inventory: bool,
        requirements: bool,
    ) -> bool {
        if self.wait > 0 {
            self.wait -= 1;
        }
        if dead || spectator || !requirements || inventory && !self.settings.ignore_inventory {
            self.target = None;
            false
        } else {
            true
        }
    }
    pub fn request_rotation(&self, rotation_ticks: i32, will_click: bool) -> bool {
        match self.settings.timing {
            1 => will_click,
            2 => rotation_ticks > 1,
            _ => true,
        }
    }
    pub fn disabled(&mut self) {
        self.target = None;
        self.failed.clear();
        self.failed_count = 0;
    }
    pub fn failed_render(&mut self) -> Vec<(V, u32)> {
        let s = &self.settings.fail;
        if !s.enabled || s.mode != 1 {
            self.failed.clear();
            return vec![];
        }
        let limit = i64::from(50i32.wrapping_mul(s.fade));
        self.failed.retain_mut(|(_, age)| {
            *age = age.wrapping_add(1);
            *age < limit
        });
        self.failed
            .iter()
            .map(|(pos, age)| {
                let a = (255. + -255. * (*age as f64) / (limit as f64)) as u32;
                (*pos, (s.color & 0xffffff) | (a << 24))
            })
            .collect()
    }
    pub fn failed_hit(&mut self, eye: V, center: V, rotation: Rotation, direction: V) {
        self.failed_count = self.failed_count.wrapping_add(1);
        if self.settings.fail.mode == 1 {
            self.failed
                .push((eye.fma(center.sub(eye).len2().sqrt(), direction), 0));
        }
        let _ = rotation;
    }
}
/// Full FightBot goal selection; the navigation adapter supplies real collision
/// and vanilla-trigonometry functions and owns an independent Agent navigator.
pub fn leader_goal(at: V, player: V, radius: f32, mut direction: impl FnMut(f32) -> V) -> V {
    (-180..=180)
        .step_by(45)
        .map(|yaw| at.fma(f64::from(radius), direction(yaw as f32)))
        .min_by(|a, b| a.sub(player).len2().total_cmp(&b.sub(player).len2()))
        .unwrap_or(at)
}
pub fn attack_goal(
    player: V,
    target: V,
    target_yaw: f32,
    range: f32,
    danger: f32,
    mut direction: impl FnMut(f32) -> V,
    mut collides: impl FnMut(V) -> bool,
) -> V {
    let fallback = target.fma(f64::from(range), direction(target_yaw));
    let mut points = (-180..=180)
        .step_by(10)
        .filter_map(|yaw| {
            let yaw = yaw as f32;
            let p = target.fma(f64::from(range), direction(yaw));
            (!collides(p) && math::wrap(target_yaw - yaw).abs() > danger).then_some(p)
        })
        .collect::<Vec<_>>();
    points.sort_by(|a, b| a.sub(fallback).len2().total_cmp(&b.sub(fallback).len2()));
    points
        .into_iter()
        .min_by(|a, b| a.sub(player).len2().total_cmp(&b.sub(player).len2()))
        .unwrap_or(fallback)
}
pub fn danger(
    eye: V,
    direction: V,
    own: B,
    range: f32,
    walls: f32,
    tolerance: f32,
    force: [f32; 2],
    mut visible: impl FnMut(V, V) -> bool,
) -> bool {
    let end = eye.fma(f64::from(range), direction);
    let Some(hit) = crate::aimbot_ray::first_hit(own.inflate(f64::from(tolerance)), eye, end)
    else {
        return false;
    };
    let d = hit.sub(eye).len2().sqrt();
    d >= f64::from(force[0]) && d <= f64::from(force[1])
        || d <= f64::from(walls)
        || d <= f64::from(range) && visible(eye, hit)
}

#[cfg(test)]
mod kill_aura_behavior_tests {
    use super::*;

    #[test]
    fn scan_range_uses_previous_closest_and_preparation_resets_invalid_target() {
        let mut a = KillAura::default();
        a.scan_addition = 2.;
        assert_eq!(a.maximum_range(3.), 4.);
        a.closest = 16.01;
        assert_eq!(a.maximum_range(3.), 6.);
        a.target = Some(42);
        a.wait = 2;
        a.settings.ignore_inventory = false;
        assert!(!a.prepare(false, false, true, true));
        assert_eq!(a.target, None);
        assert_eq!(a.wait, 1);
        assert!(a.prepare(false, false, false, true));
        assert_eq!(a.wait, 0);
    }

    #[test]
    fn selected_requirements_and_rotation_timing_gate_only_their_own_stage() {
        let mut a = KillAura::default();
        a.settings.requirements = [true, false, false, false, false, true];
        assert!(a.requirements([true, false, false, false, false, true]));
        assert!(!a.requirements([true, true, true, true, true, false]));
        a.settings.timing = 1;
        assert!(!a.request_rotation(6, false));
        assert!(a.request_rotation(6, true));
        a.settings.timing = 2;
        assert!(!a.request_rotation(1, true));
        assert!(a.request_rotation(2, false));
    }

    #[test]
    fn blink_flushes_on_item_use_and_observes_hand_changes() {
        let mut b = AutoBlock::default();
        b.wants_pose = true;
        b.tick(false);
        assert!(b.queues(true, false, 3));
        assert!(!b.queues(true, true, 3));
        assert_eq!(b.flush_ticks, 0);
        b.enforced = Some(1);
        b.outgoing(true, false);
        assert_eq!(b.enforced, Some(1));
        assert!(b.wants_pose);
        assert!(!b.queues(true, false, 3));
        b.outgoing(false, true);
        assert_eq!(b.enforced, None);
        assert!(!b.wants_pose);
        b.enforced = Some(0);
        b.wants_pose = true;
        b.outgoing(true, false);
        assert_eq!(b.enforced, None);
        assert!(!b.wants_pose);
    }

    #[test]
    fn failed_box_fade_expires_on_the_source_frame_count() {
        let mut a = KillAura::default();
        a.settings.fail.enabled = true;
        a.settings.fail.mode = 1;
        a.settings.fail.fade = 1;
        a.failed.push((V(1., 2., 3.), 0));
        for _ in 0..49 {
            assert_eq!(a.failed_render().len(), 1);
        }
        assert!(a.failed_render().is_empty());
        a.failed_count = 7;
        a.disabled();
        assert_eq!(a.failed_count, 0);
    }
}
