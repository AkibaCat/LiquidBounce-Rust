#!/usr/bin/env python3
"""Normal/Expand and all aim factories: compose and parse only, no Rust execution."""
from pathlib import Path
import ast
import hashlib
import json
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main():
    sources = [ROOT / ('native/liquidbounce-behaviors/src/' + name) for name in
               ['scaffold_aim.rs', 'scaffold_techniques.rs', 'scaffold_lifecycle.rs']]
    sources += [HERE / name for name in ['scaffold-techniques-native.rs', 'scaffold-techniques-host.rs',
                'scaffold-techniques-patch.py', 'scaffold-aim-smoke.rs', 'scaffold-techniques-smoke.rs']]
    originals = {p: p.read_bytes() for p in sources}
    generated = {}

    def once(text, old, new):
        assert text.count(old) == 1, (old, text.count(old))
        return text.replace(old, new)

    def edit(name, transform):
        if name not in generated:
            path = ROOT / 'upstream/Horizon' / name
            generated[name] = path.read_text(encoding='utf8') if path.exists() else ''
        generated[name] = transform(generated[name])

    for file in [Path(__file__), HERE / 'scaffold-techniques-patch.py']:
        ast.parse(file.read_text(encoding='utf8'))
    runpy.run_path(str(HERE / 'block-placement-patch.py'))['apply'](edit, once)
    fragment = runpy.run_path(str(HERE / 'scaffold-techniques-patch.py'))['apply'](edit, once)
    assert fragment == {'hook_names': [], 'helper_names': ['client_scaffold_technique']}
    assert len(generated) == 9
    native = ''.join(generated['crates/horizon-client/src/game/scaffold_techniques.rs'].split()).replace(',)', ')').replace(',]', ']').replace(',}', '}')
    for token in [
        'Vec3::direction_from_rotation(r[1],r[0])',
        'Vec3::new(0.0,0.0,1.0).rotate_y(yaw*DEG_TO_RAD)',
        '(jmath::atan2(-p[0],p[2])asf32)*RAD_TO_DEG',
        'super::edge_random::with_thread_random(|random|random.next_f64_between(from,to))',
        'pose(&request["pose"])?.dimensions(player.scale).eye_height',
        'location.eye_height=eye_height',
        'placement::capture_location(self,Some(predicted),rotation)',
        'rules::normal_radius(request["freeze"].as_bool()?,down)',
        'placement::Priority::Line{position:line.anchor,direction:line.direction}',
        'options.face=placement::FaceFactory::Custom(&mutfactory)',
        'options.consider_facing_away=down',
        'height.target(predicted.map(|v|v.floor()asi32))',
        'letxz=[-(jmath::sin(f64::from(radians))asf32),jmath::cos(f64::from(radians))asf32]',
        'rules::find_expand(predicted,xz,length,|candidate|{placement::find(self,height.target(candidate),&mutoptions)})',
        'ItemStack::new("minecraft:sandstone",64)',
        'placement::verify_click(self,&target,rotation,range,placement::ClickTrace::World',
        '"inside":click.inside', '"heightState":height.write()',
        'rules::reverse_rotation(p.yaw,p.pitch)',
        'placement::looking_at(target.placed.map(|v|f64::from(v)+0.5),self.eye_position()?)',
    ]:
        assert token in native, token
    assert native.index('ifself.towering||self.state.was_towering') < native.index('elseifself.down') < native.index('elseifself.ceiling') < native.index('self.state.targeted_position(')
    aim = ''.join(sources[0].read_text(encoding='utf8').split())
    for token in ['letangle=matchmode{Mode::ReverseYaw=>180.,Mode::DiagonalYaw=>75.,_=>45.,}',
                  'determinant>1e-9*a*c', 'distance<best_distance-1e-9',
                  'ifx<y{a}else{b}', 'x<=5.&&y<=5.',
                  '(-1e-9..=1.+1e-9).contains(&t)', 't.mul_add(d[i],at[i])',
                  'letfrom_player=Linear{anchor:location.eye,direction:optimal.direction,segment:false,}',
                  '(clipped.area()>=0.0001).then_some(clipped)',
                  'math.random_between(face.from[i],face.to[i])']:
        assert token in aim, token
    placement = ''.join(generated['crates/horizon-client/src/game/block_placement.rs'].split()).replace(',}', '}')
    assert 'inside:hit.inside' in placement
    assert 'Click{hit:planned_hit(target),inside:false}' in placement
    assert 'atan2(z,x)*f64::from(RAD_TO_DEG)' in placement
    assert 'rotation:host.looking_at(point,eye)' in ''.join(generated['crates/horizon-client/src/game/placement_core.rs'].split())
    hooks = generated['crates/horizon-client/src/game/native_hooks.rs']
    assert 'if !self.native_window() || !self.native_client_slot_owner()' in hooks
    with tempfile.TemporaryDirectory(prefix='liquidbounce-scaffold-techniques-') as temporary:
        paths = []
        for index, (name, value) in enumerate(generated.items()):
            path = Path(temporary) / (str(index) + '-' + Path(name).name)
            path.write_text(value, encoding='utf8')
            paths.append(str(path))
        for name in ['scaffold-aim-smoke.rs', 'scaffold-techniques-smoke.rs']:
            path = Path(temporary) / name
            path.write_text((HERE / name).read_text(encoding='utf8'), encoding='utf8')
            paths.append(str(path))
        subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths], check=True)
    assert all(p.read_bytes() == raw for p, raw in originals.items())
    evidence = {'compiled': False, 'rustExecuted': False, 'completeNativeRustSourcesParsed': 9,
        'preparedSmokeSourcesParsed': 2, 'sharedOrStagingWrites': False,
        'checks': ['eight frozen face factories and geometry domains', 'actual predicted-pose scaled eye height',
            'actual RotationManager server rotation input and movement keys', 'Freeze FULL before Down offsets',
            'line/position outline priority and source sandstone fallback', 'inclusive Expand loop and Float truncation',
            'lazy target-height mutation and tower/down/ceiling precedence', 'Telly Reverse ties-even rounding and Reset',
            'RequiresSight source fallthrough', 'actual-eye max-range world verification',
            'Down/Expand planned result fallback with inside=false', 'real clipped inside preserved',
            'Java Math rotation with widened float degree constant'],
        'sha256': {str(p.relative_to(ROOT)).replace('\\', '/'): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}}
    (HERE / 'scaffold-techniques-evidence.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    print('Scaffold Normal/Expand static checks passed: 9 complete native Rust sources and 2 prepared smoke sources parsed; no Cargo/rustc/Rust execution or shared/staging writes')


if __name__ == '__main__':
    main()
