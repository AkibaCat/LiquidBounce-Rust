#!/usr/bin/env python3
"""Source contracts and native Scaffold planner composition, without Rust execution."""
from pathlib import Path
import ast
import hashlib
import json
import runpy
import subprocess
import tempfile

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
UPSTREAM=ROOT/'upstream/Horizon'
RULES=ROOT/'native/liquidbounce-behaviors/src/scaffold_movement_planner.rs'


def main():
    sources=[RULES,HERE/'scaffold-planner-native.rs',HERE/'scaffold-planner-host.rs',
             HERE/'scaffold-planner-patch.py',HERE/'scaffold-planner-smoke.rs']
    before={p:p.read_bytes() for p in sources}
    for p in [Path(__file__),HERE/'scaffold-planner-patch.py']:
        ast.parse(p.read_text(encoding='utf8'))
    generated={}

    def once(text,old,new):
        assert text.count(old)==1,(old,text.count(old))
        return text.replace(old,new)

    def edit(name,transform):
        if name not in generated:
            path=UPSTREAM/name
            generated[name]=path.read_text(encoding='utf8') if path.exists() else ''
        generated[name]=transform(generated[name])

    fragment=runpy.run_path(str(HERE/'scaffold-planner-patch.py'))['apply'](edit,once)
    assert fragment=={'hook_names':[],'helper_names':['client_scaffold_planner']}
    assert len(generated)==4
    native=''.join(generated['crates/horizon-client/src/game/scaffold_planner.rs'].split()).replace(',)',')')
    for token in ['letbounds=player.bounds()', '(min.y.floor()asi32).wrapping_sub(1)',
                  'forxinmin.x.floor()asi32..=max.x.floor()asi32',
                  'forzinmin.z.floor()asi32..=max.z.floor()asi32',
                  '&CollisionContext::empty(false)', 'subject:None',
                  'ifshape.is_empty(){continue;}', 'rules::Candidate::from_boxes',
                  'ifinput==[false;4]{None}',
                  'rules::movement_angle(self.player.as_ref()?.yaw,input)',
                  'horizon_core::geometry::Vec3::direction_from_rotation(0.0,yaw)']:
        assert token in native,token
    rules=''.join(RULES.read_text(encoding='utf8').split())
    for token in ['constSURFACE_EPSILON:f64=1.0e-3','constOVERLAP_HYSTERESIS:f64=0.02',
                  'wrap(self.direction_angle-current).abs()<=30.0', '.round_ties_even()',
                  'while self.history.len()>=4', 'self.history.back()==Some(&block)',
                  '[self.history.back().copied(),self.last_position]',
                  'candidate.stable_against(best)', 'dot(line.direction,direction)>=0.5',
                  't.mul_add(self.direction[i],self.position[i])',
                  'self.support=None;self.last_position=None;returnNone']:
        assert token.replace(' ','') in rules,token
    source=(ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/world/scaffold/ScaffoldMovementPlanner.kt').read_text(encoding='utf8')
    for token in ['MAX_LAST_PLACE_BLOCKS: Int = 4','DIRECTION_HYSTERESIS_DEGREES = 30.0F',
                  'SUPPORT_SURFACE_EPSILON = 1.0E-3','SUPPORT_OVERLAP_HYSTERESIS = 0.02',
                  'Mth.floor(box.minY) - 1','collisionShape = blockPos.state?.getCollisionShape(world, blockPos)',
                  'lastBlocksLine.direction.dot(direction) < 0.5',
                  'preferredLastPlaced?.takeIf { it.isStableComparedTo(bestCandidate) }',
                  'lastDirectionAngle = Float.NaN']:
        assert token in source,token
    with tempfile.TemporaryDirectory(prefix='liquidbounce-scaffold-planner-') as temporary:
        paths=[]
        for index,(name,content) in enumerate(generated.items()):
            path=Path(temporary)/(str(index)+'-'+Path(name).name)
            path.write_text(content,encoding='utf8');paths.append(str(path))
        smoke=Path(temporary)/'smoke.rs';smoke.write_bytes((HERE/'scaffold-planner-smoke.rs').read_bytes());paths.append(str(smoke))
        subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',*paths],check=True)
    assert all(p.read_bytes()==raw for p,raw in before.items())
    evidence={'compiled':False,'rustExecuted':False,'generatedRustSourcesParsed':4,'preparedRustSmokeParsed':True,
              'checks':['actual empty-context support shapes at floor(minY)-1','native box iteration and support ranking',
                        'first candidate on exact tie','last placement then previous support hysteresis',
                        'preserved in-block offset','four placement history and consecutive deduplication',
                        '30-degree direction hysteresis and ties-to-even quantization',
                        'history-line projection with FMA and 60-degree divergence cutoff',
                        'real native float-table direction','no-input preserves planner state'],
              'sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}}
    (HERE/'scaffold-planner-evidence.json').write_text(json.dumps(evidence,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
    print('ScaffoldMovementPlanner static checks passed: 4 complete native Rust sources and prepared smoke parsed; no Cargo/rustc/Rust execution or shared/staging writes')


if __name__=='__main__':main()
