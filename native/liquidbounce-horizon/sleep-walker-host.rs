    /// ModuleSleepWalker.GameTick: setPose(STANDING), clearSleepingPos, setScreen(null).
    /// This changes the local synchronized sleeping position, without a wake packet.
    pub(super) fn native_client_sleep_walker_tick(&mut self) -> bool {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || self.sleeping.is_none()
            || !self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed()) {
            return false;
        }
        let player = self.player.as_mut().unwrap();
        if player.set_pose(horizon_client::pose::Pose::Standing).is_err() {
            return false;
        }
        // LivingEntity.isSleeping reads SLEEPING_POS, independently of DATA_POSE.
        self.sleeping = None;
        // Horizon caches isSleeping for Player.getDesiredPose and local travel.
        player.sleeping = false;
        // Gui.setScreen(null) calls removed(), not InBedChatScreen.onClose().
        // The existing interrupted-chat path preserves drafts without waking.
        self.pad_clear_focus();
        self.native_mod_screen = None;
        self.screen = Screen::Playing;
        self.tick_in_bed_chat();
        self.capture(true);
        true
    }
