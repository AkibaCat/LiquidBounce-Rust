#!/usr/bin/env python3
"""Transactional ModuleDebug wiring; --check validates without writing."""
from pathlib import Path
import argparse, runpy
HERE = Path(__file__).resolve().parent
Plan = runpy.run_path(str(HERE / "integrate-chest-stealer.py"))["Plan"]

def prepare():
    p = Plan()
    p.edit("../../../liquidbounce-behaviors/src/lib.rs", "pub mod nametags;", "pub mod nametags;\npub mod debug;")
    p.edit("lib.rs", "mod nametags_runtime;", "mod nametags_runtime;\n#[cfg(feature=\"behavior-hooks\")]\nmod debug_runtime;")
    p.edit("behaviors.rs", "    pub nametags:liquidbounce_behaviors::nametags::Nametags,", "    pub nametags:liquidbounce_behaviors::nametags::Nametags,\n    pub debug:crate::debug_runtime::DebugRuntime,")
    p.edit("behaviors.rs", '            "Nametags",', '            "Nametags",\n            "Debug",')
    p.edit("behaviors.rs", "            nametags:Default::default(),", "            nametags:Default::default(),\n            debug:Default::default(),")
    p.edit("behaviors.rs", '            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Nametags")==Some(&id)=>self.nametags.clear(),', '            ModuleCallback::OnEnabled(id) if self.query_ids.get("Debug")==Some(&id)=>self.debug.enabled(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Debug")==Some(&id)=>self.debug.clear(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Nametags")==Some(&id)=>self.nametags.clear(),')
    p.edit("config.rs", "pub use nametags::{NametagsSetting,serialize_nametags};", "pub use nametags::{NametagsSetting,serialize_nametags};\n#[path=\"config/debug.rs\"]\nmod debug;\npub use debug::{DebugSetting,serialize_debug};")
    p.edit("config.rs", '    Nametags=>("Nametags","Render","nametags"),', '    Nametags=>("Nametags","Render","nametags"),\n    Debug=>("Debug","Render","debug"),')
    p.edit("config.rs", "    Nametags(NametagsSetting),", "    Nametags(NametagsSetting),\n    Debug(DebugSetting),", 2)
    p.edit("config.rs", "            | ModuleKind::Nametags\n            | ModuleKind::BedPlates", "            | ModuleKind::Nametags\n            | ModuleKind::Debug\n            | ModuleKind::BedPlates")
    p.edit("config.rs", "        ModuleKind::Nametags=>nametags::parse_nametags(&mut patch,&root),", "        ModuleKind::Nametags=>nametags::parse_nametags(&mut patch,&root),\n        ModuleKind::Debug=>debug::parse_debug(&mut patch,&root),")
    p.edit("config.rs", "        SettingChange::Nametags(v)=>return ApplyResult::Runtime(RuntimeRequest::Nametags(v.clone())),", "        SettingChange::Nametags(v)=>return ApplyResult::Runtime(RuntimeRequest::Nametags(v.clone())),\n        SettingChange::Debug(v)=>return ApplyResult::Runtime(RuntimeRequest::Debug(v.clone())),")
    p.edit("actor.rs", "            ModuleKind::Nametags=>config::serialize_nametags(&b.nametags.settings,&common,format,&describe),", "            ModuleKind::Nametags=>config::serialize_nametags(&b.nametags.settings,&common,format,&describe),\n            ModuleKind::Debug=>config::serialize_debug(&b.debug.model.settings,&common,format,&describe),")
    p.edit("actor.rs", '                ApplyResult::Runtime(RuntimeRequest::Nametags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().nametags.settings,&[]){eprintln!("[LiquidBounce] Nametags config: {e}");}},', '                ApplyResult::Runtime(RuntimeRequest::Nametags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().nametags.settings,&[]){eprintln!("[LiquidBounce] Nametags config: {e}");}},\n                ApplyResult::Runtime(RuntimeRequest::Debug(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().debug.model){eprintln!("[LiquidBounce] Debug config: {e}");}},')
    p.edit("runtime.rs", '    #[cfg(feature="behavior-hooks")]\n    fn client_nametags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientNametagsEvent<\'_>){self.nametags(event);}', '    #[cfg(feature="behavior-hooks")]\n    fn client_debug(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientDebugEvent<\'_>){self.debug(event);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_nametags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientNametagsEvent<\'_>){self.nametags(event);}')
    p.edit("../../make-behavior-patch.py", "'step-patch.py'):", "'debug-patch.py', 'step-patch.py'):")
    return p
if __name__ == "__main__":
    ap=argparse.ArgumentParser();ap.add_argument("--check",action="store_true");args=ap.parse_args();p=prepare()
    if not args.check:p.commit()
    print(f"Debug {'validated' if args.check else 'integrated'}: {len(p.changed)} shared files")
