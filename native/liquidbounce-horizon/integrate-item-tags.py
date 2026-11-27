#!/usr/bin/env python3
"""Transactional complete ItemTags shared wiring; --check never writes."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']

def prepare():
    p=Plan()
    p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod radar;','pub mod radar;\npub mod item_tags;')
    p.edit('lib.rs','mod radar_runtime;','mod radar_runtime;\n#[cfg(feature="behavior-hooks")]\nmod item_tags_runtime;')
    p.edit('behaviors.rs','    pub radar:liquidbounce_behaviors::radar::Radar,','    pub radar:liquidbounce_behaviors::radar::Radar,\n    pub item_tags:liquidbounce_behaviors::item_tags::ItemTags,')
    p.edit('behaviors.rs','            "Radar",','            "Radar",\n            "ItemTags",')
    p.edit('behaviors.rs','            radar:Default::default(),','            radar:Default::default(),\n            item_tags:Default::default(),')
    p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoChatGame")==Some(&id)=>self.auto_chat_game.cancel(),','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("ItemTags")==Some(&id)=>self.item_tags.clear(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoChatGame")==Some(&id)=>self.auto_chat_game.cancel(),')
    p.edit('config.rs','pub use radar::{RadarSetting,serialize_radar};','pub use radar::{RadarSetting,serialize_radar};\n#[path="config/item_tags.rs"]\nmod item_tags;\npub use item_tags::{ItemTagsSetting,serialize_item_tags};')
    p.edit('config.rs','    Radar=>("Radar","Render","radar"),','    Radar=>("Radar","Render","radar"),\n    ItemTags=>("ItemTags","Render","itemTags"),')
    p.edit('config.rs','    Radar(RadarSetting),','    Radar(RadarSetting),\n    ItemTags(ItemTagsSetting),',2)
    p.edit('config.rs','            | ModuleKind::Radar','            | ModuleKind::Radar\n            | ModuleKind::ItemTags')
    p.edit('config.rs','        ModuleKind::Radar=>radar::parse_radar(&mut patch,&root),','        ModuleKind::Radar=>radar::parse_radar(&mut patch,&root),\n        ModuleKind::ItemTags=>item_tags::parse_item_tags(&mut patch,&root),')
    p.edit('config.rs','        SettingChange::Radar(value)=>return ApplyResult::Runtime(RuntimeRequest::Radar(value.clone())),','        SettingChange::Radar(value)=>return ApplyResult::Runtime(RuntimeRequest::Radar(value.clone())),\n        SettingChange::ItemTags(v)=>return ApplyResult::Runtime(RuntimeRequest::ItemTags(v.clone())),')
    p.edit('actor.rs','            ModuleKind::Radar=>config::serialize_radar(&b.radar,&common,format,&describe),','            ModuleKind::Radar=>config::serialize_radar(&b.radar,&common,format,&describe),\n            ModuleKind::ItemTags=>config::serialize_item_tags(&b.item_tags.settings,&common,format,&describe),')
    a='                ApplyResult::Runtime(RuntimeRequest::Radar(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().radar){eprintln!("[LiquidBounce] Radar config: {e}");}},'
    p.edit('actor.rs',a,a+'\n                ApplyResult::Runtime(RuntimeRequest::ItemTags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().item_tags.settings,&self.state.item_registry){eprintln!("[LiquidBounce] ItemTags config: {e}");}},')
    a='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
    p.edit('runtime.rs',a,'    #[cfg(feature="behavior-hooks")]\n    fn client_item_tags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientItemTagsEvent<\'_>){self.item_tags(event);}\n'+a)
    p.edit('runtime.rs','        self.state().behaviors.borrow_mut().radar.clear();','        self.state().behaviors.borrow_mut().radar.clear();\n        self.state().behaviors.borrow_mut().item_tags.clear();')
    p.edit('../../make-behavior-patch.py',"'chest-stealer-patch.py',","'chest-stealer-patch.py', 'item-tags-patch.py',")
    return p

if __name__=='__main__':
    a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
    if not args.check:p.commit()
    print(f'ItemTags {"validated"if args.check else"integrated"}: {len(p.changed)} shared files')
