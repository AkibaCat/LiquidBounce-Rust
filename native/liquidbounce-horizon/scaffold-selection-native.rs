// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual BlockItem/default-state facts for Scaffold and Clutch.
use super::{Game, ItemStack, scaffold_block_rules as rules};
use horizon_core::{
    geometry::{Direction, ShapeOp, VoxelShape},
    position::BlockPos,
};
use horizon_world::{
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
    level::{
        classes::Class,
        data::{ItemKind, state_flags},
    },
};
use serde_json::{Value, json};

fn full_cube(shape: &VoxelShape) -> bool {
    shape.is_block()
        || !VoxelShape::block()
            .join_is_not_empty(shape, ShapeOp::XOR)
            .expect("Scaffold full collision shape")
}
fn full_up_face(shape: &VoxelShape) -> bool {
    shape.is_block()
        || full_cube(
            &shape
                .face_shape(Direction::Up)
                .expect("Scaffold collision UP face"),
        )
}
fn policy(value: &Value) -> Option<rules::Policy> {
    let mut result = rules::Policy::default();
    for (name, target) in [
        ("disallowed", &mut result.disallowed),
        ("unfavorable", &mut result.unfavorable),
    ] {
        if let Some(values) = value.get(name) {
            *target = values
                .as_array()?
                .iter()
                .map(|v| v.as_str().map(str::to_owned))
                .collect::<Option<_>>()?;
        }
    }
    Some(result)
}
fn stack_json(stack: Option<&rules::BlockStack>, policy: &rules::Policy) -> Value {
    let Some(s) = stack else { return Value::Null };
    json!({"block":s.block,"count":s.count,"valid":s.valid(policy),"favorable":!s.unfavorable(policy),
        "playerCanStandUp":s.player_can_stand_up,"falling":s.falling,"baseEntity":s.base_entity,
        "fullCube":s.full_cube,"redstoneConductor":s.redstone_conductor,"friction":s.friction,
        "speedFactor":s.speed_factor,"jumpFactor":s.jump_factor,"hardness":s.hardness})
}
impl Game<'_> {
    pub(super) fn scaffold_block_facts(&self, stack: &ItemStack) -> Option<rules::BlockStack> {
        if stack.count == 0 {
            return None;
        }
        let player = self.player.as_ref()?;
        let behaviours = self.res.behaviours();
        let binding = behaviours.item_binding(&stack.item)?;
        // The registry class chain assigns precisely these kinds to BlockItem
        // and subclasses. A SolidBucketItem's block field is not a BlockItem.
        if matches!(binding.kind, ItemKind::SolidBucket | ItemKind::Other) {
            return None;
        }
        let state = behaviours.table.default_state(binding.block);
        let block = behaviours.block_data(binding.block);
        let movement = self
            .motion
            .blocks
            .get(state)
            .expect("Scaffold bound movement registry")
            .coefficients;
        let held = self
            .inventory
            .get(self.client_hotbar_slot())
            .and_then(Option::as_ref);
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(
                    self.remote
                        .as_ref()
                        .map_or(0, |r| r.entity_id as u32 as u64),
                ),
                kind: EntityContextKind::Player,
                y: player.position.y,
                descending: player.shift_key_down,
                main_hand: Some(
                    held.filter(|s| s.count > 0)
                        .map_or(CollisionItem::EMPTY, |s| CollisionItem {
                            item: &s.item,
                            count: s.count as i32,
                        }),
                ),
            },
            false,
        );
        let mut subject = player.powder_snow_subject();
        subject.leather_boots = self.armor[3]
            .as_ref()
            .is_some_and(|s| s.count > 0 && &*s.item == "minecraft:leather_boots");
        let mut world = horizon_client::collisions::SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: Some(subject),
        };
        let position = BlockPos::new(0, 0, 0);
        let supporting = self
            .collision
            .collision_shape_with_context(&mut world, state, position, &context)
            .expect("Scaffold actual player collision shape at ZERO");
        let facts = behaviours.facts(state);
        // BlockStateBase caches this predicate only when !dynamicShape. Query
        // uncached states with the empty context and real ZERO block entities.
        let full = if block.dynamic_shape {
            let shape = self
                .collision
                .collision_shape_with_context(
                    &mut world,
                    state,
                    position,
                    &CollisionContext::empty(false),
                )
                .expect("Scaffold empty-context default collision shape at ZERO");
            full_cube(&shape)
        } else {
            facts.flags & state_flags::FULL_COLLISION != 0
        };
        Some(rules::BlockStack {
            block: behaviours.table.name(state).to_owned(),
            count: stack.count as i32,
            player_can_stand_up: full_up_face(&supporting),
            falling: block.is(Class::FallingBlock),
            base_entity: block.is(Class::BaseEntity),
            full_cube: full,
            redstone_conductor: facts.flags & state_flags::CONDUCTOR != 0,
            friction: movement.friction,
            speed_factor: movement.speed_factor,
            jump_factor: movement.jump_factor,
            hardness: facts.hardness,
        })
    }
    pub(super) fn scaffold_selection_query(&self, request: &str) -> Option<String> {
        let request = serde_json::from_str::<Value>(request).ok()?;
        let auto = request["autoBlock"].as_bool()?;
        let threshold = i32::try_from(request["doNotUseBelowCount"].as_i64()?).ok()?;
        let policy = policy(&request)?;
        let hotbar: [Option<rules::BlockStack>; 9] = std::array::from_fn(|slot| {
            self.inventory
                .get(slot)
                .and_then(Option::as_ref)
                .and_then(|s| self.scaffold_block_facts(s))
        });
        let offhand = self
            .offhand
            .as_ref()
            .and_then(|s| self.scaffold_block_facts(s));
        let selected = self.selected;
        let effective = self.client_hotbar_slot();
        Some(json!({"selected":selected,"effectiveSelected":effective,
            "blockCount":rules::block_count(&hotbar, offhand.as_ref(), selected, auto, &policy),
            "bestSlot":rules::best_slot(&hotbar, threshold, &policy),
            "rawMainValid":hotbar.get(selected).and_then(Option::as_ref).is_some_and(|s| s.valid(&policy)),
            "mainValid":hotbar.get(effective).and_then(Option::as_ref).is_some_and(|s| s.valid(&policy)),
            "offhandValid":offhand.as_ref().is_some_and(|s| s.valid(&policy)),
            "hotbar":hotbar.iter().map(|s| stack_json(s.as_ref(), &policy)).collect::<Vec<_>>(),
            "offhand":stack_json(offhand.as_ref(), &policy)}).to_string())
    }
}
