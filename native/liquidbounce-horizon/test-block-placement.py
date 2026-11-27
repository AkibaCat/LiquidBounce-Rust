#!/usr/bin/env python3
"""Shared target finding kernel: static composition now; focused Rust in the owner's batch."""
from pathlib import Path
import argparse
import ast
import runpy
import subprocess
import tempfile
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]

def static_checks():
    texts={};original={};staged=ROOT/'build/staging/horizon'
    def edit(name,transform):
        if name not in texts:
            path=staged/name
            original[name]=path.read_bytes() if path.exists() else None
            texts[name]=original[name].decode('utf8') if original[name] is not None else ''
        texts[name]=transform(texts[name])
    def once(text,old,new):
        assert text.count(old)==1,(old,text.count(old))
        return text.replace(old,new)
    fragment=runpy.run_path(str(HERE/'block-placement-patch.py'))['apply'](edit,once)
    assert fragment=={'hook_names':[],'helper_names':[]}
    native=texts['crates/horizon-client/src/game/block_placement.rs']
    for token in ['EntityContextKind::Player','EntityContextKind::None','game.client_hotbar_slot()',
                  'liquidbounce_resolve_placement_context','traverse_blocks','clip_declared_outline',
                  'super::interaction_shapes::boxes','horizon_world::view_direction::view_vector',
                  'client_ghost_hand_targets','target.matches(true']:
        assert token in native,token
    compact=''.join(native.split()).replace(',}', '}')
    for token in ['geometry::trig::RAD_TO_DEG,jmath::atan2',
                  'atan2(z,x)*f64::from(RAD_TO_DEG)',
                  'Click{hit:planned_hit(target),inside:false}',
                  'inside:hit.inside', 'raytrace_block_click(game,pos,rotation,range).map(|c|c.hit)',
                  'verify_click(game,target,rotation,range,trace,on_failure).map(|c|c.hit)',
                  'cached.context_outline_shape(state,game.outline_context_kind())']:
        assert token in compact, token
    assert 'rotation: host.looking_at(point, eye)' in texts['crates/horizon-client/src/game/placement_core.rs']
    context=texts['crates/horizon-world/src/level/place.rs'].split('// Appended to horizon_world::level::place,')[1]
    assert 'scaffolding_update_context(&ctx, binding.block)'in context
    assert 'ctx.replace_clicked || sim.can_be_replaced(sim.state(ctx.pos), &ctx)'in context
    assert 'can_support_center(pos, Direction::Up)'in context
    assert all(token not in context for token in ['remote_can_place','get_state_for_placement','entity_obstruction'])
    generator=(HERE/'make-behavior-patch.py').read_text(encoding='utf8')
    applied="'block-placement-patch.py'"in generator
    plan=None if applied else runpy.run_path(str(HERE/'integrate-block-placement.py'))['build_plan']()
    final=generator if applied else plan.text['native/liquidbounce-horizon/make-behavior-patch.py']
    assert final.index("'block-placement-patch.py'")<final.index("'step-patch.py'):")
    for path in [Path(__file__),HERE/'block-placement-patch.py',HERE/'integrate-block-placement.py']:
        ast.parse(path.read_text(encoding='utf8'))
    for name,raw in original.items():
        path=staged/name
        assert (path.read_bytes()if path.exists()else None)==raw,name
    if plan:
        for name,raw in plan.original.items():assert(ROOT/name).read_bytes()==raw,name
    count='already applied'if applied else f'{len(plan.steps)} shared edit/{len(plan.text)} file'
    print(f'Placement kernel static checks passed; {len(texts)} native files in memory; {count}; no shared/staging writes')

def model_checks():
    with tempfile.TemporaryDirectory(prefix='liquidbounce-placement-')as temporary:
        temporary=Path(temporary)
        source=(HERE/'block-placement-smoke.rs').read_text(encoding='utf8').replace('"block-placement-kernel.rs"','"'+(HERE/'block-placement-kernel.rs').as_posix()+'"')
        fixture=temporary/'placement-smoke.rs';fixture.write_text(source,encoding='utf8');exe=temporary/'placement-smoke.exe'
        subprocess.run(['rustc','--edition=2024','--test',str(fixture),'-o',str(exe)],check=True)
        subprocess.run([str(exe),'placement_smoke','--nocapture'],check=True)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group();group.add_argument('--static-only',action='store_true');group.add_argument('--model-only',action='store_true')
    args=parser.parse_args()
    if not args.model_only:static_checks()
    if not args.static_only:model_checks()
    else:print('Rust execution deferred to combined batch')
if __name__=='__main__':main()
