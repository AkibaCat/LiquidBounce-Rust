#!/usr/bin/env python3
"""Transactional shared Nametags wiring; --check validates anchors without writing."""
from pathlib import Path
import argparse, runpy

HERE = Path(__file__).resolve().parent
Plan = runpy.run_path(str(HERE / "integrate-chest-stealer.py"))["Plan"]


def prepare():
    p = Plan()
    p.edit("../../../liquidbounce-behaviors/src/lib.rs", "pub mod item_tags;", "pub mod item_tags;\npub mod nametags;")
    p.edit("lib.rs", "mod item_tags_runtime;", "mod item_tags_runtime;\n#[cfg(feature=\"behavior-hooks\")]\nmod nametags_runtime;")
    p.edit("behaviors.rs", "    pub item_tags:liquidbounce_behaviors::item_tags::ItemTags,", "    pub item_tags:liquidbounce_behaviors::item_tags::ItemTags,\n    pub nametags:liquidbounce_behaviors::nametags::Nametags,")
    p.edit("behaviors.rs", '            "ItemTags",', '            "ItemTags",\n            "Nametags",')
    p.edit("behaviors.rs", "            item_tags:Default::default(),", "            item_tags:Default::default(),\n            nametags:Default::default(),")
    p.edit("behaviors.rs", '            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("BedPlates")==Some(&id)=>self.bed_plates.disable(),', '            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Nametags")==Some(&id)=>self.nametags.clear(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("BedPlates")==Some(&id)=>self.bed_plates.disable(),')

    p.edit("config.rs", "pub use item_tags::{ItemTagsSetting,serialize_item_tags};", "pub use item_tags::{ItemTagsSetting,serialize_item_tags};\n#[path=\"config/nametags.rs\"]\nmod nametags;\npub use nametags::{NametagsSetting,serialize_nametags};")
    p.edit("config.rs", '    ItemTags=>("ItemTags","Render","itemTags"),', '    ItemTags=>("ItemTags","Render","itemTags"),\n    Nametags=>("Nametags","Render","nametags"),')
    p.edit("config.rs", "    ItemTags(ItemTagsSetting),", "    ItemTags(ItemTagsSetting),\n    Nametags(NametagsSetting),", 2)
    p.edit("config.rs", "            | ModuleKind::ItemTags\n            | ModuleKind::BedPlates", "            | ModuleKind::ItemTags\n            | ModuleKind::Nametags\n            | ModuleKind::BedPlates")
    p.edit("config.rs", "        ModuleKind::ItemTags=>item_tags::parse_item_tags(&mut patch,&root),", "        ModuleKind::ItemTags=>item_tags::parse_item_tags(&mut patch,&root),\n        ModuleKind::Nametags=>nametags::parse_nametags(&mut patch,&root),")
    p.edit("config.rs", "        SettingChange::ItemTags(v)=>return ApplyResult::Runtime(RuntimeRequest::ItemTags(v.clone())),", "        SettingChange::ItemTags(v)=>return ApplyResult::Runtime(RuntimeRequest::ItemTags(v.clone())),\n        SettingChange::Nametags(v)=>return ApplyResult::Runtime(RuntimeRequest::Nametags(v.clone())),")

    p.edit("actor.rs", "            ModuleKind::ItemTags=>config::serialize_item_tags(&b.item_tags.settings,&common,format,&describe),", "            ModuleKind::ItemTags=>config::serialize_item_tags(&b.item_tags.settings,&common,format,&describe),\n            ModuleKind::Nametags=>config::serialize_nametags(&b.nametags.settings,&common,format,&describe),")
    p.edit("actor.rs", '                ApplyResult::Runtime(RuntimeRequest::ItemTags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().item_tags.settings,&self.state.item_registry){eprintln!("[LiquidBounce] ItemTags config: {e}");}},', '                ApplyResult::Runtime(RuntimeRequest::ItemTags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().item_tags.settings,&self.state.item_registry){eprintln!("[LiquidBounce] ItemTags config: {e}");}},\n                ApplyResult::Runtime(RuntimeRequest::Nametags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().nametags.settings,&[]){eprintln!("[LiquidBounce] Nametags config: {e}");}},')

    p.edit("runtime.rs", '    #[cfg(feature="behavior-hooks")]\n    fn client_item_tags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientItemTagsEvent<\'_>){self.item_tags(event);}', '    #[cfg(feature="behavior-hooks")]\n    fn client_nametags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientNametagsEvent<\'_>){self.nametags(event);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_item_tags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientItemTagsEvent<\'_>){self.item_tags(event);}')
    p.edit("../../make-behavior-patch.py", "'bed-plates-patch.py',", "'bed-plates-patch.py', 'nametags-patch.py',")
    return p


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    plan = prepare()
    if not args.check:
        plan.commit()
    print(f"Nametags {'validated' if args.check else 'integrated'}: {len(plan.changed)} shared files")

