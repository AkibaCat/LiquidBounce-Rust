// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
impl Game<'_> {
    pub(super) fn client_item_chams_frame(&self) {
        let mut output=String::new();
        self.native_client_item_chams(&mut output);
        horizon_client::gfx::item_chams::begin_frame(&output);
    }
    pub(super) fn client_item_chams_reset(&self) {
        horizon_client::gfx::item_chams::begin_frame("null");
    }
}
