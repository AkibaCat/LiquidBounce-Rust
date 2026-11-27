"""Prepare one focused real-font test; do not compile or execute Rust/Cargo."""
from pathlib import Path
import argparse,json,runpy,shutil,struct
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def face_name(path):
 data=path.read_bytes();tables=struct.unpack_from('>H',data,4)[0]
 for i in range(tables):
  tag,_,offset,_=struct.unpack_from('>4sIII',data,12+i*16)
  if tag!=b'name':continue
  _,count,strings=struct.unpack_from('>HHH',data,offset);names=[]
  for j in range(count):
   platform,encoding,language,ident,length,at=struct.unpack_from('>HHHHHH',data,offset+6+j*12)
   if ident!=4:continue
   raw=data[offset+strings+at:offset+strings+at+length]
   try:text=raw.decode('utf-16-be'if platform in [0,3]else'ascii')
   except UnicodeError:continue
   names.append(((language==0x409,platform==3),text))
  return max(names)[1]
 raise AssertionError('Font missing name table')
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out
 if out.exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True);plan=runpy.run_path(str(HERE/'integrate-theme-font.py'))['prepare']()
 patches={};wanted={'crates/horizon-client-render/Cargo.toml','crates/horizon-client-render/src/font.rs','crates/horizon-client-render/src/target-renderer-font.rs','crates/horizon-client-render/src/target-renderer-font-data.rs','crates/horizon-client-render/src/target-renderer-font-registry.rs'}
 def once(s,o,n):assert s.count(o)==1,(o,s.count(o));return s.replace(o,n)
 def edit(path,op):
  if path in wanted:
   p=ROOT/'upstream/Horizon'/path;patches[path]=op(patches.get(path,p.read_text(encoding='utf8')if p.exists()else''))
 runpy.run_path(str(HERE/'target-renderer-patch.py'))['apply'](edit,once)
 for path,text in patches.items():
  p=out/'patched'/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text,encoding='utf8')
 for key in ['liquidbounce_theme_generation:','let generation=target_font_generation();','self.reset_cache();']:assert key in patches['crates/horizon-client-render/src/font.rs']
 assert 'ttf-parser = "0.25.1"'in patches['crates/horizon-client-render/Cargo.toml']
 fonts=ROOT/'upstream/LiquidBounce/src-theme/public/fonts';names={p.name:face_name(p)for p in [fonts/'Inter-Medium.ttf',fonts/'Inter-Bold.ttf',fonts/'Inter-Light.ttf']};assert names['Inter-Medium.ttf']=='Inter Medium'and names['Inter-Light.ttf']=='Inter Light'
 body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];source=patches['crates/horizon-client-render/src/font.rs']
 harness='use ab_glyph::{Font as _,ScaleFont as _};use std::{cell::RefCell,collections::HashMap,path::{Path,PathBuf},sync::Arc};\n#[derive(Clone,Copy,Debug,Default)]pub struct Rect{pub u0:f32,pub v0:f32,pub u1:f32,pub v1:f32,pub w:u32,pub h:u32}\n'
 harness+='#[derive(Clone,Copy,Debug,Default)]'+body(source,'pub struct Glyph')+'\n#[derive(Clone,Debug)]'+body(source,'pub struct Patch')+'\n'
 harness+='type Provider=();\n'+body(source,'struct FontSet').replace('crate::fast_hash::FxHashMap','HashMap')+'\n'+body(source,'struct Region')+'\n'+body(source,'impl Region')+'\n'+body(source,'fn glyph_rect(')+'\n'
 harness+='struct Cache{sets:Vec<(String,FontSet)>,region:Region}\npub struct Font{cache:RefCell<Cache>,atlas:(u32,u32),region:(u32,u32),missing:Glyph,metrics_only:bool,generation:std::cell::Cell<u64>,liquidbounce_theme_generation:std::cell::Cell<u64>,uniform:bool,japanese:bool}\n'
 harness+='fn next_generation()->u64{static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(1);NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed)}\nimpl Default for Font{fn default()->Self{Self{cache:RefCell::new(Cache{sets:vec![],region:Region{cursor:(0,0),row:0,pending:vec![],full:false}}),atlas:(1024,1024),region:(0,1024),missing:Glyph::default(),metrics_only:false,generation:std::cell::Cell::new(next_generation()),liquidbounce_theme_generation:std::cell::Cell::new(0),uniform:false,japanese:false}}}\n'
 harness+='impl Font{'+''.join(body(source,anchor)+'\n'for anchor in ['pub fn reset_cache(','pub fn begin_frame(','pub fn epoch('])+'pub fn patch_count(&self)->usize{self.cache.borrow().region.pending.len()}pub fn first_patch_pixels(&self)->Vec<u8>{self.cache.borrow().region.pending[0].rgba.clone()}}\n'
 # Native reset_cache retains pending patches already ready for upload. The
 # harness emulates the renderer consuming them before the next frame.
 harness=harness.replace('pub fn begin_frame(&self) {','pub fn begin_frame(&self) {\n        self.cache.borrow_mut().region.pending.clear();',1)
 harness+=(HERE/'target-renderer-font.rs').read_text(encoding='utf8')
 (out/'font.rs').write_text(harness,encoding='utf8')
 for name in ['target-renderer-font-data.rs','target-renderer-font-registry.rs']:(out/name).write_text(patches['crates/horizon-client-render/src/'+name],encoding='utf8')
 shutil.copyfile(fonts/'Inter-Light.ttf',out/'fixture-light.ttf')
 lib='#![allow(dead_code,unused_imports)]\nextern crate self as horizon_client;pub mod font;pub mod skins{pub fn base64_decode(s:&str)->Option<Vec<u8>>{use base64::Engine;base64::engine::general_purpose::STANDARD.decode(s).ok()}}\n'
 lib+=f'#[path="{(HERE/"engine/src/theme_font_runtime.rs").as_posix()}"]mod theme_font_runtime;\n'+(HERE/'theme-font-smoke.rs').read_text(encoding='utf8')
 (out/'lib.rs').write_text(lib,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="liquidbounce-theme-font-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nab_glyph="0.2.32"\nttf-parser="0.25.1"\nsha2="0.10.9"\nserde_json="1"\nbase64="0.22"\n',encoding='utf8')
 report={'status':'static-passed-compile-and-focused-smoke-deferred','sharedFiles':len(plan.changed),'nativeFiles':len(patches),'realFullNames':names,'testManifest':str((out/'Cargo.toml').relative_to(ROOT)),'scope':'Real browser active theme root and Ctrl+R/metadata/font-file reload binding; native SFNT full-name matching; primary/fallback policy; cache invalidation at frame start. Rust/Cargo/GPU and game parity unexecuted. Full original theme marketplace/selection manager is outside this font bridge.'}
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report))
if __name__=='__main__':main()
