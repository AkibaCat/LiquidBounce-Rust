    /// BoatItem.use calls getEntities(player, scan, EntitySelector.CAN_BE_PICKED)
    /// and then tests the eye against each inflated physical bounding box.
    /// CAN_BE_PICKED is precisely Entity::isPickable, not isAlive/collidable.
    fn client_boat_eye_inside_pickable(&self) -> Option<bool> {
        use super::entity::Entity;
        use horizon_core::geometry::{Aabb,Vec3};
        let player=self.player.as_ref()?;
        let [x,y,z]=self.eye_position()?;
        let eye=Vec3::new(x,y,z);
        let direction=horizon_world::view_direction::view_vector(self.pitch,self.yaw);
        let scan=player.bounds().expand_towards(direction.scale(5.0)).inflate(Vec3::new(1.,1.,1.));
        let contains=|bounds:Aabb,radius:f64| {
            bounds.intersects(scan)&&bounds.inflate(Vec3::new(radius,radius,radius)).contains(eye)
        };
        // Base Entity.getPickRadius fires the original margin event. The
        // Projectile override never invokes that base method.
        let base=|id:u32,bounds:Aabb|contains(bounds,self.client_pick_margin(id as i32,0.));
        let extents=|(lo,hi):([f64;3],[f64;3])|Aabb::new(Vec3::new(lo[0],lo[1],lo[2]),Vec3::new(hi[0],hi[1],hi[2]));
        // LivingEntity only excludes removed entities; health zero is allowed.
        // Player adds its spectator test; other living types do not.
        let local_id=self.remote.as_ref().map(|r|r.entity_id as u32);
        for m in &self.mobs {
            if Some(m.id)!=local_id&&!m.removed&&(&*m.kind.id!="minecraft:player"||!m.extra.spectator)&&base(m.id,m.bounds()){return Some(true);}
        }
        for guest in self.lan.iter().flat_map(|h|h.hub.guests.iter()).filter(|g|g.placed&&g.game_mode!=3) {
            if base(guest.entity_id as u32,guest.bounds()){return Some(true);}
        }
        for p in &self.armor_stands {if !p.marker&&base(p.id,p.bounds()){return Some(true);}}
        for p in &self.paintings {if base(p.id,extents(p.bounds())){return Some(true);}}
        for p in &self.item_frames {if base(p.id,extents(p.bounds())){return Some(true);}}
        for p in &self.cushions {if base(p.id,extents(p.bounds())){return Some(true);}}
        for p in &self.boats {if base(p.id,p.bounds()){return Some(true);}}
        for p in &self.carts {if base(p.id,p.bounds()){return Some(true);}}
        for p in &self.tnts {if base(p.id,p.bounds()){return Some(true);}}
        for p in &self.falling {if base(p.id,p.bounds()){return Some(true);}}
        for (&id,p) in &self.interactions {if base(id as u32,p.bounds()){return Some(true);}}
        for p in &self.crystals {if base(p.id,p.bounds()){return Some(true);}}
        if let Some(d)=&self.dragon {
            for i in 0..8 {if base(d.id+1+i as u32,d.part_box(i)){return Some(true);}}
        }
        let knots=self.natural_knot_ids().into_iter().filter_map(|id|self.natural_knot(id).map(|block|(id,block)))
            .chain(self.remote_knots.iter().map(|(&id,&block)|(id,block)));
        for (id,block) in knots {
            let knot=horizon_client::mobs::leash::knot_state(block);
            let (p,half)=(knot.position,knot.width*0.5);
            if base(id,Aabb::new(Vec3::new(p.x-half,p.y,p.z-half),Vec3::new(p.x+half,p.y+knot.height,p.z+half))){return Some(true);}
        }
        // Projectile.isPickable reads the live entity-type tag. AbstractArrow
        // also rejects inGround; ShulkerBullet unconditionally returns true.
        let redirectable=|id:&str|self.client_item_entity_tag_contains("minecraft:redirectable_projectile",id);
        for p in &self.creatures.projectiles {
            let pickable=matches!(p.kind,horizon_client::projectiles::ProjectileKind::ShulkerBullet{..})
                ||(redirectable(p.entity_type())&&!p.arrow.as_ref().is_some_and(|a|a.in_ground));
            if pickable&&contains(p.bounds(),1.){return Some(true);}
        }
        for p in &self.rockets {if redirectable("minecraft:firework_rocket")&&contains(p.bounds(),1.){return Some(true);}}
        for p in &self.tridents {if redirectable("minecraft:trident")&&!p.in_ground&&contains(p.bounds(),1.){return Some(true);}}
        Some(false)
    }
    /// The original client copy's owner is null: AddEntity initializes only
    /// base Entity state; cloud metadata contains radius/waiting/particle.
    /// See audit-smart-eat-cloud-owner.py and its actual-JAR bytecode proof.
    /// Never substitute Horizon's authoritative server Breath.from_dragon.
    fn client_has_owned_dragon_cloud(&self)->bool {false}
