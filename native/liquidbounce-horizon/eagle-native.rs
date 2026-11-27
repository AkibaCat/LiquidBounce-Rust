    fn eagle(&mut self, config: &Value, mut keys: Input) -> Input {
        use horizon_client::eagle_model::{Input as EagleInput,Settings};
        let s=&config["eagle"];
        if s["running"]!=true {
            self.eagle.disabled();
            return keys;
        }
        self.eagle.configure(Settings {
            distance:[float(&s["distance"][0],0.4),float(&s["distance"][1],0.6)],
            conditional:s["conditional"]!=false,
            conditions:s["conditions"].as_u64().unwrap_or(1<<5) as u32,
            pitch:[float(&s["pitch"][0],-90.),float(&s["pitch"][1],90.)],
        });
        let input=world::write(|g| {
            let valid=|stack:Option<&horizon_items::items::ItemStack>| {
                let Some(stack)=stack.filter(|s|s.count>0) else{return false;};
                let Some(state)=g.res.items().get(&stack.item).and_then(|i|i.block)else{return false;};
                let facts=g.res.facts().get(state);
                if matches!(&*facts.name,"minecraft:tnt"|"minecraft:cobweb"|"minecraft:nether_portal")
                    ||facts.classes.iter().any(|c|c.rsplit('.').next()==Some("FallingBlock")){return false;}
                let Some(p)=g.player.as_ref() else{return false;};
                let context=CollisionContext::for_entity(CollisionEntity{identity:horizon_world::entity_obstruction::EntityIdentity(0),kind:horizon_world::collision_context::EntityContextKind::Player,y:p.position.y,descending:p.shift_key_down,main_hand:Some(CollisionItem::EMPTY)},false);
                let mut level=horizon_client::collisions::SceneCollisions{world:&g.world,binding:g.collision,subject:Some(p.powder_snow_subject())};
                g.collision.collision_shape_with_context(&mut level,state,horizon_core::position::BlockPos::new(0,0,0),&context)
                    .ok().is_some_and(|shape|horizon_world::block_support::SupportType::Full.is_supporting(&shape,horizon_core::geometry::Direction::Up).unwrap_or(false))
            };
            EagleInput {
                left:keys.left,right:keys.right,forwards:keys.forward,backwards:keys.backward,
                holding_blocks:valid(g.held())||valid(g.offhand.as_ref()),
                on_ground:g.player.as_ref().is_some_and(|p|p.ground.on_ground),
                sneak:keys.shift,physical_sneak:g.keys.held(horizon_client::actions::Action::Sneak),
                flying:g.player.as_ref().is_some_and(|p|p.flying),pitch:g.pitch,
            }
        });
        let close=!input.flying&&self.eagle.settings.conditions_met(input)
            &&self.close_to_edge(keys,f64::from(self.eagle.current_distance),float(&config["movementYaw"],world::read(|g|g.yaw)));
        keys.shift=self.eagle.movement_input(input,close);
        keys
    }
