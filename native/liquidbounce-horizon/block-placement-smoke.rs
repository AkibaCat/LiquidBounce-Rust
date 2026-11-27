#[path = "block-placement-kernel.rs"]
mod placement;
use placement::*;
use std::collections::{BTreeMap, BTreeSet};

const CUBE: [f64; 6] = [0., 0., 0., 1., 1., 1.];
#[derive(Default)]
struct World {
    shapes: BTreeMap<BlockPos, Vec<[f64; 6]>>,
    solid: BTreeSet<BlockPos>,
    priorities: BTreeMap<BlockPos, f64>,
    inside: Vec<bool>,
    queried: Vec<BlockPos>,
}
impl Host for World {
    fn holds_solid(&mut self, p: BlockPos) -> bool {
        self.solid.contains(&p)
    }
    fn outline(&mut self, p: BlockPos) -> Vec<[f64; 6]> {
        self.queried.push(p);
        self.shapes.get(&p).cloned().unwrap_or_default()
    }
    fn priority(&mut self, p: BlockPos) -> f64 {
        self.priorities.get(&p).copied().unwrap_or(0.)
    }
    fn sample(&mut self, face: Face, _: BlockPos, _: Location) -> Option<Position> {
        Some(face.center())
    }
    fn resolves_at(
        &mut self,
        clicked: BlockPos,
        direction: Direction,
        _: Position,
        inside: bool,
        target: BlockPos,
    ) -> bool {
        self.inside.push(inside);
        std::array::from_fn::<_, 3, _>(|i| clicked[i] + direction[i]) == target
    }
}
fn options(eye: Position) -> Options<'static> {
    Options {
        location: Location {
            position: eye,
            eye_height: 0.,
            rotation: [0.; 2],
            moving: false,
        },
        offsets: &[[0; 3]],
        range: 6.,
        consider_facing_away: false,
    }
}
fn cubes(positions: &[BlockPos]) -> World {
    World {
        shapes: positions.iter().map(|&p| (p, vec![CUBE])).collect(),
        ..Default::default()
    }
}

#[test]
fn placement_smoke_facing_beats_nearest_sample() {
    let mut world = cubes(&[[0, -1, 0], [-1, 0, 0]]);
    let target = find(&mut world, [0; 3], &options([1.5, 3., 0.5])).unwrap();
    assert_eq!(target.interacted, [0, -1, 0]);
    assert_eq!(target.direction, [0, 1, 0]);
    // The side sample is nearer, but the top face points more directly at eye.
    assert!((1.5_f64.powi(2) + 2.5_f64.powi(2)).sqrt() < (1.0_f64 + 9.0_f64).sqrt());
}

#[test]
fn placement_smoke_offsets_are_stable_and_descending() {
    let mut world = cubes(&[[0, -1, 0], [1, -1, 0]]);
    let mut o = options([0.5, 3., 0.5]);
    o.offsets = &[[1, 0, 0], [0, 0, 0]];
    assert_eq!(find(&mut world, [0; 3], &o).unwrap().placed, [1, 0, 0]);
    world.priorities.insert([0; 3], 1.);
    assert_eq!(find(&mut world, [0; 3], &o).unwrap().placed, [0; 3]);
}

#[test]
fn placement_smoke_solid_aborts_but_partial_keeps_offsets() {
    let mut world = cubes(&[[1, -1, 0]]);
    world.solid.insert([0; 3]);
    let mut o = options([0.5, 3., 0.5]);
    o.offsets = &[[0; 3], [1, 0, 0]];
    assert!(find(&mut world, [0; 3], &o).is_none());
    assert!(world.queried.is_empty());
    world.solid.clear();
    world.shapes.insert([0; 3], vec![[0., 0., 0., 1., 0.5, 1.]]);
    assert_eq!(find(&mut world, [0; 3], &o).unwrap().placed, [1, 0, 0]);
}

#[test]
fn placement_smoke_box_ranking_retains_minimum_face_y() {
    let mut world = World::default();
    world.shapes.insert(
        [-1, 0, 0],
        vec![[0., 0., 0., 0.75, 0.5, 1.], [0., 0.5, 0., 1., 1., 1.]],
    );
    let target = find(&mut world, [0; 3], &options([2., 1., 0.5])).unwrap();
    assert_eq!(target.direction, [1, 0, 0]);
    assert_eq!(target.point, [0., 0.75, 0.5]);
    assert_eq!(target.min_y, 0.5);
}

#[test]
fn placement_smoke_inside_uses_predicted_eye_and_half_open_boxes() {
    let mut world = cubes(&[[-1, 0, 0]]);
    let mut o = options([-0.25, 0.25, 0.25]);
    o.consider_facing_away = true;
    assert!(find(&mut world, [0; 3], &o).is_some());
    assert!(world.inside.iter().any(|v| *v));
    world.inside.clear();
    o.location.position = [0., 0.25, 0.25];
    assert!(find(&mut world, [0; 3], &o).is_some());
    assert!(world.inside.iter().all(|v| !*v));
}

#[test]
fn placement_smoke_range_boundary_and_backface_policy() {
    let mut world = cubes(&[[-1, 0, 0]]);
    let mut o = options([1., 0.5, 0.5]);
    o.range = 1.;
    assert!(find(&mut world, [0; 3], &o).is_some());
    o.range = 1. - f64::EPSILON;
    assert!(find(&mut world, [0; 3], &o).is_none());
    o.range = 6.;
    o.location.position = [-2., 0.5, 0.5];
    assert!(find(&mut world, [0; 3], &o).is_none());
    o.consider_facing_away = true;
    assert!(find(&mut world, [0; 3], &o).is_some());
}

#[test]
fn placement_smoke_hit_epsilon_and_direction_are_independent() {
    let target = Target {
        interacted: [2, 3, 4],
        placed: [2, 4, 4],
        direction: [0, 1, 0],
        point: [2.5, 4., 4.5],
        min_y: 4.,
        rotation: [0.; 2],
    };
    assert!(target.matches(true, [2, 3, 4], [0, 1, 0], [2.5, 4. - 0.5e-7, 4.5]));
    assert!(!target.matches(true, [2, 3, 4], [0, 1, 0], [2.5, 4. - 2e-7, 4.5]));
    assert!(!target.matches(true, [2, 3, 4], [1, 0, 0], target.point));
    assert!(!target.matches(false, [2, 3, 4], [0, 1, 0], target.point));
}

#[test]
fn placement_smoke_offset_sets_have_source_order() {
    for (radius, count) in [(1, 18), (2, 50), (4, 162)] {
        let list = offsets(radius);
        assert_eq!(list.len(), count);
        assert_eq!(
            &list[..5],
            &[[0, 0, 0], [0, -1, 0], [-1, 0, 0], [0, 0, -1], [0, 0, 1]]
        );
        assert_eq!(list.iter().copied().collect::<BTreeSet<_>>().len(), count);
    }
}

#[test]
fn placement_smoke_line_priority_handles_parallel_and_empty_shapes() {
    assert_eq!(
        line_distance_squared(&[CUBE], [0; 3], [-2., 0.5, 0.5], [1., 0., 0.]),
        0.
    );
    assert_eq!(
        line_distance_squared(&[CUBE], [0; 3], [-2., 2., 0.5], [1., 0., 0.]),
        1.
    );
    assert_eq!(
        line_distance_squared(&[], [0; 3], [-2., 2., 0.5], [1., 0., 0.]),
        2.25
    );
    assert_eq!(distance_squared(&[CUBE], [3, 0, 0], [1., 0.5, 0.5]), 4.);
    assert_eq!(distance_squared(&[], [0; 3], [0.5, 2., 0.5]), 2.25);
    assert_eq!(looking_at([0.5, 1., 1.5], [0.5, 1., 0.5]), [0., -0.]);
}
