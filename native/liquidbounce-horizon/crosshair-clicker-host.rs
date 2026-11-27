pub(super) fn native_client_crosshair_clicker_context(&self) -> Option<serde_json::Value> {
    if !self.native_client_slot_owner() {
        return None;
    }
    let main = self
        .inventory
        .get(self.client_hotbar_slot())
        .and_then(Option::as_ref);
    let off = self.offhand.as_ref();
    let main_item = main.map_or("minecraft:air", |s| &*s.item);
    let off_item = off.map_or("minecraft:air", |s| &*s.item);
    let tag = |s: &str| self.items().item_in_tag(s, main_item);
    let weapon = 1u16
        | (u16::from(tag("minecraft:swords")) << 1)
        | (u16::from(tag("minecraft:axes")) << 2)
        | (u16::from(main_item == "minecraft:mace") << 3)
        | (u16::from(tag("minecraft:spears")) << 4)
        | (u16::from(tag("minecraft:pickaxes")) << 5)
        | (u16::from(tag("minecraft:shovels")) << 6)
        | (u16::from(tag("minecraft:hoes")) << 7)
        | (u16::from(main.is_some_and(|s| s.enchantment_level("minecraft:knockback") > 0)) << 8)
        | (u16::from(main.is_some_and(|s| s.enchantment_level("minecraft:fire_aspect") > 0)) << 9);
    let entity = self.client_crosshair_entity();
    let block = entity.is_none().then(|| self.pick()).flatten();
    let attack = self.settings.keys.get("key.attack");
    let use_key = self.settings.keys.get("key.use");
    let keyboard = |key: &str| {
        horizon_client::keybinds::key_code(key).is_some_and(|key| self.physical_keys.contains(&key))
    };
    let block_item = |item: &str| self.items().get(item).is_some_and(|i| i.block.is_some());
    Some(
        serde_json::json!({"entity":entity,"block":block.is_some(),"hitBlock":block.map(|h|self.res.facts().get(self.world.block(h.pos[0],h.pos[1],h.pos[2])).name.to_string()),
            "attackKey":attack,"useKey":use_key,"attackKeyboard":keyboard(attack),"useKeyboard":keyboard(use_key),"weapon":weapon,
            "using":self.using.is_some(),"destroying":self.remote.as_ref().map_or(self.clicks.destroying.active,|r|r.is_destroying),
            "mainItem":main_item,"offItem":off_item,"mainNamed":main.is_some_and(|s|s.custom_name().is_some()),"mainBlockItem":block_item(main_item),"offBlockItem":block_item(off_item)}),
    )
}
/// Nearest actual EntityHitResult, including entities without pick-block data.
fn client_crosshair_entity(&self) -> Option<i32> {
    let mut best: Option<(f64, i32)> = None;
    let mut offer = |d: f64, id: i32| {
        if self.before_block(d) && best.is_none_or(|b| d < b.0) {
            best = Some((d, id));
        }
    };
    if let Some(h) = self.remote_object_hit(true) {
        offer(h.distance, h.id);
    }
    if let Some((i, d, _)) = self.mob_hit() {
        offer(d, self.mobs[i].id as i32);
    }
    if let Some((i, d)) = self.redirectable_hit() {
        offer(d, self.creatures.projectiles[i].id as i32);
    }
    if let Some((id, d)) = self.dragon_hit() {
        offer(d, id as i32);
    }
    best.map(|b| b.1)
}
pub(super) fn native_client_crosshair_clicker_blocks(&self) -> Vec<String> {
    let mut blocks = std::collections::BTreeSet::new();
    for id in 0..self.pack.assets.registry.len() {
        let f = self
            .res
            .facts()
            .get(horizon_world::registry::BlockStateId(id as u32));
        if ["DoorBlock", "FenceGateBlock", "TrapDoorBlock"]
            .into_iter()
            .any(|c| f.has_class(c))
        {
            blocks.insert(f.name.to_string());
        }
    }
    blocks.into_iter().collect()
}
pub(super) fn native_client_crosshair_clicker_critical(&self) -> Option<serde_json::Value> {
    let mut v = self.native_client_super_knockback_context(u32::MAX)?;
    let p = self.player.as_ref()?;
    // SuperKnockback uses the same blocking predicates with onGround; Smart
    // selection explicitly ignores that one predicate.
    let allows = if p.ground.on_ground {
        !p.fluids.was_touching_water
            && !p.is_in_lava()
            && self.riding.is_none()
            && !self.hands_busy()
            && !p.flying
            && !p.on_climbable
            && !["levitation", "blindness", "slow_falling"]
                .into_iter()
                .any(|id| self.potions.effects.get(id).is_some())
            && !self.client_crosshair_in_web()
    } else {
        v["criticalAllowed"].as_bool().unwrap_or(false)
    } && !(horizon_world::liquid_walk::mode().is_some()
        && horizon_client::liquid_walk_native::row(&self.world, self.collision, p.bounds(), 0.01));
    let speed = self.main_hand_attack_speed();
    v["allowsCritical"] = serde_json::json!(allows);
    v["hasCooldown"] = serde_json::json!(!self.legacy_rules() && speed < 20.0);
    v["jumping"] = serde_json::json!(horizon_client::reverse_step_bridge::keys()[4]);
    v["fallFlying"] = serde_json::json!(p.fall_flying);
    v["velocityY"] = serde_json::json!(p.velocity.y);
    v["missTime"] = serde_json::json!(self.clicks.miss_time);
    Some(v)
}
fn client_crosshair_in_web(&self) -> bool {
    use horizon_core::geometry::Axis;
    let Some(p) = self.player.as_ref() else {
        return false;
    };
    let b = p.bounds();
    for x in b.min(Axis::X).floor() as i32..=b.max(Axis::X).floor() as i32 {
        for y in b.min(Axis::Y).floor() as i32..=b.max(Axis::Y).floor() as i32 {
            for z in b.min(Axis::Z).floor() as i32..=b.max(Axis::Z).floor() as i32 {
                if self
                    .res
                    .facts()
                    .get(self.world.block(x, y, z))
                    .has_class("WebBlock")
                {
                    return true;
                }
            }
        }
    }
    false
}
pub(super) fn native_client_crosshair_clicker_target(&self, target: i32) -> [bool; 2] {
    let living = u32::try_from(target)
        .ok()
        .and_then(|id| self.native_entity(id))
        .and_then(|h| self.native_living(h))
        .is_some();
    [
        living,
        living && self.client_weapon_would_block_hit(target as u32),
    ]
}
pub(super) fn native_client_crosshair_clicker_click(&self, attack: bool) {
    if !self.native_client_slot_owner() {
        return;
    }
    let key = self
        .settings
        .keys
        .get(if attack { "key.attack" } else { "key.use" });
    self.client_keymapping_clicks
        .borrow_mut()
        .click(key, self.settings.keys.mappings_for(key));
}
pub(super) fn native_client_crosshair_clicker_release(&mut self) {
    if !self.native_client_slot_owner() {
        return;
    }
    self.native_client_ensure_carried();
    if self.remote.is_some() {
        self.release_remote_use();
    } else if let Some((slot, ticks)) = self.using
        && let Some(stack) = self.stack_in(slot).cloned()
    {
        self.release_used_item(&stack, ticks, false);
    }
}
pub(super) fn native_client_crosshair_clicker_attack_key(&self, pressed: &mut bool) {
    if self.native_client_slot_owner() {
        mod_api::dispatch_client_crosshair_clicker_attack_key(self, mod_api::mods(), pressed);
    }
}
pub(super) fn native_client_crosshair_clicker_falling(
    &mut self,
    ticks: i32,
    add_y: f64,
) -> Result<Option<i32>, String> {
    super::crosshair_falling::collision(self, ticks, add_y)
}
