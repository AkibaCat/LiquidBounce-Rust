"""Client setItemSlot HEAD history, read-only ViewedInventoryScreen and invsee completion."""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(edit, once):
    for name in ['native', 'screen']:
        edit(f'crates/horizon-client/src/game/inventory_tracker_{name}.rs',
             lambda _, name=name: (HERE / f'inventory-tracker-{name}.rs').read_text(encoding='utf8'))

    def game(text):
        text = once(text, 'mod native_hooks;', 'mod native_hooks;\nmod inventory_tracker_native;\nmod inventory_tracker_screen;')
        text = once(text, '    native_mod_screen: Option<String>,',
                    '    native_mod_screen: Option<String>,\n    client_inventory_tracker: inventory_tracker_native::State,\n    client_inventory_tracker_world_pending: std::cell::Cell<bool>,')
        text = once(text, '            native_mod_screen: None,',
                    '            native_mod_screen: None,\n            client_inventory_tracker: Default::default(),\n            client_inventory_tracker_world_pending: std::cell::Cell::new(false),')
        return once(text, '            Screen::Mod => {}',
                    '            Screen::Mod => self.client_inventory_tracker_draw(&mut batch),')
    edit('crates/horizon-client/src/game.rs', game)

    def hooks(text):
        anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        text = once(text, anchor, (HERE / 'inventory-tracker-host.rs').read_text(encoding='utf8') + '\n' + anchor)
        text = once(text, '    pub(super) fn native_client_tick_pre(&mut self) {',
                    '    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_inventory_tracker_tick();')
        # The existing world event intentionally accepts &self. A Cell marks
        # invalidation immediately; the first next client observation consumes
        # it before reading an old player object or equipment/history handle.
        return once(text, '        mod_api::dispatch_client_world_change(self, mod_api::mods(), dimension);',
                    '        self.client_inventory_tracker_world_pending.set(true);\n        mod_api::dispatch_client_world_change(self, mod_api::mods(), dimension);')
    edit('crates/horizon-client/src/game/native_hooks.rs', hooks)

    for target, fragment in [('perspective.rs', 'preview'), ('player_hooks.rs', 'layers')]:
        edit('crates/horizon-client/src/game/' + target,
             lambda text, fragment=fragment: text + '\n' + (HERE / f'inventory-tracker-{fragment}.rs').read_text(encoding='utf8'))

    def remote(text):
        before = '                    for (slot, item, stack) in items {\n'
        after = '''                }
                for (slot, item, stack) in items {
                    self.client_inventory_tracker_equipment(p.id, slot, stack.as_ref());
                    if let Some(m) = self.mob_index.get_mut(&mut self.mobs, p.id as u32) {
'''
        return once(text, before, after)
    edit('crates/horizon-client/src/game/remote.rs', remote)

    def keys(text):
        # Physical keys are consumed before multiple rebound actions dispatch;
        # otherwise closing E could reopen inventory on its next mapped action.
        anchor = '''    pub(super) fn mapped_key_event(
        &mut self,
        key: KeyCode,
        pressed: bool,
        repeat: bool,
    ) -> Result<()> {
'''
        return once(text, anchor, anchor + '        if pressed && self.client_inventory_tracker_key(key) { return Ok(()); }\n')
    edit('crates/horizon-client/src/game/play_input.rs', keys)

    edit('crates/horizon-client/src/game/stations.rs', lambda text: once(text,
         '        let mut lines = self.stack_tooltip_lines(stack, false, creative);',
         '        let mut lines = self.stack_tooltip_lines(stack, false, creative);\n        self.client_inventory_tracker_lore(&mut lines);'))

    def held(text):
        text = once(text, '            let mut stack = if local_player {',
                    '            let tracked_stack = (!lit).then(||self.client_inventory_tracker_held_stack(mob,off_hand)).flatten();\n            let tracked = tracked_stack.is_some();\n            let mut stack = if let Some(stack) = tracked_stack { stack } else if local_player {')
        return once(text, '            if !local_player && horizon_client::mobs::gear::enchanted(mob, slot) {',
                    '            if !tracked && !local_player && horizon_client::mobs::gear::enchanted(mob, slot) {')
    edit('crates/horizon-client/src/game/mob_blocks.rs', held)

    edit('crates/horizon-client-render/src/chat.rs', lambda text:
         text + '\n' + (HERE / 'inventory-tracker-chat.rs').read_text(encoding='utf8'))

    def chat(text):
        text = once(text, '    fn notify_chat_suggestion_completion(&mut self) {',
                    '    fn notify_chat_suggestion_completion(&mut self) {\n        self.client_inventory_tracker_complete();')
        return once(text, '    pub(super) fn draw_chat_screen(&mut self, batch: &mut HudBatch) {',
                    '    pub(super) fn draw_chat_screen(&mut self, batch: &mut HudBatch) {\n        self.client_inventory_tracker_complete();')
    edit('crates/horizon-client/src/game/chat.rs', chat)
    return {'hook_names': ['client_inventory_tracker'], 'helper_names': ['client_inventory_tracker_query']}
