#!/usr/bin/env python3
"""MultiActions' three expression queries and handleKeybinds queue ordering.

Compose after attack, FastPlace, Reach and NoBlockInteract. No source checkout
is modified: all changes go through the build transform's in-memory edit.
"""
from pathlib import Path

HERE = Path(__file__).resolve().parent
HOOK_NAMES = ['client_multi_actions']
HELPER_NAMES = []
HOOKS = (HERE / 'multi-actions-host.rs').read_text(encoding='utf-8')


def apply(edit, replace_once):
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: replace_once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HOOKS + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))

    def first_person(text):
        text = replace_once(text, '    fn using_item(&self) -> bool;\n',
            '    fn using_item(&self) -> bool;\n'
            '    fn client_multi_actions(&self) -> (bool, bool, bool) { (false, false, false) }\n')
        text = replace_once(text, 'self.miss_time > 0 || w.using_item() || w.piercing_weapon()',
            'self.miss_time > 0 || (w.using_item() && !w.client_multi_actions().2) || w.piercing_weapon()')
        text = replace_once(text, '''    pub fn start_use_item(&mut self, w: &mut impl HandWorld) {
        if self.destroying.active {
''', '''    pub fn start_use_item(&mut self, w: &mut impl HandWorld) {
        if self.destroying.active && !w.client_multi_actions().0 {
''')
        text = replace_once(text, '''        if w.using_item() {
            if !use_down {
                w.release_using();
            }
        } else {
            for _ in 0..attack_clicks {
                instant |= self.start_attack(w);
            }
            for _ in 0..use_clicks {
                self.start_use_item(w);
            }
        }
''', '''        let (mut attack_clicks, mut use_clicks) = (attack_clicks, use_clicks);
        if w.using_item() {
            if !use_down { w.release_using(); }
            if !w.client_multi_actions().1 { attack_clicks = 0; }
            use_clicks = 0;
        }
        for _ in 0..attack_clicks { instant |= self.start_attack(w); }
        for _ in 0..use_clicks { self.start_use_item(w); }
''')
        return text
    edit('crates/horizon-client/src/first_person.rs', first_person)

    def input_world(text):
        text = replace_once(text, '    fn using_item(&self) -> bool;\n',
            '    fn using_item(&self) -> bool;\n'
            '    fn client_attack_while_using(&self) -> bool { false }\n')
        text = replace_once(text, '        let [attack, use_item, pick] = std::mem::take(&mut self.clicks);\n',
            '        let [mut attack, mut use_item, mut pick] = std::mem::take(&mut self.clicks);\n')
        return replace_once(text, '''            // Vanilla drains all three click queues while using an item,
            // including the tick that releases it.
        } else {
            for _ in 0..attack {
                instant |= world.start_attack();
            }
            for _ in 0..use_item {
                world.start_use();
            }
            for _ in 0..pick {
                world.pick();
            }
        }
''', '''            // MixinMinecraft clears use/pick and conditionally attack, then
            // returns false: handleKeybinds executes its normal click loops.
            if !world.client_attack_while_using() { attack = 0; }
            use_item = 0;
            pick = 0;
        }
        for _ in 0..attack { instant |= world.start_attack(); }
        for _ in 0..use_item { world.start_use(); }
        for _ in 0..pick { world.pick(); }
''')
    edit('crates/horizon-client-input/src/interaction_input.rs', input_world)

    def remote(text):
        text = replace_once(text, '''        if self.remote.as_ref().is_some_and(|r| r.is_destroying) {
            return;
        }
''', '''        if self.remote.as_ref().is_some_and(|r| r.is_destroying) && !self.client_multi_actions().0 {
            return;
        }
''')
        text = replace_once(text, '''        if self.clicks.miss_time > 0
            || self.using.is_some()
''', '''        if self.clicks.miss_time > 0
            || (self.using.is_some() && !self.client_multi_actions().2)
''')
        anchor = "impl horizon_client::interaction_input::InteractionWorld for RemoteInputWorld<'_, '_> {\n"
        return replace_once(text, anchor, anchor + '''    fn client_attack_while_using(&self) -> bool {
        self.0.client_multi_actions().1
    }
''')
    edit('crates/horizon-client/src/game/remote.rs', remote)

    def hand(text):
        # FastPlace has already split the isDestroying and hands-busy checks.
        text = replace_once(text, '''        if self.clicks.destroying.active {
            return Ok(());
        }
''', '''        if self.clicks.destroying.active && !self.client_multi_actions().0 {
            return Ok(());
        }
''')
        anchor = "impl HandWorld for GameHands<'_, '_> {\n"
        text = replace_once(text, anchor, anchor + '''    fn client_multi_actions(&self) -> (bool, bool, bool) {
        self.game.client_multi_actions()
    }
''')
        text = replace_once(text, '    pub(super) fn tick_local_hand(&mut self) -> Result<()> {\n',
            '    pub(super) fn tick_local_hand(&mut self, was_using: bool) -> Result<()> {\n')
        text = replace_once(text, '''        let attack_clicks = std::mem::take(&mut self.attack_clicks);
        let use_clicks = std::mem::take(&mut self.use_clicks);
''', '''        let mut attack_clicks = std::mem::take(&mut self.attack_clicks);
        let mut use_clicks = std::mem::take(&mut self.use_clicks);
        let mut pick_clicks = std::mem::take(&mut self.pick_clicks);
        if was_using {
            // tick_using has run the real release; key queues still depend on
            // the initial isUsingItem, even on the tick that releases/completes.
            if !self.client_multi_actions().1 { attack_clicks = 0; }
            use_clicks = 0;
            pick_clicks = 0;
        }
''')
        text = replace_once(text, '            clicks.tick(&mut world, attack_clicks, 0, down, false);\n',
            '''            if clicks.right_click_delay > 0 { clicks.right_click_delay -= 1; }
            let mut instant = false;
            for _ in 0..attack_clicks { instant |= clicks.start_attack(&mut world); }
''')
        text = replace_once(text, '            let ticks = self.clicks.destroying.ticks;\n',
            '            let ticks = clicks.destroying.ticks;\n')
        text = replace_once(text, '''                if self.using.is_some() {
                    break;
                }
                used = self.use_with_hand();
''', '''                used = self.use_with_hand();
''')
        text = replace_once(text, '''            self.release_tick_targets();
            used?;
            // A tick of breaking:''', '''            if used.is_ok() {
                for _ in 0..pick_clicks { self.client_pick_item(); }
                if self.right && self.use_delay == 0 && !was_using && self.using.is_none() && !self.dialog_open() {
                    used = self.use_with_hand();
                }
            }
            if used.is_ok() {
                let mut clicks = std::mem::take(&mut self.clicks);
                let mut world = GameHands { game: self, error: None };
                clicks.continue_attack(&mut world, !instant && down);
                let error = world.error.take();
                if clicks.miss_time > 0 { clicks.miss_time -= 1; }
                self.clicks = clicks;
                if let Some(error) = error { used = Err(error); }
            }
            self.release_tick_targets();
            used?;
            // A tick of breaking:''')
        return replace_once(text, '        self.use_clicks = 0;\n', '        self.use_clicks = 0;\n        self.pick_clicks = 0;\n')
    edit('crates/horizon-client/src/game/hand.rs', hand)

    def game(text):
        text = replace_once(text, '    use_clicks: u32,\n', '    use_clicks: u32,\n    pick_clicks: u32,\n')
        text = replace_once(text, '            use_clicks: 0,\n', '            use_clicks: 0,\n            pick_clicks: 0,\n')
        text = replace_once(text, '        horizon_client::spans::time("tick_using", || self.tick_using());\n',
            '        let client_keybind_was_using = self.using.is_some();\n'
            '        horizon_client::spans::time("tick_using", || self.tick_using());\n')
        text = replace_once(text, '|| self.tick_local_hand())?;', '|| self.tick_local_hand(client_keybind_was_using))?;')
        start = text.index('        // Minecraft.handleKeybinds: held use repeats only while no item is in\n')
        end = text.index('        if running {\n', start)
        return text[:start] + text[end:]
    edit('crates/horizon-client/src/game.rs', game)

    def play_input(text):
        text = replace_once(text, '                if pressed && self.using.is_none() {\n                    self.use_clicks += 1;\n',
            '                if pressed {\n                    self.use_clicks += 1;\n')
        anchor = '            MouseButton::Middle if pressed => {\n'
        start = text.index(anchor)
        body = start + len(anchor)
        end = text.index('            }\n            _ => {}', body)
        pick = text[body:end]
        text = text[:body] + '                self.pick_clicks += 1;\n' + text[end:]
        method = '    pub(super) fn client_pick_item(&mut self) {\n' + '\n'.join(line[8:] for line in pick.splitlines()) + '\n    }\n'
        marker = '    /// A mouse button while playing (window or control socket).\n'
        return replace_once(text, marker, method + marker)
    edit('crates/horizon-client/src/game/play_input.rs', play_input)
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
