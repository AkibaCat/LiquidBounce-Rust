#!/usr/bin/env python3
"""ScaffoldClutchTechnique contract and native composition, without Rust execution."""
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
RULES = ROOT / 'native/liquidbounce-behaviors/src/scaffold_clutch.rs'


def main():
    sources = [RULES, HERE / 'scaffold-clutch-native.rs', HERE / 'scaffold-clutch-host.rs',
               HERE / 'scaffold-clutch-patch.py', HERE / 'scaffold-clutch-smoke.rs']
    before = {p: p.read_bytes() for p in sources}
    for p in [Path(__file__), HERE / 'scaffold-clutch-patch.py']:
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

    runpy.run_path(str(HERE / 'block-placement-patch.py'))['apply'](edit, once)
    fragment = runpy.run_path(str(HERE / 'scaffold-clutch-patch.py'))['apply'](edit, once)
    assert fragment == {'hook_names': [], 'helper_names': ['client_scaffold_clutch']}
    assert len(generated) == 7
    native = generated['crates/horizon-client/src/game/scaffold_clutch.rs']
    compact = ''.join(native.split())
    for required in [
        'placement::holds_solid(self.game,position)', 'placement::Options::center(self.stack,location)',
        'placement::Priority::Position(search.priority)', 'options.range=self.block_range',
        'placement::offsets(4)', 'ItemStack::new("minecraft:sandstone",64)',
        'letcurrent=[player.position.x,player.position.y,player.position.z]',
        'placement::ClickTrace::World', 'placement::FailedClick::Nothing',
        'placement::raytrace_block(game,target.interacted,rotation,block_range)',
        '(faces_player&&pointed_at).then(||placement::planned_click(target))',
        'self.interaction_range("minecraft:block_interaction_range",4.5,5.0)',
        'self.interaction_range("minecraft:entity_interaction_range",3.0,5.0)',
        'letverification_range=block_range.max(entity_range)',
    ]:
        assert required in compact, required
    assert compact.index('ifletSome(hit)=placement::verify_click(') < compact.index('letfaces_player=') < compact.index('letpointed_at=')
    assert '"inside":click.inside' in native
    assert 'if faces_player' not in native and 'world::read' not in native
    rules = ''.join(RULES.read_text(encoding='utf8').split())
    for required in ['lettop=feet_y.floor();feet_y=position[1];',
                     'position[1]>=top||top>from[1]', 'from[1]+f64::from(eye_height)-top>range',
                     'from[0].floor()asi32', 'from[2].floor()asi32',
                     'ifhost.can_stand_on(below){returnNone;}',
                     'below[1].wrapping_add(offset[1]).wrapping_sub(1)',
                     'priority:[from[0],top-0.5,from[2]]',
                     'if!aims_ahead{returnnow;}', 'host.can_click(&now)',
                     'let(&next,rest)=fall.split_first()?', 'target(host,next,rest,eye_height,range,full_offsets)']:
        assert required in rules, required
    source = (ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/world/scaffold/techniques/ScaffoldClutchTechnique.kt').read_text(encoding='utf8')
    for text in ['rotationTiming == NORMAL', 'target(player.position(), fall, bestStack)',
                 'now?.takeIf { it.click(RotationManager.serverRotation) != null }',
                 'target(fall.first(), fall.subList(1, fall.size), bestStack)',
                 'BlockPos.containing(from.x, top - 1, from.z)',
                 'BlockPosOffsets.FULL.offsets.filterNot { below.offset(it).below().canStandOn() }',
                 'from.copy(y = top - 0.5)', 'verifyClick(rotation)?.let { return it }',
                 'blockHitResult.takeIf { facesPlayer && pointedAt }']:
        assert text in source, text
    hooks = generated['crates/horizon-client/src/game/native_hooks.rs']
    assert 'if !self.native_window() || !self.native_client_slot_owner()' in hooks
    with tempfile.TemporaryDirectory(prefix='liquidbounce-scaffold-clutch-') as temporary:
        paths = []
        for i, (name, content) in enumerate(generated.items()):
            path = Path(temporary) / (str(i) + '-' + Path(name).name)
            path.write_text(content, encoding='utf8')
            paths.append(str(path))
        smoke = Path(temporary) / 'smoke.rs'
        smoke.write_text((HERE / 'scaffold-clutch-smoke.rs').read_text(encoding='utf8'), encoding='utf8')
        subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths, str(smoke)], check=True)
    assert all(path.read_bytes() == raw for path, raw in before.items())
    evidence = {'compiled': False, 'rustExecuted': False, 'generatedRustSourcesParsed': 7,
                'preparedRustSmokeParsed': True, 'checks': ['fall layer crossing before query',
                'fixed source xz and ground cancellation', 'FULL support filtering and priority',
                'Normal current-click then next-position aim', 'OnTick current-position aim',
                'max-range full-world verification', 'block-range target-only different-face fallback',
                'actual slot stack or source sandstone planning fallback'],
                'sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}}
    (HERE / 'scaffold-clutch-evidence.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    print('ScaffoldClutchTechnique static checks passed: 7 complete native Rust sources and prepared smoke parsed; no Cargo/rustc/Rust execution or shared/staging writes')


if __name__ == '__main__':
    main()
