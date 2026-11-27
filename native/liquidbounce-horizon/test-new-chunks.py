"""One focused smoke of real NewChunks behavior/config/runtime/native bridge."""
from pathlib import Path
import argparse, json, os, re, runpy, shutil, subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]

def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
    config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\nenum ModuleKind{NewChunks}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.newChunks"}}\n'
    for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
        if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
        config+=body(shared,anchor)+'\n'
    config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{NewChunks(settings::NewChunksSetting)}\n#[derive(Default)]pub struct SettingsPatch{pub changes:Vec<SettingChange>,pub issues:Vec<ConfigError>}\n'
    config+=f'#[path="{(HERE/"engine/src/config/new_chunks.rs").as_posix()}"]mod settings;pub use settings::*;\n'
    config+='''#[test]fn config_round_trip(){let mut s=crate::new_chunks::NewChunks::default();s.render_distance=12;s.render_y=-63.5;s.auto_y=true;s.smooth=false;s.persist=false;s.new_color=0x80775533u32 as i32;s.old_color=0x11223344;let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_new_chunks(&s,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch::default();settings::parse_new_chunks(&mut patch,&nodes(&tree,"NewChunks","NewChunks").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);assert_eq!(patch.changes.len(),7);let mut restored=crate::new_chunks::NewChunks::default();for SettingChange::NewChunks(setting)in patch.changes{setting.apply(&mut restored);}assert_eq!(restored,s);if format==JsonFormat::Interop{let values=tree["value"].as_array().unwrap();assert_eq!(values[2]["range"],json!({"from":4,"to":128}));assert_eq!(values[2]["suffix"],"chunks");assert_eq!(values[2]["key"],"liquidbounce.module.newChunks.renderDistance");}}}\n'''
    (out/'config.rs').write_text(config,encoding='utf8')
    patched={};base=ROOT/'build/staging/horizon';error=None;meta=None
    def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
    def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
    try:
        meta=runpy.run_path(str(HERE/'new-chunks-patch.py'))['apply'](edit,once)
        for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
    except Exception as e:error=repr(e)
    primitives=(base/'crates/horizon-client/src/world_primitives.rs').read_text(encoding='utf8').split('#[cfg(test)]')[0]
    (out/'world_primitives.rs').write_text(primitives,encoding='utf8')
    packets=(ROOT/'upstream/Horizon/crates/horizon-protocol/src/packets/play.rs').read_text(encoding='utf8')
    packet_fields={
        'LevelChunkWithLight': 'pub x:i32,pub z:i32',
        'ForgetLevelChunk':'pub pos:[i32;2]',
        'BlockUpdate':'pub pos:[i32;3],pub state:i32',
        'SectionBlocksUpdate':'pub section:[i32;3],pub changes:Vec<i64>'}
    for name,fields in packet_fields.items():
        source=re.search(r'\b'+name+r'\s*\{([^}]+)\}',packets).group(1)
        for field,kind in re.findall(r'pub (\w+):([^,]+)',fields):assert re.search(r'\b'+field+r':\s*'+re.escape(kind).replace(r'\;',r';'),source.replace(' ','')),(name,field,source)
    lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;extern crate self as horizon_world;extern crate self as horizon_protocol;\n'
    lib+=f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/new_chunks.rs").as_posix()}"]pub mod new_chunks;\nmod config;mod world_primitives;\n'
    lib+='pub mod packets{pub mod play{'+''.join(f'pub struct {name}'+'{'+fields+'}'for name,fields in packet_fields.items())+'pub enum Clientbound{'+','.join(f'{name}({name})'for name in packet_fields)+',Other}}}\n'
    lib+='''pub mod mod_api{pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientNewChunksEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}}
pub mod runtime{use std::{rc::Rc,cell::RefCell};pub struct Modules{pub new_chunks:crate::new_chunks::NewChunks,pub running:bool}impl Modules{pub fn query_running(&self,_:&(),_:&str)->bool{self.running}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks{state:Rc<State>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(State{behaviors:RefCell::new(Modules{new_chunks:Default::default(),running:true}),registry:()})}}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.state.clone()}}}
'''
    lib+=f'#[path="{(HERE/"engine/src/new_chunks_runtime.rs").as_posix()}"]mod new_chunks_runtime;\n'
    lib+=(HERE/'new-chunks-native-fixture.rs').read_text(encoding='utf8').replace('NATIVE_PATH',(HERE/'new-chunks-game.rs').as_posix())
    lib+=(HERE/'new-chunks-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
    (out/'Cargo.toml').write_text('[package]\nname="new-chunks-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
    locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-new-chunks-locales.py'))
    for language in ['zh_cn','zh_tw']:target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
    report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':meta,'patchedFiles':len(patched),'scope':'Production NewChunks behavior/config/runtime/native Game bridge and actual WorldPrimitives API compiled. Protocol field declarations checked against pinned source; registry/fluid/Game/Host providers are declarations. One focused Cargo smoke, native patch composed in memory, copied Chinese dictionaries. Aggregate native and visual validation remain root-owned.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)

if __name__=='__main__':main()
