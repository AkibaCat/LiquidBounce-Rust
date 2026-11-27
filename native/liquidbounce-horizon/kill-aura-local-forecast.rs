// SPDX-License-Identifier: GPL-3.0-or-later
// Appended to game::kill_aura. Replay actual input against an owned snapshot.
impl Game<'_> {
    fn kill_aura_local_future(&mut self, ticks: i32, yaw: f32) -> V {
        use super::{
            auto_walk_player::NativePlayer,
            auto_walk_simulation::NativeSimulation,
            auto_walk_world::{self as world, NativeLevel, NavigationRegistry},
        };
        use crate::navigation::{
            input::Input,
            walk_control::body::{SimulatedInput, WalkSimulation},
        };
        use horizon_client::actions::Action as M;
        let held = |key| self.keys.held(key);
        let input = Input {
            forward: held(M::Forward),
            backward: held(M::Back),
            left: held(M::Left),
            right: held(M::Right),
            jump: held(M::Jump),
            shift: held(M::Sneak),
            sprint: held(M::Sprint),
        };
        world::with_scope(self, || {
            let epoch = world::read(world::current_epoch);
            let registry = world::read(NavigationRegistry::capture);
            let player = NativePlayer::new(NativeLevel::new(registry, epoch));
            let mut input = SimulatedInput::new(input);
            input.force_stay_on_ground_surface =
                horizon_client::navigation_physics::staying_on_surface(false);
            let mut simulation = NativeSimulation::new(&player, input, yaw);
            for _ in 0..ticks.clamp(0, 40) {
                simulation.tick();
            }
            let eye = simulation.player().eye();
            V(eye.x, eye.y, eye.z)
        })
    }
}
