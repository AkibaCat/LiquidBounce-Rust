    /// Same running query exposed by LocalPlayerBody.recastsElytra.
    pub(super) fn native_client_elytra_recast_running(&self, running: &mut bool) {
        mod_api::dispatch_client_elytra_recast_running(self, mod_api::mods(), running);
    }
    pub(super) fn native_client_recasts_elytra(&self) -> bool {
        let mut running = false;
        self.native_client_elytra_recast_running(&mut running);
        running
    }
