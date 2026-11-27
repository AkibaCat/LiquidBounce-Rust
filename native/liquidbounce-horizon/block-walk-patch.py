#!/usr/bin/env python3
"""Full BlockShapeEvent boundary, including empty-cache and support scans."""
from pathlib import Path
HERE = Path(__file__).resolve().parent

HOOKS = '''    /// Original BlockShapeEvent policy is process-wide, like the module singleton.
    pub(super) fn native_client_block_walk(&self, blocks: &mut String, apply: &mut bool) {
        if !self.native_window() || self.lan_acting_uuid.is_some() { return; }
        mod_api::dispatch_client_block_walk(self, mod_api::mods(), blocks, apply);
    }
    pub(super) fn sync_native_block_walk(&self) {
        if !self.native_window() || self.lan_acting_uuid.is_some() { return; }
        let mut blocks = String::from("[]");
        let mut apply = false;
        self.native_client_block_walk(&mut blocks, &mut apply);
        let blocks = if apply { serde_json::from_str(&blocks).unwrap_or_default() } else { Default::default() };
        horizon_world::block_walk::sync(blocks);
    }

'''
TRAIT = '''    /// A BlockShapeEvent can make a cached empty collision shape nonempty.
    fn changes_collision_shapes(&self) -> bool { false }
    /// Only the collision scanner emits this event; direct shape and outline
    /// queries keep their ordinary physical/visual shape.
    fn modify_collision_shape(
        &mut self, _block: CollisionBlock, _position: BlockPos, shape: VoxelShape,
    ) -> Result<VoxelShape, Self::Error> { Ok(shape) }
'''
SCENE = '''    fn changes_collision_shapes(&self) -> bool { horizon_world::block_walk::active() }
    fn modify_collision_shape(
        &mut self, block: CollisionBlock, _: BlockPos, shape: VoxelShape,
    ) -> io::Result<VoxelShape> {
        let Some(state) = self.binding.registry().state(block.state) else { return Ok(shape); };
        Ok(horizon_world::block_walk::shape(&state.name, shape))
    }
'''
REGISTRY = '''    fn changes_collision_shapes(&self) -> bool { crate::block_walk::active() }
    fn modify_collision_shape(
        &mut self, block: CollisionBlock, _: BlockPos, shape: VoxelShape,
    ) -> Result<VoxelShape, Self::Error> {
        let Some(state) = self.binding.collisions().registry().state(block.state) else { return Ok(shape); };
        Ok(crate::block_walk::shape(&state.name, shape))
    }
'''


def apply(edit, once):
    edit('crates/horizon-world/src/block_walk.rs', lambda _: (HERE / 'block-walk-policy.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-world/src/lib.rs', lambda text: once(text, 'pub mod block_collision_scan;\n', 'pub mod block_collision_scan;\npub mod block_walk;\n'))

    def native(text):
        anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        text = once(text, anchor, HOOKS + anchor)
        return once(text, '        mod_api::dispatch_client_tick_pre(self, mod_api::mods());\n',
            '        mod_api::dispatch_client_tick_pre(self, mod_api::mods());\n        self.sync_native_block_walk();\n')
    edit('crates/horizon-client/src/game/native_hooks.rs', native)

    def scan(text):
        text = once(text, 'pub trait BlockCollisionWorld {\n', 'pub trait BlockCollisionWorld {\n' + TRAIT)
        text = once(text, '        let reaches = !empty\n',
            '        let reaches = (!empty || self.world.changes_collision_shapes())\n')
        if '    client_ghost_block_y: Option<i32>,\n' in text:
            # AirJump and BlockWalk both assign Shapes.block(), so their event
            # order commutes. Both obey MixinBlockCollisions' ShapeFlag bypass.
            text = once(text, '            if facts.empty && self.client_ghost_block_y.is_none() {\n',
                '            if facts.empty && self.client_ghost_block_y.is_none() && !self.world.changes_collision_shapes() {\n')
            text = once(text, '            if block.empty_shape && !self.client_ghost_block_y.is_some_and(|y| p.y < y) {\n',
                '            if block.empty_shape && !self.client_ghost_block_y.is_some_and(|y| p.y < y) && !self.world.changes_collision_shapes() {\n')
            text = once(text, '            let shape = if self.client_ghost_block_y.is_some_and(|y| p.y < y) {\n',
                '            let shape = if !crate::block_walk::shape_changes_suppressed() && self.client_ghost_block_y.is_some_and(|y| p.y < y) {\n')
        else:
            text = once(text, '            if facts.empty {\n',
                '            if facts.empty && !self.world.changes_collision_shapes() {\n')
            text = once(text, '            if block.empty_shape {\n',
                '            if block.empty_shape && !self.world.changes_collision_shapes() {\n')
        return once(text, '            let offset = Vec3::new(f64::from(p.x), f64::from(p.y), f64::from(p.z));\n',
            '''            let shape = self.world.modify_collision_shape(block, p, shape)
                .map_err(CollisionScanError::World)?;
            let offset = Vec3::new(f64::from(p.x), f64::from(p.y), f64::from(p.z));
''')
    edit('crates/horizon-world/src/block_collision_scan.rs', scan)

    def scene(text):
        text = once(text, '    pub fn cells_without_collision(&self, bounds: Aabb) -> bool {\n',
            '    pub fn cells_without_collision(&self, bounds: Aabb) -> bool {\n        if horizon_world::block_walk::active() { return false; }\n')
        text = once(text, "impl<'w> BlockCollisionWorld for SceneCollisions<'w, '_> {\n",
            "impl<'w> BlockCollisionWorld for SceneCollisions<'w, '_> {\n" + SCENE)
        text = once(text, "impl BlockCollisionWorld for LookupCollisions<'_, '_> {\n",
            "impl BlockCollisionWorld for LookupCollisions<'_, '_> {\n" + SCENE)
        # EntityCollisions wraps SceneCollisions for actual player motion.
        return once(text, "impl<W: BlockCollisionWorld> BlockCollisionWorld for EntityCollisions<'_, W> {\n",
            '''impl<W: BlockCollisionWorld> BlockCollisionWorld for EntityCollisions<'_, W> {
    fn changes_collision_shapes(&self) -> bool { self.blocks.changes_collision_shapes() }
    fn modify_collision_shape(
        &mut self, block: CollisionBlock, p: BlockPos, shape: VoxelShape,
    ) -> std::result::Result<VoxelShape, Self::Error> {
        self.blocks.modify_collision_shape(block, p, shape)
    }
''')
    edit('crates/horizon-client-render/src/collisions.rs', scene)

    def registry(text):
        anchor = '    type Error = CollisionQueryError<E>;\n'
        return once(text, anchor, REGISTRY + anchor)
    edit('crates/horizon-world/src/registry_collision_scan.rs', registry)

    def support(text):
        start = text.index('    /// `Level.findSupportingBlock`:')
        end = text.index('    pub fn no_collision(', start)
        before = text[start:end]
        after = '''    /// `Level.findSupportingBlock` consumes BlockCollisions, including
    /// its voxel intersection tolerance, cursor filters and BlockShapeEvent.
    pub fn supporting_block(&self, area: Aabb, from: Vec3) -> Option<[i32; 3]> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        horizon_world::ground_support::find_supporting_block(
            &mut scene,
            horizon_world::collision_context::CollisionContext::empty(false),
            from,
            area,
            usize::MAX,
        ).ok().flatten().map(|p| [p.x, p.y, p.z])
    }
'''
        return once(text, before, after)
    edit('crates/horizon-client/src/mobs/living.rs', support)
    return {'hook_names': ['client_block_walk'], 'helper_names': []}
