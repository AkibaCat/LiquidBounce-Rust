// SPDX-License-Identifier: GPL-3.0-or-later
pub(super) fn native_client_replenish_screen(&self, screen: Screen) -> Screen {
    if matches!(screen, Screen::Inventory | Screen::Creative) {
        self.native_client_inventory_observation(&serde_json::json!({"kind":"replenishScreen",
                "abstract": matches!(self.screen, Screen::Inventory | Screen::Creative),
                "inventory": self.screen == Screen::Inventory && self.client_inventory_player_menu()}).to_string());
    }
    screen
}
