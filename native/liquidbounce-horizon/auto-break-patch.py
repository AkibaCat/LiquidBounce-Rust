#!/usr/bin/env python3
"""AutoBreak isDown RETURN and KeyMapping.click; compose after MultiActions/SmartEat/inventory."""
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
HOOK_NAMES = ['client_auto_break_key']
HELPER_NAMES = []


def observe_screens(text):
    """Gui.setScreen releases every mapping's click count when a screen opens."""
    pattern = re.compile(r'\b(?P<owner>(?:self|game|g|guest)(?:\.game)?)\.screen\s*=(?![=>])\s*')
    edits = []
    for match in pattern.finditer(text):
        start = match.end()
        index, depth, quoted, escape = start, 0, False, False
        while index < len(text):
            char = text[index]
            if quoted:
                if escape: escape = False
                elif char == '\\': escape = True
                elif char == '"': quoted = False
            elif char == '"': quoted = True
            elif char in '([{': depth += 1
            elif char in ')]}':
                if depth == 0: break
                depth -= 1
            elif char in ';,' and depth == 0: break
            index += 1
        if index == len(text):
            raise ValueError('Unterminated screen assignment')
        value = '{ let client_next_screen = ' + text[start:index] + '; ' + match['owner'] + '.client_keymapping_screen(client_next_screen) }'
        edits.append((start, index, value))
    for start, end, value in reversed(edits):
        text = text[:start] + value + text[end:]
    return text


def apply(edit, once):
    edit('crates/horizon-client/src/keymapping_clicks.rs', lambda _: (HERE/'keymapping-clicks.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs', lambda text: once(text, 'pub mod block_contact;\n', 'pub mod block_contact;\npub mod keymapping_clicks;\n'))

    def native(text):
        anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        text = once(text, anchor, (HERE/'auto-break-host.rs').read_text(encoding='utf-8')+'\n'+anchor)
        return once(text, '        Some((down, label))\n', '''        if name == "key.attack" {
            let destroying = self.remote.as_ref().map_or(self.clicks.destroying.active, |remote| remote.is_destroying);
            down = self.client_auto_break_attack_key(down, self.clicks.miss_time, destroying);
        }
        Some((down, label))
''')
    edit('crates/horizon-client/src/game/native_hooks.rs', native)

    def game(text):
        text = once(text, '    client_inventory_screen_serial: std::cell::Cell<u64>,\n', '''    client_keymapping_clicks: std::cell::RefCell<horizon_client::keymapping_clicks::ClickQueues>,
    client_keymapping_spectator_effect: std::cell::Cell<bool>,
    client_inventory_screen_serial: std::cell::Cell<u64>,
''')
        return once(text, '            selected: 0,\n', '''            selected: 0,
            client_keymapping_clicks: Default::default(),
            client_keymapping_spectator_effect: std::cell::Cell::new(true),
''')
    edit('crates/horizon-client/src/game.rs', game)

    defaults = '''    fn client_attack_key(&mut self, original: bool, _miss_time: i32, _destroying: bool) -> bool { original }
    fn client_attack_query_allowed(&self) -> bool { true }
    fn client_attack_mouse_grabbed(&self) -> bool { true }
'''
    def first(text):
        text = once(text, 'pub trait HandWorld {\n', 'pub trait HandWorld {\n'+defaults)
        return once(text, '        self.continue_attack(w, !instant && attack_down);\n', '''        let down = w.client_attack_query_allowed() && !instant
            && w.client_attack_key(attack_down, self.miss_time, self.destroying.active)
            && w.client_attack_mouse_grabbed();
        self.continue_attack(w, down);
''')
    edit('crates/horizon-client/src/first_person.rs', first)

    def interaction(text):
        text = once(text, 'pub trait InteractionWorld {\n', '''pub trait InteractionWorld {
    fn client_attack_key(&mut self, original: bool) -> bool { original }
    fn client_attack_query_allowed(&self) -> bool { true }
    fn client_attack_mouse_grabbed(&self) -> bool { true }
    fn client_spectator_clicks(&mut self, _was_using: bool) {}
''')
        text = once(text, '        let mut instant = false;\n', '        let mut instant = false;\n        let client_was_using = world.using_item();\n')
        text = once(text, '        for _ in 0..pick { world.pick(); }\n', '        for _ in 0..pick { world.pick(); }\n        world.client_spectator_clicks(client_was_using);\n')
        return once(text, '        world.continue_attack(!instant && attack_down);\n', '''        let down = world.client_attack_query_allowed() && !instant
            && world.client_attack_key(attack_down) && world.client_attack_mouse_grabbed();
        world.continue_attack(down);
''')
    edit('crates/horizon-client-input/src/interaction_input.rs', interaction)

    def hand(text):
        text = once(text, '    pub(super) fn tick_local_hand(&mut self, was_using: bool) -> Result<()> {\n', '''    pub(super) fn tick_local_hand(&mut self, was_using: bool) -> Result<()> {
        self.client_keymapping_prefix();
''')
        text = once(text, '                for _ in 0..pick_clicks { self.client_pick_item(); }\n', '                for _ in 0..pick_clicks { self.client_pick_item(); }\n                self.client_keymapping_spectator_clicks(was_using);\n')
        text = once(text, '                clicks.continue_attack(&mut world, !instant && down);\n', '''                let attack_down = world.client_attack_query_allowed() && !instant
                    && world.client_attack_key(down, clicks.miss_time, clicks.destroying.active)
                    && world.client_attack_mouse_grabbed();
                clicks.continue_attack(&mut world, attack_down);
''')
        anchor = "impl HandWorld for GameHands<'_, '_> {\n"
        return once(text, anchor, anchor+'''    fn client_attack_key(&mut self, original: bool, miss_time: i32, destroying: bool) -> bool {
        self.game.client_auto_break_attack_key(original, miss_time, destroying)
    }
    fn client_attack_query_allowed(&self) -> bool { self.game.client_keymapping_query_allowed() }
    fn client_attack_mouse_grabbed(&self) -> bool { self.game.client_keymapping_mouse_grabbed() }
''')
    edit('crates/horizon-client/src/game/hand.rs', hand)

    def remote(text):
        text = once(text, '    loaded_sent: bool,\n', '    pub(super) loaded_sent: bool,\n')
        text = once(text, '    input: horizon_client::interaction_input::InteractionInput,\n', '    pub(super) input: horizon_client::interaction_input::InteractionInput,\n')
        text = once(text, '    fn remote_handle_input(&mut self) {\n', '    fn remote_handle_input(&mut self) {\n        self.client_keymapping_prefix();\n')
        anchor = "impl horizon_client::interaction_input::InteractionWorld for RemoteInputWorld<'_, '_> {\n"
        return once(text, anchor, anchor+'''    fn client_attack_key(&mut self, original: bool) -> bool {
        let destroying = self.0.remote.as_ref().is_some_and(|remote| remote.is_destroying);
        self.0.client_auto_break_attack_key(original, self.0.clicks.miss_time, destroying)
    }
    fn client_attack_query_allowed(&self) -> bool { self.0.client_keymapping_query_allowed() }
    fn client_attack_mouse_grabbed(&self) -> bool { self.0.client_keymapping_mouse_grabbed() }
    fn client_spectator_clicks(&mut self, was_using: bool) { self.0.client_keymapping_spectator_clicks(was_using); }
''')
    edit('crates/horizon-client/src/game/remote.rs', remote)

    def input(text):
        return once(text, '    pub(super) fn input_player_replaced(&mut self) {\n', '''    pub(super) fn input_player_replaced(&mut self) {
        self.client_keymapping_clicks.borrow_mut().release_all();
''')
    edit('crates/horizon-client/src/game/play_input.rs', input)
    edit('crates/horizon-client/src/game/pointer_capture.rs', lambda text: once(text,
        '        self.reset_inventory_clicks();\n',
        '        self.reset_inventory_clicks();\n        if !capture { self.client_keymapping_clicks.borrow_mut().release_all(); }\n'))
    edit('crates/horizon-client/src/game/screen_backgrounds.rs', lambda text: once(text,
        '.chain(self.spectated_post_effect.map(str::to_owned))',
        '.chain(self.spectated_post_effect.filter(|_|self.client_keymapping_spectator_effect.get()).map(str::to_owned))'))
    edit('crates/horizon-client/src/game/remote_camera.rs', lambda text: once(text,
        '    pub(super) fn check_entity_post_effect(&mut self, first_person: bool) {\n',
        '    pub(super) fn check_entity_post_effect(&mut self, first_person: bool) {\n        self.client_keymapping_spectator_effect.set(true);\n'))
    edit('crates/horizon-client/src/game/world_session.rs', lambda text: once(text,
        '    pub(super) fn close_world(&mut self) -> Result<()> {\n',
        '    pub(super) fn close_world(&mut self) -> Result<()> {\n        self.client_keymapping_clicks.borrow_mut().release_all();\n'))

    # Compose after the inventory observer; its assignments are already wrapped.
    sources = [ROOT/'upstream/Horizon/crates/horizon-client/src/game.rs']
    sources.extend((ROOT/'upstream/Horizon/crates/horizon-client/src/game').rglob('*.rs'))
    for source in sources:
        edit(source.relative_to(ROOT/'upstream/Horizon').as_posix(), observe_screens)
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
