// Planning usability smoke only; search-result differential is deferred.
struct PlanningSmokeSearch { terrain: () }
impl crate::navigation::owned_search::OwnedTerrainSearch<()> for PlanningSmokeSearch {
 type Route = Route<S>;
 fn terrain_mut(&mut self) -> &mut () { &mut self.terrain }
 fn advance(&mut self) -> crate::navigation::search::Outcome<Self::Route> { crate::navigation::search::Outcome::AlreadyThere }
 fn preview(&mut self) -> Option<Vec<Vec3>> { None }
}
struct PlanningSmokeExecutor;
impl crate::navigation::background::Executor for PlanningSmokeExecutor {
 fn execute(&self, task: Box<dyn FnOnce() + Send>) { task(); }
}
#[test]
fn planning_constructor_stuck_drop_and_reset_smoke() {
 use crate::navigation::walk_control::planning::Planning;
 let mut planning = Planning::<S, (), PlanningSmokeSearch>::new(
  Arc::new(PlanningSmokeExecutor), Arc::new(|terrain, _, _, _, _| PlanningSmokeSearch { terrain }));
 assert!(!planning.is_searching());
 assert!(!planning.failed());
 assert!(!planning.is_settled());
 assert!(planning.preview().is_none());
 planning.tick(Vec3::ZERO);
 let spot = BlockPos::new(1, 0, 0);
 planning.stuck_at(spot);
 assert_eq!(planning.diagnostics(), (0, true, true, false, None));
 let version = planning.stuck_spots().version();
 planning.stuck_at(spot);
 assert_eq!(planning.stuck_spots().version(), version);
 planning.drop_route();
 assert_eq!(planning.diagnostics(), (0, false, false, false, None));
 assert!(planning.stuck_spots().blocks.contains(&spot.packed()));
 planning.stuck_at(spot);
 planning.reset();
 assert_eq!(planning.diagnostics(), (0, false, false, false, None));
 planning.forget_stuck_spots();
 assert!(planning.stuck_spots().blocks.is_empty());
 planning.block_changed(spot);
 planning.level_changed();
 assert!(!planning.is_searching());
}
