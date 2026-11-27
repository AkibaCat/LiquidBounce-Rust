"""Read-only shared/native composition; prepare aggregate Rust smoke, never execute it."""
from pathlib import Path
import argparse,json,runpy,shutil,hashlib
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'test-reset-vl.py').is_file()else HERE.parents[1];ROOT=NATIVE.parents[1]
def owned(name):
 if(HERE/name).is_file():return HERE/name
 return {'indicator.rs':ROOT/'native/liquidbounce-behaviors/src/indicator.rs','tnt_timer.rs':ROOT/'native/liquidbounce-behaviors/src/tnt_timer.rs','tnt_timer_config.rs':NATIVE/'engine/src/config/tnt_timer.rs','tnt_timer_runtime.rs':NATIVE/'engine/src/tnt_timer_runtime.rs'}[name]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out;assert not out.exists();out.mkdir(parents=True)
 plan=runpy.run_path(str(HERE/'integrate-tnt-timer.py'))['prepare']();body=runpy.run_path(str(NATIVE/'test-reset-vl.py'))['body'];base=ROOT/'build/staging/horizon';patched={}
 def once(t,a,b):assert t.count(a)==1,(a,t.count(a));return t.replace(a,b)
 def edit(path,fn):patched[path]=fn(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).is_file()else''))
 meta=runpy.run_path(str(HERE/'tnt-timer-patch.py'))['apply'](edit,once)
 for p,t in patched.items():target=out/'patched'/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 native=patched['crates/horizon-client/src/game/tnt_timer.rs'];assert't.bounds().center()'in native and'self.tnts.iter()'in native and'camera.position'in native
 geometry=patched['crates/horizon-client/src/game/tnt.rs'];assert'self.mark_outline(&out[first..],color)'in geometry and'self.client_tnt_timer_glow(t.fuse)'in geometry
 queue=patched['crates/horizon-client/src/game/indicator.rs'];assert'entries.sort_by_key(|v| v.priority)'in queue and'native_physical_window_size'in queue and'for v in &mut batch.vertices[first..]'in queue
 game=patched['crates/horizon-client/src/game.rs'];assert game.index('indicator::clear();')<game.index('let mut hud = horizon_client::spans::time("hud", || self.hud(size));');assert game.index('self.client_tnt_timer_submit(&mut batch);')<game.index('self.gui_layer("minecraft:hotbar"');assert'native_color(color)'in queue
 gfx=patched['crates/horizon-client-render/src/gfx.rs'];assert'indicator_at<=blur_at'in gfx and'indicator_at>blur_at&&indicator_at<=frame.hud.vertices.len()as u32'in gfx
 for name in ['indicator.rs','tnt_timer.rs','tnt_timer_config.rs','tnt_timer_runtime.rs']:(out/name).write_text(owned(name).read_text(encoding='utf8'),encoding='utf8')
 for name in ['indicator-render.rs','indicator.wgsl']:(out/name.replace('-','_')if name.endswith('.rs')else out/name).write_text((HERE/name).read_text(encoding='utf8'),encoding='utf8')
 shutil.copyfile(HERE/'tnt-timer-time-vectors.json',out/'tnt-timer-time-vectors.json')
 with(out/'indicator_render.rs').open('a',encoding='utf8')as f:f.write('\n#[cfg(test)]pub(crate)fn pending_vertices()->Vec<Vertex>{PENDING.with_borrow_mut(std::mem::take)}\n')
 (out/'hud_coverage.rs').write_text((ROOT/'upstream/Horizon/crates/horizon-client-render/src/gfx/hud_coverage.rs').read_text(encoding='utf8').split('#[cfg(test)]')[0],encoding='utf8')
 node=(ROOT/'native/liquidbounce-behaviors/src/target_renderer.rs').read_text(encoding='utf8');(out/'target_renderer.rs').write_text('use serde_json::Value;\n#[derive(Clone,Debug)]\n'+body(node,'pub struct Node')+'\n',encoding='utf8')
 shared=(NATIVE/'engine/src/config.rs').read_text(encoding='utf8');config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]TNTTimer}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.tntTimer"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{TNTTimer(TNTTimerSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n#[path="tnt_timer_config.rs"]mod settings;pub use settings::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code,unused_imports)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;mod target_renderer;pub mod indicator;pub mod tnt_timer;mod config;mod hud_coverage;#[path="indicator_render.rs"]mod render;\n'
 curve=(ROOT/'native/liquidbounce-behaviors/src/tracers_curve.rs').as_posix();lib+=f'pub mod tracers{{#[path="{curve}"]pub mod curve;}}\n'
 lib+='''
pub mod mod_api{pub struct ClientTntTimerEvent<'a>{pub input:&'a str,pub output:&'a mut String}pub trait Host{}impl Host for(){}}
pub mod runtime{use std::{cell::RefCell,rc::Rc};pub struct Modules{pub tnt_timer:crate::tnt_timer::TNTTimer,pub active:bool}impl Modules{pub fn query_running(&self,_:&(),n:&str)->bool{self.active&&n=="TNTTimer"}}pub struct State{pub registry:(),pub behaviors:RefCell<Modules>}#[derive(Clone)]pub struct LiquidBounceHooks(Rc<State>);impl Default for LiquidBounceHooks{fn default()->Self{Self(Rc::new(State{registry:(),behaviors:RefCell::new(Modules{tnt_timer:Default::default(),active:true})}))}}impl LiquidBounceHooks{pub fn state(&self)->&State{&self.0}}}
#[path="tnt_timer_runtime.rs"]mod tnt_timer_runtime;
'''+(HERE/'tnt-timer-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 manifest=(ROOT/'build/potion-fx-static-04/Cargo.toml').read_text(encoding='utf8').replace('name="potion-fx-smoke"','name="tnt-timer-smoke"');(out/'Cargo.toml').write_text(manifest,encoding='utf8')
 updater=runpy.run_path(str(HERE/'update-tnt-timer-locales.py'));counts={};locale=out/'localization';locale.mkdir()
 for lang in ['zh_cn','zh_tw']:
  path=locale/f'display_{lang}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json',path);updater['update'](lang,path);data=json.loads(path.read_text(encoding='utf8'));expected={f'liquidbounce.module.tntTimer.{k}'for k in updater['TEXTS']};assert expected<=data['descriptions'].keys();counts[lang]=len(expected)
 report={'module':'TNTTimer','status':'static-passed-awaiting-parent-grouped-compile','sharedFiles':len(plan.changed),'nativeFiles':len(patched),'metadata':meta,'localeDescriptions':counts,'manifest':str((out/'Cargo.toml').relative_to(ROOT)),'sharedPlan':{str(p.relative_to(ROOT)):hashlib.sha256(t.encode()).hexdigest()for p,t in plan.changed.items()},'noRustCargoGpuExecuted':True,'scope':'Complete ESP actual original TNT geometry outline, ShowTimer independent toggle, original CurveUtil scale/current center/owner styled Text/GameTimeUnit/visibility, reusable native current-frame stable priority queue, original screen projection/behind-camera direction/border/tag containment and original analytic concave arrow SDF including HDR HUD coverage. Correct hotbar HEAD submission, clear before HUD extraction, SDF draw at its actual HUD boundary before native menus/blur and GPU UI, retained buffers for generated frames.','limits':['Actual native text/entity/window providers are statically composed; fixture does not establish actual-game parity.','Owner measurement and drawing use shared native Font.width_text/HudBatch.text_styled; global NameProtect integration is handled by that shared service.','Decimal seconds uses one-decimal f64 formatting; default JVM number-locale decoration is not emulated.'],'focusedTests':6,'decimalFormatOracle':{'vectors':4101,'evidence':'build/tnt-timer-time-oracle-01','source':'GameTimeUnit.SECONDS DecimalFormat("0.#s").format(ticks*0.05)','execution':'Java reference executed; Rust comparison prepared but deferred to parent.'}};(out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
