// SPDX-License-Identifier: GPL-3.0-or-later
use crate::elytra_recast::{Chest, ElytraRecast, RecastWorld};
use std::cell::RefCell;

fn oracle() -> &'static serde_json::Value {
    static VALUE: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    VALUE.get_or_init(|| serde_json::from_str(include_str!("oracle.json")).unwrap())
}
struct World {
    bits: u32,
    glide: bool,
    delay: i32,
    trace: RefCell<Vec<&'static str>>,
    fail_climb: bool,
}
impl World {
    fn new(bits: u32) -> Self {
        Self {
            bits,
            glide: bits & 2 != 0,
            delay: 7,
            trace: RefCell::new(Vec::new()),
            fail_climb: false,
        }
    }
    fn bit(&self, index: u32, label: &'static str) -> bool {
        self.trace.borrow_mut().push(label);
        self.bits & index != 0
    }
}
impl RecastWorld for World {
    type Error = &'static str;
    fn chest(&mut self) -> Chest {
        self.trace.borrow_mut().push("chest");
        Chest {
            glider: self.bits & 128 != 0,
            next_damage_will_break: self.bits & 256 != 0,
        }
    }
    fn flying(&self) -> bool {
        self.bit(4, "flying")
    }
    fn passenger(&self) -> bool {
        self.bit(8, "passenger")
    }
    fn climbable(&mut self) -> Result<bool, Self::Error> {
        let value = self.bit(16, "climbable");
        if self.fail_climb {
            Err("climb-provider")
        } else {
            Ok(value)
        }
    }
    fn in_water(&self) -> bool {
        self.bit(32, "water")
    }
    fn levitating(&self) -> bool {
        self.bit(64, "levitation")
    }
    fn fall_flying(&self) -> bool {
        self.trace.borrow_mut().push("fallFlying");
        self.glide
    }
    fn jump_key(&self) -> bool {
        self.bit(512, "jumpKey")
    }
    fn input_jump(&self) -> bool {
        self.bit(1024, "inputJump")
    }
    fn stop_elytra_sound(&mut self) {
        self.trace.borrow_mut().push("stop");
    }
    fn start_fall_flying(&mut self) {
        self.trace.borrow_mut().push("start");
        self.glide = true;
    }
    fn send_start_fall_flying(&mut self) {
        self.trace.borrow_mut().push("send");
    }
    fn reset_jump_delay(&mut self) {
        self.trace.borrow_mut().push("reset");
        self.delay = 0;
    }
}

#[test]
fn complete_state_space_matches_unchanged_kotlin_and_exact_original_mixin() {
    for row in oracle()["flows"].as_array().unwrap() {
        let bits = row["bits"].as_u64().unwrap() as u32;
        let mut state = ElytraRecast {
            previous_elytra: row["was"].as_bool().unwrap(),
        };
        let mut world = World::new(bits);
        if bits & 2048 != 0 {
            state.ai_step_tail(bits & 1 != 0, &mut world).unwrap();
        }
        // Actual Input record accessor and primitive cooldown assignment cannot
        // be intercepted by provider shells; their outcomes remain asserted.
        let trace: Vec<_> = world
            .trace
            .into_inner()
            .into_iter()
            .filter(|label| !matches!(*label, "inputJump" | "reset"))
            .collect();
        assert_eq!(world.glide, row["glide"].as_bool().unwrap(), "{bits}");
        assert_eq!(
            state.previous_elytra,
            row["previous"].as_bool().unwrap(),
            "{bits}"
        );
        assert_eq!(world.delay, row["delay"].as_i64().unwrap() as i32, "{bits}");
        assert_eq!(serde_json::json!(trace), row["trace"], "{bits}");
    }
}

#[test]
fn public_recast_matches_unchanged_kotlin_for_all_2048_cases() {
    for row in oracle()["direct"].as_array().unwrap() {
        let bits = row["bits"].as_u64().unwrap() as u32;
        let mut world = World::new(bits);
        let result = ElytraRecast::recast_elytra(&mut world).unwrap();
        assert_eq!(result, row["result"].as_bool().unwrap(), "{bits}");
        assert_eq!(world.glide, row["glide"].as_bool().unwrap(), "{bits}");
        assert_eq!(world.delay, row["delay"].as_i64().unwrap() as i32, "{bits}");
        let trace: Vec<_> = world
            .trace
            .into_inner()
            .into_iter()
            .filter(|label| *label != "inputJump")
            .collect();
        assert_eq!(serde_json::json!(trace), row["trace"], "{bits}");
    }
}

#[test]
fn actual_mc_stack_components_match_all_durability_and_glider_cases() {
    use horizon_assets::item_model::{Components, ComponentsView};
    for row in oracle()["stacks"].as_array().unwrap() {
        let components: Components = serde_json::from_value(row["components"].clone()).unwrap();
        let chest = crate::elytra_recast_chest::chest(ComponentsView::of(&components));
        assert_eq!(
            chest.glider,
            row["glider"].as_bool().unwrap(),
            "{}",
            row["label"]
        );
        assert_eq!(
            chest.next_damage_will_break,
            row["nextDamageWillBreak"].as_bool().unwrap(),
            "{}",
            row["label"]
        );
    }
}

#[test]
fn disabled_history_lifecycle_and_pre_recast_sample_survive_toggle_and_clone() {
    let mut state = ElytraRecast::default();
    let mut world = World::new(128 | 512);
    world.glide = true;
    state.ai_step_tail(false, &mut world).unwrap();
    assert!(state.previous_elytra);
    world.glide = false;
    state.ai_step_tail(false, &mut world).unwrap();
    assert!(!state.previous_elytra);
    world.trace.borrow_mut().clear();
    state.ai_step_tail(true, &mut world).unwrap();
    assert_eq!(*world.trace.borrow(), ["fallFlying"]);
    world.glide = true;
    state.ai_step_tail(false, &mut world).unwrap();
    world.glide = false;
    state.ai_step_tail(true, &mut world).unwrap();
    assert!(world.glide);
    assert!(!state.previous_elytra);
    world.trace.borrow_mut().clear();
    state.ai_step_tail(true, &mut world).unwrap();
    assert!(state.previous_elytra);
    assert_eq!(*world.trace.borrow(), ["fallFlying"]);
    let cloned_body_state = state;
    assert_eq!(state, cloned_body_state); // body clones/dimension retention keep it
    assert!(!ElytraRecast::default().previous_elytra); // replacement entity starts false
}

#[test]
fn public_recast_has_no_running_ground_lava_or_equippable_guard() {
    let mut world = World::new(128 | 1024);
    assert!(ElytraRecast::recast_elytra(&mut world).unwrap());
    assert!(world.glide);
    assert_eq!(world.delay, 7, "public function never resets cooldown");
    assert_eq!(
        &world.trace.borrow()[world.trace.borrow().len() - 2..],
        ["start", "send"]
    );
}

#[test]
fn provider_failure_preserves_history_and_never_fabricates_success() {
    let mut state = ElytraRecast {
        previous_elytra: true,
    };
    let mut world = World::new(128 | 512);
    world.fail_climb = true;
    assert_eq!(state.ai_step_tail(true, &mut world), Err("climb-provider"));
    assert!(state.previous_elytra);
    assert!(!world.glide);
    assert_eq!(world.delay, 7);
    assert_eq!(
        world.trace.into_inner(),
        [
            "fallFlying",
            "stop",
            "chest",
            "flying",
            "passenger",
            "climbable"
        ]
    );
}
