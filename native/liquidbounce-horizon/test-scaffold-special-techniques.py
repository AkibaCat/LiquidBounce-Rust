#!/usr/bin/env python3
"""Compose GodBridge/Breezily and phase replay; prepare five batch-only tests."""
from pathlib import Path
import argparse
import ast
import hashlib
import json
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/scaffold-special-focused-01')
    args = parser.parse_args()
    out = ROOT / args.out
    out.mkdir(parents=True, exist_ok=True)
    files = [HERE / name for name in ['scaffold-special-techniques-patch.py',
        'scaffold-replay-patch.py', 'scaffold-special-techniques-native.rs',
        'scaffold-special-techniques-host.rs', 'scaffold-special-techniques-smoke.rs']]
    files.append(ROOT / 'native/liquidbounce-behaviors/src/scaffold_special_techniques.rs')
    for path in files:
        if path.suffix == '.py':
            ast.parse(path.read_text(encoding='utf8'))
    source = (HERE / 'make-behavior-patch.py').read_text(encoding='utf8')
    source = source[:source.index('\noutput = []')]
    patches = ['falling-forecast-patch.py', 'scaffold-planner-patch.py',
        'scaffold-prediction-patch.py', 'scaffold-techniques-patch.py',
        'scaffold-features-patch.py', 'scaffold-replay-patch.py',
        'scaffold-special-techniques-patch.py']
    additions = ''.join(repr(name) + ', ' for name in patches if repr(name) not in source)
    if additions:
        anchor = "'step-patch.py'):"
        assert source.count(anchor) == 1
        source = source.replace(anchor, additions + anchor)
    context = {'__file__': str(HERE / 'make-behavior-patch.py'), '__name__': 'scaffold_special_static'}
    exec(compile(source, context['__file__'], 'exec'), context)
    native = {name: after for name, (_, after) in context['changes'].items()}
    falling = native['crates/horizon-client/src/game/falling_forecast.rs']
    assert 'continues_tick && tick == 0' in falling
    assert 'after_movement(motion.player, tick, backed_off_edge)' in falling
    assert 'visit_with_keys_probe(g, ticks, add_y, keys, false,' in falling
    motion = native['crates/horizon-client/src/motion.rs']
    assert 'self.local_tick_from_phase(keys, sprint, ai_tail, false)' in motion
    assert 'self.liquidbounce_forecast_tick_from_phase(keys, false)' in motion
    assert 'if !continues_tick {' in motion
    techniques = native['crates/horizon-client/src/game/scaffold_techniques.rs']
    assert '"Normal" | "Expand" | "GodBridge" | "Breezily"' in techniques
    assert 'else if technique == "Expand"' in techniques
    assert 'trace_from_point(game,start,rotation,range)' in native['crates/horizon-client/src/game/block_placement.rs']
    hooks = native['crates/horizon-client/src/game/native_hooks.rs']
    assert hooks.count('fn native_client_scaffold_special_technique(') == 1
    assert context['helper_names'].count('client_scaffold_special_technique') == 1
    selected = ['crates/horizon-client/src/motion.rs',
        'crates/horizon-client/src/game/falling_forecast.rs',
        'crates/horizon-client/src/game/scaffold_techniques.rs',
        'crates/horizon-client/src/game/block_placement.rs',
        'crates/horizon-client/src/game/scaffold_special_rules.rs',
        'crates/horizon-client/src/game/scaffold_special_techniques.rs']
    with tempfile.TemporaryDirectory(prefix='scaffold-special-static-') as folder:
        parsed = []
        for index, name in enumerate(selected):
            target = Path(folder) / (str(index) + '.rs')
            target.write_text(native[name], encoding='utf8')
            parsed.append(str(target))
        smoke_copy = Path(folder) / 'smoke.rs'
        smoke_copy.write_text((HERE / 'scaffold-special-techniques-smoke.rs').read_text(encoding='utf8'), encoding='utf8')
        parsed.append(str(smoke_copy))
        # Format only temporary copies, never shared native or upstream files.
        subprocess.run(['rustfmt', '+stable', '--edition', '2024', '--config', 'skip_children=true', *parsed], check=True)
    rules = (ROOT / 'native/liquidbounce-behaviors/src/scaffold_special_techniques.rs').as_posix()
    smoke = (HERE / 'scaffold-special-techniques-smoke.rs').as_posix()
    (out / 'lib.rs').write_text(f'#[path="{rules}"]mod rules;\ninclude!("{smoke}");\n', encoding='utf8')
    (out / 'Cargo.toml').write_text('[package]\nname="scaffold-special-focused"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n', encoding='utf8')
    evidence = {'compiled': False, 'rustExecuted': False, 'testsPrepared': 5,
        'nativeRustSourcesParsed': len(selected), 'sharedWrites': False,
        'upstreamWrites': False, 'scope': 'Complete existing generator composed with owned pending patches before Step. Full-tick wrappers retained; explicit first-tick aiStep continuation and independent per-tick sneak backoff callback. Production GodBridge/Breezily rules and five focused tests prepared.',
        'sourceSha256': {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}}
    (out / 'preparation.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf8')
    (HERE / 'scaffold-special-techniques-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf8')
    print('GodBridge/Breezily + phase replay composed and parsed; five tests prepared; no compiler invoked')


if __name__ == '__main__':
    main()
