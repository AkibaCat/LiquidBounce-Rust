// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientItemChamsEvent;
impl LiquidBounceHooks {
    pub(crate) fn item_chams(&self, event: &mut ClientItemChamsEvent<'_>) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if !modules.query_running(&state.registry, "ItemChams") {
            *event.output = "null".into();
            return;
        }
        // Util.getMillis is a monotonic clock. Keep source f32 seconds/phase precision.
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let time = START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_millis() as f32
            / 1000.;
        *event.output = modules.item_chams.render_settings(time).to_string();
    }
}
