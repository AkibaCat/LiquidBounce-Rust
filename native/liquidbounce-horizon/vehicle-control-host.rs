    /// Real local controller and physical key snapshot, including the stored rehook target.
    pub(super) fn native_client_vehicle_control_state(&mut self,target:i32)->Option<serde_json::Value>{self.vehicle_control_state(target)}
    pub(super) fn native_client_vehicle_control_apply(&mut self,request:&str)->bool{self.vehicle_control_apply(request)}
