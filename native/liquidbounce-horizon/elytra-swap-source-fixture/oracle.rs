use crate::elytra_swap::{ElytraSwap, Item};
use serde_json::{Value, json};

fn item(bits: i32) -> Item {
    Item {
        empty: bits & 1 != 0,
        glider: bits & 2 != 0,
        next_damage_will_break: bits & 4 != 0,
        chest_armor: bits & 8 != 0,
    }
}

#[test]
fn unchanged_kotlin_plans_capture_direction_and_suspend_before_disable() {
    let commands: Vec<Vec<Value>> = serde_json::from_str(include_str!("commands.json")).unwrap();
    let expected: Vec<Vec<Value>> = serde_json::from_str(include_str!("oracle.json")).unwrap();
    let mut module = ElytraSwap::default();
    for (i, (commands, expected)) in commands.iter().zip(expected.iter()).enumerate() {
        let mut stacks = [item(1); 41];
        let mut enabled = false;
        let mut pending = false;
        for (j, (c, e)) in commands.iter().zip(expected.iter()).enumerate() {
            let mut reads = Vec::new();
            let mut actions = Vec::new();
            match c["op"].as_str().unwrap() {
                "enable" => {
                    let next = c["value"].as_bool().unwrap();
                    if next && !enabled {
                        module.on_enabled();
                    }
                    enabled = next;
                }
                "stack" => stacks[c["slot"].as_u64().unwrap() as usize] = item(c["bits"].as_i64().unwrap() as i32),
                "start" if enabled && !pending => {
                    reads.push(38u8);
                    actions = module.pickups(stacks[38], |slot| {
                        reads.push(slot);
                        stacks[usize::from(slot)]
                    });
                    pending = !actions.is_empty();
                    if !pending {
                        enabled = false;
                    }
                }
                "resume" if pending => {
                    pending = false;
                    enabled = false;
                }
                "cancel" => pending = false,
                "start" | "resume" => {}
                _ => panic!("unknown command"),
            }
            assert_eq!(json!({"enabled":enabled,"wants":module.wants_glider,"actions":actions,"reads":reads,"pending":pending}), *e, "row {i} command {j}: {c}");
        }
    }
}
