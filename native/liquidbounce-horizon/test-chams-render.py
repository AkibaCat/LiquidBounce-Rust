#!/usr/bin/env python3
"""Actual Chams renderer/Uploads/entity WGSL, offscreen GPU pixel tests."""
from pathlib import Path
import hashlib,json,runpy,subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
UP=ROOT/'upstream/Horizon'
OUT=ROOT/'build/chams-render-tests'

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/'gfx').mkdir(exist_ok=True)
    (OUT/'chams-green.jpg').write_bytes((HERE/'fixtures/chams-green.jpg').read_bytes())
    block=runpy.run_path(str(HERE/'test-camera-control-patch.py'))['block']
    changed={}
    def edit(path,transform):
        base=UP/path
        changed[path]=transform(changed[path] if path in changed else base.read_text(encoding='utf-8') if base.exists() else '')
    def once(source,before,after):
        if source.count(before)!=1:raise ValueError(f'Expected one Chams patch anchor, got {source.count(before)}: {before[:100]!r}')
        return source.replace(before,after,1)
    for name in ['reach','hitbox','camera-control','chams']:
        helper=runpy.run_path(str(HERE/(name+'-patch.py')))
        contract=helper['apply'](edit,once)
    hooks=helper['HOOKS']
    api=OUT/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text("impl Game<'_>{"+hooks+'}',encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True,exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'helpers':[],
        'hooks':[{'name':name,'target':"horizon_client::game::native_hooks::Game<'_>::native_"+name,'mod_api':{'contract':'listen'}} for name in contract['hook_names']]},indent=2)+'\n',encoding='utf-8')
    gfx=(UP/'crates/horizon-client-render/src/gfx.rs').read_text(encoding='utf-8')
    hud=(UP/'crates/horizon-client-render/src/hud.rs').read_text(encoding='utf-8')
    mesher=(UP/'crates/horizon-client-render/src/mesher.rs').read_text(encoding='utf-8')
    vertex=block(mesher,'pub struct Vertex {')
    globals_=block(gfx,'struct Globals {')
    decoder=hud[hud.index('enum ImageFormat {'):hud.index('/// The most texels of OptiFine random entity variants')]
    decoder+='\n'+block(hud,'fn png_bytes_size(')
    (OUT/'hud.rs').write_text('use crate::Result;\n'+decoder,encoding='utf-8')
    for name,source in {
        'chams.rs':HERE/'chams-render.rs','chams.wgsl':HERE/'chams.wgsl',
        'behavior_chams.rs':ROOT/'native/liquidbounce-behaviors/src/chams.rs',
        'entity.wgsl':UP/'crates/horizon-client-render/src/entity.wgsl',
        'hdr_light.wgsl':UP/'crates/horizon-client-render/src/hdr_light.wgsl',
        'overlay.rs':UP/'crates/horizon-client-render/src/overlay.rs',
        'test_gpu.rs':UP/'crates/horizon-client-render/src/gfx/test_gpu.rs',
    }.items():
        destination=OUT/'gfx'/name if name in ('chams.rs','chams.wgsl','test_gpu.rs') else OUT/name
        destination.write_bytes(source.read_bytes())
    for name in ['upload','linear_blend']:
        source=(UP/f'crates/horizon-client-render/src/gfx/{name}.rs').read_text(encoding='utf-8').split('#[cfg(test)]')[0]
        (OUT/'gfx'/(name+'.rs')).write_text(source,encoding='utf-8')
    source=(HERE/'chams-fixture.rs').read_text(encoding='utf-8').replace('__VERTEX__',vertex).replace('__GLOBALS__',globals_)
    source+='\n'+(HERE/'chams-hooks-fixture.rs').read_text(encoding='utf-8').replace('__HOOKS__',hooks.replace('pub(super)','pub(crate)'))
    (OUT/'lib.rs').write_text(source,encoding='utf-8')
    (OUT/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}',encoding='utf-8')
    (OUT/'Cargo.toml').write_text('''[package]
name="chams-render-tests"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
wgpu={version="30.0.1",default-features=false,features=["std","parking_lot","vulkan","dx12","wgsl"]}
serde={version="1",features=["derive"]}
serde_json="1"
bytemuck={version="1.24",features=["derive"]}
half="2.7"
glam="0.30"
pollster="0.4"
png="0.18"
zune-jpeg={version="0.4",default-features=false,features=["std"]}
image-webp={version="0.2",default-features=false}
'''+'\n[build-dependencies]\nhorizon-mod-api-gen={path="'+(UP/'crates/horizon-mod-api-gen').as_posix()+'"}\n[patch.crates-io]\n'+''.join(f'{name}={{path="{(UP/"third_party"/name).as_posix()}"}}\n' for name in ('naga','wgpu-hal')),encoding='utf-8')
    # Resolve this small fixture against the same cached dependency revisions as production.
    (OUT/'Cargo.lock').write_bytes((UP/'Cargo.lock').read_bytes())
    checks=[]
    for name,argv in [('test',['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--','--test-threads=1','--nocapture']),
        ('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        result=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (OUT/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr,flush=True)
        checks.append({'name':name,'exitCode':result.returncode})
        if result.returncode:break
    report={'passed':all(c['exitCode']==0 for c in checks),'checks':checks,
        'sourceSha256':{str(p.relative_to(OUT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in OUT.rglob('*') if p.suffix in ('.rs','.wgsl') and 'target' not in p.parts},
        'patchSourceSha256':{name:hashlib.sha256(text.encode()).hexdigest() for name,text in changed.items()},
        'apiGenerator':'horizon_mod_api_gen::generate(Target::Client), complete Host/events/dispatch',
        'boundaries':['actual capture/composite/Uploads and complete original entity shader','offscreen synthetic scene, real GPU adapter required','actual Game capture anchors applied; real generated hooks/dispatch compiled with Game shell','full game integration and actual entity render submissions are parent-owned']}
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(0 if report['passed'] else 1)

if __name__=='__main__':main()
