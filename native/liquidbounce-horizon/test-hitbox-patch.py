#!/usr/bin/env python3
"""Compile real Hitbox consumers, actual generated Host/events, JAR oracle facts."""
import hashlib
import json
from pathlib import Path
import runpy
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
UP=ROOT/'upstream/Horizon'
OUT=ROOT/'build/hitbox-hook-tests'


def main():
    changed={}
    def edit(path,fn):
        changed[path]=fn(changed.get(path,(UP/path).read_text(encoding='utf-8')))
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one anchor ({text.count(before)}):{before[:100]!r}')
        return text.replace(before,after,1)
    reach=runpy.run_path(str(HERE/'reach-patch.py'))
    hitbox=runpy.run_path(str(HERE/'hitbox-patch.py'))
    reach['apply'](edit,once)
    contract=hitbox['apply'](edit,once)
    block=runpy.run_path(str(HERE/'test-camera-control-patch.py'))['block']
    take=lambda path,anchor:block(changed.get(path,(UP/path).read_text(encoding='utf-8')),anchor)
    native=changed['crates/horizon-client/src/game/native_hooks.rs']
    contains=block(native,'pub(super) fn client_attack_range_contains(')
    hooks=hitbox['HOOKS']
    OUT.mkdir(parents=True,exist_ok=True)
    for name in ['targeting','targeting_facts','hitbox']:
        (OUT/(name+'.rs')).write_bytes((ROOT/'native/liquidbounce-behaviors/src'/(name+'.rs')).read_bytes())
    (OUT/'gizmos.rs').write_text(changed['crates/horizon-client/src/gizmos.rs'],encoding='utf-8')
    api=OUT/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text("impl Game<'_>{"+hooks+'}',encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True,exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'helpers':[{'name':name,'target':"horizon_client::game::native_hooks::Game<'_>::native_"+name} for name in contract['helper_names']],
        'hooks':[{'name':name,'target':"horizon_client::game::native_hooks::Game<'_>::native_"+name,'mod_api':{'contract':'listen'}} for name in contract['hook_names']]},indent=2)+'\n',encoding='utf-8')
    source=(HERE/'hitbox-fixture.rs').read_text(encoding='utf-8')
    replacements={'HOOKS':hooks,'CONTAINS':contains,
        'MOB_PICK':take('crates/horizon-client/src/game.rs','pub(super) fn mob_hit_within('),
        'STAND_PICK':take('crates/horizon-client/src/game/armor_stands.rs','pub(super) fn armor_stand_hit('),
        'ATTACK_PICK':take('crates/horizon-client/src/game/hand.rs','pub(super) fn attack_range_pick(').replace('super::view_vector','crate::view_vector').replace('super::spear::','crate::spear::'),
        'ATTACK_REACH':take('crates/horizon-client/src/game/hand.rs','fn attack_reach('),
        'RAY_BOX':take('crates/horizon-client/src/game.rs','fn ray_box('),
        'HITS_ALONG':take('crates/horizon-client/src/spear.rs','fn hits_along('),
        'HITS_IN_REACH':take('crates/horizon-client/src/spear.rs','pub fn hits_in_reach(')}
    oracle=json.loads((ROOT/'research/target-entity-facts.json').read_text(encoding='utf-8'))
    assertions=''
    for row in oracle['margins']:
        size=str(row['size'])+'f32'
        assertions+='''{let size='''+size+''';let bounds=Aabb::new(Vec3::new(-0.3,64.,-0.3),Vec3::new(0.3,65.8,0.3)).inflate(Vec3::new(f64::from(size),f64::from(size),f64::from(size)));
        let lo=bounds.min_position();let hi=bounds.max_position();
        assert_eq!([lo.x,lo.y,lo.z,hi.x,hi.y,hi.z].map(f64::to_bits),'''+str([int(v) for v in row['inflatedBits']]).replace(' ','')+''');
        assert_eq!(hitbox::Hitbox{size,..Default::default()}.component_margin(0.125).to_bits(),'''+str(row['componentMarginBits'])+''');
        let mut g=gizmos::Gizmos::default();g.hitbox_with_margin([0.,64.,0.],(0.6,1.8),1.62,true,(0.,0.),size);
        assert_eq!(g.lines[12].start[1],64.-f64::from(size)+f64::from(1.62f32)-f64::from(0.01f32));
        assert_eq!(g.lines[24].start[1],64.+f64::from(1.62f32));}\n'''
    replacements['ORACLE_ASSERTIONS']=assertions
    for name,value in replacements.items():source=source.replace('__'+name+'__',value)
    source=source.replace('pub(super)','pub(crate)')
    (OUT/'lib.rs').write_text(source,encoding='utf-8')
    (OUT/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}',encoding='utf-8')
    (OUT/'Cargo.toml').write_text('''[package]
name="hitbox-hook-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[dependencies]
serde_json="1"
glam="0.30"
horizon-core={path="'''+(UP/'crates/horizon-core').as_posix()+'''"}
[build-dependencies]
horizon-mod-api-gen={path="'''+(UP/'crates/horizon-mod-api-gen').as_posix()+'"}\n',encoding='utf-8')
    checks=[]
    for name,command in [('test',['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--','--test-threads=1']),('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (OUT/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr)
        checks.append({'name':name,'exitCode':result.returncode})
        if result.returncode:break
    report={'passed':all(check['exitCode']==0 for check in checks),'checks':checks,'contract':contract,
        'sourceSha256':{name:hashlib.sha256(text.encode()).hexdigest() for name,text in changed.items()},
        'productionBodies':list(replacements),'javaEntityRows':len(oracle['entities']),'javaMarginRows':len(oracle['margins']),
        'apiGenerator':'horizon_mod_api_gen::generate(Target::Client), complete Host/events/dispatch',
        'boundaries':['Game shell actual entity values','engine registry lifecycle and global REST integration handled by root','full game screenshots separate']}
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(0 if report['passed']else 1)


if __name__=='__main__':main()
