// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual BlockPlaceContext/outline/ray adapter shared by Scaffold and liquid use.
use super::placement_core as core;
use super::{Game, Hit, ItemStack};
pub(super) use core::{Face, Location, Target};
use horizon_core::{
    geometry::{Aabb, Direction, Vec3, traverse_blocks},
    position::BlockPos,
};
use horizon_world::{collision_context::EntityContextKind, level::PlaceContext};

pub(super) fn looking_at(point: [f64; 3], from: [f64; 3]) -> [f32; 2] {
    use horizon_core::{geometry::trig::RAD_TO_DEG, jmath::atan2};
    let [x, y, z] = std::array::from_fn(|i| point[i] - from[i]);
    let wrap = |a: f32| {
        let a = a % 360.0;
        if a >= 180.0 {
            a - 360.0
        } else if a < -180.0 {
            a + 360.0
        } else {
            a
        }
    };
    [
        wrap((atan2(z, x) * f64::from(RAD_TO_DEG)) as f32 - 90.0),
        wrap(-((atan2(y, (x * x + z * z).sqrt()) * f64::from(RAD_TO_DEG)) as f32)),
    ]
}
/// The original block result's inside bit must survive direct placement checks.
#[derive(Clone, Copy)]
pub(super) struct Click {
    pub hit: Hit,
    pub inside: bool,
}

pub(super) type Sample<'a> =
    dyn FnMut(&Game<'_>, Face, [i32; 3], Location) -> Option<[f64; 3]> + 'a;
pub(super) enum FaceFactory<'a> {
    Center,
    Custom(&'a mut Sample<'a>),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Priority {
    Position([f64; 3]),
    Line {
        position: [f64; 3],
        direction: [f64; 3],
    },
}
pub(super) struct Options<'a> {
    pub stack: &'a ItemStack,
    pub location: Location,
    pub offsets: Vec<[i32; 3]>,
    pub priority: Priority,
    pub face: FaceFactory<'a>,
    pub range: f64,
    pub consider_facing_away: bool,
}
impl<'a> Options<'a> {
    pub fn center(stack: &'a ItemStack, location: Location) -> Self {
        Self {
            stack,
            location,
            offsets: vec![[0; 3]],
            priority: Priority::Position(location.position),
            face: FaceFactory::Center,
            range: f64::MAX,
            consider_facing_away: false,
        }
    }
}
pub(super) fn capture_location(
    game: &Game<'_>,
    position: Option<[f64; 3]>,
    rotation: [f32; 2],
) -> Option<Location> {
    let p = game.player.as_ref()?;
    let keys = horizon_client::reverse_step_bridge::keys();
    Some(Location {
        position: position.unwrap_or([p.position.x, p.position.y, p.position.z]),
        eye_height: p.dimensions.eye_height,
        rotation,
        moving: keys[..4].iter().any(|v| *v),
    })
}
pub(super) fn offsets(radius: i32) -> Vec<[i32; 3]> {
    core::offsets(radius)
}
pub(super) fn planned_hit(target: &Target) -> Hit {
    Hit {
        pos: target.interacted,
        face: target.direction,
        point: target.point,
    }
}
pub(super) fn planned_click(target: &Target) -> Click {
    Click {
        hit: planned_hit(target),
        inside: false,
    }
}
fn block(pos: [i32; 3]) -> BlockPos {
    BlockPos::new(pos[0], pos[1], pos[2])
}
fn vec(pos: [f64; 3]) -> Vec3 {
    Vec3::new(pos[0], pos[1], pos[2])
}
fn direction(normal: [i32; 3]) -> Direction {
    Direction::ALL
        .into_iter()
        .find(|d| d.normal() == normal)
        .expect("placement direction")
}
pub(super) fn holds_solid(game: &mut Game<'_>, pos: [i32; 3]) -> bool {
    if !game.world.chunks.contains_key(&(pos[0] >> 4, pos[2] >> 4)) {
        return false;
    }
    game.with_level(|b, l, t| b.liquidbounce_placement_holds_solid(l, t, block(pos)))
}

/// Shape coordinates include the block offset but stay relative to the block.
/// Target finding always uses the actual local player, even when the camera is
/// another entity. Comparator priority uses the no-entity outline instead.
pub(super) fn outline(game: &Game<'_>, pos: [i32; 3], player: bool) -> Vec<[f64; 6]> {
    if !game.world.chunks.contains_key(&(pos[0] >> 4, pos[2] >> 4)) {
        return Vec::new();
    }
    let state = game.world.block(pos[0], pos[1], pos[2]);
    let facts = game.res.facts().get(state);
    let held = game
        .inventory
        .get(game.client_hotbar_slot())
        .and_then(Option::as_ref);
    let special = if matches!(&*facts.name, "minecraft:light" | "minecraft:scaffolding") {
        (player && held.is_some_and(|s| s.count > 0 && s.item == facts.name))
            .then_some([0., 0., 0., 1., 1., 1.])
    } else if facts.name.ends_with("shulker_box") {
        game.context_outline(pos, state)
    } else {
        None
    };
    let kind = if player {
        EntityContextKind::Player
    } else {
        EntityContextKind::None
    };
    let declared = game
        .collision
        .cached_shapes()
        .context_outline_shape(state, kind);
    let mut boxes = if let Some(bounds) = special {
        vec![bounds]
    } else if let Some(shape) = declared {
        shape
            .boxes()
            .iter()
            .map(|b| {
                let a = b.min_position();
                let z = b.max_position();
                [a.x, a.y, a.z, z.x, z.y, z.z]
            })
            .collect()
    } else {
        facts.outline.to_vec()
    };
    if facts.outline_offset {
        let offset = game
            .pack
            .context
            .tables
            .offset(state, pos[0], pos[2])
            .unwrap_or([0.; 3]);
        for bounds in &mut boxes {
            for i in 0..6 {
                bounds[i] += offset[i % 3];
            }
        }
    }
    boxes
}
struct Adapter<'g, 'r, 'o, 's> {
    game: &'g mut Game<'r>,
    options: &'o mut Options<'s>,
}
impl core::Host for Adapter<'_, '_, '_, '_> {
    fn looking_at(&mut self, point: [f64; 3], eye: [f64; 3]) -> [f32; 2] {
        looking_at(point, eye)
    }
    fn holds_solid(&mut self, pos: [i32; 3]) -> bool {
        holds_solid(self.game, pos)
    }
    fn outline(&mut self, pos: [i32; 3]) -> Vec<[f64; 6]> {
        outline(self.game, pos, true)
    }
    fn priority(&mut self, pos: [i32; 3]) -> f64 {
        let boxes = outline(self.game, pos, false);
        match self.options.priority {
            Priority::Position(p) => -core::distance_squared(&boxes, pos, p),
            Priority::Line {
                position,
                direction,
            } => -core::line_distance_squared(&boxes, pos, position, direction),
        }
    }
    fn sample(&mut self, face: Face, pos: [i32; 3], location: Location) -> Option<[f64; 3]> {
        match &mut self.options.face {
            FaceFactory::Center => Some(face.center()),
            FaceFactory::Custom(f) => f(self.game, face, pos, location),
        }
    }
    fn resolves_at(
        &mut self,
        clicked: [i32; 3],
        normal: [i32; 3],
        point: [f64; 3],
        inside: bool,
        target: [i32; 3],
    ) -> bool {
        let p = self.game.player.as_ref().expect("placement player");
        let input = PlaceContext {
            clicked_pos: block(clicked),
            face: direction(normal),
            hit: vec(point),
            inside,
            yaw: p.yaw,
            pitch: p.pitch,
            sneaking: p.shift_key_down,
            item: self.options.stack.item.to_string(),
            can_use_game_master_blocks: self.game.can_use_game_master_blocks(),
        };
        self.game.with_level(|b, l, t| {
            b.liquidbounce_resolve_placement_context(l, t, &input)
                .is_some_and(|ctx| ctx.pos == block(target))
        })
    }
}
pub(super) fn find(
    game: &mut Game<'_>,
    pos: [i32; 3],
    options: &mut Options<'_>,
) -> Option<Target> {
    if game.player.is_none() {
        return None;
    }
    let offsets = options.offsets.clone();
    let core_options = core::Options {
        location: options.location,
        offsets: &offsets,
        range: options.range,
        consider_facing_away: options.consider_facing_away,
    };
    core::find(&mut Adapter { game, options }, pos, &core_options)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClickTrace {
    World,
    TargetShape,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FailedClick {
    Nothing,
    PlannedHit,
}
fn clip_player_outline(
    game: &Game<'_>,
    pos: [i32; 3],
    start: [f64; 3],
    view: [f64; 3],
    range: f64,
) -> Option<Click> {
    let boxes = outline(game, pos, true);
    let state = game.world.block(pos[0], pos[1], pos[2]);
    let shapes = boxes
        .iter()
        .map(|b| Aabb::new(Vec3::new(b[0], b[1], b[2]), Vec3::new(b[3], b[4], b[5])))
        .collect::<Vec<_>>();
    let cached = game.collision.cached_shapes();
    let fallback;
    let interactions = if let Some(shape) = cached.interaction_shape(state) {
        shape.boxes()
    } else {
        let facts = game.res.facts().get(state);
        let facing = game
            .pack
            .assets
            .registry
            .state(state)
            .and_then(|s| s.properties.get("facing"))
            .map(String::as_str);
        fallback = super::interaction_shapes::boxes(&facts.name, facing)
            .iter()
            .map(|b| Aabb::new(Vec3::new(b[0], b[1], b[2]), Vec3::new(b[3], b[4], b[5])))
            .collect::<Vec<_>>();
        &fallback
    };
    let hit = horizon_world::interaction_shapes::clip_with_interaction_override(
        shapes.iter().copied(),
        interactions.iter().copied(),
        vec(start),
        vec(std::array::from_fn(|i| start[i] + view[i] * range)),
        block(pos),
    )?;
    Some(Click {
        hit: Hit {
            pos,
            face: hit.direction.normal(),
            point: [hit.location.x, hit.location.y, hit.location.z],
        },
        inside: hit.inside,
    })
}
pub(super) fn raytrace_block(
    game: &Game<'_>,
    pos: [i32; 3],
    rotation: [f32; 2],
    range: f64,
) -> Option<Hit> {
    raytrace_block_click(game, pos, rotation, range).map(|c| c.hit)
}
pub(super) fn raytrace_block_click(
    game: &Game<'_>,
    pos: [i32; 3],
    rotation: [f32; 2],
    range: f64,
) -> Option<Click> {
    let start = game
        .camera_entity()
        .map(|c| {
            [
                c.position[0],
                c.position[1] + f64::from(c.eye_height),
                c.position[2],
            ]
        })
        .or_else(|| game.eye_position())?;
    let d = horizon_world::view_direction::view_vector(rotation[1], rotation[0]);
    clip_player_outline(game, pos, start, [d.x, d.y, d.z], range)
}
pub(super) fn trace_world(game: &Game<'_>, rotation: [f32; 2], range: f64) -> Option<Hit> {
    trace_world_click(game, rotation, range).map(|c| c.hit)
}
fn clip_declared_click(
    game: &Game<'_>,
    pos: [i32; 3],
    start: [f64; 3],
    view: [f64; 3],
    range: f64,
) -> Option<Click> {
    use std::borrow::Cow;
    let state = game.world.block(pos[0], pos[1], pos[2]);
    let facts = game.res.facts().get(state);
    let cached = game.collision.cached_shapes();
    let shape = cached.context_outline_shape(state, game.outline_context_kind());
    let boxes: Cow<'_, [Aabb]> = if let Some(shape) = shape {
        Cow::Borrowed(shape.boxes())
    } else if let Some(b) = game.context_outline(pos, state) {
        Cow::Owned(vec![Aabb::new(
            Vec3::new(b[0], b[1], b[2]),
            Vec3::new(b[3], b[4], b[5]),
        )])
    } else {
        Cow::Owned(
            facts
                .outline
                .iter()
                .map(|b| Aabb::new(Vec3::new(b[0], b[1], b[2]), Vec3::new(b[3], b[4], b[5])))
                .collect(),
        )
    };
    let offset = if facts.outline_offset {
        game.pack
            .context
            .tables
            .offset(state, pos[0], pos[2])
            .unwrap_or([0.0; 3])
    } else {
        [0.0; 3]
    };
    let offset = vec(offset);
    let interactions: Cow<'_, [Aabb]> = if let Some(shape) = cached.interaction_shape(state) {
        Cow::Borrowed(shape.boxes())
    } else {
        let facing = game
            .pack
            .assets
            .registry
            .state(state)
            .and_then(|s| s.properties.get("facing"))
            .map(String::as_str);
        Cow::Owned(
            super::interaction_shapes::boxes(&facts.name, facing)
                .into_iter()
                .map(|b| Aabb::new(Vec3::new(b[0], b[1], b[2]), Vec3::new(b[3], b[4], b[5])))
                .collect(),
        )
    };
    let hit = horizon_world::interaction_shapes::clip_with_interaction_override(
        boxes.iter().map(|b| b.move_by(offset)),
        interactions.iter().copied(),
        vec(start),
        vec(std::array::from_fn(|i| start[i] + view[i] * range)),
        block(pos),
    )?;
    Some(Click {
        hit: Hit {
            pos,
            face: hit.direction.normal(),
            point: [hit.location.x, hit.location.y, hit.location.z],
        },
        inside: hit.inside,
    })
}
pub(super) fn trace_world_click(game: &Game<'_>, rotation: [f32; 2], range: f64) -> Option<Click> {
    let p = game.player.as_ref()?;
    let start = [
        p.position.x,
        p.position.y + f64::from(p.dimensions.eye_height),
        p.position.z,
    ];
    let view = horizon_world::view_direction::view_vector(rotation[1], rotation[0]);
    let d = [view.x, view.y, view.z];
    let end = std::array::from_fn(|i| start[i] + d[i] * range);
    let max_visits =
        ((end[0] - start[0]).abs() + (end[1] - start[1]).abs() + (end[2] - start[2]).abs()).ceil()
            as usize;
    // The same declared outline as Game::clip_declared_outline, retaining inside.
    let visit = |pos: BlockPos| clip_declared_click(game, [pos.x, pos.y, pos.z], start, d, range);
    if let Some(targets) = game.client_ghost_hand_targets() {
        let selected = traverse_blocks(vec(start), vec(end), max_visits.saturating_add(8), |pos| {
            let state = game.world.block(pos.x, pos.y, pos.z);
            if targets.contains(&*game.res.facts().get(state).name) {
                visit(pos)
            } else {
                None
            }
        })
        .ok()
        .flatten();
        if selected.is_some() {
            return selected;
        }
    }
    traverse_blocks(vec(start), vec(end), max_visits.saturating_add(8), visit)
        .ok()
        .flatten()
}
pub(super) fn verify(
    game: &Game<'_>,
    target: &Target,
    rotation: [f32; 2],
    range: f64,
    trace: ClickTrace,
    on_failure: FailedClick,
) -> Option<Hit> {
    verify_click(game, target, rotation, range, trace, on_failure).map(|c| c.hit)
}
pub(super) fn verify_click(
    game: &Game<'_>,
    target: &Target,
    rotation: [f32; 2],
    range: f64,
    trace: ClickTrace,
    on_failure: FailedClick,
) -> Option<Click> {
    let hit = match trace {
        ClickTrace::World => trace_world_click(game, rotation, range),
        ClickTrace::TargetShape => raytrace_block_click(game, target.interacted, rotation, range),
    };
    hit.filter(|c| target.matches(true, c.hit.pos, c.hit.face, c.hit.point))
        .or_else(|| (on_failure == FailedClick::PlannedHit).then(|| planned_click(target)))
}
