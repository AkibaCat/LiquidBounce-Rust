#!/usr/bin/env python3
"""Compile transformed production interpolation methods and check client outcomes."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
UPSTREAM = ROOT / 'upstream/Horizon'
OUT = ROOT / 'build/interpolation-hook-tests'


def block(text, anchor):
    start = text.index(anchor)
    opening = text.index('{', start)
    level = 1
    end = opening + 1
    while level:
        if text[end] == '{':
            level += 1
        elif text[end] == '}':
            level -= 1
        end += 1
    return text[start:end]


def main():
    changes = {}
    def edit(path, transform):
        changes[path] = transform(changes.get(path, (UPSTREAM / path).read_text(encoding='utf-8')))
    def once(text, source, target):
        if text.count(source) != 1:
            raise ValueError(f'Expected exactly one anchor: {source[:100]}')
        return text.replace(source, target, 1)
    spec = importlib.util.spec_from_file_location('interpolation_patch', HERE / 'interpolation-patch.py')
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    helper.apply(edit, once)
    OUT.mkdir(parents=True, exist_ok=True)
    world = changes['crates/horizon-world/src/stepped_interpolation.rs']
    (OUT / 'stepped.rs').write_text(world, encoding='utf-8')
    fish = changes['crates/horizon-client/src/game/fishing.rs']
    # Includes actual production movement history and the complete linear handler.
    linear = block(fish, '#[derive(Default)]\npub(super) struct HookLerp') + '\n' + block(fish, 'impl HookLerp')
    resting = block(fish, 'fn resting(')
    moves = []
    for name in ('Hook', 'RemoteHook'):
        begin = fish.index(f'impl {name} {{', fish.index('pub(super) struct HookLerp'))
        impl = block(fish[begin:], f'impl {name} {{')
        moves.append(f'impl {name} {{' + block(impl, 'pub(super) fn move_to(')
                     + block(impl, 'pub(super) fn move_to_with_steps(') + '}')
    source = '''#![cfg(test)]
#![allow(dead_code)]
use horizon_core::geometry::Vec3;
mod stepped;
use stepped::{Path, PosRot, Start, SteppedInterpolation};
''' + resting + '\n' + linear + '''
struct Hook { on_ground: bool, position: Vec3, previous: Vec3, lerp: HookLerp }
struct RemoteHook { on_ground: bool, position: Vec3, previous: Vec3, lerp: HookLerp }
''' + '\n'.join(moves) + '''
#[test]
fn stepped_policy_preserves_interval_and_ongoing_paths_then_resets_on_zero() {
    let mut handler = SteppedInterpolation::new(3);
    let entity = PosRot::default();
    handler.set_client_step_reduction(Some(2));
    assert_eq!(handler.interpolate_to(entity, Some(Path::Linear([6.,0.,0.])), Some((60.,30.))), Start::Interpolating);
    assert_eq!(handler.interpolate().unwrap(), PosRot { position:[6.,0.,0.], yaw:60.,pitch:30. });
    handler.set_client_step_reduction(None);
    handler.cancel();
    handler.interpolate_to(entity, Some(Path::Linear([6.,0.,0.])), None);
    assert!((handler.interpolate().unwrap().position[0] - 2.).abs() < 1e-6);
    handler.set_client_step_reduction(Some(3));
    assert_eq!(handler.interpolate_to(entity, Some(Path::Linear([12.,0.,0.])), None), Start::Snap(PosRot {position:[12.,0.,0.],yaw:0.,pitch:0.}));
    assert!(!handler.active());
}
#[test]
fn stepped_paths_keep_their_own_tick_lengths_until_zero_requests_snap() {
    let mut handler = SteppedInterpolation::new(3);
    handler.set_client_step_reduction(Some(1));
    let path = Path::Stepped(vec![([6.,0.,0.],3),([12.,0.,0.],3)]);
    handler.interpolate_to(PosRot::default(), Some(path.clone()), None);
    assert!((handler.interpolate().unwrap().position[0] - 2.).abs() < 1e-6);
    handler.set_client_step_reduction(Some(3));
    assert_eq!(handler.interpolate_to(PosRot::default(), Some(path), Some((42.,14.))), Start::Snap(PosRot {position:[12.,0.,0.],yaw:42.,pitch:14.}));
}
#[test]
fn linear_hook_snaps_in_packet_handler_and_restores_three_ticks_after_disable() {
    let mut hook = Hook {on_ground:false,position:Vec3::ZERO,previous:Vec3::ZERO,lerp:HookLerp::default()};
    hook.move_to_with_steps([9.,0.,0.], true, 0);
    assert_eq!(hook.position.x, 9.);
    assert_eq!(hook.previous.x, 9.);
    assert!(hook.on_ground);
    hook.move_to([18.,0.,0.], false);
    let at = hook.lerp.step(hook.position);
    assert_eq!(at.x,12.);
    let mut other = RemoteHook {on_ground:false,position:Vec3::ZERO,previous:Vec3::ZERO,lerp:HookLerp::default()};
    other.move_to_with_steps([9.,0.,0.], false, 2);
    assert_eq!(other.position.x,0.);
    assert_eq!(other.lerp.step(other.position).x,4.5);
    other.move_to_with_steps([12.,0.,0.], false, 0);
    assert_eq!(other.position.x,12.);
    assert_eq!(other.lerp.steps,0);
}
#[test]
fn raw_config_subtraction_matches_kotlin_overflow_before_max() {
    use liquidbounce_behaviors::no_interpolation::NoInterpolation;
    for (reduce, expected) in [(1,2),(2,1),(3,0),(4,0),(-2,5),(i32::MIN,0)] {
        let module = NoInterpolation {reduce_by:reduce};
        assert_eq!(module.steps(true,3),expected);
        assert_eq!(module.steps(false,3),3);
    }
}
'''
    # Only crate visibility changes: methods are compiled at the fixture root.
    (OUT / 'lib.rs').write_text(source.replace('pub(super)', 'pub(crate)'), encoding='utf-8')
    core = (UPSTREAM / 'crates/horizon-core').as_posix()
    behaviors = (ROOT / 'native/liquidbounce-behaviors').as_posix()
    (OUT / 'Cargo.toml').write_text(f'''[package]
name = "liquidbounce-interpolation-fixture"
version = "0.1.0"
edition = "2024"
[lib]
path = "lib.rs"
[dependencies]
horizon-core = {{path="{core}"}}
liquidbounce-behaviors = {{path="{behaviors}"}}
''', encoding='utf-8')
    command = ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(OUT / 'Cargo.toml')]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
    (OUT / 'cargo-test.log').write_text(result.stdout + result.stderr, encoding='utf-8')
    report = {'passed': result.returncode == 0, 'exitCode': result.returncode, 'command':command,
              'sourceSha256': {path:hashlib.sha256(text.encode()).hexdigest() for path,text in changes.items()},
              'scope':'transformed_production_methods', 'fullGameIntegrationPassed':False}
    (OUT / 'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(result.stdout + result.stderr)
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
