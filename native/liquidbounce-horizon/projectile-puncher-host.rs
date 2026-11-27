    pub(super) fn native_client_projectile_puncher_plan(&self,range:f32,projectiles:u8,only_air:bool,fov:f32,yaw:f32,pitch:f32)->Option<serde_json::Value>{
        if !self.native_client_slot_owner(){return None;}self.client_projectile_puncher_plan(range,projectiles,only_air,fov,yaw,pitch)
    }
    pub(super) fn native_client_projectile_puncher_ready(&self,target:&str,range:f32,yaw:f32,pitch:f32)->bool{
        self.native_client_slot_owner()&&self.client_projectile_puncher_ready(target,range,yaw,pitch)
    }
    pub(super) fn native_client_projectile_puncher_attack(&mut self,target:u32,swing:u8){
        if self.native_client_slot_owner(){self.client_projectile_puncher_attack(target,swing);}
    }
