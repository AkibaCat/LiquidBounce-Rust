// SPDX-License-Identifier: GPL-3.0-or-later
impl Dragon {
    pub(super) fn client_damage_particles_ticks(&self) -> i64 {
        self.ticks as i64
    }
    pub(super) fn client_damage_particles_entity(&self, identity: u64) -> serde_json::Value {
        let p = self.position;
        let v = self.velocity;
        serde_json::json!({"identity":identity,"own":false,"ticked":self.ticks!=0,"dead":self.down(),"health":self.health,"max":MAX_HEALTH,"center":[p.x,p.y+4.,p.z],"movement":[v.x,v.y,v.z]})
    }
    fn client_lb_set_health(&mut self, value: f32) {
        let p = self.position;
        let c = horizon_client::damage_particles_capture::context(
            false,
            self.ticks != 0,
            Vec3::new(p.x, p.y + 4., p.z),
            self.velocity,
        );
        horizon_client::damage_particles_capture::record(
            c,
            self.health.max(0.),
            value.clamp(0., MAX_HEALTH),
            MAX_HEALTH,
        );
        self.health = value;
    }
}
