    pub(super) fn native_client_free_cam(&self,input:&str,output:&mut String) {
        if self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none() {
            mod_api::dispatch_client_free_cam(self,mod_api::mods(),input,output);
        }
    }
    pub(super) fn native_client_free_cam_context(&self)->Option<String>{self.free_cam_context()}
    pub(super) fn native_client_free_cam_trace(&self,position:[f64;3],rotation:[f32;2])->Option<[f64;3]>{self.free_cam_trace(position,rotation)}
    pub(super) fn native_client_free_cam_navigation_config(&mut self,value:&str){self.free_cam_navigation_config(value);}
    pub(super) fn native_client_free_cam_navigation_result(&self)->Option<String>{self.free_cam_navigation_result()}
