    pub(super) fn native_client_interaction_range(&self, attribute: &str, original: f64, range: &mut f64) {
        if !self.client_reach_player() { return; }
        mod_api::dispatch_client_interaction_range(self, mod_api::mods(), attribute, original, range);
    }
    pub(super) fn native_client_attack_range(&self, min_reach: &mut f32, max_reach: &mut f32, min_creative_reach: &mut f32, max_creative_reach: &mut f32) {
        if !self.client_reach_player() { return; }
        mod_api::dispatch_client_attack_range(self, mod_api::mods(), min_reach, max_reach, min_creative_reach, max_creative_reach);
    }
    pub(super) fn native_client_reach_policy(&self, through_walls: &mut f32) {
        if !self.client_reach_player() || self.camera_entity().is_some() { return; }
        mod_api::dispatch_client_reach_policy(self, mod_api::mods(), through_walls);
    }
    pub(super) fn native_client_rotation_aim(&self, yaw: &mut f32, pitch: &mut f32, managed: &mut bool) {
        if !self.client_reach_player() || self.camera_entity().is_some() { return; }
        mod_api::dispatch_client_rotation_aim(self, mod_api::mods(), yaw, pitch, managed);
    }
    fn client_reach_player(&self) -> bool {
        self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_reach_queries_allowed())
    }
    pub(super) fn client_attack_range_contains(&self, range: &serde_json::Value, distance: f64) -> bool {
        let value = |key: &str, default: f32| range[key].as_f64().map_or(default, |v| v as f32);
        let mut min = value("min_reach", 0.0);
        let mut max = value("max_reach", 3.0);
        let mut creative_min = value("min_creative_reach", 0.0);
        let mut creative_max = value("max_creative_reach", 5.0);
        self.native_client_attack_range(&mut min, &mut max, &mut creative_min, &mut creative_max);
        let margin = value("hitbox_margin", 0.3);
        let (min, max) = if self.creative { (creative_min, creative_max) } else { (min, max) };
        distance >= f64::from(min - margin) && distance <= f64::from(max + margin)
    }
    fn client_reach_root_vehicle(&self, entity: u32) -> u32 {
        let mut root = entity;
        let mut visited = Vec::new();
        loop {
            if visited.contains(&root) { return root; }
            visited.push(root);
            let parent = self.remote.as_ref().and_then(|r| r.passenger_vehicles.get(&root).copied())
                .or_else(|| self.mobs.iter().find(|m| m.id == root).and_then(|m| m.extra.mount))
                .or_else(|| self.armor_stands.iter().find(|a| a.id == root).and_then(|a| a.vehicle))
                .or_else(|| self.interactions.get(&(root as i32)).and_then(|a| a.vehicle));
            let Some(parent) = parent else { return root; };
            root = parent;
        }
    }
    pub(super) fn client_block_aim(&self) -> Option<([f64; 3], [f64; 3])> {
        if self.camera_entity().is_some() || !self.client_reach_player() { return self.aim(); }
        let (mut yaw, mut pitch, mut managed) = (self.yaw, self.pitch, false);
        self.native_client_rotation_aim(&mut yaw, &mut pitch, &mut managed);
        Some((self.eye_position()?, super::client_rotation_direction(yaw, pitch)))
    }
    /// The replacement EntityHitResult at the Entity.pick expression in
    /// LocalPlayer.pick. A non-miss active attack-range ray takes precedence.
    pub(super) fn client_reach_target(&self) -> Option<super::ClientReachPick> {
        use horizon_core::geometry::{Aabb, Vec3};
        let mut range = 0.0;
        self.native_client_reach_policy(&mut range);
        if range <= 0.0 { return None; }
        if self.active_attack_range().is_some_and(|r| self.attack_range_pick(&r).is_some()) { return None; }
        let player = self.player.as_ref()?;
        let (eye, direction) = self.client_block_aim()?;
        let root = self.riding.map(|r| self.client_reach_root_vehicle(r));
        let mut candidates = Vec::new();
        let mut offer = |kind: u8, index: usize, id: u32, local_id: u32, position: [f64; 3], bounds: Aabb, pick_radius: f64| {
            candidates.push(super::ClientReachCandidate {
                kind, index, id: id as i32, local_id, position, bounds, pick_radius,
                same_root_vehicle: root.is_some_and(|r| r == self.client_reach_root_vehicle(id)),
                can_pick_inside: true,
            });
        };
        for (i, m) in self.mobs.iter().enumerate().filter(|(_, m)| !m.removed && !m.extra.spectator) {
            offer(0, i, m.id, m.id, [m.position.x, m.position.y, m.position.z], m.bounds(), 0.0);
        }
        let box_of = |(lo, hi): ([f64; 3], [f64; 3])| Aabb::new(Vec3::new(lo[0],lo[1],lo[2]),Vec3::new(hi[0],hi[1],hi[2]));
        for (i, p) in self.paintings.iter().enumerate() {
            let (lo, hi) = p.bounds();
            offer(1, i, p.id, p.id, std::array::from_fn(|a| (lo[a]+hi[a])*0.5), box_of((lo,hi)), 0.0);
        }
        for (i, p) in self.item_frames.iter().enumerate() {
            let (lo, hi) = p.bounds();
            offer(2, i, p.id, p.id, std::array::from_fn(|a| (lo[a]+hi[a])*0.5), box_of((lo,hi)), 0.0);
        }
        for (i, p) in self.armor_stands.iter().enumerate().filter(|(_,p)| !p.marker) {
            offer(3, i, p.id, p.id, [p.position.x,p.position.y,p.position.z], p.bounds(), 0.0);
        }
        for (i, p) in self.cushions.iter().enumerate() {
            offer(4, i, p.id, p.id, [p.position.x,p.position.y,p.position.z], box_of(p.bounds()), 0.0);
        }
        for (i, p) in self.boats.iter().enumerate() {
            offer(5, i, p.id, p.id, [p.position.x,p.position.y,p.position.z], p.bounds(), 0.0);
        }
        for (i, p) in self.carts.iter().enumerate() {
            offer(6, i, p.id, p.id, [p.position.x,p.position.y,p.position.z], p.bounds(), 0.0);
        }
        for (i, (&id, p)) in self.interactions.iter().enumerate() {
            offer(7, i, id as u32, id as u32, [p.position.x,p.position.y,p.position.z], p.bounds(), 0.0);
        }
        for (i, p) in self.creatures.projectiles.iter().enumerate().filter(|(_,p)| super::deflect::attackable(p)) {
            let radius = if matches!(p.kind,horizon_client::projectiles::ProjectileKind::ShulkerBullet { .. }) { 0.0 } else { 1.0 };
            offer(8, i, p.id, p.id, [p.position.x,p.position.y,p.position.z], p.bounds(), radius);
        }
        for (i, (&id, &block)) in self.remote_knots.iter().enumerate() {
            let knot = horizon_client::mobs::leash::knot_state(block);
            let p = knot.position;
            let half = knot.width * 0.5;
            offer(9, i, id, id, [p.x,p.y,p.z], Aabb::new(Vec3::new(p.x-half,p.y,p.z-half),Vec3::new(p.x+half,p.y+knot.height,p.z+half)), 0.0);
        }
        for (i, p) in self.crystals.iter().enumerate() {
            let pos = p.position;
            offer(10, i, p.id, super::dragon::CRYSTAL_IDS+i as u32, [pos.x,pos.y,pos.z],
                Aabb::new(Vec3::new(pos.x-1.0,pos.y,pos.z-1.0),Vec3::new(pos.x+1.0,pos.y+2.0,pos.z+1.0)), 0.0);
        }
        if let Some(d) = self.dragon.as_ref().filter(|d| d.alive()) {
            for i in 0..8 {
                let p = d.parts[i].0;
                offer(11, i, d.id+1+i as u32, super::dragon::SPECIAL+i as u32, [p.x,p.y,p.z], d.part_box(i), 0.0);
            }
        }
        super::client_reach_pick(candidates, Vec3::new(eye[0],eye[1],eye[2]), Vec3::new(direction[0],direction[1],direction[2]), player.bounds(), f64::from(range))
    }

