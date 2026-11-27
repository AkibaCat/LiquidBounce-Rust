pub(super) fn native_client_aimbot_query(&mut self, text: &str) -> String {
    if !self.native_window() || !self.native_client_slot_owner() {
        return "null".into();
    }
    self.aimbot_query(text)
}
pub(super) fn native_client_aimbot_render(
    &self,
    partial: f32,
    screen: bool,
    container: bool,
    yaw: &mut f32,
    pitch: &mut f32,
    apply: &mut bool,
) {
    if self.native_window() {
        mod_api::dispatch_client_aimbot_render(
            self,
            mod_api::mods(),
            partial,
            screen,
            container,
            yaw,
            pitch,
            apply,
        );
    }
}
