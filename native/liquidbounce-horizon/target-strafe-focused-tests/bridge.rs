// SPDX-License-Identifier: GPL-3.0-or-later
//! Window-thread TargetStrafe state. Solvers never call back into an owned Game.
use crate::target_strafe::{self as model, Context, Plan, Planner, Settings, Target};
use crate::target_strafe_math::V;
use std::cell::RefCell;
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub settings: Settings,
    pub target: Option<Target>,
    pub input: u8,
    pub initial_moving: bool,
    pub speed_running: bool,
}
#[derive(Default)]
struct State {
    snapshot: Option<Snapshot>,
    planner: Planner,
    should_strafe: bool,
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::default());}
pub fn configure(snapshot: Option<Snapshot>) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if snapshot.is_none() {
            s.planner.reset();
        } else if s.snapshot.as_ref().is_some_and(|old| old.settings.visuals)
            && snapshot.as_ref().is_some_and(|new| !new.settings.visuals)
        {
            s.planner.reset();
        }
        s.snapshot = snapshot;
    });
}
pub fn clear() {
    configure(None);
}
/// SpeedHypixel20250125's shared static flag, written by its tick/jump handlers.
pub fn set_hypixel_should_strafe(value: bool) {
    STATE.with(|s| s.borrow_mut().should_strafe = value);
}
pub fn render() -> Option<(Settings, Plan)> {
    STATE.with(|s| {
        let s = s.borrow();
        Some((s.snapshot.as_ref()?.settings.clone(), s.planner.render?))
    })
}
pub fn plan(world: &mut impl model::World, context: Context, input_mode: bool) -> Option<Plan> {
    STATE.with(|slot| {
        let mut s = slot.borrow_mut();
        let snapshot = s.snapshot.clone()?;
        if snapshot.settings.input_mode != input_mode {
            return None;
        }
        let moving = if input_mode {
            ((context.input & 1 != 0) != (context.input & 2 != 0)
                || (context.input & 4 != 0) != (context.input & 8 != 0))
        } else {
            snapshot.initial_moving
        };
        if !moving {
            s.planner.reset();
            return None;
        }
        s.planner.compute(
            &snapshot.settings,
            world,
            Context {
                input: snapshot.input,
                ..context
            },
            snapshot.target,
        )
    })
}
pub fn input_event(world: &mut impl model::World, context: Context) -> Option<u8> {
    plan(world, context, true)
        .filter(|p| p.valid)
        .map(|p| model::directional_input(p.direction, context.yaw))
}
pub fn move_event(
    world: &mut impl model::World,
    context: Context,
    movement: V,
    on_ground: bool,
) -> V {
    let Some(plan) = plan(world, context, false) else {
        return movement;
    };
    STATE.with(|s| {
        let s = s.borrow();
        let snapshot = s.snapshot.as_ref().unwrap();
        model::movement(
            plan,
            movement,
            context.speed,
            snapshot.settings.hypixel,
            snapshot.speed_running,
            on_ground,
            s.should_strafe,
        )
    })
}

#[cfg(test)]
mod bridge_tests {
    use super::*;
    use crate::target_strafe_math::B;
    struct Flat;
    impl model::World for Flat {
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
    fn snapshot(input_mode: bool, initial_moving: bool) -> Snapshot {
        Snapshot {
            settings: Settings {
                input_mode,
                ..Default::default()
            },
            target: Some(Target {
                id: 1,
                position: V::default(),
                bounds: B {
                    lo: V(-0.3, 0., -0.3),
                    hi: V(0.3, 1.8, 0.3),
                },
            }),
            input: 1,
            initial_moving,
            speed_running: false,
        }
    }
    fn context(input: u8) -> Context {
        Context {
            feet: V(3., 0., 0.),
            speed: 0.53,
            yaw: 0.,
            horizontal_collision: false,
            input,
        }
    }
    #[test]
    fn actual_move_uses_event_speed_and_initial_movement_gate() {
        clear();
        configure(Some(snapshot(false, false)));
        let old = V(0.2, 0.1, 0.4);
        assert_eq!(move_event(&mut Flat, context(1), old, true), old);
        assert!(render().is_none());
        configure(Some(snapshot(false, true)));
        let out = move_event(&mut Flat, context(1), old, true);
        assert!((out.0.hypot(out.2) - 0.53).abs() < 1e-12);
        assert_eq!(out.1, 0.1);
        assert!(render().is_some());
        clear();
        assert!(render().is_none());
    }
    #[test]
    fn input_mode_uses_cancelled_keys_and_does_not_change_motion() {
        clear();
        configure(Some(snapshot(true, false)));
        assert!(input_event(&mut Flat, context(15)).is_none());
        assert!(render().is_none());
        assert!(input_event(&mut Flat, context(1)).is_some());
        let old = V(1., 2., 3.);
        assert_eq!(move_event(&mut Flat, context(1), old, false), old);
        clear();
    }
}
