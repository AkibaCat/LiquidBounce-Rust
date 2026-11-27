pub(super) fn native_client_anti_staff_chat(&mut self, message: &str, key: &str) {
    let ticks = self.visual_ticks();
    let focused = self.screen == Screen::Chat;
    self.ui.chat.add_client_id(
        &self.gui.atlas,
        key,
        Text::styled(message, horizon_client::text::Style::color(0xffb4b4)),
        ticks,
        focused,
    );
}
