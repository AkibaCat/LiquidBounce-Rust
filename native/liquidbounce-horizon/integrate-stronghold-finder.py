"""Validate then publish the complete StrongholdFinder engine transaction."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod new_chunks;','pub mod new_chunks;\npub mod stronghold_finder;')
 p.edit('lib.rs','mod new_chunks_runtime;','mod new_chunks_runtime;\n#[cfg(feature="behavior-hooks")]\nmod stronghold_finder_runtime;')
 p.edit('behaviors.rs','    pub new_chunks: liquidbounce_behaviors::new_chunks::NewChunks,','    pub new_chunks: liquidbounce_behaviors::new_chunks::NewChunks,\n    pub stronghold_finder:liquidbounce_behaviors::stronghold_finder::StrongholdFinder,')
 p.edit('behaviors.rs','            "NewChunks",','            "NewChunks",\n            "StrongholdFinder",')
 p.edit('behaviors.rs','            new_chunks: Default::default(),','            new_chunks: Default::default(),\n            stronghold_finder:Default::default(),')
 a='            ModuleCallback::OnDisabled(id) if self.query_ids.get("NewChunks") == Some(&id) => { self.new_chunks.clear(); }'
 p.edit('behaviors.rs',a,a+'\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("StrongholdFinder")==Some(&id)=>self.stronghold_finder.clear(),')
 p.edit('config.rs','pub use new_chunks::{NewChunksSetting,serialize_new_chunks};','pub use new_chunks::{NewChunksSetting,serialize_new_chunks};\n#[path="config/stronghold_finder.rs"]\nmod stronghold_finder;\npub use stronghold_finder::{StrongholdFinderSetting,serialize_stronghold_finder};')
 p.edit('config.rs','    NewChunks => ("NewChunks", "Render", "newChunks"),','    NewChunks => ("NewChunks", "Render", "newChunks"),\n    StrongholdFinder=>("StrongholdFinder","World","strongholdFinder"),')
 p.edit('config.rs','"InventoryManager"=>Some(Self::InventoryCleaner),','"Triangulation"=>Some(Self::StrongholdFinder), "InventoryManager"=>Some(Self::InventoryCleaner),')
 p.edit('config.rs','    NewChunks(NewChunksSetting),','    NewChunks(NewChunksSetting),\n    StrongholdFinder(StrongholdFinderSetting),',2)
 p.edit('config.rs','            | ModuleKind::NewChunks','            | ModuleKind::NewChunks\n            | ModuleKind::StrongholdFinder')
 p.edit('config.rs','        ModuleKind::NewChunks => new_chunks::parse_new_chunks(&mut patch,&root),','        ModuleKind::NewChunks => new_chunks::parse_new_chunks(&mut patch,&root),\n        ModuleKind::StrongholdFinder=>stronghold_finder::parse_stronghold_finder(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::NewChunks(value) => { return ApplyResult::Runtime(RuntimeRequest::NewChunks(*value)); }','        SettingChange::NewChunks(value) => { return ApplyResult::Runtime(RuntimeRequest::NewChunks(*value)); }\n        SettingChange::StrongholdFinder(value)=>return ApplyResult::Runtime(RuntimeRequest::StrongholdFinder(value.clone())),')
 p.edit('actor.rs','            ModuleKind::NewChunks => config::serialize_new_chunks(&b.new_chunks,&common,format,&describe),','            ModuleKind::NewChunks => config::serialize_new_chunks(&b.new_chunks,&common,format,&describe),\n            ModuleKind::StrongholdFinder=>config::serialize_stronghold_finder(&b.stronghold_finder,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::NewChunks(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().new_chunks); }','                ApplyResult::Runtime(RuntimeRequest::NewChunks(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().new_chunks); }\n                ApplyResult::Runtime(RuntimeRequest::StrongholdFinder(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().stronghold_finder){eprintln!("[LiquidBounce] StrongholdFinder config: {e}");}},')
 p.edit('runtime.rs','        self.state().behaviors.borrow_mut().new_chunks.clear();','        self.state().behaviors.borrow_mut().new_chunks.clear();\n        self.state().behaviors.borrow_mut().stronghold_finder.world_change();')
 a='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.edit('runtime.rs',a,'    #[cfg(feature="behavior-hooks")]\n    fn client_stronghold_finder(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientStrongholdFinderEvent<\'_>){self.stronghold_finder(event);}\n'+a)
 # Needs final SmartEat/AirPlace/EasyPearl wrappers and the section cache seam.
 p.edit('../../make-behavior-patch.py',"'block-placement-patch.py', 'step-patch.py'","'block-placement-patch.py', 'stronghold-finder-patch.py', 'step-patch.py'")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('StrongholdFinder '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
