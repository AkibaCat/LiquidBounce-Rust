// SPDX-License-Identifier: GPL-3.0-or-later
/// Included in motion.rs: exact move-event speed, context shapes and collision phases.
mod target_strafe_terrain {
    use super::*;
    use crate::target_strafe::{self as model, Context};
    use crate::target_strafe_math::{self as math, B, V};
    use horizon_core::geometry::VoxelShape;
    use horizon_world::world_ray::{RayBlockMode, RayFluidMode, RayWorld, clip_world};
    struct TargetStrafeRay<'a, 'r, 'c> {
        scene: SceneCollisions<'a, 'r>,
        context: CollisionContext<'c>,
    }
    impl RayWorld for TargetStrafeRay<'_, '_, '_> {
        type Error = std::io::Error;
        type Block = BlockStateId;
        type Fluid = ();
        fn block_state(&mut self, p: BlockPos) -> std::result::Result<BlockStateId, Self::Error> {
            Ok(self.scene.world.block(p.x, p.y, p.z))
        }
        fn fluid_state(&mut self, _: BlockPos) -> std::result::Result<(), Self::Error> {
            Ok(())
        }
        fn block_shape(
            &mut self,
            state: BlockStateId,
            p: BlockPos,
            _: RayBlockMode,
        ) -> std::result::Result<VoxelShape, Self::Error> {
            let binding = self.scene.binding;
            binding
                .collision_shape_with_context(&mut self.scene, state, p, &self.context)
                .map_err(|e| std::io::Error::other(format!("{e:?}")))
        }
        fn interaction_shape(
            &mut self,
            _: BlockStateId,
            _: BlockPos,
        ) -> std::result::Result<VoxelShape, Self::Error> {
            Ok(VoxelShape::empty())
        }
        fn fluid_is_source(&mut self, _: ()) -> std::result::Result<bool, Self::Error> {
            Ok(false)
        }
        fn fluid_is_empty(&mut self, _: ()) -> std::result::Result<bool, Self::Error> {
            Ok(true)
        }
        fn fluid_is_water(&mut self, _: ()) -> std::result::Result<bool, Self::Error> {
            Ok(false)
        }
        fn fluid_shape(
            &mut self,
            _: (),
            _: BlockPos,
        ) -> std::result::Result<VoxelShape, Self::Error> {
            Ok(VoxelShape::empty())
        }
    }

    fn point(v: V) -> Vec3 {
        Vec3::new(v.0, v.1, v.2)
    }
    impl MotionWorld<'_, '_, '_> {
        fn target_strafe_box(&self, p: V, shrink: f64) -> Aabb {
            let r = f64::from(self.player.dimensions.width) * 0.5 - shrink;
            Aabb::new(
                Vec3::new(p.0 - r, p.1, p.2 - r),
                Vec3::new(
                    p.0 + r,
                    p.1 + f64::from(self.player.dimensions.height),
                    p.2 + r,
                ),
            )
        }
        pub(super) fn target_strafe_context(&self, input: u8) -> Context {
            let p = &*self.player;
            Context {
                feet: V(p.position.x, p.position.y, p.position.z),
                speed: p.velocity.x.hypot(p.velocity.z),
                yaw: p.yaw,
                horizontal_collision: p.collision.horizontal,
                input,
            }
        }
        pub(super) fn target_strafe_move(&mut self, movement: Vec3) -> Vec3 {
            if !self.player.client_side || !self.player.native_client_queries_allowed() {
                return movement;
            }
            let c = self.target_strafe_context(0);
            let on_ground = self.player.ground.on_ground;
            let result = crate::target_strafe_bridge::move_event(
                self,
                c,
                V(movement.x, movement.y, movement.z),
                on_ground,
            );
            point(result)
        }
        pub fn target_strafe_input(&mut self, input: u8) -> Option<u8> {
            let c = self.target_strafe_context(input);
            crate::target_strafe_bridge::input_event(self, c)
        }
    }
    impl model::World for MotionWorld<'_, '_, '_> {
        fn collision_free(&mut self, p: V) -> bool {
            self.no_player_collision(self.target_strafe_box(p, 0.))
                .unwrap_or(false)
        }
        fn edge_safe(&mut self, p: V, fall: f32) -> bool {
            let p = V(p.0, p.1.floor() - f64::from(fall), p.2);
            !self
                .no_player_collision(self.target_strafe_box(p, 0.05))
                .unwrap_or(true)
        }
        fn void_safe(&mut self, p: V, expand: f32) -> bool {
            if p.1 < -64. {
                return false;
            }
            let b = self.target_strafe_box(p, -f64::from(expand));
            let lo = b.min_position();
            let hi = b.max_position();
            let b = Aabb::new(Vec3::new(lo.x, -64., lo.z), hi);
            let context = self.context();
            horizon_world::block_collision_scan::BlockCollisionScan::any_collision(
                &mut self.collisions(),
                context,
                b,
                false,
                1_000_000,
            )
            .unwrap_or(false)
        }
        fn sees(&mut self, feet: V, b: B, max_points: i32) -> bool {
            let eye = feet.add(V(0., f64::from(self.player.dimensions.eye_height), 0.));
            if b.contains(eye) {
                return true;
            }
            let context = self.context();
            let mut ray = TargetStrafeRay {
                scene: self.collisions(),
                context,
            };
            math::projected_points_limit(eye, b, max_points)
                .into_iter()
                .any(|to| {
                    clip_world(
                        &mut ray,
                        point(eye),
                        point(to),
                        RayBlockMode::Collider,
                        RayFluidMode::None,
                        4096,
                    )
                    .is_ok_and(|r| r.miss)
                })
        }
    }
}
