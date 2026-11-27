use crate::{auto_tool, inventory::*};
use liquidbounce_behaviors::{
    auto_tool::{AutoTool, ToolStack, inventory::StackRecord},
    inventory_manager::*,
    random::ThreadRandom,
    vomit::{Constraints, Vomit},
};

fn context(creative: bool) -> Context {
    Context {
        in_game: true,
        creative,
        no_screen: true,
        menu: MenuState {
            stacks: vec![Stack::default(); 46],
            ..Default::default()
        },
        inventory_menu: Geometry {
            object: 1,
            slots: vec![Position::default(); 46],
        },
        player_stacks: vec![Stack::default(); 41],
        requirements: Requirements {
            rotation_matches: true,
            breaking: Some(false),
            ..Default::default()
        },
        ..Default::default()
    }
}
fn set(c: &mut Context, id: u8, stack: Stack) {
    c.player_stacks[usize::from(id)] = stack.clone();
    c.menu.stacks[Slot::Player(id).server_id(None).unwrap() as usize] = stack;
}
fn item(name: &str, count: i32) -> Stack {
    Stack {
        components: name.into(),
        count,
        wire: format!("wire:{name}"),
    }
}
fn tools(c: &Context) -> auto_tool::Snapshot {
    auto_tool::Snapshot {
        block: "minecraft:stone".into(),
        tick: 0,
        creative: c.creative,
        sneaking: false,
        server_slot: 0,
        beds: vec![],
        stacks: c
            .player_stacks
            .iter()
            .take(36)
            .enumerate()
            .map(|(id, stack)| ToolStack {
                slot: id as u8,
                destroy_speed: 1.,
                efficiency: 0,
                silk_touch: 0,
                max_damage: 0,
                damage: 0,
                empty: stack.empty(),
                identity: format!("{}:{}", stack.components, stack.count),
            })
            .collect(),
        records: vec![
            StackRecord {
                identity: String::new(),
                components: String::new(),
                count: 0
            };
            36
        ],
        context: Default::default(),
    }
}
fn module() -> AutoTool {
    let mut m = AutoTool::default();
    m.consider_inventory = true;
    m.constraints.start_delay = [0; 2];
    m.constraints.click_delay = [0; 2];
    m.constraints.close_delay = [0; 2];
    m.constraints.slot_multiplier = 0.;
    m
}
fn vomit() -> Vomit {
    Vomit {
        constraints: Constraints {
            start_delay: [0; 2],
            click_delay: [0; 2],
            close_delay: [0; 2],
            slot_multiplier: 0.,
            ..Default::default()
        },
        ..Default::default()
    }
}
fn native(a: &mut InventoryActor, c: &mut Context, effect: &Effect) {
    match effect {
        Effect::Action(Action::Click {
            slot: Slot::Player(id),
            input: Input::Swap,
            button,
            ..
        }) => {
            let left = c.player_stacks[usize::from(*id)].clone();
            let right = c.player_stacks[*button as usize].clone();
            set(c, *id, right);
            set(c, *button as u8, left);
            a.manager.sent_click(0);
        }
        Effect::Action(Action::Click {
            slot: Slot::Player(id),
            input: Input::Pickup,
            ..
        }) => {
            let old = c.player_stacks[usize::from(*id)].clone();
            let carried = std::mem::replace(&mut c.menu.carried, old);
            set(c, *id, carried);
            a.manager.sent_click(0);
        }
        Effect::Action(Action::Click {
            slot: Slot::Player(id),
            input: Input::Throw,
            ..
        }) => {
            set(c, *id, Stack::default());
            a.manager.sent_click(0);
        }
        Effect::CloseInventory => {
            a.manager.outgoing_close(0);
        }
        Effect::MoveCursor(cursor) => c.visible_cursor = *cursor,
        _ => {}
    }
}
#[test]
fn completed_swap_rechecks_live_owner_before_running_anchor_tail() {
    let mut a = InventoryActor::default();
    let mut m = module();
    let v = vomit();
    let mut c = context(false);
    set(&mut c, 9, item("pickaxe", 1));
    m.swaps.request(9);
    a.begin_tick(&c);
    a.submit_auto_tool(&mut m, true, &tools(&c), &c);
    let mut rng = ThreadRandom::seeded(8, 1);
    let mut running = true;
    let mut actions = 0;
    while let Some(emission) = {
        let owners = a.owners(&m, running, &tools(&c), &v, false, c.creative);
        a.manager
            .step(
                &c,
                &mut |id| owners.iter().find(|(key, _)| *key == id).unwrap().1.clone(),
                &mut rng,
            )
            .unwrap()
    } {
        if let Effect::Outcome(outcome) = &emission.effect {
            a.outcome(outcome, &mut m, running, &c);
        }
        native(&mut a, &mut c, &emission.effect);
        if matches!(emission.effect, Effect::Action(_)) {
            actions += 1;
            running = false;
        }
        assert!(a.manager.acknowledge(emission.ticket, true, &c));
    }
    assert_eq!(actions, 1);
    assert!(m.swaps.active.is_none());
    assert!(m.swaps.anchor_action.is_none());
    assert_eq!(c.player_stacks[0].components, "pickaxe");
}
#[test]
fn successful_swap_may_withdraw_request_before_unchanged_source_tail() {
    let mut a = InventoryActor::default();
    let mut m = module();
    let v = vomit();
    let mut c = context(false);
    set(&mut c, 9, item("pickaxe", 1));
    m.swaps.request(9);
    a.begin_tick(&c);
    a.submit_auto_tool(&mut m, true, &tools(&c), &c);
    let mut rng = ThreadRandom::seeded(2, 1);
    while let Some(emission) = {
        let owners = a.owners(&m, true, &tools(&c), &v, false, c.creative);
        a.manager
            .step(
                &c,
                &mut |id| owners.iter().find(|(key, _)| *key == id).unwrap().1.clone(),
                &mut rng,
            )
            .unwrap()
    } {
        if let Effect::Outcome(outcome) = &emission.effect {
            a.outcome(outcome, &mut m, true, &c);
        }
        native(&mut a, &mut c, &emission.effect);
        if matches!(emission.effect, Effect::Action(_)) {
            m.swaps.clear_requested();
        }
        assert!(a.manager.acknowledge(emission.ticket, true, &c));
    }
    assert_eq!(m.swaps.anchor_hotbar, Some(0));
    assert_eq!(m.swaps.anchor_action.unwrap().source, 9);
}
#[test]
fn shared_queue_cancels_vomit_capture_after_autotool_changes_full_menu() {
    let mut a = InventoryActor::default();
    let mut m = module();
    let mut v = vomit();
    let mut c = context(false);
    set(&mut c, 9, item("pickaxe", 1));
    m.swaps.request(9);
    let s = Snapshot {
        context: c.clone(),
        block_registry_size: 1286,
        elytra_swap_stacks: vec![],
    };
    let mut rng = ThreadRandom::seeded(3, 1);
    a.begin_tick(&c);
    a.submit_auto_tool(&mut m, true, &tools(&c), &c);
    a.submit_vomit(&mut v, true, &s, &mut rng).unwrap();
    let mut actions = vec![];
    let mut cancelled = 0;
    while let Some(emission) = {
        let owners = a.owners(&m, true, &tools(&c), &v, true, c.creative);
        a.manager
            .step(
                &c,
                &mut |id| owners.iter().find(|(key, _)| *key == id).unwrap().1.clone(),
                &mut rng,
            )
            .unwrap()
    } {
        if let Effect::Outcome(outcome) = &emission.effect {
            cancelled += usize::from(matches!(outcome, Outcome::Cancelled { .. }));
            a.outcome(outcome, &mut m, true, &c);
        }
        if let Effect::Action(action) = &emission.effect {
            actions.push(action.clone());
        }
        native(&mut a, &mut c, &emission.effect);
        assert!(a.manager.acknowledge(emission.ticket, true, &c));
    }
    assert_eq!(actions.len(), 1);
    assert_eq!(cancelled, 1);
    assert_eq!(c.player_stacks[0].components, "pickaxe");
    assert!(matches!(
        actions[0],
        Action::Click {
            input: Input::Swap,
            ..
        }
    ));
}
#[test]
fn hidden_zero_delay_vomit_keeps_complete_seventy_two_action_chain_after_stop() {
    let mut a = InventoryActor::default();
    let mut m = module();
    let mut v = vomit();
    let mut c = context(true);
    let snapshot = Snapshot {
        context: c.clone(),
        block_registry_size: 1286,
        elytra_swap_stacks: vec![],
    };
    let mut rng = ThreadRandom::seeded(13, 1);
    a.begin_tick(&c);
    a.submit_vomit(&mut v, true, &snapshot, &mut rng).unwrap();
    let mut actions = vec![];
    let mut running = true;
    while let Some(emission) = {
        let owners = a.owners(&m, false, &tools(&c), &v, running, c.creative);
        a.manager
            .step(
                &c,
                &mut |id| owners.iter().find(|(key, _)| *key == id).unwrap().1.clone(),
                &mut rng,
            )
            .unwrap()
    } {
        if let Effect::Outcome(outcome) = &emission.effect {
            a.outcome(outcome, &mut m, false, &c);
        }
        if let Effect::Action(action) = &emission.effect {
            actions.push(action.clone());
            running = false;
        }
        native(&mut a, &mut c, &emission.effect);
        assert!(a.manager.acknowledge(emission.ticket, true, &c));
    }
    assert_eq!(actions.len(), 72);
    assert!(
        actions[..36]
            .iter()
            .all(|a| matches!(a, Action::Creative { slot: Some(_), .. }))
    );
    assert!(actions[36..].iter().all(|a| matches!(
        a,
        Action::Click {
            input: Input::Throw,
            ..
        }
    )));
    assert!(c.player_stacks.iter().all(Stack::empty));
}

fn elytra_snapshot(c: &Context) -> Snapshot {
    use liquidbounce_behaviors::elytra_swap::Item;
    Snapshot {
        context: c.clone(),
        block_registry_size: 1286,
        elytra_swap_stacks: c.player_stacks.iter().map(|stack| Item {
            empty: stack.empty(),
            glider: stack.components == "custom_glider",
            next_damage_will_break: stack.components == "broken_glider",
            chest_armor: stack.components == "tagged_chest_armor",
        }).collect(),
    }
}

#[test]
fn elytra_swap_atomic_pickups_disable_only_on_completed_and_leave_empty_cursor() {
    use liquidbounce_behaviors::elytra_swap::ElytraSwap;
    for source in [0_u8, 9, 35, 40] {
        for occupied in [false, true] {
            let mut actor = InventoryActor::default();
            let mut module = ElytraSwap { constraints: vomit().constraints, ..Default::default() };
            let mut c = context(false);
            if occupied { set(&mut c, 38, item("tagged_chest_armor", 1)); }
            set(&mut c, source, item("custom_glider", 1));
            actor.begin_tick(&c);
            assert!(!actor.submit_elytra_swap(&mut module, true, &elytra_snapshot(&c)).unwrap());
            assert_eq!(module.wants_glider, Some(true));
            let mut random = ThreadRandom::seeded(83, 1);
            let mut running = true;
            let mut pickups = Vec::new();
            let mut completed = 0;
            while let Some(emission) = {
                let (id, owner) = actor.elytra_swap_owner(&module, running);
                actor.manager.step(&c, &mut |asked| { assert_eq!(id, asked); owner.clone() }, &mut random).unwrap()
            } {
                if let Effect::Action(Action::Click { slot: Slot::Player(slot), input: Input::Pickup, .. }) = &emission.effect {
                    assert!(running);
                    pickups.push(*slot);
                }
                if let Effect::Outcome(outcome) = &emission.effect {
                    let disable = actor.elytra_swap_outcome(outcome);
                    assert_eq!(disable, matches!(outcome, Outcome::Completed { .. }));
                    if disable { completed += 1; running = false; actor.cancel_elytra_swap(); }
                }
                native(&mut actor, &mut c, &emission.effect);
                assert!(actor.manager.acknowledge(emission.ticket, true, &c));
            }
            assert_eq!(completed, 1);
            assert!(!running);
            assert_eq!(pickups, if occupied { vec![source, 38, source] } else { vec![source, 38] });
            assert_eq!(c.player_stacks[38].components, "custom_glider");
            assert_eq!(c.player_stacks[usize::from(source)].components, if occupied { "tagged_chest_armor" } else { "" });
            assert!(c.menu.carried.empty());
        }
    }
}

#[test]
fn elytra_swap_cancelled_pending_plan_retains_direction_until_on_enabled() {
    use liquidbounce_behaviors::elytra_swap::ElytraSwap;
    let mut actor = InventoryActor::default();
    let mut module = ElytraSwap { constraints: vomit().constraints, ..Default::default() };
    let mut c = context(false);
    set(&mut c, 9, item("custom_glider", 1));
    actor.begin_tick(&c);
    assert!(!actor.submit_elytra_swap(&mut module, true, &elytra_snapshot(&c)).unwrap());
    actor.cancel_elytra_swap();
    let mut random = ThreadRandom::seeded(83, 1);
    let emission = actor.manager.step(&c, &mut |_| panic!("cancelled pending owner must not be queried"), &mut random).unwrap().unwrap();
    let Effect::Outcome(outcome @ Outcome::Cancelled { .. }) = &emission.effect else { panic!("cancel outcome") };
    assert!(!actor.elytra_swap_outcome(outcome));
    assert_eq!(module.wants_glider, Some(true));
    assert!(actor.manager.acknowledge(emission.ticket, true, &c));
    set(&mut c, 38, item("custom_glider", 1));
    assert!(actor.submit_elytra_swap(&mut module, true, &elytra_snapshot(&c)).unwrap());
    assert_eq!(module.wants_glider, Some(true));
    module.on_enabled();
    assert_eq!(module.wants_glider, None);
    assert!(actor.submit_elytra_swap(&mut module, true, &elytra_snapshot(&c)).unwrap());
    assert_eq!(module.wants_glider, Some(false));
}
