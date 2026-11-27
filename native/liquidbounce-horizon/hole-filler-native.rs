// SPDX-License-Identifier: GPL-3.0-or-later
//! HoleFiller reads the shared resident-world HoleManager and real BlockItem stacks.
use super::Game;
use horizon_world::level::data::ItemKind;
use serde_json::{Value,json};
impl Game<'_>{
    pub(super) fn hole_filler_query(&self,text:&str)->Option<String>{
        let q:Value=serde_json::from_str(text).ok()?;
        match q["op"].as_str()?{
            "slots"=>{let b=self.res.behaviours();let slots=std::iter::once(40).chain(0..9).filter_map(|index|{let stack=self.stack_in(index).filter(|s|s.count>0)?;let binding=b.item_binding(&stack.item).filter(|v|!matches!(v.kind,ItemKind::SolidBucket|ItemKind::Other))?;Some(json!({"index":index,"count":stack.count as i32,"block":b.table.name(b.table.default_state(binding.block))}))}).collect::<Vec<_>>();Some(json!({"slots":slots,"selected":self.client_hotbar_slot()}).to_string())},
            "snapshot"=>{
                self.client_hole_esp_tick();
                let player=self.player.as_ref()?;let cache=self.client_hole_esp_cache.borrow();
                let holes=cache.manager.tracker.holes.values().map(|h|json!({"position":h.position,"size":h.size,"kind":h.kind})).collect::<Vec<_>>();drop(cache);
                let entities=super::auto_bow::entities(self).into_iter().map(|e|json!({"id":e.id,"position":[e.position.x,e.position.y,e.position.z],"previous":[e.previous.x,e.previous.y,e.previous.z]})).collect::<Vec<_>>();
                Some(json!({"player":[player.position.x,player.position.y,player.position.z],"creative":self.creative,"holes":holes,"entities":entities}).to_string())
            },_=>None
        }
    }
}
