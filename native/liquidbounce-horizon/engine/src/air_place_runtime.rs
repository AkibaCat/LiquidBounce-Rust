// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use serde_json::json;
impl LiquidBounceHooks {
    pub(crate) fn air_place_configuration(&self, configuration: &mut String) {
        let state = self.state();
        let b = state.behaviors.borrow();
        let s = &b.air_place;
        *configuration = json!({"enabled":b.query_running(&state.registry,"AirPlace"),
            "preview":s.preview,"outlineOnly":s.outline_only,"color":s.color as u32,
            "outlineColor":s.outline_color as u32,"placeInLiquid":s.place_in_liquid,
            "customRange":s.custom_range,"range":s.range,
            "liquidPlace":b.query_running(&state.registry,"LiquidPlace")})
        .to_string();
    }
    pub(crate) fn air_place_scroll(&self, host: &dyn Host, vertical: f64) {
        let scan = self.state().behaviors.borrow().air_place.modifier.value;
        let down = host.client_keyboard_scan_down(scan);
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let running = b.query_running(&state.registry, "AirPlace");
        b.air_place.scroll(running, down, vertical);
    }
    pub(crate) fn air_place_cancel_scroll(&self, host: &dyn Host) -> bool {
        let state = self.state();
        let b = state.behaviors.borrow();
        b.air_place.can_scroll(
            b.query_running(&state.registry, "AirPlace"),
            host.client_keyboard_scan_down(b.air_place.modifier.value),
        )
    }
}
