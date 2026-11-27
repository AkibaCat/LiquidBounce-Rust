    /// The actual stored MainHand option, readable in a menu as at object init.
    pub(super) fn native_client_hand_derp_hand(&self) -> i32 {
        self.native_publish_hand_derp_options();
        i32::from(!self.settings.main_hand_left)
    }
    /// Complete current client information, changing only the advertised arm.
    /// The real mainHand option and sent_info option-broadcast cache stay intact.
    pub(super) fn native_client_hand_derp_send(&mut self, hand: i32) -> bool {
        if !self.native_window() || !(0..=1).contains(&hand) { return false; }
        self.native_publish_hand_derp_options();
        let Some(packet) = horizon_client::hand_derp_bridge::packet(hand) else { return false; };
        self.send(packet);
        self.remote.is_some()
    }
    pub(super) fn native_publish_hand_derp_options(&self) {
        if self.native_window() {
            // Vanilla casts the raw integer view distance to a protocol byte;
            // the separate normal Horizon broadcast may clamp it for its UI.
            horizon_client::hand_derp_bridge::publish(self.settings.client_information(self.settings.render_distance as i8));
        }
    }
