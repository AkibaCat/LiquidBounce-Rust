// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::blocks::Fluid;
use horizon_client::collisions::SceneCollisions;
use horizon_core::geometry::Direction;
use horizon_world::{
    block_support::{BlockSupportRegistry, SupportType},
    registry::BlockTableFile,
};
use std::{cell::RefCell, path::PathBuf};
type Support = (PathBuf, usize, Option<BlockSupportRegistry<'static>>);
thread_local! { static SUPPORT: RefCell<Option<Support>> = const { RefCell::new(None) }; }
impl Game<'_> {
    pub(super) fn clip_context(&self) -> Option<String> {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() {
            return None;
        }
        let p = self
            .player
            .as_ref()
            .filter(|p| p.native_client_queries_allowed())?;
        use horizon_client::actions::Action;
        Some(serde_json::json!({"position":[p.position.x,p.position.y,p.position.z],"yaw":self.yaw,
            "collision":p.collision.horizontal,"keys":([Action::Forward,Action::Back,Action::Left,Action::Right,Action::Sneak,Action::Jump].map(|key|self.keys.held(key)))}).to_string())
    }
    pub(super) fn clip_snap(&mut self, position: [f64; 3], reset: bool) {
        if self.clip_context().is_none() || !position.iter().all(|v| v.is_finite()) {
            return;
        }
        let at = horizon_core::geometry::Vec3::new(position[0], position[1], position[2]);
        if let Some(p) = self.player.as_mut() {
            p.position = at;
            if reset {
                p.velocity = horizon_core::geometry::Vec3::ZERO;
            }
        }
        self.previous_position = at;
    }
    pub(super) fn clip_target(
        &self,
        direction: i32,
        length: i32,
        liquids: u32,
        stand: bool,
    ) -> Option<[f64; 3]> {
        if !(0..6).contains(&direction) || self.clip_context().is_none() {
            return None;
        }
        let p = self.player.as_ref()?;
        let start = [
            p.position.x.floor() as i32,
            p.position.y.floor() as i32,
            p.position.z.floor() as i32,
        ];
        let registry = self.res.collision().registry();
        SUPPORT.with(|cache| {
            let mut cache = cache.borrow_mut();
            let key = registry as *const _ as usize;
            if cache
                .as_ref()
                .is_none_or(|(root, id, _)| *root != self.options.resources || *id != key)
            {
                let support = (|| {
                    let file = BlockTableFile::open(
                        &self.options.resources.join("cached-support.nbt"),
                        registry,
                    )
                    .ok()?;
                    BlockSupportRegistry::from_table(file.table("support").ok()?).ok()
                })();
                *cache = Some((self.options.resources.clone(), key, support));
            }
            let support = cache.as_ref().and_then(|(_, _, s)| s.as_ref());
            let target = super::client_clip_logic::find_target(
                start,
                direction as usize,
                length,
                |pos, need_stand| {
                    let loaded =
                        |p: [i32; 3]| self.world.chunks.contains_key(&(p[0] >> 4, p[2] >> 4));
                    let above = [pos[0], pos[1].wrapping_add(1), pos[2]];
                    if !loaded(pos) || !loaded(above) {
                        return false;
                    }
                    if stand && need_stand {
                        let below = [pos[0], pos[1].wrapping_sub(1), pos[2]];
                        if !loaded(below) {
                            return false;
                        }
                        let Some(support) = support else {
                            return false;
                        };
                        let mut world = SceneCollisions {
                            world: &self.world,
                            binding: self.collision,
                            subject: None,
                        };
                        if !support
                            .is_face_sturdy(
                                &mut world,
                                self.world.block(below[0], below[1], below[2]),
                                below,
                                Direction::Up,
                                SupportType::Center,
                            )
                            .unwrap_or(false)
                        {
                            return false;
                        }
                    }
                    let state = self.world.block(pos[0], pos[1], pos[2]);
                    let above = self.world.block(above[0], above[1], above[2]);
                    let facts = self.res.facts().get(state);
                    let top = self.res.facts().get(above);
                    let air = |id| self.res.facts().is_air(id);
                    (air(state) && air(above))
                        || ((liquids & 1 != 0 && facts.fluid == Fluid::Water)
                            || (liquids & 2 != 0 && facts.fluid == Fluid::Lava))
                            && (air(above) || facts.fluid == top.fluid)
                },
            );
            target.map(|[x, y, z]| [f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5])
        })
    }
}
