// SPDX-License-Identifier: GPL-3.0-or-later
// Prepared for the coordinator's centralized Rust batch; not run alone.
#[path = "../liquidbounce-behaviors/src/scaffold_lifecycle.rs"]
mod lifecycle;
use lifecycle::*;

fn main() {
    let mut state = State::default();
    state.enabled(70);
    assert_eq!((state.placement_y, state.start_y, state.jumps), (69, 70, 2));
    state.tick_begin(71, true, false);
    assert_eq!((state.placement_y, state.jumps, state.was_towering), (70, 3, false));
    state.tick_begin(72, false, true);
    assert_eq!((state.start_y, state.jumps), (72, 2));
    assert!(state.is_towering(true, true));
    assert!(state.was_towering);
    assert!(!state.is_towering(false, true));
    assert!(state.was_towering);
    state.force_sneak = 3;
    state.reset();
    assert_eq!(state.force_sneak, 0);
    assert_eq!((state.placement_y, state.start_y, state.jumps, state.was_towering), (70, 72, 2, true));
    assert!(state.ledge_sneak(2));
    assert!(!state.ledge_sneak(1));
    assert!(state.take_forced_sneak());
    assert!(state.take_forced_sneak());
    assert!(!state.take_forced_sneak());

    let mut facts = PositionFacts { block:[2,75,4], moving:true, ..Default::default() };
    assert_eq!(state.targeted_position(facts, SameY::Off), [2,74,4]);
    assert_eq!(state.targeted_position(facts, SameY::On), [2,70,4]);
    assert_eq!(state.targeted_position(facts, SameY::JumpKey), [2,70,4]);
    facts.physical_jump = true;
    assert_eq!(state.targeted_position(facts, SameY::JumpKey), [2,74,4]);
    facts.velocity_y = 0.2;
    assert_eq!(state.targeted_position(facts, SameY::Falling), [2,74,4]);
    facts.velocity_y = 0.199999999;
    assert_eq!(state.targeted_position(facts, SameY::Falling), [2,70,4]);
    facts.velocity_y = -0.15233518685055708;
    assert_eq!(state.targeted_position(facts, SameY::Hypixel), [2,72,4]);
    assert_eq!(state.jumps, 0);
    assert_eq!(state.targeted_position(facts, SameY::Hypixel), [2,71,4]);
    facts.input_jump = true;
    facts.horizontal_collision = true;
    assert_eq!(state.targeted_position(facts, SameY::On), [2,74,4]);
    facts.horizontal_collision = false;
    facts.moving = false;
    assert_eq!(state.targeted_position(facts, SameY::On), [2,74,4]);

    assert_eq!(select_block(true,false,false,Some(8)), Selection {action:SlotAction::Select(8),has_main:true});
    assert_eq!(select_block(true,false,true,Some(8)), Selection {action:SlotAction::Reset,has_main:false});
    assert_eq!(select_block(false,false,false,Some(8)), Selection {action:SlotAction::Reset,has_main:false});
    assert_eq!(select_block(true,false,false,None), Selection {action:SlotAction::Reset,has_main:false});
    assert_eq!(hand(true,true),Some(Hand::Main));
    assert_eq!(hand(false,true),Some(Hand::Off));
    assert_eq!(hand(false,false),None);
    assert!(valid_crosshair([0.1,0.0,0.1],[0.0,0.0,0.0],[0,1,0],0.25));
    assert!(!valid_crosshair([0.1,0.0,1.0],[0.0,0.0,0.0],[1,0,0],0.25));
    assert!(valid_crosshair([0.25,0.0,0.0],[0.0,0.0,0.0],[-1,0,0],0.25));
    assert!(!valid_crosshair([1.0,0.0,0.1],[0.0,0.0,0.0],[0,0,-1],0.25));

    let base = Attempt { enabled:true,failed_only:false,suitable_hand:true,block_hit:true,
        can_place_on_face:false,same_y:SameY::Off,physical_jump:false,clicked_y:69,
        clicked_up:true,placement_y:69,player_block_y:70 };
    assert!(simulate_attempt(base));
    assert!(!simulate_attempt(Attempt { can_place_on_face:true,..base }));
    assert!(simulate_attempt(Attempt { can_place_on_face:true,clicked_up:false,..base }));
    assert!(!simulate_attempt(Attempt { clicked_y:70,..base }));
    assert!(!simulate_attempt(Attempt { enabled:false,..base }));
    assert!(!simulate_attempt(Attempt { suitable_hand:false,..base }));
    assert!(!simulate_attempt(Attempt { block_hit:false,..base }));
    assert!(!simulate_attempt(Attempt { failed_only:true,can_place_on_face:true,..base }));
    assert!(simulate_attempt(Attempt { failed_only:true,clicked_y:75,..base }));
    assert!(!simulate_attempt(Attempt { same_y:SameY::On,clicked_y:68,..base }));
    // Source deliberately tests physical_jump == true here, unlike SameY's target height.
    assert!(simulate_attempt(Attempt { same_y:SameY::JumpKey,clicked_y:68,..base }));
    assert!(!simulate_attempt(Attempt { same_y:SameY::JumpKey,physical_jump:true,clicked_y:68,..base }));
}
