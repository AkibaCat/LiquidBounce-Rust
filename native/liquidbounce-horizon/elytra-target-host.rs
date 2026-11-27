    pub(super) fn native_client_elytra_target_query(&mut self,value:&str)->String{
        if !self.native_window()||!self.native_client_slot_owner(){return "null".into();}
        self.elytra_target_query(value)
    }
