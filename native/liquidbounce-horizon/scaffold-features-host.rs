    pub(super) fn native_client_scaffold_features(&mut self,request:&str)->String {
        if !self.native_window() || !self.native_client_slot_owner() { return "null".into(); }
        self.scaffold_features_query(request).unwrap_or_else(||"null".into())
    }
