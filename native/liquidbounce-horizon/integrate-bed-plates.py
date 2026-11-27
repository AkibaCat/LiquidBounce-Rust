#!/usr/bin/env python3
"""Transactional complete BedPlates shared wiring; --check never writes."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
    p=Plan()
    p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod item_tags;','pub mod item_tags;\npub mod bed_plates;')
    p.edit('lib.rs','mod item_tags_runtime;','mod item_tags_runtime;\n#[cfg(feature="behavior-hooks")]\nmod bed_plates_runtime;')
    p.edit('behaviors.rs','    pub item_tags:liquidbounce_behaviors::item_tags::ItemTags,','    pub item_tags:liquidbounce_behaviors::item_tags::ItemTags,\n    pub bed_plates:liquidbounce_behaviors::bed_plates::BedPlates,')
    p.edit('behaviors.rs','            "ItemTags",','            "ItemTags",\n            "BedPlates",')
    p.edit('behaviors.rs','            item_tags:Default::default(),','            item_tags:Default::default(),\n            bed_plates:Default::default(),')
    a='            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("ItemTags")==Some(&id)=>self.item_tags.clear(),'
    p.edit('behaviors.rs',a,'            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("BedPlates")==Some(&id)=>self.bed_plates.disable(),\n'+a)
    p.edit('config.rs','pub use item_tags::{ItemTagsSetting,serialize_item_tags};','pub use item_tags::{ItemTagsSetting,serialize_item_tags};\n#[path="config/bed_plates.rs"]\nmod bed_plates;\npub use bed_plates::{BedPlatesSetting,serialize_bed_plates};')
    p.edit('config.rs','    ItemTags=>("ItemTags","Render","itemTags"),','    ItemTags=>("ItemTags","Render","itemTags"),\n    BedPlates=>("BedPlates","Render","bedPlates"),')
    p.edit('config.rs','    ItemTags(ItemTagsSetting),','    ItemTags(ItemTagsSetting),\n    BedPlates(BedPlatesSetting),',2)
    p.edit('config.rs','            | ModuleKind::ItemTags','            | ModuleKind::ItemTags\n            | ModuleKind::BedPlates')
    p.edit('config.rs','        ModuleKind::ItemTags=>item_tags::parse_item_tags(&mut patch,&root),','        ModuleKind::ItemTags=>item_tags::parse_item_tags(&mut patch,&root),\n        ModuleKind::BedPlates=>bed_plates::parse_bed_plates(&mut patch,&root),')
    p.edit('config.rs','        SettingChange::ItemTags(v)=>return ApplyResult::Runtime(RuntimeRequest::ItemTags(v.clone())),','        SettingChange::ItemTags(v)=>return ApplyResult::Runtime(RuntimeRequest::ItemTags(v.clone())),\n        SettingChange::BedPlates(v)=>return ApplyResult::Runtime(RuntimeRequest::BedPlates(v.clone())),')
    p.edit('actor.rs','            ModuleKind::ItemTags=>config::serialize_item_tags(&b.item_tags.settings,&common,format,&describe),','            ModuleKind::ItemTags=>config::serialize_item_tags(&b.item_tags.settings,&common,format,&describe),\n            ModuleKind::BedPlates=>config::serialize_bed_plates(&b.bed_plates.settings,&common,format,&describe),')
    a='                ApplyResult::Runtime(RuntimeRequest::ItemTags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().item_tags.settings,&self.state.item_registry){eprintln!("[LiquidBounce] ItemTags config: {e}");}},'
    p.edit('actor.rs',a,a+'\n                ApplyResult::Runtime(RuntimeRequest::BedPlates(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().bed_plates,&self.state.block_registry){eprintln!("[LiquidBounce] BedPlates config: {e}");}},')
    a='    #[cfg(feature="behavior-hooks")]\n    fn client_item_tags('
    p.edit('runtime.rs',a,'    #[cfg(feature="behavior-hooks")]\n    fn client_bed_plates(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBedPlatesEvent<\'_>){self.bed_plates(event);}\n'+a)
    p.edit('../../make-behavior-patch.py',"'item-tags-patch.py',","'item-tags-patch.py', 'bed-plates-patch.py',")
    return p
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args();p=prepare()
    if not args.check:p.commit()
    print(f'BedPlates {"validated"if args.check else"integrated"}: {len(p.changed)} shared files')
