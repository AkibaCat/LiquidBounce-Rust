#!/usr/bin/env python3
"""Client-only Entity.isSuppressingBounce for dropped items and primed TNT.

The existing movement query hook supplies the policy. Shared/server entry
points keep their original signature and delegate with suppression disabled.
No entity state or constructor receives client-specific module flags.
"""

ITEM_SIGNATURE = '''    pub fn tick_move_before_block_effects(
        &mut self,
        world: &World,
        facts: &BlockFacts,
        collision: &BlockCollisionRegistry<'_>,
        no_physics: bool,
        escape_speed: Option<f32>,
        (water_height, lava_height): (f64, f64),
        entity_boxes: &[Aabb],
        border: Option<&BorderGeometry>,
        blocks: Option<&BoundMovementProperties>,
    ) -> ItemStep {
'''
ITEM_WRAPPER = '''    pub fn tick_move_before_block_effects(
        &mut self,
        world: &World,
        facts: &BlockFacts,
        collision: &BlockCollisionRegistry<'_>,
        no_physics: bool,
        escape_speed: Option<f32>,
        heights: (f64, f64),
        entity_boxes: &[Aabb],
        border: Option<&BorderGeometry>,
        blocks: Option<&BoundMovementProperties>,
    ) -> ItemStep {
        self.tick_move_before_block_effects_with_bounce(
            world, facts, collision, no_physics, escape_speed, heights,
            entity_boxes, border, blocks, false,
        )
    }
    /// The actual client-level move may suppress bounce. The shared/server
    /// entry above always supplies false and retains its original behavior.
'''

TNT_SIGNATURE = '''    pub(super) fn tick_among(
        &mut self,
        world: &horizon_client::world::World,
        collision: &horizon_world::block_collision::BlockCollisionRegistry<'_>,
        assets: &horizon_client::assets::Assets,
        facts: &horizon_client::blocks::BlockFacts,
        fluids: &horizon_client::motion::FluidResources,
        border: Option<&horizon_core::geometry::border::BorderGeometry>,
        blocks: Option<&horizon_world::movement_properties::BoundMovementProperties>,
        entities: &[Aabb],
    ) -> Result<(Vec<BlockPos>, bool)> {
'''
TNT_WRAPPER = '''    pub(super) fn tick_among(
        &mut self,
        world: &horizon_client::world::World,
        collision: &horizon_world::block_collision::BlockCollisionRegistry<'_>,
        assets: &horizon_client::assets::Assets,
        facts: &horizon_client::blocks::BlockFacts,
        fluids: &horizon_client::motion::FluidResources,
        border: Option<&horizon_core::geometry::border::BorderGeometry>,
        blocks: Option<&horizon_world::movement_properties::BoundMovementProperties>,
        entities: &[Aabb],
    ) -> Result<(Vec<BlockPos>, bool)> {
        self.tick_among_with_bounce(
            world, collision, assets, facts, fluids, border, blocks, entities, false,
        )
    }
    /// Only the local client-level TNT route supplies module suppression.
    #[allow(clippy::too_many_arguments)]
'''

CAPTURE = '''        let mut client_movement_queries = horizon_client::motion::ClientMovementQueries::default();
        if self.remote.is_some() {
            self.native_client_movement_queries(
                &mut client_movement_queries.reset_jump_delay,
                &mut client_movement_queries.suppress_bounce,
                &mut client_movement_queries.ignore_levitation,
                &mut client_movement_queries.ignore_slow_falling,
            );
        }
        let client_suppress_bounce = client_movement_queries.suppress_bounce;
'''


def apply(edit, replace_once):
    def items(text):
        signature = ITEM_SIGNATURE.replace(
            "fn tick_move_before_block_effects(",
            "fn tick_move_before_block_effects_with_bounce(",
        ).replace(
            "        blocks: Option<&BoundMovementProperties>,\n",
            "        blocks: Option<&BoundMovementProperties>,\n        suppress_bounce: bool,\n",
        )
        text = replace_once(text, ITEM_SIGNATURE, ITEM_WRAPPER + signature)
        return replace_once(text, '''                    self.on_ground,
                    slime,
                );
''', '''                    self.on_ground,
                    slime && !suppress_bounce,
                );
''')

    edit("crates/horizon-client/src/entities.rs", items)

    def item_tables(text):
        text = replace_once(text, '''pub(in crate::game) struct ItemTables<'a> {
''', '''pub(in crate::game) struct ItemTables<'a> {
    client_suppress_bounce: bool,
''')
        text = replace_once(text, '''    pub(in crate::game) fn item_tables(&self) -> ItemTables<'_> {
        ItemTables {
''', '''    pub(in crate::game) fn item_tables(&self) -> ItemTables<'_> {
''' + CAPTURE + '''        ItemTables {
            client_suppress_bounce,
''')
        text = replace_once(text, "        let step = item.tick_move_before_block_effects(\n", "        let step = item.tick_move_before_block_effects_with_bounce(\n")
        return replace_once(text, '''            Some(&self.motion.blocks),
        );
        item.check_fall(&step, |fall, from, moved| {
''', '''            Some(&self.motion.blocks),
            self.client_suppress_bounce,
        );
        item.check_fall(&step, |fall, from, moved| {
''')

    edit("crates/horizon-client/src/game/item_entities/ahead.rs", item_tables)

    def tnt(text):
        signature = TNT_SIGNATURE.replace("fn tick_among(\n", "fn tick_among_with_bounce(\n").replace(
            "        entities: &[Aabb],\n",
            "        entities: &[Aabb],\n        suppress_bounce: bool,\n",
        )
        text = replace_once(text, TNT_SIGNATURE, TNT_WRAPPER + signature)
        return replace_once(text, '''                &*facts.get(below).name == "minecraft:slime_block",
            );
''', '''                &*facts.get(below).name == "minecraft:slime_block" && !suppress_bounce,
            );
''')

    edit("crates/horizon-client/src/game/tnt.rs", tnt)

    def remote_tnt(text):
        text = replace_once(text, '''        let (world, collision, assets, facts, fluids, border, blocks) = (
''', CAPTURE + '''        let (world, collision, assets, facts, fluids, border, blocks) = (
''')
        return replace_once(text, '''            let _ = t.tick(world, collision, assets, facts, fluids, border, blocks);
            t.fuse > 0
''', '''            if client_suppress_bounce {
                let _ = t.tick_among_with_bounce(
                    world, collision, assets, facts, fluids, border, blocks, &[], true,
                );
            } else {
                let _ = t.tick(world, collision, assets, facts, fluids, border, blocks);
            }
            t.fuse > 0
''')

    edit("crates/horizon-client/src/game/remote_projectiles.rs", remote_tnt)
    return {"hook_names": [], "helper_names": []}
