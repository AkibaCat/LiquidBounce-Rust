extern crate self as liquidbounce_behaviors;
#[path="__SMART_PATH__"]pub mod smart_eat;
#[path="config-fixture.rs"]mod config;
use smart_eat::*;
fn stack(slot:Slot,item:&str,nutrition:Option<i32>)->Stack {Stack {slot,item:item.into(),food:nutrition.map(|nutrition|Food {nutrition,can_always_eat:false}),instant_health:false}}
fn context()->Context {Context {health:20.,hunger:14,can_eat:true,server_slot:0,main_mining_tool:true,using_food_or_drink:false,stacks:vec![stack(Slot::Hotbar(1),"minecraft:bread",Some(5))]}}
#[test]fn all_original_defaults_and_distinct_slot_owners() {let m=SmartEat::default();assert_eq!((m.swap_back_delay,m.combat_pause_time,m.min_hunger,m.render_offset),(5,0,15,40));assert_eq!((m.prefer_gapple_health,m.prefer_notch_health,m.prefer_health_pot_health),(9.,2.,12.));assert!(m.silent_offhand&&m.auto_eat&&m.render_slot&&!m.not_during_combat);assert_ne!(SILENT_OFFHAND_REQUESTER,AUTO_EAT_REQUESTER);assert_eq!(Slot::Offhand.native(),-1);}
#[test]fn nutrition_and_actual_maxwith_distance_order() {let m=SmartEat::default();let mut c=context();c.server_slot=4;c.stacks=vec![stack(Slot::Offhand,"apple",Some(5)),stack(Slot::Hotbar(4),"bread",Some(5)),stack(Slot::Hotbar(0),"bread",Some(5)),stack(Slot::Hotbar(8),"bread",Some(5))];assert_eq!(m.best_food(&c).unwrap().slot,Slot::Hotbar(0));c.stacks[3].food.as_mut().unwrap().nutrition=6;assert_eq!(m.best_food(&c).unwrap().slot,Slot::Hotbar(8));c.stacks.truncate(2);assert_eq!(m.best_food(&c).unwrap().slot,Slot::Offhand);}
#[test]fn urgent_health_uses_threshold_integer_inverse_then_nutrition() {let m=SmartEat::default();let mut c=context();c.health=1.;c.stacks.extend([stack(Slot::Hotbar(2),"minecraft:golden_apple",Some(4)),stack(Slot::Hotbar(3),"minecraft:enchanted_golden_apple",Some(4)),Stack{instant_health:true,..stack(Slot::Offhand,"minecraft:potion",None)}]);assert_eq!(m.best_food(&c).unwrap().slot,Slot::Hotbar(3));c.health=5.;assert_eq!(m.best_food(&c).unwrap().slot,Slot::Hotbar(2));c.health=10.;assert_eq!(m.best_food(&c).unwrap().slot,Slot::Offhand);c.health=13.;assert_eq!(m.best_food(&c).unwrap().slot,Slot::Hotbar(1));}
#[test]fn potion_branch_has_no_custom_food_fallback_at_threshold() {let m=SmartEat::default();let mut c=context();c.stacks=vec![stack(Slot::Offhand,"minecraft:potion",Some(99))];c.health=12.;assert!(m.best_food(&c).is_none());c.health=12.01;assert_eq!(m.best_food(&c).unwrap().slot,Slot::Offhand);c.stacks[0].instant_health=true;c.health=12.;assert_eq!(m.best_food(&c).unwrap().slot,Slot::Offhand);}
#[test]fn splash_and_lingering_potions_are_not_drinkable_health_candidates() {let m=SmartEat::default();let mut c=context();c.health=1.;c.stacks=vec![Stack{instant_health:true,..stack(Slot::Offhand,"minecraft:splash_potion",None)},Stack{instant_health:true,..stack(Slot::Hotbar(0),"minecraft:lingering_potion",None)}];assert!(m.best_food(&c).is_none());}
#[test]fn floats_use_jvm_casts_and_nan_comparisons() {let mut m=SmartEat::default();let mut c=context();m.prefer_gapple_health=9.9;m.prefer_health_pot_health=9.1;c.health=9.;c.stacks=vec![stack(Slot::Hotbar(1),"minecraft:golden_apple",Some(4)),Stack{instant_health:true,..stack(Slot::Hotbar(2),"minecraft:potion",None)}];assert_eq!(m.best_food(&c).unwrap().slot,Slot::Hotbar(1));c.health=f32::NAN;assert_eq!(m.best_food(&c).unwrap().slot,Slot::Hotbar(1));}
#[test]fn pass_checks_food_hunger_combat_and_main_mining_tool() {let mut m=SmartEat::default();let mut c=context();assert_eq!(m.interacted_pass(true,false,&c).selection.unwrap().ticks,5);m.swap_back_delay=1;assert_eq!(m.interacted_pass(true,false,&c).selection.unwrap().ticks,5);assert_eq!(m.interacted_pass(true,false,&c).pause_combat,Some(0));assert_eq!(m.interacted_pass(false,false,&c),Effects::default());c.can_eat=false;assert_eq!(m.interacted_pass(true,false,&c),Effects::default());c.stacks[0].food.as_mut().unwrap().can_always_eat=true;assert!(m.interacted_pass(true,false,&c).selection.is_some());m.not_during_combat=true;assert_eq!(m.interacted_pass(true,true,&c),Effects::default());c.main_mining_tool=false;assert_eq!(m.interacted_pass(true,false,&c),Effects::default());}
#[test]fn healing_potions_bypass_hunger_gate_and_offhand_selection_is_retained() {let m=SmartEat::default();let mut c=context();c.health=10.;c.can_eat=false;c.stacks=vec![Stack{instant_health:true,..stack(Slot::Offhand,"minecraft:potion",None)}];assert_eq!(m.interacted_pass(true,false,&c).selection.unwrap().slot,Slot::Offhand);}
#[test]fn offhand_tick_refreshes_only_its_owner_and_eat_or_drink() {let mut m=SmartEat::default();m.swap_back_delay=1;let mut c=context();c.using_food_or_drink=true;c.server_slot=4;assert_eq!(m.silent_offhand_tick(true,Some(SILENT_OFFHAND_REQUESTER),&c).selection,Some(Selection{requester:SILENT_OFFHAND_REQUESTER,slot:Slot::Hotbar(4),ticks:1}));for owner in [None,Some(AUTO_EAT_REQUESTER)] {assert_eq!(m.silent_offhand_tick(true,owner,&c),Effects::default());}c.using_food_or_drink=false;assert_eq!(m.silent_offhand_tick(true,Some(SILENT_OFFHAND_REQUESTER),&c),Effects::default());}
#[test]fn autoeat_first_eat_is_next_tick_and_equality_keeps_waiting() {let mut m=SmartEat::default();let mut c=context();assert_eq!(m.start_run(true,false,&c),Effects {pause_combat:Some(0),selection:None});assert!(m.waiting()&&!m.force_use_key());assert_eq!(m.start_run(true,false,&c),Effects::default());c.hunger=15;assert!(m.advance_waiter(&c).selection.is_some());assert!(m.waiting()&&m.force_use_key());c.hunger=16;assert!(m.advance_waiter(&c).selection.is_some());assert!(!m.waiting()&&!m.force_use_key());}
#[test]fn autoeat_checks_combat_only_at_entry_and_no_food_does_not_end_wait() {let mut m=SmartEat::default();m.not_during_combat=true;let mut c=context();assert_eq!(m.start_run(true,true,&c),Effects::default());assert!(m.start_run(true,false,&c).pause_combat.is_some());c.stacks.clear();assert_eq!(m.advance_waiter(&c),Effects::default());assert!(m.waiting()&&!m.force_use_key());c.stacks.push(stack(Slot::Hotbar(7),"bread",Some(5)));assert!(m.advance_waiter(&c).selection.is_some());assert!(m.waiting()&&m.force_use_key());}
#[test]fn cancellation_preserves_force_flag_and_reenable_forces_until_successful_run_finishes() {let mut m=SmartEat::default();let mut c=context();m.start_run(true,false,&c);m.advance_waiter(&c);m.cancel_auto_eat();assert!(!m.waiting()&&m.force_use_key());let mut pressed=false;assert_eq!(m.use_key(false,&mut pressed),Effects::default());assert!(!pressed);assert_eq!(m.use_key(true,&mut pressed).pause_combat,Some(0));assert!(pressed);c.hunger=16;assert_eq!(m.start_run(true,false,&c),Effects::default());assert!(m.force_use_key());c.hunger=14;m.start_run(true,false,&c);c.hunger=16;m.advance_waiter(&c);assert!(!m.force_use_key());}
#[test]fn each_key_query_pauses_but_does_not_mutate_stored_physical_key() {let mut m=SmartEat::default();m.combat_pause_time=8;let c=context();m.start_run(true,false,&c);m.advance_waiter(&c);for _ in 0..3 {let physical=false;let mut pressed=physical;assert_eq!(m.use_key(true,&mut pressed).pause_combat,Some(8));assert!(pressed&&!physical);}m.auto_eat=false;let mut pressed=false;assert_eq!(m.use_key(true,&mut pressed),Effects::default());}
#[test]fn overlay_uses_integer_center_and_original_draw_coordinates() {let mut m=SmartEat::default();let c=context();assert_eq!(render_positions(321,240,40),([3,221],[0,217,29,24]));assert_eq!(render_positions(320,240,40),render_positions(321,240,40));assert!(m.render_food(true,&c).is_some());m.render_slot=false;assert!(m.render_food(true,&c).is_none());m.render_slot=true;m.silent_offhand=false;assert!(m.render_food(true,&c).is_none());}
#[test]fn emergency_apple_without_food_aborts_estimator_and_waiter_but_preserves_force() {let mut m=SmartEat::default();let mut c=context();m.start_run(true,false,&c);m.advance_waiter(&c);assert!(m.force_use_key());c.health=1.;c.stacks.push(stack(Slot::Hotbar(2),"minecraft:golden_apple",None));assert_eq!(m.try_best_food(&c),Err(FoodError::EmergencyAppleWithoutFood));assert_eq!(m.interacted_pass(true,false,&c),Effects::default());assert_eq!(m.advance_waiter(&c),Effects::default());assert!(!m.waiting()&&m.force_use_key());}

#[test]fn autoeat_fifo_callback_identity_survives_cancel_and_replacement() {
    let mut m=SmartEat::default();let mut c=context();
    m.start_run(true,false,&c);let first=m.take_wait_registrations()[0];
    assert!(m.take_wait_registrations().is_empty());
    assert!(m.advance_callback(first,true,Some(&c)).0);
    assert!(m.waiting()&&m.force_use_key());
    m.cancel_auto_eat();m.start_run(true,false,&c);
    let second=m.take_wait_registrations()[0];assert_ne!(first,second);
    assert_eq!(m.advance_callback(first,true,None),(false,Effects::default()));
    assert!(m.waiting());
    c.stacks[0].slot=Slot::Hotbar(8);
    let (keep,effects)=m.advance_callback(second,true,Some(&c));
    assert!(keep);assert_eq!(effects.selection.unwrap().slot,Slot::Hotbar(8));
    assert_eq!(m.advance_callback(second,true,None),(false,Effects::default()));
    assert!(!m.waiting()&&m.force_use_key());
    m.start_run(true,false,&c);let third=m.take_wait_registrations()[0];
    assert_ne!(second,third);c.hunger=16;
    assert!(!m.advance_callback(third,true,Some(&c)).0);
    assert!(!m.waiting()&&!m.force_use_key());
}

#[test]fn autoeat_tickuntil_checks_listener_running_only_when_continuation_resumes() {
    let mut m=SmartEat::default();let mut c=context();
    m.start_run(true,false,&c);let callback=m.take_wait_registrations()[0];
    let (keep,effects)=m.advance_callback(callback,false,Some(&c));
    assert!(keep&&effects.selection.is_some()&&m.force_use_key());
    c.hunger=16;
    let (keep,effects)=m.advance_callback(callback,false,Some(&c));
    assert!(!keep&&effects.selection.is_some());
    assert!(!m.waiting()&&m.force_use_key());
}
