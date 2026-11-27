    /// AttackEntityEvent is MultiPlayerGameMode.attack HEAD, before carried sync
    /// and the ordinary attack packet. A cancelled event still ran its handlers.
    pub(super) fn native_client_attack_entity(&mut self, target: u32, living: bool, would_block_hit: bool, cancel: &mut bool) {
        if !self.native_client_slot_owner() { return; }
        mod_api::dispatch_client_attack_entity(self, mod_api::mods(), target, living, would_block_hit, cancel);
    }
    /// The actual getCurrentItemAttackStrengthDelay attribute expression.
    pub(super) fn native_client_attack_speed(&self, original: f64, speed: &mut f64) {
        if !self.native_client_slot_owner() { return; }
        mod_api::dispatch_client_attack_speed(self, mod_api::mods(), original, speed);
    }
    pub(super) fn native_client_ensure_carried(&self) {
        if self.native_client_slot_owner() { self.remote_send_carried(); }
    }
    /// Actual item component calculation in declaration order. MC26.3's
    /// ItemAttributeModifiers.compute is one pass, not AttributeInstance's
    /// three operation buckets (confirmed against the target game's bytecode).
    fn client_weapon_item_attribute(&self, stack: &horizon_client::items::ItemStack, attribute: &str, base: f64) -> f64 {
        let mut value = base;
        for (kind, _, amount, operation, group) in self.items().attribute_modifiers(stack) {
            if kind != attribute || !matches!(group.as_str(), "any" | "hand" | "mainhand" | "main_hand") { continue; }
            value += match operation.as_str() {
                "add_value" => amount,
                "add_multiplied_base" => amount * base,
                "add_multiplied_total" => amount * value,
                _ => continue,
            };
        }
        self.player.as_ref().and_then(|player| {
            let catalog = player.attributes.catalog();
            catalog.id(attribute).map(|id| catalog.attribute(id).definition.sanitize(value))
        }).unwrap_or(value)
    }
    fn client_weapon_stack(&self, slot: i32, stack: &horizon_client::items::ItemStack, bases: [f64; 2]) -> serde_json::Value {
        use serde_json::json;
        let items = self.items();
        let id = &*stack.item;
        let tag = |name: &str| items.item_tags.get(name).is_some_and(|members| members.contains(id));
        let is = |class: &str| client_weapon_item_facts::item_is(id, class);
        let sword = tag("minecraft:swords"); let axe = tag("minecraft:axes");
        let pickaxe = tag("minecraft:pickaxes"); let shovel = tag("minecraft:shovels");
        let spear = tag("minecraft:spears"); let hoe = tag("minecraft:hoes");
        let mace = id == "minecraft:mace";
        let registry = &self.stations().enchantments;
        let level = |name: &str| {
            let key = format!("minecraft:{name}");
            if registry.get(&key).is_none() { 0 } else { stack.enchantment_level(&key) as i32 }
        };
        let sharpness = level("sharpness");
        let legacy = self.legacy_rules();
        let base_damage = if legacy {
            let tool = if sword {4.0} else if axe {3.0} else if pickaxe {2.0} else if shovel {1.0} else {0.0};
            let material = id.strip_prefix("minecraft:").unwrap_or(id).split('_').next().unwrap_or("");
            let bonus = if tool == 0.0 {0.0} else {match material { "stone" => 1.0, "iron" => 2.0, "diamond" | "netherite" => 3.0, _ => 0.0 }};
            bases[0] + tool + bonus
        } else { self.client_weapon_item_attribute(stack, "minecraft:attack_damage", bases[0]) };
        let damage = base_damage + if legacy {f64::from(sharpness) * 1.25} else if sharpness == 0 {0.0} else {f64::from(sharpness) * 0.5 + 0.5};
        let speed = self.client_weapon_item_attribute(stack, "minecraft:attack_speed", bases[1]);
        let view = horizon_client::item_render::stack_view(stack);
        let components = self.pack.item_renderer.models.components_view(&view);
        let enchantable = components.get("minecraft:enchantable").and_then(|v| v.get("value")).and_then(serde_json::Value::as_i64).unwrap_or(0) as i32;
        let excluded = ["BowItem","CrossbowItem","ArrowItem","FishingRodItem","BlockItem","BucketItem","PotionItem","BottleItem","EnderpearlItem","EggItem","SnowballItem","WindChargeItem"].into_iter().any(is)
            // ShieldItem is absent from NoInteract's selected class table;
            // the builtin shield is its sole registered MC26.3 instance.
            || matches!(id,"minecraft:shield"|"minecraft:milk_bucket"|"minecraft:totem_of_undying"|"minecraft:golden_apple"|"minecraft:enchanted_golden_apple");
        // AutoArmor's default curse filter accepts all player armour. Its policy
        // bridge can override this input when that module is ported.
        let armor = ["minecraft:head_armor","minecraft:chest_armor","minecraft:leg_armor","minecraft:foot_armor"].into_iter().any(tag);
        let specialization = if excluded {0} else if axe {if (5..100).contains(&sharpness) {3} else {0}}
            else if armor {0} else if sword {1} else if spear {2} else if mace {4} else {0};
        let kinds = (u16::from(sword)<<1) | (u16::from(axe)<<2) | (u16::from(mace)<<3)
            | (u16::from(spear)<<4) | (u16::from(pickaxe)<<5) | (u16::from(shovel)<<6) | (u16::from(hoe)<<7);
        json!({"slot":slot,"identity":stack.object_identity(),"identityHash":stack.object_identity_hash(),
            "empty":stack.count==0 || id=="minecraft:air","kinds":kinds,"specialization":specialization,
            "damage":damage,"speed":speed,"durability":(items.stack_max_damage(stack) as i32).wrapping_sub(stack.damage as i32),"enchantable":enchantable,
            "enchantments":[sharpness,level("fire_aspect"),level("smite"),level("bane_of_arthropods"),level("knockback"),level("looting"),level("unbreaking"),level("mending"),level("vanishing_curse"),level("sweeping_edge"),level("density"),level("breach"),level("wind_burst")]})
    }
    /// Snapshot reads actual objects; their clone/read snapshots preserve opaque
    /// identity. Player attribute bases deliberately exclude potion modifiers.
    pub(super) fn native_client_weapon_context(&self) -> Option<serde_json::Value> {
        if !self.native_client_slot_owner() { return None; }
        let player = self.player.as_ref()?;
        let damage = player.attributes.base_value_of("minecraft:attack_damage").ok()?;
        let base_speed = player.attributes.base_value_of("minecraft:attack_speed").ok()?;
        let current_speed = player.attributes.computed_value_of("minecraft:attack_speed").ok()?;
        let can_smash = player.fall_distance > 1.5 && !player.fall_flying;
        let bases = [damage,base_speed];
        let hotbar: Vec<_> = self.inventory.iter().take(9).enumerate().filter_map(|(slot,stack)| stack.as_ref().map(|stack|self.client_weapon_stack(slot as i32,stack,bases))).collect();
        let stick = self.client_weapon_stack(-1,&horizon_client::items::ItemStack::new("minecraft:stick",1),bases);
        let consuming = self.using.is_some_and(|(slot,_)| slot != super::shield::OFFHAND_SLOT && self.stack_in(slot).is_some_and(|stack| {
            let view = horizon_client::item_render::stack_view(stack);
            self.pack.item_renderer.models.components_view(&view).get("minecraft:consumable").is_some()
        }));
        Some(serde_json::json!({"hotbar":hotbar,"stick":stick,"legacy":self.legacy_rules(),"canSmash":can_smash,
            "consumingMainHand":consuming,"currentSpeed":current_speed,"selected":self.client_hotbar_slot()}))
    }
    /// LiquidBounce's getBlockedDamage(playerAttack,1.0F), including the actual
    /// used stack component, use delay, damage tag bypass, yHeadRot and angle.
    fn client_weapon_would_block_hit(&self, target: u32) -> bool {
        if self.legacy_rules() { return false; }
        let Some(mob) = self.mobs.iter().find(|mob| mob.id == target && !mob.removed) else { return false; };
        if mob.extra.using_item <= 0 { return false; }
        let Some(stack) = horizon_client::mobs::finalize::item_stack(mob, usize::from(mob.extra.using_off_hand)) else { return false; };
        let Some(component) = self.blocks_attacks(&stack) else { return false; };
        if mob.extra.using_item - 1 < (f64::from(component.delay_seconds * 20.0) + 0.5).floor() as i32 { return false; }
        let matches = |spec: &str| match spec.strip_prefix('#') {
            Some(tag) => self.stations().enchantments.damage_tags.get(tag).is_some_and(|types|types.contains("minecraft:player_attack")),
            None => spec == "minecraft:player_attack",
        };
        if component.bypassed_by.as_deref().is_some_and(matches) { return false; }
        let Some(player) = self.player.as_ref() else { return false; };
        let yaw = -mob.head_yaw * horizon_core::geometry::trig::DEG_TO_RAD;
        let view = [f64::from(horizon_core::geometry::trig::sin(f64::from(yaw))),f64::from(horizon_core::geometry::trig::cos(f64::from(yaw)))];
        let dx = player.position.x - mob.position.x; let dz = player.position.z - mob.position.z;
        let length = (dx*dx+dz*dz).sqrt();
        let direction = if length < 1.0e-4 {[0.0,0.0]} else {[dx/length,dz/length]};
        let angle = horizon_core::jmath::acos(direction[0]*view[0]+direction[1]*view[1]);
        let mut blocked = 0.0_f32;
        for reduction in &component.reductions {
            if angle > f64::from(0.017_453_292_f32 * reduction.horizontal_angle) || reduction.types.as_ref().is_some_and(|types|!types.iter().any(|spec|matches(spec))) { continue; }
            let value = reduction.base + reduction.factor;
            blocked += if value < 0.0 {0.0} else if value.is_nan() {f32::NAN} else {value.min(1.0)};
        }
        blocked > 0.0
    }
    pub(super) fn client_weapon_before_attack(&mut self, target: u32, living: bool) -> bool {
        let blocked = living && self.client_weapon_would_block_hit(target);
        let mut cancel = false;
        self.native_client_attack_entity(target,living,blocked,&mut cancel);
        !cancel
    }
