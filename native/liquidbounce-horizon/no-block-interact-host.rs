    pub(super) fn client_no_block_player(&self) -> bool {
        self.window.is_some() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_reach_queries_allowed())
    }
    pub(super) fn native_client_start_use_item(&mut self) {
        if !self.client_no_block_player() { return; }
        if self.remote.is_some() { self.remote_use(); } else { let _ = self.use_with_hand(); }
    }
    pub(super) fn client_no_block_item(&self, held: Option<&ItemStack>) -> bool {
        let Some(held) = held else { return false; };
        let view = horizon_client::item_render::stack_view(held);
        let components = self.pack.item_renderer.models.components_view(&view);
        let equippable = components.get("minecraft:equippable");
        let slot = equippable.and_then(|value| value.get("slot")).and_then(serde_json::Value::as_str);
        let current = match slot {
            Some("mainhand") => self.stack_in(self.client_hotbar_slot()),
            Some("offhand") => self.offhand.as_ref(),
            Some("head") => self.armor[0].as_ref(), Some("chest") => self.armor[1].as_ref(),
            Some("legs") => self.armor[2].as_ref(), Some("feet") => self.armor[3].as_ref(),
            _ => None,
        };
        let allowed = equippable.and_then(|value| value.get("allowed_entities")).is_none_or(|values| {
            let matches = |value: &serde_json::Value| {
                value.as_str().is_some_and(|id| {
                    if let Some(tag) = id.strip_prefix('#') {
                        let tag = if tag.contains(':') { tag.to_owned() } else { format!("minecraft:{tag}") };
                        self.items().enchantments.entity_tags.get(&tag).is_some_and(|values| values.contains("minecraft:player"))
                    } else { id == "minecraft:player" || id == "player" }
                })
            };
            values.as_array().map_or_else(|| matches(values), |values| values.iter().any(matches))
        });
        let prevent_change = current.is_some_and(|stack| stack.enchantments().iter().any(|(id, _)| {
            self.items().enchantments.get(id).is_some_and(|enchantment| enchantment.effects.get("minecraft:prevent_armor_change").is_some())
        }));
        let arrow = |stack: &ItemStack| stack.count > 0 && client_no_block_rules::item_facts::item_is(&stack.item, "ArrowItem");
        let main = self.stack_in(self.client_hotbar_slot());
        let hand_firework = [main, self.offhand.as_ref()].into_iter().flatten().any(|stack| stack.count > 0 && client_no_block_rules::item_facts::item_is(&stack.item, "FireworkRocketItem"));
        let arrow = self.inventory.iter().flatten().chain(self.offhand.iter()).chain(self.armor.iter().flatten()).any(arrow);
        client_no_block_rules::item_is_interactable(&held.item, held.count, |key| components.get(key).cloned(), client_no_block_rules::ItemContext {
            // Player.canUseSlot returns true for every EquipmentSlot in MC26.3.
            equippable_allowed: allowed,
            equipped_same_components: current.is_some_and(|stack| stack.same_item_same_components(held)),
            equipped_prevents_armor_change: prevent_change,
            creative: self.creative,
            has_inventory_arrow: arrow,
            hand_has_firework: hand_firework,
        })
    }
    pub(super) fn native_client_block_interact(&self, should_sneak: bool, canceled: &mut bool) {
        if !self.client_no_block_player() { return; }
        mod_api::dispatch_client_block_interact(self, mod_api::mods(), should_sneak, canceled);
    }
    pub(super) fn client_cancel_block_interact(&self, hit: &Hit) -> bool {
        if !self.client_no_block_player() { return false; }
        let shift = self.player.as_ref().is_some_and(|player| player.shift_key_down);
        if shift { return false; }
        let state = self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
        let facts = self.res.facts().get(state);
        let properties = self.pack.assets.registry.state(state);
        let interactable = client_no_block_rules::block_is_interactable(
            |class| facts.has_class(class) || client_no_block_rules::item_facts::block_is(&facts.name, class),
            |name| properties.and_then(|state| state.properties.get(name)).map(String::as_str),
            self.player.as_ref().is_some_and(|player| player.health.food.food < 20), self.can_use_game_master_blocks());
        let main = self.client_no_block_item(self.stack_in(self.client_hotbar_slot()));
        let off = self.client_no_block_item(self.offhand.as_ref());
        let should_sneak = client_no_block_rules::should_sneak(shift, interactable, main, off);
        let mut canceled = false;
        self.native_client_block_interact(should_sneak, &mut canceled);
        canceled
    }
