#!/usr/bin/env python3
"""UseCooldownEvent after the real rightClickDelay assignment, for both clients."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
HOOK_NAMES = ['client_use_cooldown']
HELPER_NAMES = []
HELPERS = '''    pub(super) fn native_client_use_cooldown(&self, main_kind: u8, off_kind: u8,
        mouse_button: i32, cooldown: &mut i32) {
        if self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed()) {
            mod_api::dispatch_client_use_cooldown(self, mod_api::mods(), main_kind, off_kind,
                mouse_button, cooldown);
        }
    }
    /// Minecraft.startUseItem: the write and event follow the breaking guard,
    /// and precede hands-busy, mod interactions, targeting and item use.
    pub(super) fn native_begin_use_cooldown(&mut self) {
        self.use_delay = 4;
        let classify = |stack: Option<&horizon_client::items::ItemStack>| -> u8 {
            stack.map_or(0, |stack| {
                u8::from(horizon_client::fast_place_items::is_projectile_item(&stack.item))
                    | (u8::from(self.items().get(&stack.item).is_some_and(|item| item.block.is_some())) << 1)
            })
        };
        let main_kind = classify(self.stack_in(self.client_hotbar_slot()));
        let off_kind = classify(self.offhand.as_ref());
        let mouse_button = match self.settings.keys.get("key.use") {
            "key.mouse.left" => 0,
            "key.mouse.right" => 1,
            "key.mouse.middle" => 2,
            key => key.strip_prefix("key.mouse.").and_then(|n| n.parse::<i32>().ok())
                .filter(|n| (4..=32).contains(n)).map_or(-1, |n| n - 1),
        };
        let mut cooldown = self.use_delay;
        self.native_client_use_cooldown(main_kind, off_kind, mouse_button, &mut cooldown);
        self.use_delay = cooldown;
    }

'''


def apply(edit, replace_once):
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda t: replace_once(t,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HELPERS + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    edit('crates/horizon-client/src/fast_place_items.rs', lambda _:
        (HERE.parent / 'liquidbounce-behaviors/src/projectile_items.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs', lambda t: replace_once(t,
        'pub mod first_person;\n', 'pub mod fast_place_items;\npub mod first_person;\n'))

    def game(text):
        text = replace_once(text, '    use_delay: u32,\n', '    use_delay: i32,\n')
        return replace_once(text, '            self.use_with_hand()?;\n            self.use_delay = 4;\n',
            '            self.use_with_hand()?;\n')
    edit('crates/horizon-client/src/game.rs', game)
    def guests(text):
        anchor = '    use_delay: u32,\n'
        if text.count(anchor) != 2:
            raise ValueError('Expected both guest/actor use-delay storage fields')
        return text.replace(anchor,'    use_delay: i32,\n')
    edit('crates/horizon-client/src/game/lan_actions.rs', guests)
    def hand(text):
        text = replace_once(text, '                used = self.use_with_hand();\n                self.use_delay = 4;\n',
            '                used = self.use_with_hand();\n')
        return replace_once(text, '''        if self.clicks.destroying.active || self.hands_busy() {
            return Ok(());
        }
''', '''        if self.clicks.destroying.active {
            return Ok(());
        }
        self.native_begin_use_cooldown();
        if self.hands_busy() {
            return Ok(());
        }
''')
    edit('crates/horizon-client/src/game/hand.rs', hand)
    def remote(text):
        text = replace_once(text,'        self.use_delay = 4;\n','        self.native_begin_use_cooldown();\n')
        return replace_once(text,'    fn use_delay(&self) -> u32 {\n        self.0.use_delay\n    }',
            '    fn use_delay(&self) -> u32 {\n        self.0.use_delay as u32\n    }')
    edit('crates/horizon-client/src/game/remote.rs', remote)
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
