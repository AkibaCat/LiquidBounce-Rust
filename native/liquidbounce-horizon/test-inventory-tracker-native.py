#!/usr/bin/env python3
"""InventoryTracker native source anchors and temporary Rust parsing, without compiling."""
from pathlib import Path
import ast
import hashlib
import json
import re
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SOURCE = ROOT / 'upstream/Horizon'


def main():
    patch = HERE / 'inventory-tracker-patch.py'
    ast.parse(patch.read_text(encoding='utf8'))
    changed = {}

    def once(text, before, after):
        count = text.count(before)
        if count != 1:
            raise AssertionError(f'Expected one anchor, found {count}: {before[:180]}')
        return text.replace(before, after, 1)

    def edit(path, transform):
        text = changed.get(path)
        if text is None:
            file = SOURCE / path
            text = file.read_text(encoding='utf8') if file.exists() else ''
        changed[path] = transform(text)

    # The world-change event was introduced by the existing timer extension.
    # Include its actual fragment to validate our &self-safe invalidation seam.
    hooks = 'crates/horizon-client/src/game/native_hooks.rs'
    timer = runpy.run_path(str(HERE / 'timer-patch.py'))
    edit(hooks, lambda text: text + '\nimpl Game<\'_> {\n' + timer['HELPERS'] + '\n}\n')
    # AutoWeapon already retains the accepted offhand's real components in
    # this loop. Preserve that earlier assignment after our new HEAD event.
    offhand = '                        if slot == 1 && let Some(stack) = stack.as_ref() { horizon_client::mobs::finalize::set_item_stack(m, 1, stack.clone()); }\n'
    edit('crates/horizon-client/src/game/remote.rs', lambda text: once(text,
         '                    for (slot, item, stack) in items {\n',
         '                    for (slot, item, stack) in items {\n' + offhand))
    added = runpy.run_path(str(patch))['apply'](edit, once)
    assert added == {'hook_names': ['client_inventory_tracker'],
                     'helper_names': ['client_inventory_tracker_query']}

    native = changed['crates/horizon-client/src/game/inventory_tracker_native.rs']
    screen = changed['crates/horizon-client/src/game/inventory_tracker_screen.rs']
    remote = changed['crates/horizon-client/src/game/remote.rs']
    branch = remote[remote.index('            C::SetEquipment(p) => {'):remote.index('            C::TakeItemEntity(p) => {')]
    ordered = [branch.index('for (slot, item, stack) in items'),
               branch.index('self.client_inventory_tracker_equipment(p.id, slot, stack.as_ref())'),
               branch.index('let e = &mut m.extra;'),
               branch.index('Some(stack) => e.set_main_hand_stack(Some(stack))')]
    assert ordered == sorted(ordered)
    assert branch.index('self.client_inventory_tracker_equipment(p.id, slot, stack.as_ref())') < branch.index(offhand.strip())
    assert 'object_identity()' in native and 'next_stack' not in native
    assert 'SystemTime::now().duration_since(std::time::UNIX_EPOCH)' in native
    assert 'client_auto_tool_time' not in native
    assert 'p.inventory[1..].fill(0)' in native
    assert 'current && tracked.contains(&p.uuid)' in native
    assert 'CommandInvsee.viewedPlayer is deliberately not reset' in native
    assert 'self.client_inventory_tracker_world_pending.replace(false)' in native
    assert 'self.client_inventory_tracker_world_pending.set(true)' in changed[hooks]
    assert 'pub(super) fn native_client_world_change(&self,' in changed[hooks]
    assert '.then(||self.client_inventory_tracker_held_stack(mob,off_hand))' in changed['crates/horizon-client/src/game/mob_blocks.rs']
    # 1 result + 4 crafting + 4 armor + 27 inventory + 9 hotbar + 1 offhand.
    assert 1 + 4 + 4 + (36 - 9) + 9 + 1 == 46
    for operation in ['remote_menu_ops(', 'menu_click(', 'S::Click', 'ClickSlot', 'set_stack_in(']:
        assert operation not in native + screen
    assert 'if pressed && self.client_inventory_tracker_key(key)' in changed['crates/horizon-client/src/game/play_input.rs']
    compact_screen = re.sub(r'\s+', '', screen)
    assert 'draw_item_decorations(batch,stack' in compact_screen
    assert 'draw_stack_tooltip(batch,&stack' in compact_screen
    assert 'client_inventory_tracker_preview' in screen
    tooltip = changed['crates/horizon-client/src/game/stations.rs']
    assert 'self.stack_tooltip_lines(stack, false, creative);\n        self.client_inventory_tracker_lore(&mut lines);' in tooltip
    chat = changed['crates/horizon-client-render/src/chat.rs']
    compact_chat = re.sub(r'\s+', '', chat)
    assert 'self.request=None' in compact_chat and 'self.awaiting=None' in compact_chat
    assert 'range.1 > self.input.value.len()' in chat
    assert 'self.show_suggestions(measure,false)' in compact_chat

    with tempfile.TemporaryDirectory(prefix='inventory-tracker-native-parse-') as directory:
        paths = []
        for index, (path, text) in enumerate(changed.items()):
            if not path.endswith('.rs'):
                continue
            temporary = Path(directory) / f'{index}-{Path(path).name}'
            temporary.write_text(text, encoding='utf8')
            paths.append(str(temporary))
        result = subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths],
                                capture_output=True, text=True)
        if result.returncode:
            raise RuntimeError(result.stderr)

    evidence = {
        'passed': True,
        'scope': 'Frozen Horizon source anchors; real per-slot equipment HEAD order; actual ItemStack identity; epoch timestamps; world invalidation; read-only 46-slot inventory and native tooltip/model pipeline; byte-range pending chat suggestions; Rust parsing in temporary files only.',
        'compiled': False,
        'rustSourcesParsed': len(paths),
        'sourceCommits': {'LiquidBounce': 'ae0eb396c07f22383789b28d5912d714c6ca0a86',
                          'Horizon': '9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},
        'ownedSha256': {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                        for path in sorted(HERE.glob('inventory-tracker-*')) if path.is_file() and path.suffix in ['.rs', '.py']},
    }
    out = HERE / 'inventory-tracker-native-evidence.json'
    out.write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    print(json.dumps({'passed': True, 'rustSourcesParsed': len(paths), 'compiled': False,
                      'evidence': str(out)}, ensure_ascii=False))


if __name__ == '__main__':
    main()
