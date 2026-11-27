// SPDX-License-Identifier: GPL-3.0-or-later
// Appended to horizon_world::level::place, retaining private vanilla queries.
impl Behaviours {
    /// BlockItem.getPlacementState(BlockPlaceContext(UseOnContext)) directly,
    /// as Scaffold's simulated-attempt predicate calls it. This is before
    /// BlockItem.place/canPlace/updatePlacementContext gates. The virtual
    /// GameMasterBlockItem.getPlacementState permission check still applies.
    pub fn liquidbounce_scaffold_placement_state<L: LevelAccess + ?Sized>(
        &self,
        level: &mut L,
        ticks: &mut Ticks,
        input: &PlaceContext,
        mut unobstructed: impl FnMut(&L, BlockPos, BlockStateId, bool) -> bool,
    ) -> Option<BlockStateId> {
        let mut sim = self.sim(level, ticks);
        let binding = self.item_binding(&input.item)?;
        if matches!(binding.kind, ItemKind::SolidBucket | ItemKind::Other) {
            return None;
        }
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
        if binding.game_master && !input.can_use_game_master_blocks {
            return None;
        }
        match binding.kind {
            ItemKind::StandingAndWall | ItemKind::HangingSign => {
                let wall = binding
                    .wall_block
                    .and_then(|block| sim.state_for_placement(block, &ctx));
                let attachment = binding.attachment.unwrap_or(Direction::Down);
                let mut chosen = None;
                for direction in ctx.nearest_looking_directions() {
                    if direction == attachment.opposite() {
                        continue;
                    }
                    let candidate = if direction == attachment {
                        sim.state_for_placement(binding.block, &ctx)
                    } else {
                        wall
                    };
                    if let Some(candidate) = candidate {
                        let hanging_ok = binding.kind != ItemKind::HangingSign
                            || !self.instance_of(candidate, Class::WallHangingSign)
                            || sim.wall_hanging_sign_can_place(candidate, ctx.pos);
                        if hanging_ok && sim.can_survive(candidate, ctx.pos) {
                            chosen = Some(candidate);
                            break;
                        }
                    }
                }
                chosen.filter(|state| unobstructed(sim.level, ctx.pos, *state, false))
            }
            _ => {
                let must_survive = binding.kind != ItemKind::Scaffolding;
                sim.state_for_placement(binding.block, &ctx)
                    .filter(|state| {
                        (!must_survive || sim.can_survive(*state, ctx.pos))
                            && unobstructed(sim.level, ctx.pos, *state, true)
                    })
            }
        }
    }
}
