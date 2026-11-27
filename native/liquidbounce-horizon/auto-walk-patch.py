#!/usr/bin/env python3
"""Owned Agent navigation and real Horizon body adapters for AutoWalk."""
from pathlib import Path
HERE = Path(__file__).resolve().parent
NAV = HERE.parent / 'liquidbounce-behaviors/src/navigation'

def apply(edit, once):
    for source in sorted(NAV.rglob('*.rs')):
        relative = source.relative_to(NAV).as_posix()
        content = source.read_text(encoding='utf8')
        edit('crates/horizon-client/src/navigation/' + relative, lambda _, value=content: value)
    edit('crates/horizon-client/src/lib.rs', lambda s: once(once(s,
        '#![forbid(unsafe_code)]\n', '#![deny(unsafe_code)]\n'),
        'pub mod game;\n', 'pub mod game;\npub mod navigation;\n'))
    modules = ('world', 'player', 'simulation', 'host', 'vehicle', 'travel', 'runtime')
    for module in modules:
        content = (HERE / ('auto-walk-' + module + '.rs')).read_text(encoding='utf8')
        edit('crates/horizon-client/src/game/auto_walk_' + module + '.rs', lambda _, value=content: value)
    declarations = ''.join(('#[allow(unsafe_code)]\n' if module == 'world' else '') + 'mod auto_walk_' + module + ';\n' for module in modules)
    edit('crates/horizon-client/src/game.rs', lambda s: once(s, 'mod native_hooks;\n', 'mod native_hooks;\n' + declarations))
    content = (HERE / 'auto-walk-physics.rs').read_text(encoding='utf8')
    edit('crates/horizon-client/src/navigation_physics.rs', lambda _: content)
    edit('crates/horizon-client/src/client_lib.rs', lambda s: once(s, 'pub mod motion;\n', 'pub mod motion;\npub mod navigation_physics;\n'))

    def motion(s):
        s = once(s, '        Ok(self.player.shift_key_down)\n', '        Ok(crate::navigation_physics::staying_on_surface(self.player.shift_key_down))\n')
        return once(s, '''        edge_backoff::back_off_from_edge(self, input)
            .map_err(|e| io::Error::other(e.to_string()).into())
''', '''        let result = edge_backoff::back_off_from_edge(self, input)
            .map_err(|e| -> Box<dyn std::error::Error> { io::Error::other(e.to_string()).into() })?;
        Ok(crate::navigation_physics::finish_backoff(movement, result, self.player.shift_key_down))
''')
    edit('crates/horizon-client/src/motion.rs', motion)
    def remote(s):
        if '    is_destroying: bool,\n' in s:
            s = once(s, '    is_destroying: bool,\n', '    pub(super) is_destroying: bool,\n')
        elif s.count('    pub(super) is_destroying: bool,\n') != 1:
            raise ValueError('Missing unique native navigation is_destroying field')
        for method in ('remote_mining', 'remote_break_restricted', 'remote_use_item_on', 'remote_use_item', 'remote_send_carried'):
            signature = '    fn ' + method + '('
            promoted = '    pub(super) fn ' + method + '('
            if signature in s:
                s = once(s, signature, promoted)
            elif promoted not in s:
                raise ValueError('Missing native navigation method: ' + method)
        s = once(s, '    fn server_block(&mut self, pos: [i32; 3], state: i32) {\n', '    fn server_block(&mut self, pos: [i32; 3], state: i32) {\n        super::auto_walk_runtime::confirmed(super::auto_walk_world::current_epoch(self), pos, state.max(0) as u32);\n')
        s = once(s, '            self.sync_block_entity((pos[0], pos[1], pos[2]), Some(previous), state);\n', '            if previous != state { super::auto_walk_runtime::changed(super::auto_walk_world::current_epoch(self), pos); }\n            self.sync_block_entity((pos[0], pos[1], pos[2]), Some(previous), state);\n')
        s = once(s, '    fn remote_teleport(&mut self, p: &play::PlayerPosition) -> Result<()> {\n', '    fn remote_teleport(&mut self, p: &play::PlayerPosition) -> Result<()> {\n        super::auto_walk_runtime::corrected(super::auto_walk_world::current_epoch(self));\n')
        s = once(s, '    fn remote_continue_attack(&mut self, down: bool) {\n', '    fn remote_continue_attack(&mut self, down: bool) {\n        if super::auto_walk_runtime::keeps_destroying() { return; }\n')
        s = once(s, '    fn release_using(&mut self) {\n', '    fn release_using(&mut self) {\n        if super::auto_walk_runtime::keeps_using() && self.0.using.is_some() { return; }\n')
        return s
    edit('crates/horizon-client/src/game/remote.rs', remote)
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n','''    pub(super) fn native_client_auto_walk_config(&mut self, value: &str) { self.auto_walk_set_config(value); }
    pub(super) fn native_client_auto_walk_result(&self) -> Option<String> { self.auto_walk_get_result() }
    /// Player FOV scale, after vanilla effects and before zoom and easing.
'''))
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    fn client_reach_root_vehicle(', '    pub(super) fn client_reach_root_vehicle('))
    def hooks(s):
        s = once(s, '    pub(super) fn native_client_directional_input(&self,', '    pub(super) fn native_client_directional_input(&mut self,')
        target='        mod_api::dispatch_client_directional_input(self, mod_api::mods(), forward, backward, left, right, jump, shift, sprint);\n'
        return once(s,target,'''        let navigation = self.native_client_auto_walk_input([*forward,*backward,*left,*right,*jump,*shift,*sprint]);
        if let Some((keys,_)) = navigation { [*forward,*backward,*left,*right,*jump,*shift,*sprint] = keys; }
'''+target+'''        if let Some((keys,true)) = navigation { [*forward,*backward,*left,*right,*jump,*shift] = [keys[0],keys[1],keys[2],keys[3],keys[4],keys[5]]; }
''')
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    def lifecycle(s):
        target='        mod_api::dispatch_client_world_change(self, mod_api::mods(), dimension);\n'
        return once(s,target,'        super::auto_walk_runtime::level_changed();\n'+target)
    edit('crates/horizon-client/src/game/native_hooks.rs',lifecycle)
    return {'hook_names': [], 'helper_names': ['client_auto_walk_config', 'client_auto_walk_result']}
