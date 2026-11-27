    pub(super) fn native_client_multi_actions(&self, place_while_breaking: &mut bool,
        attack_while_using: &mut bool, break_while_using: &mut bool) {
        if self.window.is_some() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_reach_queries_allowed()) {
            mod_api::dispatch_client_multi_actions(self, mod_api::mods(), place_while_breaking,
                attack_while_using, break_while_using);
        }
    }
    pub(super) fn client_multi_actions(&self) -> (bool, bool, bool) {
        let (mut place, mut attack, mut breaking) = (false, false, false);
        self.native_client_multi_actions(&mut place, &mut attack, &mut breaking);
        (place, attack, breaking)
    }

