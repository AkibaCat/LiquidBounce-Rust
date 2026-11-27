#!/usr/bin/env python3
"""Scaffold prediction/edge contracts and complete native composition, static only."""
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
RULES=ROOT/'native/liquidbounce-behaviors/src/scaffold_prediction.rs'


def main():
    sources=[RULES,HERE/'scaffold-prediction-native.rs',HERE/'scaffold-prediction-host.rs',
             HERE/'scaffold-prediction-patch.py',HERE/'scaffold-prediction-smoke.rs',HERE/'scaffold-edge-native.rs',
             HERE/'falling-forecast-native.rs']
    before={p:p.read_bytes() for p in sources}
    for path in [Path(__file__),HERE/'scaffold-prediction-patch.py']:
        ast.parse(path.read_text(encoding='utf8'))
    generated={}

    def once(text,old,new):
        assert text.count(old)==1,(old,text.count(old))
        return text.replace(old,new)

    def edit(name,transform):
        if name not in generated:
            path=UPSTREAM/name
            generated[name]=path.read_text(encoding='utf8') if path.exists() else ''
            if name=='crates/horizon-client/src/game.rs':
                generated[name]=once(generated[name],'mod native_hooks;\n','mod native_hooks;\nmod crosshair_falling;\n')
        generated[name]=transform(generated[name])

    runpy.run_path(str(HERE/'scaffold-planner-patch.py'))['apply'](edit,once)
    runpy.run_path(str(HERE/'falling-forecast-patch.py'))['apply'](edit,once)
    fragment=runpy.run_path(str(HERE/'scaffold-prediction-patch.py'))['apply'](edit,once)
    assert fragment=={'hook_names':[],'helper_names':['client_scaffold_prediction']}
    assert len(generated)==9,generated.keys()
    native=''.join(generated['crates/horizon-client/src/game/scaffold_prediction.rs'].split()).replace(',)',')')
    for token in ['horizon_core::jmath::atan2(line.direction[2],line.direction[0])asf32',
                  'vector(point).rotate_y(angle)', 'letnearest=line.nearest(array(player.position))',
                  'rules::with_length(line.direction,3.0)', 'Some([edge.x,player.position.y,edge.z])',
                  'jump:false,shift:false,sprint:player.sprinting||keys[6]',
                  'super::falling_forecast::visit_with_keys(self,1,0.0,input',
                  'horizontal.length_squared()>0.003*0.003',
                  'Vec3::direction_from_rotation(', 'distance.mul_add(direction.x,from.x)',
                  '.inflate(Vec3::new(-0.05,0.0,-0.05))',
                  '.move_by(Vec3::new(0.0,player.fall_distance-step,0.0))',
                  'self.player_entity_collisions()', 'world_movement::no_world_collision(',
                  'self.inventory.get(self.client_hotbar_slot())', 'CollisionContext::for_entity(',
                  'descending:player.shift_key_down', 'subject.leather_boots=',
                  'self.scaffold_would_fall(predicted.add_vector(horizontal))',
                  'self.scaffold_planner_state.support.map(|s|[s.offset_x,s.offset_z])']:
        assert token in native,token
    edge=''.join((HERE/'scaffold-edge-native.rs').read_text(encoding='utf8').split())
    for token in ['direction.length_squared()<=1.0e-12', 'Pose::Standing.dimensions(player.scale)',
                  'fma(from,-1000.0,direction)', 'fma(to,1000.0,direction)',
                  '&CollisionContext::empty(false)', 'subject:None',
                  'min.x-0.3,min.y-1.0,min.z-0.3',
                  'max.y+f64::from(allowed_drop)+0.05',
                  'ifcontaining.is_empty(){returnSome(at);}',
                  'ifcontaining.iter().any(|b|b.contains(to)){returnNone;}',
                  'bounds.clip(extended_to,extended_from)', 'boxes.retain(|b|!containing.contains(b))']:
        assert token in edge,token
    assert edge.index('forzinminimum[2]')<edge.index('foryinminimum[1]')<edge.index('forxinminimum[0]')
    rules=''.join(RULES.read_text(encoding='utf8').split())
    for token in ['bootstrap_backoff:0.2','cutoff_distance:0.05','warmup_placements:2',
                  'squared.abs()<1.0e-5','ifself.history.len()>4',
                  'rotate_y(offset,angle)','rotate_y(average,-angle)',
                  'ifsettings.warmup_placements<=0',
                  'bootstrap[i]+blend*(predicted[i]-bootstrap[i])']:
        assert token in rules,token
    source=(ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/world/scaffold/features/ScaffoldMovementPrediction.kt').read_text(encoding='utf8')
    for token in ['MAX_PLACEMENT_OFFSETS = 4','atan2(optimalLine.direction.z, optimalLine.direction.x).toFloat()',
                  '(player.position() - fallOffPoint).yRot(lineDirAngle)',
                  'last.yRot(-lineDirAngle)','bootstrapPos.add(it.offsetX, 0.0, it.offsetZ)',
                  'bootstrapPos.lerp(predictedPos, getWarmupBlendFactor())']:
        assert token in source,token
    with tempfile.TemporaryDirectory(prefix='liquidbounce-scaffold-prediction-') as temporary:
        paths=[]
        for index,(name,content) in enumerate(generated.items()):
            path=Path(temporary)/(str(index)+'-'+Path(name).name)
            path.write_text(content,encoding='utf8');paths.append(str(path))
        smoke=Path(temporary)/'smoke.rs';smoke.write_bytes((HERE/'scaffold-prediction-smoke.rs').read_bytes());paths.append(str(smoke))
        subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',*paths],check=True)
    assert all(path.read_bytes()==raw for path,raw in before.items())
    evidence={'compiled':False,'rustExecuted':False,'generatedRustSourcesParsed':9,'preparedRustSmokeParsed':True,
              'checks':['source edge support scan with actual standing dimensions','empty collision context and real moving block geometry',
                        'reverse clip edge walk returns point','one full native body replay with jump/sneak false',
                        'two-tick fall checks include collidable entities','four offset history in native float yRot coordinates',
                        'Math atan2 and float angle','bootstrap backoff and initial lateral offset',
                        'source warmup lerp and cutoff distance'],
              'sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}}
    (HERE/'scaffold-prediction-evidence.json').write_text(json.dumps(evidence,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
    print('ScaffoldMovementPrediction static checks passed: 9 complete native Rust sources and prepared smoke parsed; no Cargo/rustc/Rust execution or shared/staging writes')


if __name__=='__main__':main()
