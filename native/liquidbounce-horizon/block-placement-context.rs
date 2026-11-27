// SPDX-License-Identifier: GPL-3.0-or-later
// Appended to horizon_world::level::place, so it shares vanilla's private rules.
impl Behaviours {
    /// LiquidBounce resolvePlacementAt: BlockPlaceContext replacement and the
    /// BlockItem's context update, without getPlacementState/survival/obstruction.
    /// Non-block items such as water buckets still use BlockPlaceContext.canPlace.
    pub fn liquidbounce_resolve_placement_context<L: LevelAccess + ?Sized>(
        &self,
        level: &mut L,
        ticks: &mut Ticks,
        input: &PlaceContext,
    ) -> Option<PlacementContext> {
        let sim = self.sim(level, ticks);
        let mut ctx = PlacementContext {
            pos: input.clicked_pos,
            hit_pos: input.clicked_pos,
            face: input.face,
            hit: input.hit,
            inside: input.inside,
            yaw: input.yaw,
            pitch: input.pitch,
            sneaking: input.sneaking,
            item: input.item.clone(),
            replace_clicked: true,
        };
        ctx.replace_clicked = sim.can_be_replaced(sim.state(input.clicked_pos), &ctx);
        if !ctx.replace_clicked {
            ctx.pos = rel(input.clicked_pos, input.face);
        }
        if let Some(binding) = self.item_binding(&input.item)
            && binding.kind == ItemKind::Scaffolding
        {
            ctx = sim.scaffolding_update_context(&ctx, binding.block)?;
        }
        (ctx.replace_clicked || sim.can_be_replaced(sim.state(ctx.pos), &ctx)).then_some(ctx)
    }
    /// BlockState.isFaceSturdy(UP,CENTER), including moving and open block entities.
    pub fn liquidbounce_placement_holds_solid<L: LevelAccess + ?Sized>(
        &self,
        level: &mut L,
        ticks: &mut Ticks,
        pos: BlockPos,
    ) -> bool {
        self.sim(level, ticks)
            .can_support_center(pos, Direction::Up)
    }
}
