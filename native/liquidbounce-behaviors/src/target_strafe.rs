// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTargetStrafe's shared planner, source movement and complete option state.
pub const PRIORITIES: [&str; 6] = ["Type", "Health", "Distance", "Direction", "HurtTime", "Age"];
use crate::auto_bow_math::{B, V, wrap};
use serde_json::{Value, json};
pub const REQUIREMENTS: [&str; 4] = ["Space", "Speed", "KillAura", "Ground"];
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub input_mode: bool,
    pub hypixel: bool,
    pub range: f32,
    pub follow_range: f32,
    pub vertical_range: f32,
    pub fov: f32,
    pub hurt_time: i32,
    pub priority: Vec<usize>,
    pub requirements: u32,
    pub planner: bool,
    pub control_direction: bool,
    pub validation: bool,
    pub edge: bool,
    pub max_fall: f32,
    pub void: bool,
    pub safety_expand: f32,
    pub adaptive: bool,
    pub max_range: f32,
    pub range_step: f32,
    pub visibility: bool,
    pub search_distance: f32,
    pub visuals: bool,
    pub width: f32,
    pub height_offset: f32,
    pub outer: u32,
    pub inner: u32,
    pub outline: u32,
    pub show_point: bool,
    pub point_radius: f32,
    pub point: u32,
    pub point_outline: u32,
    pub invalid: u32,
    pub invalid_outline: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            input_mode: false,
            hypixel: false,
            range: 2.95,
            follow_range: 4.,
            vertical_range: 2.,
            fov: 180.,
            hurt_time: 10,
            priority: vec![0, 1],
            requirements: 0,
            planner: true,
            control_direction: true,
            validation: true,
            edge: true,
            max_fall: 1.2,
            void: true,
            safety_expand: 0.1,
            adaptive: false,
            max_range: 4.,
            range_step: 0.5,
            visibility: true,
            search_distance: 4.,
            visuals: true,
            width: 0.12,
            height_offset: 0.05,
            outer: 0x644677ff,
            inner: 0x144677ff,
            outline: 0xb44677ff,
            show_point: true,
            point_radius: 0.18,
            point: 0x5a4677ff,
            point_outline: 0xb44677ff,
            invalid: 0x5aff5a5a,
            invalid_outline: 0xb4ff5a5a,
        }
    }
}
impl Settings {
    pub fn requirements_met(&self, met: [bool; 4]) -> bool {
        met.into_iter()
            .enumerate()
            .all(|(i, v)| self.requirements & (1 << i) == 0 || v)
    }
    pub fn in_range(&self, feet: V, target: V) -> bool {
        let h = (target.1 - feet.1).abs();
        h <= f64::from(self.vertical_range)
            && h < f64::from(self.range)
            && target.sub(feet).len() <= f64::from(self.follow_range)
    }
    pub fn wire(&self) -> Value {
        json!({"input":self.input_mode,"hypixel":self.hypixel,"range":self.range,"follow":self.follow_range,"vertical":self.vertical_range,
        "planner":self.planner,"control":self.control_direction,"validation":self.validation,"edge":self.edge,"fall":self.max_fall,"void":self.void,"expand":self.safety_expand,"adaptive":self.adaptive,"max":self.max_range,"step":self.range_step,"visibility":self.visibility,"search":self.search_distance,
        "visuals":self.visuals,"width":self.width,"offset":self.height_offset,"outer":self.outer,"inner":self.inner,"outline":self.outline,"show":self.show_point,"radius":self.point_radius,"point":self.point,"pointOutline":self.point_outline,"invalid":self.invalid,"invalidOutline":self.invalid_outline})
    }
    pub fn from_wire(v: &Value) -> Self {
        let mut s = Self::default();
        macro_rules! b {($($f:ident:$k:literal),*)=>{$(if let Some(v)=v[$k].as_bool(){s.$f=v;})*};}
        macro_rules! f {($($f:ident:$k:literal),*)=>{$(if let Some(v)=v[$k].as_f64(){s.$f=v as f32;})*};}
        macro_rules! c {($($f:ident:$k:literal),*)=>{$(if let Some(v)=v[$k].as_u64(){s.$f=v as u32;})*};}
        b!(input_mode:"input",hypixel:"hypixel",planner:"planner",control_direction:"control",validation:"validation",edge:"edge",void:"void",adaptive:"adaptive",visibility:"visibility",visuals:"visuals",show_point:"show");
        f!(range:"range",follow_range:"follow",vertical_range:"vertical",max_fall:"fall",safety_expand:"expand",max_range:"max",range_step:"step",search_distance:"search",width:"width",height_offset:"offset",point_radius:"radius");
        c!(outer:"outer",inner:"inner",outline:"outline",point:"point",point_outline:"pointOutline",invalid:"invalid",invalid_outline:"invalidOutline");
        s
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub id: i32,
    pub position: V,
    pub bounds: B,
}
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub feet: V,
    pub speed: f64,
    pub yaw: f32,
    pub horizontal_collision: bool,
    pub input: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub target: Target,
    pub radius: f32,
    pub direction: V,
    pub next: V,
    pub valid: bool,
}
/// Real terrain adapter, using Level.noCollision and COLLIDER line of sight.
pub trait World {
    fn collision_free(&mut self, point: V) -> bool;
    fn edge_safe(&mut self, point: V, max_fall: f32) -> bool;
    fn void_safe(&mut self, point: V, expand: f32) -> bool;
    fn sees(&mut self, feet: V, bounds: B, max_points: i32) -> bool;
}
#[derive(Clone, Debug)]
pub struct Planner {
    pub direction: i32,
    pub render: Option<Plan>,
}
impl Default for Planner {
    fn default() -> Self {
        Self {
            direction: 1,
            render: None,
        }
    }
}
impl Planner {
    pub fn reset(&mut self) {
        self.render = None;
    }
    fn valid(s: &Settings, w: &mut impl World, p: V) -> bool {
        w.collision_free(p)
            && (!s.validation
                || ((!s.edge || w.edge_safe(p, s.max_fall))
                    && (!s.void || w.void_safe(p, s.safety_expand))))
    }
    fn steps_to_sight(
        &self,
        s: &Settings,
        w: &mut impl World,
        c: Context,
        t: Target,
        radius: f32,
        side: i32,
        steps: i32,
    ) -> Option<i32> {
        let mut point = c.feet;
        for step in 1..=steps {
            let d = t.position.sub(point);
            let yaw = d.2.atan2(d.0);
            let distance = (d.0 * d.0 + d.2 * d.2).sqrt();
            point = point.add(direction_vector(yaw, distance, 0.5, radius, side));
            if !Self::valid(s, w, point) {
                return None;
            }
            if w.sees(point, t.bounds, 32) {
                return Some(step);
            }
        }
        None
    }
    fn steer(
        &mut self,
        s: &Settings,
        w: &mut impl World,
        c: Context,
        t: Target,
        radius: f32,
    ) -> bool {
        if w.sees(c.feet, t.bounds, 256) {
            if self
                .steps_to_sight(s, w, c, t, radius, self.direction, 1)
                .is_none()
                && self
                    .steps_to_sight(s, w, c, t, radius, -self.direction, 1)
                    .is_some()
            {
                self.direction = -self.direction;
            }
            return true;
        }
        let steps = (f64::from(s.search_distance) / 0.5) as i32;
        let ahead = self.steps_to_sight(s, w, c, t, radius, self.direction, steps.max(1));
        let behind = self.steps_to_sight(s, w, c, t, radius, -self.direction, steps.max(1));
        if behind.is_some_and(|b| ahead.is_none_or(|a| b < a)) {
            self.direction = -self.direction;
        }
        ahead.is_some() || behind.is_some()
    }
    pub fn compute(
        &mut self,
        s: &Settings,
        w: &mut impl World,
        c: Context,
        t: Option<Target>,
    ) -> Option<Plan> {
        let Some(t) = t.filter(|t| s.in_range(c.feet, t.position)) else {
            self.reset();
            return None;
        };
        let height = t.position.1 - c.feet.1;
        let radius =
            |range: f32| ((f64::from(range * range) - height * height).max(0.).sqrt()) as f32;
        if c.horizontal_collision {
            self.direction = -self.direction;
        }
        if s.control_direction && c.input & 12 != 12 {
            if c.input & 4 != 0 {
                self.direction = -1;
            } else if c.input & 8 != 0 {
                self.direction = 1;
            }
        }
        if s.visibility && !self.steer(s, w, c, t, radius(s.range)) {
            self.reset();
            return None;
        }
        let d = t.position.sub(c.feet);
        let yaw = d.2.atan2(d.0);
        let distance = (d.0 * d.0 + d.2 * d.2).sqrt();
        let create = |side: i32, range: f32, w: &mut _| {
            let r = radius(range);
            let direction = direction_vector(yaw, distance, c.speed, r, side);
            let next = c.feet.add(direction);
            Plan {
                target: t,
                radius: r,
                direction,
                next,
                valid: Self::valid(s, w, next),
            }
        };
        let mut plan = create(self.direction, s.range, w);
        if !plan.valid {
            if !s.adaptive {
                self.direction = -self.direction;
                plan = create(self.direction, s.range, w);
            } else {
                let mut current = s.range_step;
                while !plan.valid {
                    plan = create(self.direction, current, w);
                    current += s.range_step;
                    if current > s.max_range {
                        self.direction = -self.direction;
                        plan = create(self.direction, s.range, w);
                        break;
                    }
                }
            }
        }
        self.render = Some(plan);
        Some(plan)
    }
}
pub fn direction_vector(
    strafe_yaw: f64,
    distance: f64,
    speed: f64,
    range: f32,
    direction: i32,
) -> V {
    let yaw = strafe_yaw - f64::from(std::f32::consts::FRAC_PI_2);
    let encirclement = (-speed).max(distance - f64::from(range));
    V(
        -yaw.sin() * encirclement - strafe_yaw.sin() * speed * f64::from(direction),
        0.,
        yaw.cos() * encirclement + strafe_yaw.cos() * speed * f64::from(direction),
    )
}
pub fn movement(
    plan: Plan,
    old: V,
    speed: f64,
    hypixel: bool,
    speed_running: bool,
    on_ground: bool,
    should_strafe: bool,
) -> V {
    if !plan.valid {
        return old;
    }
    // Vec3.yaw rounds to Float before withStrafe converts the degrees back to Double.
    let yaw = (-plan.direction.0).atan2(plan.direction.2).to_degrees() as f32;
    let speed = if hypixel && speed_running {
        speed.max(if on_ground { 0.48 } else { 0.281 })
    } else {
        speed
    };
    let strength = if hypixel && speed_running && !should_strafe {
        0.02
    } else {
        1.
    };
    let angle = f64::from(yaw).to_radians();
    V(
        old.0 * (1. - strength) - angle.sin() * speed * strength,
        old.1,
        old.2 * (1. - strength) + angle.cos() * speed * strength,
    )
}
pub fn directional_input(direction: V, yaw: f32) -> u8 {
    let optimal = (-direction.0).atan2(direction.2) as f32 * (180. / std::f32::consts::PI);
    let d = wrap(optimal - wrap(yaw));
    u8::from(d > -70. && d < 70.)
        | u8::from(d < -110. || d > 110.) << 1
        | u8::from(d > -160. && d < -20.) << 2
        | u8::from(d > 20. && d < 160.) << 3
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Flat;
    impl World for Flat {
        fn collision_free(&mut self, _: V) -> bool {
            true
        }
        fn edge_safe(&mut self, _: V, _: f32) -> bool {
            true
        }
        fn void_safe(&mut self, _: V, _: f32) -> bool {
            true
        }
        fn sees(&mut self, _: V, _: B, _: i32) -> bool {
            true
        }
    }
    fn context() -> Context {
        Context {
            feet: V(3., 0., 0.),
            speed: 0.3,
            yaw: 0.,
            horizontal_collision: false,
            input: 1,
        }
    }
    fn target() -> Target {
        Target {
            id: 5,
            position: V(0., 0., 0.),
            bounds: B {
                lo: V(-0.3, 0., -0.3),
                hi: V(0.3, 1.8, 0.3),
            },
        }
    }
    #[test]
    fn sphere_slice_requirements_and_direction_control() {
        let mut s = Settings::default();
        s.requirements = 9;
        assert!(!s.requirements_met([true, false, false, false]));
        assert!(s.requirements_met([true, false, false, true]));
        let mut t = target();
        t.position.1 = 1.;
        let mut p = Planner::default();
        let mut c = context();
        c.input = 4;
        let plan = p.compute(&s, &mut Flat, c, Some(t)).unwrap();
        assert_eq!(p.direction, -1);
        assert!((plan.radius - ((s.range * s.range - 1.).sqrt())).abs() < 1e-6);
        t.position.1 = s.range as f64;
        assert!(p.compute(&s, &mut Flat, c, Some(t)).is_none());
        assert!(p.render.is_none());
    }
    #[test]
    fn motion_preserves_vertical_and_hypixel_blends() {
        let s = Settings::default();
        let p = Planner::default()
            .compute(&s, &mut Flat, context(), Some(target()))
            .unwrap();
        let old = V(0.2, -0.1, 0.3);
        let out = movement(p, old, 0.3, false, true, false, false);
        assert_eq!(out.1, -0.1);
        assert!((out.0.hypot(out.2) - 0.3).abs() < 1e-12);
        let boost = movement(p, old, 0.1, true, true, true, true);
        assert!((boost.0.hypot(boost.2) - 0.48).abs() < 1e-12);
        assert_eq!(directional_input(V(0., 0., 1.), 0.), 1);
        // MovementUtils.getDirectionalInputForDegrees: positive yaw selects right.
        assert_eq!(directional_input(V(-1., 0., 0.), 0.), 8);
        assert_eq!(directional_input(V(1., 0., 0.), 0.), 4);
    }
    #[test]
    fn wire_keeps_planner_and_visual_settings() {
        let mut s = Settings::default();
        s.input_mode = true;
        s.adaptive = true;
        s.inner = 0xaabbccdd;
        s.safety_expand = 0.8;
        let copy = Settings::from_wire(&s.wire());
        assert_eq!(s, copy);
        assert_eq!(PRIORITIES.len(), 6);
    }
    struct Unsafe {
        safe: bool,
    }
    impl World for Unsafe {
        fn collision_free(&mut self, _: V) -> bool {
            self.safe
        }
        fn edge_safe(&mut self, _: V, _: f32) -> bool {
            panic!("disabled terrain check")
        }
        fn void_safe(&mut self, _: V, _: f32) -> bool {
            panic!("disabled terrain check")
        }
        fn sees(&mut self, _: V, _: B, _: i32) -> bool {
            true
        }
    }
    #[test]
    fn disabling_validation_never_allows_collisions_and_failed_plan_is_rendered() {
        let mut s = Settings::default();
        s.validation = false;
        s.visibility = false;
        let mut planner = Planner::default();
        let plan = planner
            .compute(&s, &mut Unsafe { safe: false }, context(), Some(target()))
            .unwrap();
        assert!(!plan.valid);
        assert_eq!(planner.direction, -1);
        assert!(planner.render.is_some());
        assert!(
            planner
                .compute(&s, &mut Unsafe { safe: true }, context(), Some(target()))
                .unwrap()
                .valid
        );
    }
    #[test]
    fn visibility_probes_both_sides_and_selects_earlier_sight() {
        struct Sight(bool);
        impl World for Sight {
            fn collision_free(&mut self, _: V) -> bool {
                true
            }
            fn edge_safe(&mut self, _: V, _: f32) -> bool {
                true
            }
            fn void_safe(&mut self, _: V, _: f32) -> bool {
                true
            }
            fn sees(&mut self, p: V, _: B, maximum: i32) -> bool {
                assert!(maximum == 32 || maximum == 256);
                self.0 && p.2 > 0.1
            }
        }
        let mut planner = Planner::default();
        let s = Settings::default();
        assert!(
            planner
                .compute(&s, &mut Sight(true), context(), Some(target()))
                .unwrap()
                .valid
        );
        assert_eq!(planner.direction, -1);
        assert!(
            planner
                .compute(&s, &mut Sight(false), context(), Some(target()))
                .is_none()
        );
        assert!(planner.render.is_none());
    }
    #[test]
    fn adaptive_probes_smaller_ranges_before_reversing() {
        struct Radius;
        impl World for Radius {
            fn collision_free(&mut self, p: V) -> bool {
                p.0 < 1.1
            }
            fn edge_safe(&mut self, _: V, _: f32) -> bool {
                true
            }
            fn void_safe(&mut self, _: V, _: f32) -> bool {
                true
            }
            fn sees(&mut self, _: V, _: B, _: i32) -> bool {
                true
            }
        }
        let mut s = Settings::default();
        s.visibility = false;
        s.adaptive = true;
        let mut planner = Planner::default();
        let plan = planner
            .compute(&s, &mut Radius, context(), Some(target()))
            .unwrap();
        assert!(plan.valid);
        assert_eq!(plan.radius, 0.5);
        assert_eq!(planner.direction, 1);
    }
}
