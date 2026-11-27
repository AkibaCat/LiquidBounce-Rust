// SPDX-License-Identifier: GPL-3.0-or-later
//! The actual aiStep tail, using disjoint sound and Session borrows.
use crate::{
    elytra_recast::{Chest, RecastWorld},
    motion::MotionWorld,
};

pub fn ai_step_tail(
    world: &mut MotionWorld<'_, '_, '_>,
    running: bool,
    physical_jump: bool,
    input_jump: bool,
    chest: Chest,
    stop: &mut dyn FnMut(),
    send: &mut dyn FnMut(),
) -> crate::Result<()> {
    // Taking only this tiny state avoids aliasing world.player in RecastWorld.
    let mut state = std::mem::take(&mut world.player.client_elytra_recast);
    let result = state.ai_step_tail(
        running,
        &mut Context {
            world,
            physical_jump,
            input_jump,
            chest,
            stop,
            send,
        },
    );
    world.player.client_elytra_recast = state;
    result
}

struct Context<'a, 'p, 'w, 'r> {
    world: &'a mut MotionWorld<'p, 'w, 'r>,
    physical_jump: bool,
    input_jump: bool,
    chest: Chest,
    stop: &'a mut dyn FnMut(),
    send: &'a mut dyn FnMut(),
}
impl RecastWorld for Context<'_, '_, '_, '_> {
    type Error = Box<dyn std::error::Error>;
    fn chest(&mut self) -> Chest {
        self.chest
    }
    fn flying(&self) -> bool {
        self.world.player.flying
    }
    fn passenger(&self) -> bool {
        self.world.player.is_passenger
    }
    fn climbable(&mut self) -> crate::Result<bool> {
        self.world.climbable()
    }
    fn in_water(&self) -> bool {
        self.world.player.fluids.was_touching_water
    }
    fn levitating(&self) -> bool {
        self.world.player.status.levitation.is_some()
    }
    fn fall_flying(&self) -> bool {
        self.world.player.fall_flying
    }
    fn jump_key(&self) -> bool {
        self.physical_jump
    }
    fn input_jump(&self) -> bool {
        self.input_jump
    }
    fn stop_elytra_sound(&mut self) {
        (self.stop)();
    }
    fn start_fall_flying(&mut self) {
        self.world.player.fall_flying = true;
    }
    fn send_start_fall_flying(&mut self) {
        (self.send)();
    }
    fn reset_jump_delay(&mut self) {
        self.world.player.no_jump_delay = 0;
    }
}
