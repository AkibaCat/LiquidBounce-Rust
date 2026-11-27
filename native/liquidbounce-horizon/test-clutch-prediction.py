#!/usr/bin/env python3
"""Static Clutch native composition, frozen-source rules, and Rust syntax."""
from pathlib import Path
import ast
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'


def main():
    names = ['clutch-prediction-native.rs', 'clutch-prediction-host.rs', 'clutch-prediction-patch.py']
    before = {name: (HERE / name).read_bytes() for name in names}
    for name in [Path(__file__).name, 'clutch-prediction-patch.py']:
        ast.parse((HERE / name).read_text(encoding='utf8'))
    generated = {}

    def once(s, old, new):
        assert s.count(old) == 1, (old, s.count(old))
        return s.replace(old, new)

    def edit(name, transform):
        if name not in generated:
            path = UPSTREAM / name
            generated[name] = path.read_text(encoding='utf8') if path.exists() else ''
            if name == 'crates/horizon-client/src/game.rs':
                generated[name] = once(generated[name], 'mod native_hooks;\n', 'mod native_hooks;\nmod falling_forecast;\n')
        generated[name] = transform(generated[name])

    fragment = runpy.run_path(str(HERE / 'clutch-prediction-patch.py'))['apply'](edit, once)
    assert fragment == {'hook_names': [], 'helper_names': ['client_clutch_query']}
    assert len(generated) == 3
    native = generated['crates/horizon-client/src/game/clutch_prediction.rs']
    assert 'const MAX_FALL_TICKS: i32 = 60;' in native
    assert 'self.interaction_range("minecraft:block_interaction_range", 4.5, 5.0) as f32' in native
    assert 'f64::from(range + 1.0)' in native and 'f64::from(range * range)' in native
    assert 'for &(cx, cz) in game.world.chunks.keys()' in native
    assert 'minimum[1].max(game.world.shape.min_y)' in native
    assert 'maximum[1].min(game.world.shape.max_y() - 1)' in native
    assert 'game.res.facts().get(state).replaceable' in native
    assert 'super::block_placement::outline(game, pos, true)' in native
    assert '!boxes.is_empty()' in native
    assert native.count('horizon_world::block_walk::without_shape_changes(||') == 2
    assert native.count('super::falling_forecast::visit(self, MAX_FALL_TICKS, 0.0,') == 2
    assert native.index('if allowed && !on_ground') < native.index('reachable = in_reach(self, range)') < native.index('if !reachable')
    assert '!passenger && !fall_flying && !flying && !scaffold_enabled && block_count > 0' in native
    prediction = native[native.index('if guarding {'):]
    assert 'letplayer=self.player.as_ref()' in ''.join(prediction.split())
    assert 'let steered = keys[..4].iter().any(|down| *down)' in prediction
    assert 'player.ground.on_ground && (steered || horizontal == 0.0)' in prediction
    assert prediction.index('positions.push(') < prediction.index('if player.fall_distance > f64::from(minimum_fall)') < prediction.index('player.ground.on_ground\n')
    assert '"path":must_catch.then_some(positions)' in native
    assert 'request["blockCount"].as_i64()' in native
    assert 'request["scaffoldEnabled"].as_bool()' in native
    assert 'unwrap_or(' not in native
    assert 'world::read' not in native and 'with_scope' not in native
    hooks = generated['crates/horizon-client/src/game/native_hooks.rs']
    assert 'if !self.native_window() || !self.native_client_slot_owner()' in hooks
    assert 'self.clutch_query(request)' in hooks
    assert generated['crates/horizon-client/src/game.rs'].count('mod clutch_prediction;') == 1

    source = (ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/world/ModuleClutch.kt').read_text(encoding='utf8')
    falling = (ROOT / 'upstream/Minecraft-Agent/v26_3/adapter/src/main/kotlin/net/ccbluex/agent/simulation/FallingPlayer.kt').read_text(encoding='utf8')
    shape = (ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/math/ShapeExtensions.kt').read_text(encoding='utf8')
    assert 'ModuleScaffold.enabled || ModuleScaffold.blockCount <= 0' in source
    assert 'player.fallingPlayer().findCollision(MAX_FALL_TICKS)' in source
    assert 'if (this.isEmpty) return Double.POSITIVE_INFINITY' in shape
    assert falling.index('if (simulation.fallDistance > distance)') < falling.index('if (simulation.onGround() && (steered ||')
    with tempfile.TemporaryDirectory(prefix='liquidbounce-clutch-static-') as temporary:
        paths = []
        for i, (name, content) in enumerate(generated.items()):
            path = Path(temporary) / (str(i) + '-' + Path(name).name)
            path.write_text(content, encoding='utf8')
            paths.append(str(path))
        subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths], check=True)
    for name, raw in before.items():
        assert (HERE / name).read_bytes() == raw, name
    print('Clutch prediction static checks passed: three native files in memory, real range/outline/loaded chunks, complete FallingPlayer and source guard/order rules, temporary rustfmt; no Rust execution or shared/staging writes')


if __name__ == '__main__':
    main()
