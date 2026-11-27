// SPDX-License-Identifier: GPL-3.0-or-later
// Exact-source extension appended to Horizon's title module.
impl Title {
    pub fn liquidbounce_menu_screen(&mut self, name: &str) -> Result<(), String> {
        self.focused_button = None;
        self.search.clear();
        self.search_focused = false;
        self.scroll.set(crate::ui::Scroll::default());
        match name {
            "title" => self.mode = Mode::Main,
            "singleplayer" => {
                self.create_return_to_main = false;
                self.mode = Mode::Select {
                    worlds: list_worlds(&self.saves),
                    selected: None,
                };
            }
            "multiplayer" => {
                if !self.allows_multiplayer {
                    return Err("Multiplayer is disabled for this account".into());
                }
                self.mode = if self.skip_multiplayer_warning {
                    Mode::Multiplayer(crate::multiplayer::Screen::list(&self.saves))
                } else {
                    Mode::MultiplayerWarning { check: false }
                };
            }
            "create_world" => {
                self.create_return_to_main = false;
                self.preparing = true;
            }
            "multiplayer_realms" => {
                if !self.allows_multiplayer {
                    return Err("Multiplayer is disabled for this account".into());
                }
                self.mode =
                    Mode::Realms(crate::realms::Screen::from_title(Some(self.saves.clone())));
            }
            _ => return Err(format!("Unsupported native menu screen: {name}")),
        }
        Ok(())
    }

    /// Reuses the same world lock, recovery, compatibility and upgrade prompts
    /// as selecting the native world's Play/Edit/Delete buttons.
    pub fn liquidbounce_world_action(
        &mut self,
        folder: &str,
        action: &str,
    ) -> Result<Action, String> {
        let world = list_worlds(&self.saves)
            .into_iter()
            .find(|world| world.folder == folder)
            .ok_or_else(|| "Saved world was not found".to_owned())?;
        match action {
            "world_join" if world.can_play() => Ok(self.play(world)),
            "world_edit" if world.can_edit() => {
                self.mode = Mode::Edit {
                    name: world.name.clone(),
                    world,
                    focused: true,
                };
                Ok(Action::None)
            }
            "world_delete" if !world.locked => {
                self.mode = Mode::Delete { world };
                Ok(Action::None)
            }
            _ => Err("The saved world is locked or is not compatible with this action".into()),
        }
    }
}
