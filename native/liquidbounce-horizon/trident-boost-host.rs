    /// TridentItem.use/releaseUsing's evaluated isInWaterOrRain expression.
    /// Querying the real weather first preserves vanilla's evaluation effects.
    pub(super) fn native_trident_water(&mut self) -> bool {
        let original = self.in_water_or_rain();
        if !self.native_client_slot_owner() { return original; }
        let mut allowed = original;
        self.native_client_trident_water(original, &mut allowed);
        allowed
    }
    /// Only the Player.push arguments change. The grounded lift, spin attack,
    /// sound, damage and pre-existing velocity retain the ordinary item path.
    pub(super) fn native_trident_impulse(&self, original: [f64;3]) -> [f64;3] {
        if !self.native_client_slot_owner() { return original; }
        let mut impulse = original;
        self.native_client_trident_impulse(original, &mut impulse);
        impulse
    }
    pub(super) fn native_client_trident_water(&self, original: bool, allowed: &mut bool) {
        mod_api::dispatch_client_trident_water(self, mod_api::mods(), original, allowed);
    }
    pub(super) fn native_client_trident_impulse(&self, original: [f64;3], impulse: &mut [f64;3]) {
        mod_api::dispatch_client_trident_impulse(self, mod_api::mods(), original, impulse);
    }
