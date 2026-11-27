// SPDX-License-Identifier: GPL-3.0-or-later
//! LocalPlayerHooks.move reads the source player, not the speculative body.
use crate::motion::{MotionWorld, PlayerMotion};
use horizon_core::geometry::Vec3;
use std::cell::RefCell;
thread_local! {static SOURCE:RefCell<Option<PlayerMotion>>=const{RefCell::new(None)};}
pub struct Scope {
    previous: Option<PlayerMotion>,
    active: bool,
}
impl Scope {
    pub fn enter(source: PlayerMotion) -> Self {
        Self {
            previous: SOURCE.with(|s| s.replace(Some(source))),
            active: true,
        }
    }
    pub fn finish(mut self) -> PlayerMotion {
        let source = SOURCE.with(|s| s.replace(self.previous.take()));
        self.active = false;
        source.expect("falling source scope")
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        if self.active {
            SOURCE.with(|s| {
                s.replace(self.previous.take());
            });
        }
    }
}
pub fn movement(
    world: &mut MotionWorld<'_, '_, '_>,
    movement: Vec3,
) -> Result<Vec3, Box<dyn std::error::Error>> {
    SOURCE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(source) = slot.as_mut() else {
            return Ok(movement);
        };
        let mut movement = movement;
        if let Some((settings, input)) = source.client_strafe {
            let [x, y, z] = settings.move_event(
                true,
                crate::strafe::StrafeContext {
                    self_movement: true,
                    on_ground: source.ground.on_ground,
                    moving: source.move_vector[0] != 0.0 || source.move_vector[1] != 0.0,
                    input,
                    yaw: source.yaw,
                },
                [movement.x, movement.y, movement.z],
                |angle| {
                    [
                        horizon_core::jmath::sin(angle),
                        horizon_core::jmath::cos(angle),
                    ]
                },
            );
            movement = Vec3::new(x, y, z);
        }
        let mut live_world = MotionWorld {
            entities: world.entities,
            player: source,
            scene: world.scene,
            assets: world.assets,
            collision_binding: world.collision_binding,
            resources: world.resources,
            health: world.health,
        };
        movement = crate::terrain_speed_motion::move_event(&mut live_world, movement)?;
        if let Some(position) = crate::terrain_speed_bridge::take_snap() {
            live_world.player.old_position = position;
        }
        Ok(crate::anchor_bridge::movement(
            live_world.player.position,
            live_world.player.bounds(),
            movement,
        ))
    })
}
pub fn flush() {
    SOURCE.with(|s| {
        if let Some(source) = s.borrow_mut().as_mut() {
            crate::terrain_speed_bridge::flush(&mut source.velocity);
        }
    });
}
