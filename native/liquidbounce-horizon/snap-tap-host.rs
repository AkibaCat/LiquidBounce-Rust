    /// Matches the physical keyboard event against current movement bindings.
    /// The mask preserves all conflicts; ModuleSnapTap's when uses left first.
    pub(super) fn native_client_snap_tap_key_mask(&self, key: KeyCode) -> u8 {
        if !self.native_window() {
            return 0;
        }
        let Some(name) = keybinds::key_name(key) else {
            return 0;
        };
        [
            ("key.left", 1),
            ("key.right", 2),
            ("key.back", 4),
            ("key.forward", 8),
        ]
        .into_iter()
        .fold(0, |mask, (mapping, bit)| {
            mask | if self.settings.keys.get(mapping) == name {
                bit
            } else {
                0
            }
        })
    }
