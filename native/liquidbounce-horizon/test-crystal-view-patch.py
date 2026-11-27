#!/usr/bin/env python3
"""Compile complete transformed crystal model/renderer and real generated event dispatch."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
UPSTREAM=ROOT/'upstream/Horizon'
OUT=ROOT/'build/crystal-view-hook-tests'

def block(text,anchor):
    start=text.index(anchor);opening=text.index('{',start);depth=1;end=opening+1
    while depth:
        if text[end]=='{':depth+=1
        elif text[end]=='}':depth-=1
        end+=1
    return text[start:end]

def main():
    changed={}
    def edit(path,transform):changed[path]=transform(changed.get(path,(UPSTREAM/path).read_text(encoding='utf-8')))
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one anchor: {before!r}')
        return text.replace(before,after,1)
    spec=importlib.util.spec_from_file_location('crystal_view',HERE/'crystal-view-patch.py')
    helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
    contract=helper.apply(edit,once)
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/'end_crystal.rs').write_text(changed['crates/horizon-client/src/end_crystal.rs'],encoding='utf-8')
    (OUT/'behavior.rs').write_text((ROOT/'native/liquidbounce-behaviors/src/crystal_view.rs').read_text(encoding='utf-8'),encoding='utf-8')
    source=(HERE/'crystal-view-fixture.rs').read_text(encoding='utf-8')
    native=block(changed['crates/horizon-client/src/game/native_hooks.rs'],'pub(super) fn native_client_crystal_view(')
    renderer=block(changed['crates/horizon-client/src/game/end.rs'],'fn end_crystal_geometry(')
    source=source.replace('__NATIVE_CRYSTAL_HOOK__',native).replace('__END_CRYSTAL_GEOMETRY__',renderer)
    (OUT/'lib.rs').write_text(source,encoding='utf-8')
    api=OUT/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text("impl Game<'_>{"+native+'}\n',encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True,exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'helpers':[],'hooks':[{'name':'client_crystal_view',
        'target':"horizon_client::game::native_hooks::Game<'_>::native_client_crystal_view",'mod_api':{'contract':'listen'}}]},indent=2)+'\n',encoding='utf-8')
    (OUT/'build.rs').write_text('''fn main(){
        let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");
        let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();
    }''',encoding='utf-8')
    (OUT/'Cargo.toml').write_text('''[package]
name="crystal-view-hook-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[dependencies]
glam="0.30"
horizon-core={path="'''+(UPSTREAM/'crates/horizon-core').as_posix()+'''"}
[build-dependencies]
horizon-mod-api-gen={path="'''+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n',encoding='utf-8')
    result=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,capture_output=True,text=True)
    (OUT/'test.log').write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr)
    (OUT/'result.json').write_text(json.dumps({'hookContract':contract,
        'sourceFiles':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in sorted(changed.items())},
        'testedProductionBodies':['complete end_crystal model module','Game::end_crystal_geometry','Game::native_client_crystal_view'],
        'apiGenerator':'horizon_mod_api_gen::generate(Target::Client); actual generated events and dispatch compiled',
        'boundaries':['model Part/Poses storage shell','render draw mesh shell','atlas/clock/world state shell'],
        'fixtureResult':'passed' if result.returncode==0 else 'failed','fullGameIntegrationPassed':False},indent=2)+'\n',encoding='utf-8')
    raise SystemExit(result.returncode)

if __name__=='__main__':main()
