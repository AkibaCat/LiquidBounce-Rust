"""Prepare the focused production SwordBlock helper/service smoke without compiling."""
from pathlib import Path
import argparse,json,subprocess,ast
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def prepare(out):
 out.mkdir(parents=True,exist_ok=True)
 model=HERE.parent/'liquidbounce-behaviors/src/sword_block.rs'
 source='''#![allow(dead_code,unused_imports)]
extern crate self as horizon_client;extern crate self as liquidbounce_behaviors;
pub mod items { #[derive(Clone)]pub struct ItemStack{pub item:String} }
mod shield{pub const OFFHAND_SLOT:usize=40;}
struct Game{inventory:Vec<Option<items::ItemStack>>,selected:usize,offhand:Option<items::ItemStack>,using:Option<(usize,u32)>,right:bool,flags:u32,aura:bool,owner:bool}
impl Game{fn native_window(&self)->bool{true}fn native_client_slot_owner(&self)->bool{self.owner}fn stack_in(&self,slot:usize)->Option<&items::ItemStack>{if slot==40{self.offhand.as_ref()}else{self.inventory.get(slot)?.as_ref()}}fn client_kill_aura_sword(&self,s:&items::ItemStack)->bool{s.item=="minecraft:iron_sword"}fn client_kill_aura_block_pose(&self)->bool{self.aura}}
mod mod_api{pub fn mods(){}pub fn dispatch_client_sword_block_state(g:&crate::Game,_:(),flags:&mut u32){*flags=g.flags;}}
fn item(name:&str)->items::ItemStack{items::ItemStack{item:name.into()}}
fn game()->Game{Game{inventory:vec![Some(item("minecraft:iron_sword"))],selected:0,offhand:Some(item("minecraft:shield")),using:None,right:false,flags:9,aura:false,owner:true}}
'''
 source+=f'#[path="{model.as_posix()}"]pub mod sword_block;\n#[path="{(HERE/"engine/src/sword_block_runtime.rs").as_posix()}"]mod service;\n'
 source+='mod native{use crate::mod_api;impl super::Game{'+(HERE/'sword-block-host.rs').read_text(encoding='utf8')+'}}\n'
 source+=(HERE/'sword-block-smoke.rs').read_text(encoding='utf8')
 (out/'smoke.rs').write_text(source,encoding='utf8')
 for file in ['sword-block-patch.py','integrate-sword-block.py','update-sword-block-locales.py']:ast.parse((HERE/file).read_text(encoding='utf8'))
 (out/'preparation.json').write_text(json.dumps({'status':'prepared-no-compiler-invoked','scope':'Actual native methods, shared settings model and atomic service. Render consumers and File/Interop tree checked with the grouped official batch.'},indent=2)+'\n',encoding='utf8')
 return out/'smoke.rs'
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--run',action='store_true');args=p.parse_args();out=ROOT/args.out;source=prepare(out)
 if args.run:
  command=['rustc','+stable','--edition=2024','--test',str(source),'-o',str(out/'smoke.exe')]
  compile=subprocess.run(command,capture_output=True,text=True,encoding='utf8',errors='replace');log=compile.stdout+compile.stderr;result=compile
  if compile.returncode==0:
   result=subprocess.run([str(out/'smoke.exe'),'--test-threads=1'],capture_output=True,text=True,encoding='utf8',errors='replace');log+=result.stdout+result.stderr
  (out/'test.log').write_text(log,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':result.returncode==0,'scope':'Local sword/shield/key guards and setting service only; no GPU/full-suite/game.'},indent=2)+'\n',encoding='utf8');print(log);raise SystemExit(result.returncode)
 print('SwordBlock smoke prepared; no compiler invoked')
