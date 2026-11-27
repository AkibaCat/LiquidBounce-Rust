#!/usr/bin/env python3
"""Scaffold source/registry contract and full native-source syntax, without Rust execution."""
from pathlib import Path
import ast
import hashlib
import json
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'
RULES = ROOT / 'native/liquidbounce-behaviors/src/scaffold_block_selection.rs'


def main():
    sources = [RULES, HERE / 'scaffold-selection-native.rs', HERE / 'scaffold-selection-host.rs',
               HERE / 'scaffold-selection-patch.py', HERE / 'scaffold-selection-smoke.rs']
    before = {p: p.read_bytes() for p in sources}
    for p in [Path(__file__), HERE / 'scaffold-selection-patch.py']:
        ast.parse(p.read_text(encoding='utf8'))
    generated = {}

    def once(s, old, new):
        assert s.count(old) == 1, (old, s.count(old))
        return s.replace(old, new)

    def edit(name, transform):
        if name not in generated:
            path = UPSTREAM / name
            generated[name] = path.read_text(encoding='utf8') if path.exists() else ''
        generated[name] = transform(generated[name])

    fragment = runpy.run_path(str(HERE / 'scaffold-selection-patch.py'))['apply'](edit, once)
    assert fragment == {'hook_names': [], 'helper_names': ['client_scaffold_selection']}
    assert len(generated) == 4
    native = generated['crates/horizon-client/src/game/scaffold_selection.rs']
    compact = ''.join(native.split())
    for required in [
        'behaviours.table.default_state(binding.block)', 'ItemKind::SolidBucket|ItemKind::Other',
        'y:player.position.y', 'descending:player.shift_key_down',
        'self.client_hotbar_slot()', 'item:&s.item,count:s.countasi32',
        'subject.leather_boots=self.armor[3]', 'BlockPos::new(0,0,0)',
        'block.dynamic_shape', 'CollisionContext::empty(false)',
        'block.is(Class::FallingBlock)', 'block.is(Class::BaseEntity)',
        'self.motion.blocks.get(state)', 'letselected=self.selected;',
        'request["autoBlock"].as_bool()?', 'request["doNotUseBelowCount"].as_i64()?',
        'shape.face_shape(Direction::Up)', 'ShapeOp::XOR',
    ]:
        assert required in compact, required
    assert compact.count('collision_shape_with_context(') == 2
    assert 'holds_solid' not in native and 'SupportType' not in native
    assert 'world::read' not in native and 'with_scope' not in native
    rules = RULES.read_text(encoding='utf8')
    ordered = ['!a.unfavorable(policy)', 'a.redstone_conductor.cmp', 'a.full_cube.cmp',
               'f64::from(a.friction)', 'f64::from(a.jump_factor)', 'f64::from(a.speed_factor)',
               'b.hardness_distance(true)', 'b.count.cmp(&a.count)', 'b.hardness_distance(false)']
    start = rules.index('pub fn compare(')
    for needle in ordered:
        start = rules.index(needle, start) + len(needle)
    assert 'Ordering::Greater' in rules and 'choose(true).or_else(|| choose(false))' in rules
    assert 'stack.count<=threshold' in ''.join(rules.split()) and 'hotbar.get(selected)' in ''.join(rules.split())
    assert 'pub always: bool' in rules and 'slot_reset_delay: 5' in rules
    source_root = ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
    selection = (source_root / 'features/module/modules/world/scaffold/ScaffoldBlockItemSelection.kt').read_text(encoding='utf8')
    scaffold = (source_root / 'features/module/modules/world/scaffold/ModuleScaffold.kt').read_text(encoding='utf8')
    comparators = (source_root / 'utils/item/ItemStackComparators.kt').read_text(encoding='utf8')
    feature = (source_root / 'features/module/modules/world/scaffold/features/ScaffoldAutoBlockFeature.kt').read_text(encoding='utf8')
    assert 'entityCanStandOnFace(world, BlockPos.ZERO, player, Direction.UP)' in selection
    assert 'stack.count > doNotUseBelowCount' in scaffold
    assert 'for (i in 0..8)' in scaffold
    assert 'player.inventory.getItem(player.inventory.selectedSlot).blockCount()' in scaffold
    assert 'PREFER_MORE: Comparator<ItemStack> = PREFER_FEWER.reversed()' in comparators
    assert 'comparingDouble { it.friction.toDouble() }' in comparators
    assert 'int("SlotResetDelay", 5, 0..40' in feature
    javap = (HERE / 'scaffold-BlockStateBase.javap.txt').read_text(encoding='utf8')
    method = javap[javap.index('public final boolean entityCanStandOnFace('):]
    method = method[:method.index('\n  public ', 1)]
    assert 'CollisionContext.of:' in method and 'getCollisionShape:' in method and 'Block.isFaceFull:' in method
    loader = (UPSTREAM / 'crates/horizon-world/src/level/data.rs').read_text(encoding='utf8')
    for name in ['HangingSignItem', 'StandingAndWallBlockItem', 'DoubleHighBlockItem', 'ScaffoldingBlockItem', 'PlaceOnWaterBlockItem', 'SolidBucketItem', 'BlockItem']:
        assert 'has("' + name + '")' in loader
    hooks = generated['crates/horizon-client/src/game/native_hooks.rs']
    assert 'if !self.native_window() || !self.native_client_slot_owner()' in hooks
    with tempfile.TemporaryDirectory(prefix='liquidbounce-scaffold-selection-') as temporary:
        paths = []
        for i, (name, content) in enumerate(generated.items()):
            path = Path(temporary) / (str(i) + '-' + Path(name).name)
            path.write_text(content, encoding='utf8')
            paths.append(str(path))
        smoke = Path(temporary) / 'smoke.rs'
        smoke.write_text((HERE / 'scaffold-selection-smoke.rs').read_text(encoding='utf8'), encoding='utf8')
        subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths, str(smoke)], check=True)
    assert all(path.read_bytes() == raw for path, raw in before.items())
    evidence = {'compiled': False, 'rustExecuted': False, 'generatedRustSourcesParsed': 4,
                'preparedRustSmokeParsed': True, 'checks': ['original comparator directions and stable tie',
                'strict threshold with all-valid fallback', 'real BlockItem/default state',
                'actual player/held/boots collision context at ZERO', 'dynamic empty-context full cube',
                'raw-selected blockCount and offhand', 'full native patch composition'],
                'sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}}
    (HERE / 'scaffold-selection-evidence.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    print('Scaffold selection static checks passed: 4 complete native Rust sources and prepared smoke parsed; no Cargo/rustc/Rust execution or shared/staging writes')


if __name__ == '__main__':
    main()
