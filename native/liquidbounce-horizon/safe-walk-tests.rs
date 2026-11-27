// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{random::ThreadRandom, safe_walk::*};
use serde_json::{Value, json};

fn double(v: &Value) -> f64 {
    f64::from_bits(v.as_str().unwrap().parse().unwrap())
}
fn float(v: &Value) -> f32 {
    f32::from_bits(v.as_str().unwrap().parse().unwrap())
}
fn vector(v: &Value) -> [f64; 3] {
    std::array::from_fn(|i| double(&v[i]))
}
fn direction(mask: u64) -> Direction {
    Direction {
        forwards: mask & 1 != 0,
        backwards: mask & 2 != 0,
        left: mask & 4 != 0,
        right: mask & 8 != 0,
    }
}
fn mask(value: Direction) -> u64 {
    u64::from(value.forwards)
        + 2 * u64::from(value.backwards)
        + 4 * u64::from(value.left)
        + 8 * u64::from(value.right)
}
#[derive(Default)]
struct World {
    edge: bool,
    fall: bool,
    next: [f64; 3],
    trace: Vec<String>,
}
impl Queries for World {
    fn close_to_edge(&mut self, _: Direction, distance: f64) -> bool {
        self.trace.push(format!("edge:{}", distance.to_bits()));
        self.edge
    }
    fn next_simulated_position(&mut self) -> [f64; 3] {
        self.trace.push("next".into());
        self.next
    }
    fn close_to_fall_off(&mut self, _: [f64; 3]) -> bool {
        self.trace.push("fall".into());
        self.fall
    }
    fn debug_input(&mut self, _: Direction) {
        self.trace.push("debug:InputOnEdge".into());
    }
    fn debug_center(&mut self, _: [f64; 3], _: f64, _: f64) {
        self.trace.extend([
            "debugGeometry:Center".into(),
            "debug:CurrentDistance".into(),
            "debug:NextDistance".into(),
        ]);
    }
    fn debug_overwrite_ticks(&mut self, _: i32) {
        self.trace.push("debug:OverwriteInputTicks".into());
    }
}

#[test]
fn untouched_original_matches_500_sequential_state_and_lazy_query_cases() {
    let cases: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/inputs.json"
    )))
    .unwrap();
    let oracle: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/oracle.json"
    )))
    .unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 500);
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let thread_id = oracle[index][0]["threadId"].as_u64().unwrap();
        let seed: i64 = case["seed"].as_str().unwrap().parse().unwrap();
        let mut random = ThreadRandom::seeded(seed as u64, thread_id);
        let mut module = SafeWalk::new(&mut random).unwrap();
        module.settings.mode = Mode::OnEdge;
        let mut running = true;
        let mut snapshot = Snapshot {
            on_ground: true,
            position: [0.5, 1.0, 0.5],
            block_bottom_center: [0.5, 1.0, 0.5],
            ..Snapshot::default()
        };
        let mut world = World {
            next: snapshot.position,
            ..World::default()
        };
        for (step_index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            world.trace.clear();
            let mut input = None;
            let mut safe = None;
            let result = match step["op"].as_str().unwrap() {
                "snapshot" => {
                    snapshot.on_ground = step["ground"].as_bool().unwrap();
                    snapshot.horizontal_speed = double(&step["speed"]);
                    snapshot.position = vector(&step["pos"]);
                    snapshot.yaw = float(&step["yaw"]);
                    snapshot.block_bottom_center = [
                        snapshot.position[0].floor() + 0.5,
                        snapshot.position[1].floor(),
                        snapshot.position[2].floor() + 0.5,
                    ];
                    world.next = vector(&step["next"]);
                    world.edge = step["edge"].as_bool().unwrap();
                    world.fall = step["fall"].as_bool().unwrap();
                    Ok(())
                }
                "input" => {
                    let value = input.insert(Input {
                        direction: direction(step["mask"].as_u64().unwrap()),
                        jump: step["jump"].as_bool().unwrap(),
                        sneak: step["sneak"].as_bool().unwrap(),
                    });
                    module.movement(running, snapshot, value, &mut world, &mut random)
                }
                "safe" => {
                    safe = Some(module.safe_walk(running, step["value"].as_bool().unwrap()));
                    Ok(())
                }
                "running" => {
                    let active = step["value"].as_bool().unwrap();
                    let result = if running && !active && module.settings.mode == Mode::OnEdge {
                        module.disable_on_edge(&mut random)
                    } else {
                        Ok(())
                    };
                    running = active;
                    result
                }
                "mode" => module.set_mode(
                    Mode::parse(step["value"].as_str().unwrap()).unwrap(),
                    running,
                    &mut random,
                ),
                "disable" => module.disable_on_edge(&mut random),
                "settings" => {
                    let mut result = Ok(());
                    if let Some(v) = step.get("distance") {
                        result = module.set_distance([float(&v[0]), float(&v[1])], &mut random);
                    }
                    if result.is_ok() {
                        if let Some(v) = step.get("keep") {
                            module.settings.keep =
                                std::array::from_fn(|i| v[i].as_i64().unwrap() as i32);
                        }
                        if let Some(v) = step.get("sneak") {
                            module.settings.sneak =
                                std::array::from_fn(|i| v[i].as_i64().unwrap() as i32);
                        }
                        if let Some(v) = step.get("jump") {
                            module.settings.jump = v.as_bool().unwrap();
                        }
                        if let Some(v) = step.get("edgeMode") {
                            module.settings.edge_mode =
                                EdgeMode::parse(v.as_str().unwrap()).unwrap();
                        }
                    }
                    result
                }
                "state" => {
                    if let Some(v) = step.get("center") {
                        module.center = (!v.is_null()).then(|| vector(v));
                    }
                    if let Some(v) = step.get("overwrite") {
                        module.overwrite_ticks = v.as_i64().unwrap() as i32;
                    }
                    if let Some(v) = step.get("sneaks") {
                        module.sneak_ticks = v.as_i64().unwrap() as i32;
                    }
                    Ok(())
                }
                op => panic!("unexpected operation {op}"),
            };
            let mut probe = random;
            let mut actual = json!({
                "overwrite":module.overwrite_ticks,
                "sneaks":module.sneak_ticks,
                "distance":module.edge_distance.to_bits().to_string(),
                "threadId":thread_id,
                "nextRandom":(probe.next_i64() as u64).to_string(),
                "trace":world.trace,
            });
            if let Some(center) = module.center {
                actual["center"] = json!(center.map(|v| v.to_bits().to_string()));
            }
            if let Some(input) = input {
                actual["mask"] = json!(mask(input.direction));
                actual["jump"] = json!(input.jump);
                actual["sneak"] = json!(input.sneak);
            }
            if let Some(safe) = safe {
                actual["safe"] = json!(safe);
            }
            if result.is_err() {
                actual["error"] = json!("IllegalArgumentException");
            }
            let mut expected = oracle[index][step_index].clone();
            expected.as_object_mut().unwrap().remove("seed");
            assert_eq!(actual, expected, "case {index} step {step_index}: {step}");
        }
    }
}
