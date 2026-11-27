// SPDX-License-Identifier: GPL-3.0-or-later
use crate::terrain_speed::*;
use serde_json::Value;
use std::cell::RefCell;

fn flag(row: &Value, name: &str) -> bool {
    row[name].as_bool().unwrap()
}
fn float(row: &Value, name: &str) -> f32 {
    row[name].as_f64().unwrap() as f32
}
fn vector(row: &Value, name: &str) -> [f64; 3] {
    let a = row[name].as_array().unwrap();
    std::array::from_fn(|i| a[i].as_f64().unwrap())
}
fn bits(v: [f64; 3]) -> Value {
    serde_json::json!(v.map(|v| v.to_bits().to_string()))
}
struct World {
    row: Value,
    position: [f64; 3],
    snaps: Vec<Value>,
    trace: RefCell<Vec<String>>,
    start_y: i32,
}
impl ClimbWorld for World {
    type Error = std::convert::Infallible;
    fn horizontal_collision(&self) -> bool {
        flag(&self.row, "collision")
    }
    fn on_climbable(&mut self) -> Result<bool, Self::Error> {
        self.trace.borrow_mut().push("climbable".into());
        Ok(flag(&self.row, "climbable"))
    }
    fn forward_key(&self) -> bool {
        self.trace.borrow_mut().push("forward".into());
        flag(&self.row, "forward")
    }
    fn position(&self) -> [f64; 3] {
        self.trace.borrow_mut().push("position".into());
        self.position
    }
    fn block_position(&self) -> [i32; 3] {
        self.trace.borrow_mut().push("blockPosition".into());
        self.position.map(|v| v.floor() as i32)
    }
    fn ladder_or_vine(&mut self, p: [i32; 3]) -> Result<bool, Self::Error> {
        self.trace
            .borrow_mut()
            .push(format!("block:{}:{}:{}", p[0], p[1], p[2]));
        let dy = p[1].wrapping_sub(self.start_y);
        Ok(match self.row["pattern"].as_u64().unwrap() {
            0 | 5 => true,
            1 => dy <= 3,
            2 | 3 => false,
            4 => dy < 8,
            _ => unreachable!(),
        })
    }
    fn direction(&self) -> Direction {
        self.trace.borrow_mut().push("direction".into());
        [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ][self.row["direction"].as_u64().unwrap() as usize]
    }
    fn abs_snap_to(&mut self, p: [f64; 3]) -> Result<(), Self::Error> {
        self.trace.borrow_mut().push("snap".into());
        self.position = p;
        self.snaps.push(bits(p));
        Ok(())
    }
}
#[test]
fn unchanged_original_four_kotlin_files_match_7040_exact_bit_cases() {
    let oracle: Value = serde_json::from_str(include_str!("oracle.json")).unwrap();
    let rows = oracle.as_array().unwrap();
    assert_eq!(rows.len(), 7040);
    let mut counts = [0; 4];
    for (index, expected) in rows.iter().enumerate() {
        let r = &expected["input"];
        let s = TerrainSpeed {
            fast_climb: FastClimb {
                enabled: flag(r, "climbEnabled"),
                mode: if r["climbMode"] == "Motion" {
                    ClimbMode::Motion
                } else {
                    ClimbMode::Clip
                },
                motion: float(r, "climbMotion"),
            },
            ice_speed: IceSpeed {
                enabled: flag(r, "iceEnabled"),
                slipperiness: float(r, "slipperiness"),
                motion_enabled: flag(r, "iceMotionEnabled"),
                horizontal_motion: float(r, "iceMotion"),
            },
            water_speed: WaterSpeed {
                enabled: flag(r, "waterEnabled"),
                auto_swim: flag(r, "autoSwim"),
                base_horizontal: float(r, "baseHorizontal"),
                base_vertical: float(r, "baseVertical"),
                sprint_enabled: flag(r, "sprintEnabled"),
                sprint_horizontal: float(r, "sprintHorizontal"),
                sprint_vertical: float(r, "sprintVertical"),
            },
        };
        let running = flag(r, "running");
        let p = vector(r, "position");
        let mut world = World {
            row: r.clone(),
            position: p,
            snaps: vec![],
            trace: RefCell::new(vec![]),
            start_y: p[1].floor() as i32,
        };
        let mut velocity = vector(r, "velocity");
        match r["event"].as_str().unwrap() {
            "move" => {
                counts[0] += 1;
                let mut movement = vector(r, "movement");
                s.fast_climb
                    .move_event(running, &mut world, &mut movement)
                    .unwrap();
                assert_eq!(
                    bits(movement),
                    expected["movementBits"],
                    "movement case {index}: {r}"
                );
                assert_eq!(
                    serde_json::json!(world.trace.borrow().as_slice()),
                    expected["trace"],
                    "climb provider order {index}: {r}"
                );
            }
            "input" => {
                counts[1] += 1;
                let mut jump = flag(r, "jump");
                s.water_speed.movement_input(
                    running,
                    flag(r, "water"),
                    flag(r, "sneakKey"),
                    &mut jump,
                );
                assert_eq!(jump, flag(expected, "jump"), "input case {index}: {r}");
            }
            "tick" => {
                counts[2] += 1;
                s.water_speed.player_tick(
                    running,
                    WaterContext {
                        in_water: flag(r, "water"),
                        physical_sprint: flag(r, "sprintKey"),
                        physical_jump: flag(r, "jumpKey"),
                        physical_sneak: flag(r, "sneakKey"),
                        moving: flag(r, "moving"),
                        input: r["input"].as_u64().unwrap() as u8,
                        yaw: float(r, "yaw"),
                    },
                    &mut velocity,
                    |a| [horizon_core::jmath::sin(a), horizon_core::jmath::cos(a)],
                );
            }
            "friction" => {
                counts[3] += 1;
                let block = [
                    "minecraft:ice",
                    "minecraft:blue_ice",
                    "minecraft:frosted_ice",
                    "minecraft:packed_ice",
                    "minecraft:slime_block",
                    "minecraft:air",
                ][r["block"].as_u64().unwrap() as usize];
                let mut f = float(r, "friction");
                for _ in 0..r["repeat"].as_u64().unwrap() {
                    f = s.ice_speed.block_friction(running, block, f, &mut velocity);
                }
                assert_eq!(
                    f.to_bits().to_string(),
                    expected["frictionBits"].as_str().unwrap(),
                    "friction case {index}: {r}"
                );
            }
            _ => unreachable!(),
        }
        assert_eq!(
            bits(world.position),
            expected["positionBits"],
            "position case {index}: {r}"
        );
        assert_eq!(
            bits(velocity),
            expected["velocityBits"],
            "velocity case {index}: {r}"
        );
        assert_eq!(
            serde_json::json!(world.snaps),
            expected["snaps"],
            "snap case {index}: {r}"
        );
    }
    assert_eq!(counts, [2304, 64, 4096, 576]);
}
