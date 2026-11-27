// SPDX-License-Identifier: GPL-3.0-or-later
    fn client_inventory_cleaner_stack(&self, slot:i32, stack:&horizon_client::items::ItemStack, bases:[f64;2], policy:&serde_json::Value)->serde_json::Value {
        use serde_json::{Value,json};
        let items=self.items();let id=&*stack.item;
        let view=horizon_client::item_render::stack_view(stack);
        let components=self.pack.item_renderer.models.components_view(&view).to_map();
        let get=|key:&str|components.get(key);
        let is=|class:&str|client_weapon_item_facts::item_is(id,class);
        let mut classes=["BowItem","CrossbowItem","ArrowItem","FishingRodItem","BlockItem","BucketItem","PotionItem","EnderpearlItem","EggItem","SnowballItem","WindChargeItem"].into_iter().filter(|class|is(class)).collect::<Vec<_>>();
        if id=="minecraft:splash_potion" {classes.push("SplashPotionItem");}else if id=="minecraft:lingering_potion" {classes.push("LingeringPotionItem");}
        if get("minecraft:glider").is_some() {classes.push("Glider");}
        let mut weapon=self.client_weapon_stack(slot,stack,bases);
        let names=["sharpness","fire_aspect","smite","bane_of_arthropods","knockback","looting","unbreaking","mending","vanishing_curse","sweeping_edge","density","breach","wind_burst","power","punch","flame","infinity","quick_charge","multishot","piercing","silk_touch","fortune","protection","projectile_protection","fire_protection","blast_protection","feather_falling","thorns","respiration","aqua_affinity","binding_curse"];
        let registry=&self.stations().enchantments;
        let enchantments=names.into_iter().map(|name| {let key=format!("minecraft:{name}");(name,if registry.get(&key).is_some(){stack.enchantment_level(&key) as i32}else{0})}).collect::<std::collections::BTreeMap<_,_>>();
        let tag=|name:&str|items.item_in_tag(name,id);
        let player_armor=["minecraft:foot_armor","minecraft:leg_armor","minecraft:chest_armor","minecraft:head_armor"].into_iter().any(tag);
        let equip=get("minecraft:equippable").and_then(|v|v.get("slot")).and_then(Value::as_str);
        let armor=if player_armor {match equip {Some("feet")=>Some(0),Some("legs")=>Some(1),Some("chest")=>Some(2),Some("head")=>Some(3),_=>None}}else{None};
        let attribute=|attribute:&str| {let mut value=0.;for (kind,_,amount,op,group) in items.attribute_modifiers(stack) {
            if kind!=attribute||!match equip {None=>true,Some(slot)=>group=="any"||group==slot||group=="armor"||(group=="hand"&&matches!(slot,"mainhand"|"offhand"|"main_hand"|"off_hand"))} {continue;}
            value+=match op.as_str(){"add_value"=>amount,"add_multiplied_base"=>0.,"add_multiplied_total"=>amount*value,_=>0.};
        } self.player.as_ref().and_then(|p|{let catalog=p.attributes.catalog();catalog.id(attribute).map(|id|catalog.attribute(id).definition.sanitize(value))}).unwrap_or(value)};
        let mut effects=vec![];
        let potion=get("minecraft:potion_contents");
        if let Some(base)=potion.and_then(|v|v.get("potion")).and_then(Value::as_str).and_then(horizon_client::brewing::potion) {
            effects.extend(base.effects.iter().map(|(id,amplifier,duration)|json!({"id":id,"amplifier":amplifier,"duration":duration})));
        }
        if let Some(custom)=potion.and_then(|v|v.get("custom_effects")).and_then(Value::as_array) {for effect in custom {effects.push(json!({"id":effect["id"].as_str().unwrap_or("").strip_prefix("minecraft:").unwrap_or(effect["id"].as_str().unwrap_or("")),"amplifier":effect["amplifier"].as_i64().unwrap_or(0),"duration":effect["duration"].as_i64().unwrap_or(0)}));}}
        let tool=get("minecraft:tool");
        let mining=tool.map_or(0.,|tool|tool["rules"].as_array().and_then(|rules|rules.iter().find(|rule|rule["correct_for_drops"]==true)).and_then(|rule|rule["speed"].as_f64()).unwrap_or_else(||tool["default_mining_speed"].as_f64().unwrap_or(1.)));
        let block=if is("BlockItem") {items.get(id).and_then(|i|i.block).map(|state| {
            let facts=self.res.facts().get(state);let movement=self.motion.blocks.get(state).expect("bound movement registry").coefficients;
            let forbidden=|name:&str,defaults:&[&str]|policy[name].as_array().map_or_else(||defaults.contains(&&*facts.name),|list|list.iter().any(|v|v.as_str()==Some(&*facts.name)));
            let mut level=horizon_client::collisions::SceneCollisions{world:&self.world,binding:self.collision,subject:self.player.as_ref().map(|p|p.powder_snow_subject())};
            let player=self.player.as_ref().expect("local player checked");
            let context=horizon_world::collision_context::CollisionContext::for_entity(horizon_world::collision_context::CollisionEntity {identity:horizon_world::entity_obstruction::EntityIdentity(0),kind:horizon_world::collision_context::EntityContextKind::Player,y:player.position.y,descending:player.shift_key_down,main_hand:Some(horizon_world::collision_context::CollisionItem::EMPTY)},false);
            let shape=self.collision.collision_shape_with_context(&mut level,state,horizon_core::position::BlockPos::new(0,0,0),&context).ok();
            let supporting=shape.as_ref().is_some_and(|shape|horizon_world::block_support::SupportType::Full.is_supporting(shape,horizon_core::geometry::Direction::Up).unwrap_or(false));
            let class=|name:&str|facts.classes.iter().any(|c|c.rsplit('.').next()==Some(name));
            let valid=supporting&&!class("FallingBlock")&&!forbidden("disallowed",&["minecraft:tnt","minecraft:cobweb","minecraft:nether_portal"]);
            let favourable=movement.friction<=0.6&&movement.speed_factor>=1.&&movement.jump_factor>=1.&&!class("BaseEntityBlock")&&facts.full_collision&&!forbidden("unfavourable",&["minecraft:crafting_table","minecraft:jigsaw","minecraft:smithing_table","minecraft:fletching_table","minecraft:enchanting_table","minecraft:cauldron","minecraft:magma_block"]);
            let solid=self.res.behaviours().facts(state).flags&horizon_world::level::data::state_flags::CONDUCTOR!=0;
            json!({"valid":valid,"favourable":favourable,"full":facts.full_collision,"solid":solid,"friction":movement.friction,"speed":movement.speed_factor,"jump":movement.jump_factor,"hardness":facts.hardness})
        })}else{None};
        let maximum=get("minecraft:max_stack_size").and_then(Value::as_i64).unwrap_or(1);
        let default_stack=horizon_client::items::ItemStack::new(id,1);
        let default_view=horizon_client::item_render::stack_view(&default_stack);
        let default_maximum=self.pack.item_renderer.models.components_view(&default_view).get("minecraft:max_stack_size").and_then(Value::as_i64).unwrap_or(1);
        let food=get("minecraft:food").map(|v|json!([v["nutrition"].as_i64().unwrap_or(0),v["saturation"].as_f64().unwrap_or(0.)]));
        let mut identity_stack=stack.clone();identity_stack.count=1;
        let patch=horizon_items::item_data::canon_stack(Some(&identity_stack));
        let enchantment_count=get("minecraft:enchantments").and_then(Value::as_object).map_or(0,|v|v.len());
        let has_effect=|effect:&str|get("minecraft:enchantments").and_then(Value::as_object).is_some_and(|levels|levels.keys().any(|id|registry.get(id).is_some_and(|e|e.effects.get(effect).is_some())));
        let curse=u8::from(has_effect("minecraft:prevent_armor_change"))|(u8::from(has_effect("minecraft:prevent_equipment_drop"))<<1);
        let bucket=if id=="minecraft:lava_bucket" {1}else if matches!(id,"minecraft:water_bucket"|"minecraft:cod_bucket"|"minecraft:salmon_bucket"|"minecraft:pufferfish_bucket"|"minecraft:tropical_fish_bucket"|"minecraft:axolotl_bucket"|"minecraft:tadpole_bucket") {0}else{3};
        // Keep the useful weapon comparator's real native ItemStack facts.
        weapon["slot"]=json!(slot);
        json!({"item":id,"components":json!([id,components]).to_string(),"patch":patch,"count":stack.count,"maximum":maximum,"defaultMaximum":default_maximum,
            "stackable":maximum>1&&(items.stack_max_damage(stack)==0||stack.damage==0||get("minecraft:unbreakable").is_some()),"weapon":weapon,"classes":classes,"enchants":enchantments,
            "enchantmentCount":enchantment_count,"armor":armor,"defense":attribute("minecraft:armor"),"toughness":attribute("minecraft:armor_toughness"),"curses":curse,
            "food":food,"block":block,"mining":mining,"named":get("minecraft:custom_name").is_some(),"effects":effects,"bucket":bucket})
    }
    pub(super) fn native_client_inventory_cleaner_context(&mut self, policy:&str)->Option<String> {
        use serde_json::{Value,json};
        if !self.client_auto_tool_player(){return None;}
        let p=self.player.as_ref()?;let bases=[p.attributes.base_value_of("minecraft:attack_damage").ok()?,p.attributes.base_value_of("minecraft:attack_speed").ok()?];
        let policy=serde_json::from_str::<Value>(policy).unwrap_or(Value::Null);
        let mut output=vec![];
        let slots=[0,1,2,3,4,5,6,7,8,40,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39];
        for slot in slots {let stack=match slot {0..=35=>self.inventory.get(slot).and_then(Option::as_ref),36..=39=>self.armor.get(39-slot).and_then(Option::as_ref),_=>self.offhand.as_ref()};
            if let Some(stack)=stack.filter(|s|s.count>0&&&*s.item!="minecraft:air"){output.push(json!({"player":slot,"facts":self.client_inventory_cleaner_stack(slot as i32,stack,bases,&policy)}));}}
        if !self.client_inventory_player_menu() {let mapping=self.client_inventory_mapping(false);for (slot,internal) in mapping.into_iter().enumerate() {if internal.is_some_and(|i|i<36||(500..=504).contains(&i)){continue;}if let Some(stack)=internal.and_then(|i|self.slot_view(i)).filter(|s|s.count>0&&&*s.item!="minecraft:air") {output.push(json!({"container":slot,"facts":self.client_inventory_cleaner_stack(-2,&stack,bases,&policy)}));}}}
        Some(json!({"items":output,"legacy":self.legacy_rules(),"stick":self.client_weapon_stack(-1,&horizon_client::items::ItemStack::new("minecraft:stick",1),bases),"offhandSwap":true}).to_string())
    }
