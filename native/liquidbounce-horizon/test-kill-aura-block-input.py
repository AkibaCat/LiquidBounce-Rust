#!/usr/bin/env python3
"""Static source composition and release/render boundary checks; no Rust runs."""
from pathlib import Path
import ast
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'
NATIVE = 'crates/horizon-client/src/game/native_hooks.rs'
FOV = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
GUARD = '        if super::auto_walk_runtime::keeps_using() && self.0.using.is_some() { return; }\n'


def main():
    owned = {name: (HERE / name).read_bytes() for name in [
        'kill-aura-block-input-host.rs', 'kill-aura-block-input-patch.py',
        'kill-aura-host.rs', 'kill-aura-patch.py',
    ]}
    apply = runpy.run_path(str(HERE / 'kill-aura-block-input-patch.py'))['apply']
    for name in ['kill-aura-block-input-patch.py', Path(__file__).name]:
        ast.parse((HERE / name).read_text(encoding='utf8'))

    def once(s, old, new):
        assert s.count(old) == 1, (old, s.count(old))
        return s.replace(old, new)

    def compose(auto_walk_first, baseline):
        original = {}
        result = {}

        def edit(path, transform):
            if path not in result:
                if path == NATIVE:
                    original[path] = "impl Game<'_> {\n" + owned['kill-aura-host.rs'].decode('utf8').replace('\r\n', '\n') + FOV + '    fn fov_static_anchor(&self) {}\n}\n'
                else:
                    source = ROOT / 'build/staging/horizon' if baseline == 'staged' else UPSTREAM
                    original[path] = (source / path).read_text(encoding='utf8')
                    if baseline in ['smart-eat', 'auto-bow']:
                        if path.endswith('/remote.rs'):
                            original[path] = once(original[path], 'self.mod_use_tick(slot, ticks, !self.right || !in_hand)', 'self.mod_use_tick(slot, ticks, !self.client_use_key_down() || !in_hand)')
                            original[path] = once(original[path], 'let (attack, use_item) = (self.left, self.right);', 'let (attack, use_item) = (self.left, self.client_use_key_down());')
                        elif path.endswith('/using_item.rs'):
                            original[path] = once(original[path], 'if !self.right || hand_changed || self.screen != Screen::Playing', 'if !self.client_use_key_down() || hand_changed || self.screen != Screen::Playing')
                        elif path.endswith('/frame_geometry.rs'):
                            original[path] = once(original[path], 'let legacy_block = self.pvp_legacy_look()\n            && self.right', 'let legacy_block = self.pvp_legacy_look()\n            && self.client_use_key_down()')
                    if baseline == 'auto-bow':
                        if path.endswith('/remote.rs'):
                            original[path] = once(original[path], 'self.mod_use_tick(slot, ticks, !self.client_use_key_down() || !in_hand)', 'self.mod_use_tick(slot, ticks, (!super::auto_bow::updating()&&!self.client_use_key_down()) || !in_hand)')
                        elif path.endswith('/using_item.rs'):
                            original[path] = once(original[path], 'if !self.client_use_key_down() || hand_changed || self.screen != Screen::Playing', 'if hand_changed || (!super::auto_bow::updating()&&(!self.client_use_key_down()||self.screen!=Screen::Playing))')
                result[path] = original[path]
            result[path] = transform(result[path])

        def navigation(s):
            return once(s, '    fn release_using(&mut self) {\n',
                        '    fn release_using(&mut self) {\n' + GUARD)

        remote_path = 'crates/horizon-client/src/game/remote.rs'
        if auto_walk_first and baseline != 'staged':
            edit(remote_path, navigation)
        assert apply(edit, once) == {'hook_names': [], 'helper_names': []}
        if not auto_walk_first and baseline != 'staged':
            edit(remote_path, navigation)
        assert len(result) == 7
        remote = result[remote_path]
        assert remote.count(GUARD) == 1
        assert 'if self.0.using.is_some() && self.0.client_kill_aura_use_down() { return; }' in remote
        if baseline in ['auto-bow', 'staged']:
            assert 'self.mod_use_tick(slot, ticks, (!super::auto_bow::updating()&&!self.client_kill_aura_use_down()) || !in_hand)' in remote
        else:
            assert 'self.mod_use_tick(slot, ticks, !self.client_kill_aura_use_down() || !in_hand)' in remote
        use_key = 'self.right' if baseline == 'vanilla' else 'self.client_use_key_down()'
        assert f'let (attack, use_item) = (self.left, {use_key});' in remote
        assert 'input.tick(&mut RemoteInputWorld(self), attack, use_item);' in remote
        assert remote.count('client_kill_aura_use_down()') == 2
        integrated = result['crates/horizon-client/src/game/using_item.rs']
        if baseline in ['auto-bow', 'staged']:
            assert 'if hand_changed || (!super::auto_bow::updating()&&(!self.client_kill_aura_use_down()||self.screen!=Screen::Playing))' in integrated
        else:
            assert 'if !self.client_kill_aura_use_down() || hand_changed || self.screen != Screen::Playing' in integrated
        # The start/repeat key in Game is deliberately outside this patch.
        game = (UPSTREAM / 'crates/horizon-client/src/game.rs').read_text(encoding='utf8')
        assert 'if self.right\n' in game or 'if self.right &&' in game
        assert 'crates/horizon-client/src/game.rs' not in result
        host = result[NATIVE]
        assert 'self.client_use_key_down() || matches!(enforced, 0 | 1)' in host
        assert 'item_tags.get("minecraft:swords")' in host
        assert 'ends_with("_sword")' not in host
        assert host.count('fn native_client_kill_aura_state(') == 1
        assert 'pose && self.inventory.get(self.selected)' in host
        assert 'mod_api::dispatch_client_kill_aura_state(self,mod_api::mods(),enforced,pose)' in ''.join(host.split())

        frame_path = 'crates/horizon-client/src/game/frame_geometry.rs'
        frame = result[frame_path]
        original_frame = original[frame_path]
        # Legacy presentation and offhand hiding belong to a different option.
        legacy_start = '        let legacy_block = self.pvp_legacy_look()'
        legacy_end = '        for (i, (hand, item))'
        assert frame[frame.index(legacy_start):frame.index(legacy_end)] == original_frame[original_frame.index(legacy_start):original_frame.index(legacy_end)]
        assert frame.count('if hand == Hand::Off && legacy_block {') == 1
        assert 'let render_using = if kill_aura_block_pose {\n                hand == Hand::Main' in frame
        assert 'let render_remaining = if kill_aura_block_pose {\n                7200' in frame
        assert '&& self.client_kill_aura_sword(&stack) => ItemPose::Using' in frame
        assert 'owner.using_this_item &= used_hand == Some(hand);' in frame
        assert 'owner.use_item_remaining_ticks =' not in frame
        assert 'owner.use_duration =' not in frame

        perspective = result['crates/horizon-client/src/game/perspective.rs']
        for actual in ['m.extra.using_item = using;', 'm.extra.using_off_hand = using_off_hand;', 'm.extra.blocking = blocking;']:
            assert actual in perspective
        assert 'm.extra.client_main_hand_block_pose = client_main_hand_block_pose;' in perspective
        anim_path = 'crates/horizon-client/src/mobs/anim.rs'
        anim = result[anim_path]
        off_start = '            let off = if player && m.extra.off_hand.is_some()'
        off_end = '            let mut h = if m.main_arm_left()'
        start = anim.index(off_start)
        end = anim.index(off_end, start)
        original_start = original[anim_path].index(off_start)
        original_end = original[anim_path].index(off_end, original_start)
        assert anim[start:end] == original[anim_path][original_start:original_end]
        assert 'Some(_) if m.extra.client_main_hand_block_pose => ArmPose::Block,' in anim
        return result

    for baseline in ['vanilla', 'smart-eat', 'auto-bow']:
        first = compose(True, baseline)
        second = compose(False, baseline)
        assert set(first) == set(second)
        for name in first:
            # Release guards commute; every other native output is identical.
            assert first[name].replace(GUARD, '') == second[name].replace(GUARD, ''), name
    staged_remote = ROOT / 'build/staging/horizon/crates/horizon-client/src/game/remote.rs'
    if staged_remote.exists() and 'client_kill_aura_use_down()' not in staged_remote.read_text(encoding='utf8'):
        first = compose(True, 'staged')
    with tempfile.TemporaryDirectory(prefix='liquidbounce-block-input-static-') as temporary:
        paths = []
        for index, (name, value) in enumerate(first.items()):
            path = Path(temporary) / (str(index) + '-' + Path(name).name)
            path.write_text(value, encoding='utf8')
            paths.append(str(path))
        subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths], check=True)
    for name, raw in owned.items():
        assert (HERE / name).read_bytes() == raw, name
    print('KillAura block input static checks passed: seven native files in memory, vanilla/SmartEat/AutoBow/staged chains, both AutoWalk orders, original starts, release guards, sword tags, isolated first/third-person pose, temporary rustfmt; no Rust execution or shared/staging writes')


if __name__ == '__main__':
    main()
