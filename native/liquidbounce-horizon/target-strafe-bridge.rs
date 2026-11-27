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
