#[test]
fn walking_runtime_owns_actions_planning_and_setbacks() {
 use crate::navigation::walking_runtime::{StablePlacementOrder,WalkingRuntime};
 let events=Arc::new(std::sync::Mutex::new(Vec::new()));
 let world=World{events:events.clone(),registry:Rc::new(RefCell::new(Registry::default())),data:Rc::new(RefCell::new(Data{min:-16,count:64,border_min:-100.0,border_max:100.0,legacy:false,floor_y:-1,floor_state:1,blocks:HashMap::new(),unknown:HashSet::new()}))};
 let w=WalkFixture::new(world);let mut meta=ActionMeta::default();
 let mut host=ActionBody{inner:TargetBody{player:&w.player,targets:&w.targets,sent:ViewRotation::ZERO},meta:&mut meta,script:&w.script,method:Bridging::Auto,risky:false,sneak_speed:0.3};
 let mut runtime=WalkingRuntime::<S,(),PlanningSmokeSearch,StablePlacementOrder>::new(Arc::new(PlanningSmokeExecutor),Arc::new(|terrain,_,_,_,_|PlanningSmokeSearch{terrain}),StablePlacementOrder::default());
 runtime.start_tick(&mut host);assert_eq!(runtime.actions.timing().0,1);
 let pos=BlockPos::new(1,0,0);assert!(!runtime.block_confirmed(&mut host,pos,&w.player.level.state_id(0)));
 runtime.actions.mine(&mut host,pos);assert!(runtime.actions.is_active());runtime.reset(&mut host);assert!(!runtime.actions.is_active());
 assert_eq!(&*events.lock().unwrap(), &["release"]);
 assert!(runtime.corrected(Vec3::new(0.1,0.0,0.0),1).is_empty());
 assert_eq!(runtime.corrected(Vec3::new(1.1,0.0,0.0),2),[BlockPos::new(0,0,0),pos]);
 assert!(runtime.corrected(Vec3::new(1.2,0.0,0.0),2).is_empty());
 assert!(runtime.planning.stuck_spots().blocks.contains(&pos.packed()));
 runtime.level_changed();assert!(runtime.planning.stuck_spots().blocks.is_empty());assert!(!runtime.planning.is_searching());
}
#[test]
fn walking_runtime_pending_order_preserves_bucket_identity_and_capacity() {
 use crate::navigation::walking_runtime::StablePlacementOrder;
 let mut order=StablePlacementOrder::default();let p=|x|BlockPos::new(x,0,0);
 for x in [16,0,1,-1,32,17]{order.inserted(p(x));}
 assert_eq!(order.keys(),[16,0,-1,32,1,17].map(p));
 let first=order.identity(p(16));order.inserted(p(16));assert_eq!(order.identity(p(16)),first);
 order.removed(p(0));assert_eq!(order.keys(),[16,-1,32,1,17].map(p));
 let capacity=order.capacity();order.clear();assert_eq!(order.capacity(),capacity);assert!(order.keys().is_empty());
 for i in 0..11{order.inserted(BlockPos::new(-31*i,i,0));}
 assert_eq!(order.capacity(),64);assert!(order.has_tree_bin());assert_eq!(order.keys().len(),11);
}
