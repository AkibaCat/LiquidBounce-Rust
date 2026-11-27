// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{ghost_hand::GhostHand, native_ray};
use horizon_core::{
    geometry::{Aabb, ShapeOp, Vec3, VoxelShape},
    position::BlockPos,
};
use horizon_world::world_ray::{RayBlockMode, RayError, RayFluidMode, RayWorld};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub(crate) fn oracle() -> &'static Value {
    static ORACLE: OnceLock<Value> = OnceLock::new();
    ORACLE.get_or_init(|| serde_json::from_str(include_str!("oracle.json")).unwrap())
}
fn vector(value: &Value) -> Vec3 {
    Vec3::new(
        value[0].as_f64().unwrap(),
        value[1].as_f64().unwrap(),
        value[2].as_f64().unwrap(),
    )
}
pub(crate) fn shape(value: &Value) -> VoxelShape {
    let mut shape = VoxelShape::empty();
    for b in value.as_array().unwrap() {
        let part = VoxelShape::from_box(Aabb::new(
            Vec3::new(
                b[0].as_f64().unwrap(),
                b[1].as_f64().unwrap(),
                b[2].as_f64().unwrap(),
            ),
            Vec3::new(
                b[3].as_f64().unwrap(),
                b[4].as_f64().unwrap(),
                b[5].as_f64().unwrap(),
            ),
        ))
        .unwrap();
        shape = shape.join(&part, ShapeOp::OR).unwrap();
    }
    shape
}
fn block_mode(mode: &str) -> RayBlockMode {
    match mode {
        "OUTLINE" => RayBlockMode::Outline,
        "COLLIDER" => RayBlockMode::Collider,
        "VISUAL" => RayBlockMode::Visual,
        "FALLDAMAGE_RESETTING" => RayBlockMode::FallDamageResetting,
        _ => panic!("unknown mode"),
    }
}
fn fluid_mode(mode: &str) -> RayFluidMode {
    match mode {
        "NONE" => RayFluidMode::None,
        "SOURCE_ONLY" => RayFluidMode::SourceOnly,
        "ANY" => RayFluidMode::Any,
        "WATER" => RayFluidMode::Water,
        _ => panic!("unknown fluid mode"),
    }
}
#[derive(Default)]
pub(crate) struct World {
    pub(crate) cells: BTreeMap<[i32; 3], usize>,
    pub(crate) queries: Vec<String>,
    pub(crate) fail_state: bool,
}
impl RayWorld for World {
    type Error = &'static str;
    type Block = usize;
    type Fluid = usize;
    fn block_state(&mut self, p: BlockPos) -> Result<usize, Self::Error> {
        self.queries.push(format!("block:{},{},{}", p.x, p.y, p.z));
        if self.fail_state {
            return Err("state failure");
        }
        Ok(*self.cells.get(&[p.x, p.y, p.z]).unwrap_or(&0))
    }
    fn fluid_state(&mut self, p: BlockPos) -> Result<usize, Self::Error> {
        self.queries.push(format!("fluid:{},{},{}", p.x, p.y, p.z));
        Ok(*self.cells.get(&[p.x, p.y, p.z]).unwrap_or(&0))
    }
    fn block_shape(
        &mut self,
        state: usize,
        _: BlockPos,
        mode: RayBlockMode,
    ) -> Result<VoxelShape, Self::Error> {
        self.queries.push(format!("shape:{state}"));
        let mode = match mode {
            RayBlockMode::Collider => "COLLIDER",
            RayBlockMode::Outline => "OUTLINE",
            RayBlockMode::Visual => "VISUAL",
            RayBlockMode::FallDamageResetting => "FALLDAMAGE_RESETTING",
        };
        Ok(shape(&oracle()["states"][state]["shapes"][mode]))
    }
    fn interaction_shape(&mut self, state: usize, _: BlockPos) -> Result<VoxelShape, Self::Error> {
        self.queries.push(format!("interaction:{state}"));
        Ok(shape(&oracle()["states"][state]["interaction"]))
    }
    fn fluid_is_source(&mut self, state: usize) -> Result<bool, Self::Error> {
        Ok(oracle()["states"][state]["source"].as_bool().unwrap())
    }
    fn fluid_is_empty(&mut self, state: usize) -> Result<bool, Self::Error> {
        Ok(oracle()["states"][state]["empty"].as_bool().unwrap())
    }
    fn fluid_is_water(&mut self, state: usize) -> Result<bool, Self::Error> {
        Ok(oracle()["states"][state]["water"].as_bool().unwrap())
    }
    fn fluid_shape(&mut self, state: usize, _: BlockPos) -> Result<VoxelShape, Self::Error> {
        Ok(shape(&oracle()["states"][state]["fluid"]))
    }
}

#[test]
fn untouched_original_java_tweaked_methods_matches_all_4608_actual_shape_cases() {
    let facts = oracle();
    assert_eq!(facts["cases"].as_array().unwrap().len(), 4608);
    for (index, case) in facts["cases"].as_array().unwrap().iter().enumerate() {
        let mut world = World::default();
        for cell in case["cells"].as_array().unwrap() {
            let p = &cell["pos"];
            world.cells.insert(
                [
                    p[0].as_i64().unwrap() as i32,
                    p[1].as_i64().unwrap() as i32,
                    p[2].as_i64().unwrap() as i32,
                ],
                cell["state"].as_u64().unwrap() as usize,
            );
        }
        let accepts = |state: usize| {
            case["targets"]
                .as_array()
                .unwrap()
                .contains(&facts["states"][state]["id"])
        };
        let targets = case["running"]
            .as_bool()
            .unwrap()
            .then_some(&accepts as &dyn Fn(usize) -> bool);
        let result = native_ray::clip(
            &mut world,
            vector(&case["start"]),
            vector(&case["end"]),
            block_mode(case["mode"].as_str().unwrap()),
            fluid_mode(case["fluids"].as_str().unwrap()),
            128,
            targets,
        )
        .unwrap();
        let expected = &case["result"];
        assert_eq!(
            result.miss,
            expected["miss"].as_bool().unwrap(),
            "case {index}"
        );
        assert_eq!(
            [result.hit.block.x, result.hit.block.y, result.hit.block.z],
            std::array::from_fn::<_, 3, _>(|i| expected["pos"][i].as_i64().unwrap() as i32),
            "case {index}"
        );
        assert_eq!(
            result.hit.direction.normal(),
            std::array::from_fn::<_, 3, _>(|i| expected["face"][i].as_i64().unwrap() as i32),
            "case {index}"
        );
        assert_eq!(
            result.hit.inside,
            expected["inside"].as_bool().unwrap(),
            "case {index}"
        );
        let actual = [
            result.hit.location.x,
            result.hit.location.y,
            result.hit.location.z,
        ];
        for (axis, actual) in actual.into_iter().enumerate() {
            let expected = expected["point"][axis].as_f64().unwrap();
            assert!(
                (actual - expected).abs() < 1e-12,
                "case {index} axis {axis}: {actual} != {expected}"
            );
        }
    }
}

#[test]
fn defaults_match_actual_chests_and_all_sixteen_dyed_shulker_blocks() {
    let expected: Vec<_> = oracle()["defaults"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let module = GhostHand::default();
    assert_eq!(module.sorted_targeted_blocks(), expected);
    assert_eq!(module.targeted_blocks.len(), 20);
}

#[test]
fn registry_list_resolves_before_replacing_and_preserves_identifier_sort_order() {
    let registry = ["minecraft:air", "minecraft:chest", "mod:a", "minecraft:z"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut module = GhostHand::default();
    for bad in [
        json!(null),
        json!(false),
        json!({}),
        json!(["chest", false]),
        json!(["chest", null]),
        json!([["chest"]]),
    ] {
        let original = module.clone();
        assert!(module.deserialize_targeted_blocks(&bad, &registry).is_err());
        assert_eq!(module, original);
    }
    module
        .deserialize_targeted_blocks(
            &json!(["chest", ":chest", "MISSING!", 17, "mod:a", "minecraft:z"]),
            &registry,
        )
        .unwrap();
    assert_eq!(
        module.sorted_targeted_blocks(),
        ["mod:a", "minecraft:air", "minecraft:chest", "minecraft:z"]
    );
    module
        .deserialize_targeted_blocks(&json!([]), &registry)
        .unwrap();
    assert!(module.targeted_blocks.is_empty());
    module
        .deserialize_targeted_blocks(&json!("chest"), &registry)
        .unwrap();
    assert_eq!(module.sorted_targeted_blocks(), ["minecraft:chest"]);
}

#[test]
fn selected_pass_never_queries_fluids_or_nonselected_shapes_and_fallback_restarts() {
    let start = Vec3::new(0.5, 0.5, 0.5);
    let end = Vec3::new(4.5, 0.5, 0.5);
    let mut world = World::default();
    world.cells.insert([1, 0, 0], 1); // nearer stone wall
    world.cells.insert([3, 0, 0], 2); // selected chest
    let hit = native_ray::clip(
        &mut world,
        start,
        end,
        RayBlockMode::Outline,
        RayFluidMode::Any,
        20,
        Some(&|s| s == 2),
    )
    .unwrap();
    assert_eq!(hit.hit.block.x, 3);
    assert!(!world.queries.iter().any(|q| q.starts_with("fluid:")));
    assert!(!world.queries.iter().any(|q| q == "shape:1"));
    let mut world = World::default();
    world.cells.insert([1, 0, 0], 1);
    let hit = native_ray::clip(
        &mut world,
        start,
        end,
        RayBlockMode::Outline,
        RayFluidMode::None,
        20,
        Some(&|_| false),
    )
    .unwrap();
    assert_eq!(hit.hit.block.x, 1);
    assert_eq!(
        world
            .queries
            .iter()
            .filter(|q| q.as_str() == "block:0,0,0")
            .count(),
        2
    );
    assert!(world.queries.iter().any(|q| q == "fluid:0,0,0"));
}

#[test]
fn provider_and_budget_errors_propagate_without_fabricating_fallback_hits() {
    let mut world = World {
        fail_state: true,
        ..World::default()
    };
    let result = native_ray::clip(
        &mut world,
        Vec3::ZERO,
        Vec3::new(1., 1., 1.),
        RayBlockMode::Outline,
        RayFluidMode::None,
        10,
        Some(&|_| true),
    );
    assert!(matches!(result, Err(RayError::World("state failure"))));
    assert_eq!(world.queries.len(), 1);
    let mut world = World::default();
    let result = native_ray::clip(
        &mut world,
        Vec3::ZERO,
        Vec3::new(1., 1., 1.),
        RayBlockMode::Outline,
        RayFluidMode::None,
        0,
        Some(&|_| true),
    );
    assert!(matches!(result, Err(RayError::Traversal(_))));
    assert!(world.queries.is_empty());
}
