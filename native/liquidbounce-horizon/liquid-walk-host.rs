// SPDX-License-Identifier: GPL-3.0-or-later
    pub(super) fn native_client_liquid_walk_context(&self)->Option<[bool;5]>{
        if !self.native_client_slot_owner(){return None;}
        horizon_world::liquid_walk::physical_shift(self.keys.held(horizon_client::actions::Action::Sneak));
        let p=self.player.as_ref()?;
        horizon_client::liquid_walk_native::shape_player(&self.world,p,self.collision);
        let c=horizon_client::liquid_walk_native::context(&self.world,p,self.collision);
        Some([c[0],c[1],c[2],c[3],self.keys.held(horizon_client::actions::Action::Sneak)])
    }
    pub(super) fn liquid_walk_input(&self,sneak:bool,jump:&mut bool){
        if !self.native_client_slot_owner(){return;}
        horizon_world::liquid_walk::physical_shift(self.keys.held(horizon_client::actions::Action::Sneak));
        let Some(p)=self.player.as_ref()else{return;};
        let inside=horizon_client::liquid_walk_native::row(&self.world,self.collision,p.bounds(),0.);
        *jump|=horizon_world::liquid_walk::input_jump(sneak,inside);
    }
