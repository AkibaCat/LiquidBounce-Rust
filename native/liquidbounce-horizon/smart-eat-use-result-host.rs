    /// Pre-use MultiPlayerGameMode.useItem result predicate. It reads actual
    /// effective components and a dedicated Item.getPlayerPOVHitResult ray;
    /// native success/consumed booleans never enter this classification.
    pub(super) fn client_item_use_pass(&self, stack:&horizon_client::items::ItemStack) -> Option<bool> {
        use client_smart_eat_use::{UseFacts,UseOwner,passes};
        if !self.client_smart_eat_player() {return None;}
        if self.spectator() {return Some(true);}
        let view=horizon_client::item_render::stack_view(stack);
        let components=self.pack.item_renderer.models.components_view(&view);
        let group=components.get("minecraft:use_cooldown").and_then(|v|v.get("cooldown_group")).and_then(serde_json::Value::as_str).unwrap_or(&stack.item);
        let mut facts=UseFacts {spectator:self.spectator(),cooldown:self.cooldowns.is_on_cooldown(group),..UseFacts::default()};
        if facts.spectator||facts.cooldown {return Some(true);}
        let owner=client_smart_eat_use::owner(&stack.item)?;
        match owner {
            UseOwner::Item=> {
                facts.consumable=components.contains_key("minecraft:consumable");
                let equipment=components.get("minecraft:equippable");
                facts.swappable_equippable=equipment.is_some_and(|v|v.get("swappable").is_none_or(|v|v.as_bool().unwrap_or_else(||v.as_i64().is_some_and(|n|n!=0))));
                facts.equipment_allows_player=equipment.and_then(|v|v.get("allowed_entities")).is_none_or(|v|self.client_equipment_allows_player(v));
                facts.blocks_attacks=components.contains_key("minecraft:blocks_attacks");
                facts.kinetic_weapon=components.contains_key("minecraft:kinetic_weapon");
            },
            UseOwner::BoatItem=> {
                facts.boat_ray_hit= !self.client_item_use_ray(horizon_world::world_ray::RayFluidMode::Any)?.miss;
                if facts.boat_ray_hit {facts.boat_eye_inside_pickable=self.client_boat_eye_inside_pickable()?;}
            },
            UseOwner::BottleItem=> {
                let hit=self.client_item_use_ray(horizon_world::world_ray::RayFluidMode::SourceOnly)?;
                facts.source_ray_hit= !hit.miss;
                let p=hit.hit.block;
                facts.source_is_water=self.res.facts().get(self.world.block(p.x,p.y,p.z)).fluid==horizon_client::blocks::Fluid::Water;
                facts.collectible_dragon_cloud=self.client_has_owned_dragon_cloud();
            },
            UseOwner::BucketItem=> {
                facts.empty_bucket=&*stack.item=="minecraft:bucket";
                let mode=if facts.empty_bucket {horizon_world::world_ray::RayFluidMode::SourceOnly}else {horizon_world::world_ray::RayFluidMode::None};
                let hit= !self.client_item_use_ray(mode)?.miss;
                facts.source_ray_hit=hit;facts.solid_ray_hit=hit;
            },
            UseOwner::EnderEyeItem=> {
                let hit=self.client_item_use_ray(horizon_world::world_ray::RayFluidMode::None)?;
                let p=hit.hit.block;
                facts.end_portal_frame= !hit.miss&&&*self.block_name([p.x,p.y,p.z])=="minecraft:end_portal_frame";
            },
            UseOwner::FireworkRocketItem=>facts.gliding=self.player.as_ref().is_some_and(|p|p.fall_flying),
            UseOwner::SpawnEggItem=>facts.source_ray_hit= !self.client_item_use_ray(horizon_world::world_ray::RayFluidMode::SourceOnly)?.miss,
            _=>{},
        }
        Some(passes(owner,&facts))
    }
    fn client_equipment_allows_player(&self,spec:&serde_json::Value)->bool {
        let one=|value:&serde_json::Value| {
            let Some(id)=value.as_str() else {return false;};
            let (tag,id)=id.strip_prefix('#').map_or((false,id),|s|(true,s));
            let id=if id.contains(':') {id.to_owned()}else {format!("minecraft:{id}")};
            if tag {self.client_item_entity_tag_contains(&id,"minecraft:player")}
            else {id=="minecraft:player"}
        };
        spec.as_array().map_or_else(||one(spec),|values|values.iter().any(one))
    }
    fn client_item_use_ray(&self,fluid:horizon_world::world_ray::RayFluidMode)->Option<horizon_world::world_ray::RayResult> {
        client_smart_eat_ray::trace(self,fluid)
    }
    /// Entity tags can be replaced in configuration or a live UpdateTags
    /// packet. A synced registry's missing tag is empty, never a local fallback.
    fn client_item_entity_tag_contains(&self,tag:&str,entity:&str)->bool {
        if let Some(remote)=&self.remote
            && let Some(tags)=remote.registries.tags.get("minecraft:entity_type") {
            return remote.builtin.get("minecraft:entity_type").and_then(|ids|ids.id(entity))
                .is_some_and(|id|tags.get(tag).is_some_and(|members|members.contains(&id)));
        }
        self.stations().enchantments.entity_tags.get(tag).is_some_and(|ids|ids.contains(entity))
    }
