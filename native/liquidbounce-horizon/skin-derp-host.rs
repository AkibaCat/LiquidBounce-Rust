    /// Actual stored model-parts option, including when no world is active.
    pub(super) fn native_client_skin_derp_parts(&self) -> i32 {
        i32::from(self.settings.file.model_customisation())
    }
    /// Options.setModelPart mutates its set only. Normal client-info change
    /// detection broadcasts the resulting real options at its ordinary tick.
    pub(super) fn native_client_skin_derp_set_parts(&mut self, parts: i32) -> bool {
        if !self.native_window() || !(0..=127).contains(&parts) { return false; }
        const PARTS: [&str; 7] = ["cape", "jacket", "left_sleeve", "right_sleeve", "left_pants_leg", "right_pants_leg", "hat"];
        for (bit, part) in PARTS.into_iter().enumerate() {
            self.settings.file.set(&format!("modelPart_{part}"), horizon_client::options_file::V::Bool(parts & (1 << bit) != 0));
        }
        true
    }
