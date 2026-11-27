    pub(super) fn native_client_better_tab(&self,players:&str,policy:&mut String){if self.native_window(){mod_api::dispatch_client_better_tab(self,mod_api::mods(),players,policy);}}
    pub(super) fn client_better_tab_scope(&self,players:&[horizon_client::tab_list::TabPlayer])->horizon_client::better_tab::Scope {
        let rows=players.iter().map(|p|serde_json::json!({"id":p.uuid.to_string(),"name":p.name,"display":p.display_name.as_ref().map(|d|horizon_client::scoreboard::comp_text(d).plain())})).collect::<Vec<_>>();
        let input=serde_json::json!({"own":self.own_profile_name(),"players":rows}).to_string();let mut policy=String::new();self.native_client_better_tab(&input,&mut policy);horizon_client::better_tab::scope(&policy)
    }
