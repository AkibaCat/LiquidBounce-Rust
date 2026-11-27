// SPDX-License-Identifier: GPL-3.0-or-later
    pub(super) fn native_client_freeze_frame(&mut self){
        if self.native_client_slot_owner(){mod_api::dispatch_client_freeze_frame(self,mod_api::mods());}
    }
    pub(super) fn native_client_freeze_flag(&mut self){
        if self.native_client_slot_owner(){mod_api::dispatch_client_freeze_flag(self,mod_api::mods());}
    }
    pub(super) fn native_client_freeze_context(&self)->Option<[f64;9]>{
        if !self.native_client_slot_owner(){return None;}
        let p=self.player.as_ref()?;
        let hurt=self.remote.as_ref().and_then(|r|r.builtin.get("minecraft:sound_event")).and_then(|r|r.id("minecraft:entity.player.hurt")).unwrap_or(-1);
        Some([f64::from(u8::from(p.ground.on_ground)),f64::from(u8::from(p.move_vector!=[0.,0.])),
            f64::from(u8::from(p.fluids.was_touching_water||p.is_in_lava())),f64::from(u8::from(p.position.y<=self.world.shape.min_y as f64)),
            f64::from(u8::from(self.using.is_some())),f64::from(self.yaw),f64::from(self.pitch),f64::from(u8::from(p.collision.horizontal)),f64::from(hurt)])
    }
