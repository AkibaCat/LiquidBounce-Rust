// SPDX-License-Identifier: GPL-3.0-or-later
// Prepared for the coordinator's next centralized Rust batch; not run alone.
#[path = "../liquidbounce-behaviors/src/scaffold_clutch.rs"]
mod clutch;
use clutch::*;
use std::collections::BTreeSet;

#[derive(Default)]
struct World {
    solid: BTreeSet<BlockPos>,
    searches: Vec<(BlockPos, Vec<BlockPos>, Position, Position)>,
    clickable: bool,
    click_queries: usize,
    fail_searches: usize,
}
impl Host for World {
    type Target = BlockPos;
    fn can_stand_on(&mut self, p: BlockPos) -> bool { self.solid.contains(&p) }
    fn find_target(&mut self, s: Search<'_>) -> Option<BlockPos> {
        self.searches.push((s.below, s.offsets.to_vec(), s.from, s.priority));
        if self.fail_searches > 0 { self.fail_searches -= 1; None } else { Some(s.below) }
    }
    fn can_click(&mut self, _: &BlockPos) -> bool { self.click_queries += 1; self.clickable }
}
fn main() {
    let current = [0.4, 10.8, 1.2];
    let fall = [[4.0,10.2,6.0], [5.0,9.9,6.0], [6.0,8.9,6.0]];
    let offsets = [[0,0,0], [1,0,0], [0,-1,0]];
    let mut world = World::default();
    world.solid.insert([1,8,1]);
    assert_eq!(target(&mut world, current, &fall, 1.62, 4.5, &offsets), Some([0,9,1]));
    assert_eq!(world.searches[0].1, vec![[0,0,0], [0,-1,0]]);
    assert_eq!(world.searches[0].2, current);
    assert_eq!(world.searches[0].3, [0.4,9.5,1.2]);
    world.solid.insert([0,9,1]);
    let before = world.searches.len();
    assert_eq!(target(&mut world, current, &fall, 1.62, 4.5, &offsets), None);
    assert_eq!(world.searches.len(), before);
    let mut world = World::default();
    assert_eq!(target(&mut world, current, &fall, 1.62, 2.0, &offsets), None);
    assert!(world.searches.is_empty());
    let mut world = World { fail_searches:1, ..World::default() };
    assert_eq!(target(&mut world, current, &fall, 1.62, 4.5, &offsets), Some([0,8,1]));
    assert_eq!(world.searches.len(), 2);
    let mut world = World::default();
    assert_eq!(find_placement_target(&mut world, current, &fall, 1.62, 4.5, &offsets, false), Some([0,9,1]));
    assert_eq!(world.click_queries, 0);
    let mut world = World { clickable:true, ..World::default() };
    assert_eq!(find_placement_target(&mut world, current, &fall, 1.62, 4.5, &offsets, true), Some([0,9,1]));
    assert_eq!(world.searches.len(), 1);
    let mut world = World::default();
    assert_eq!(find_placement_target(&mut world, current, &fall, 1.62, 4.5, &offsets, true), Some([4,9,6]));
    assert_eq!(world.click_queries, 1);
    assert_eq!(world.searches[1].2, fall[0]);
    let mut world = World::default();
    assert_eq!(find_placement_target(&mut world, current, &[], 1.62, 4.5, &offsets, true), None);
}
