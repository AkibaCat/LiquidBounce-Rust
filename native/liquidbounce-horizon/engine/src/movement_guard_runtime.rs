// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::rotations::movement_yaw;
impl LiquidBounceHooks {
    pub(crate) fn movement_guard_prepare(&self, host: &mut dyn Host) {
        let [yaw, _] = host.client_player_rotation();
        let config = {
            let state = self.state();
            let m = state.behaviors.borrow();
            let mut r = m.rotations.lock().expect("RotationManager");
            let correction = r
                .active_target(|p| m.rotation_provider_running(&state.registry, p))
                .map(|t| t.movement_correction);
            let a = m.anchor;
            let h = m.avoid_hazards;
            serde_json::json!({"movementYaw":movement_yaw(yaw,r.current,correction),"anchor":{"running":m.query_running(&state.registry,"Anchor"),"distance":a.max_distance,"horizontal":a.horizontal_speed,"vertical":a.vertical_speed},"hazards":{"running":m.query_running(&state.registry,"AvoidHazards"),"mode":h.mode.tag(),"avoid":h.avoid}}).to_string()
        };
        host.client_movement_guard_prepare(&config);
    }
}
