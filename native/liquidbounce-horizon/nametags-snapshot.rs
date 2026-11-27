// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game,ItemStack};
use horizon_client::text::Text;
use horizon_world::player_inventory::EquipmentSlot as S;
use serde_json::{json,Value};
use std::collections::HashMap;
#[derive(Clone)]pub(super) struct Source{pub stacks:[Option<ItemStack>;8],pub position:[f64;3],pub eye:f64}
#[derive(Clone)]pub(super) struct UseRef{pub hand:usize,pub handle:u64,pub ticks:u32}
pub(super) fn empty(stack:&ItemStack)->bool{stack.count==0||stack.count as i32<=0||&*stack.item=="minecraft:air"}
impl Game<'_>{
    fn client_nametags_enchantments(&self,stack:&ItemStack)->Value{
        let registry=self.enchantments();json!(stack.crafting_enchantments().iter().filter_map(|(id,level)|{let e=registry.get(id)?;let name=Text::translate_with_fallback(&e.description_key,&e.description,&[]).plain();Some(json!({"id":id.as_ref(),"level":*level as i32,"name":name,"curse":registry.in_tag("minecraft:curse",id)}))}).collect::<Vec<_>>())
    }
    pub(super) fn client_nametags_snapshot(&self)->(Vec<Value>,HashMap<u64,Source>,HashMap<u64,UseRef>){
        let mut sources=HashMap::new();let mut uses=HashMap::new();let mut entries=Vec::new();let previous=self.client_nametags.borrow().uses.clone();let local=self.player.as_ref().map(|p|[p.position.x,p.position.y,p.position.z]).unwrap_or([0.;3]);
        let slots=[S::MainHand,S::OffHand,S::Feet,S::Legs,S::Chest,S::Head,S::Body,S::Saddle];
        for mut e in self.client_tracers_entities(1.){
            let Some(identity)=e["identity"].as_u64()else{continue;};let id=e["id"].as_i64().unwrap_or(0)as i32;let player=e["kind"]=="minecraft:player";let local_entity=identity==1;
            let mut stacks:[Option<ItemStack>;8]=Default::default();let(mut eye,mut absorption,mut crouching)=(0.,0.,false);let mut using=None;let mut holder=None;
            let mut name=Text::translate(&format!("entity.{}",e["kind"].as_str().unwrap_or("").replace(':',".")),&[]);
            let info=player.then(||self.client_anti_bot_player(id)).flatten();
            if let Some(p)=info.as_ref(){name=Text::literal(p["name"].as_str().unwrap_or(""));holder=p["name"].as_str().map(str::to_owned);e["uuid"]=p["uuid"].clone();e["ping"]=p["ping"].clone();e["mode"]=p["mode"].clone();}
            if local_entity{if let Some(p)=self.player.as_ref(){eye=f64::from(p.dimensions.eye_height);absorption=p.health.absorption();crouching=p.local_crouching;stacks=[self.stack_in(self.selected).cloned(),self.offhand.clone(),self.armor[3].clone(),self.armor[2].clone(),self.armor[1].clone(),self.armor[0].clone(),None,None];using=self.using.map(|(slot,ticks)|(usize::from(slot==crate::game::shield::OFFHAND_SLOT),ticks));}}
            else if identity&7==0{if let Some(m)=self.mobs.iter().find(|m|m.join.get()<<3==identity){
                eye=m.eye_height();absorption=m.absorption;crouching=m.extra.crouching;
                stacks=slots.map(|s|{let mut stack=horizon_client::mobs::finalize::equipment_stack(m,s);if s==S::Body{if let Some(body)=stack.as_mut(){body.extra=m.extra.body_extra.clone();}}stack});
                if !player{name=m.custom_name_text.clone().or_else(||m.custom_name.as_ref().map(Text::literal)).unwrap_or_else(||m.command_name());holder=self.persist.uuids.get(&m.id).map(|u|horizon_protocol::crypto::uuid_string(horizon_client::lan::playerdata::ints_uuid(*u)));}
                if m.extra.using_item>0{using=Some((usize::from(m.extra.using_off_hand),m.extra.using_item as u32));}
            }}else if identity&7==2{if let Some(a)=self.armor_stands.iter().find(|a|(a.join.get()<<3)|2==identity){eye=if a.marker{0.}else{a.probe_dimensions().2};for(i,s)in a.equipment.iter().enumerate(){stacks[i]=s.clone();}name=a.custom_name.clone().unwrap_or_else(||Text::translate("entity.minecraft.armor_stand",&[]));holder=self.persist.uuids.get(&a.id).map(|u|horizon_protocol::crypto::uuid_string(horizon_client::lan::playerdata::ints_uuid(*u)));}}
            else if identity==3{eye=6.8;if let Some(d)=self.dragon.as_ref(){holder=self.persist.uuids.get(&d.id).map(|u|horizon_protocol::crypto::uuid_string(horizon_client::lan::playerdata::ints_uuid(*u)));}}
            else if identity&7==4{if let Some(h)=self.lan.as_ref(){if let Some(g)=h.hub.guests.iter().find(|g|(u64::from(g.conn)<<3)|4==identity){eye=g.eye_height();if let Some(p)=g.motion.as_ref(){absorption=p.health.absorption();crouching=p.local_crouching;}stacks=[g.inventory.get(g.selected).cloned().flatten(),g.offhand.clone(),g.armor[3].clone(),g.armor[2].clone(),g.armor[1].clone(),g.armor[0].clone(),None,None];using=h.play.get(&g.conn).and_then(|p|p.using()).map(|(slot,ticks)|(usize::from(slot==crate::game::shield::OFFHAND_SLOT),ticks));name=Text::literal(&g.name);holder=Some(g.name.clone());}}}
            if let Some(team)=holder.as_deref().and_then(|h|self.score.board.players_team(h)){name=team.formatted_name(name);}
            e["name"]=name.resolve_legacy().component();e["local"]=json!(local_entity);e["player"]=json!(player);e["absorption"]=json!(absorption);e["crouching"]=json!(crouching);
            // getActualHealth's scoreboard also gates absorption when no score is present.
            e["scoreboard"]=json!(self.score.board.display_objective("below_name").is_some_and(|o|["❤","HP","Health","Здоровья","Здоровье"].iter().any(|s|o.display_text().plain().contains(s))));
            let pos=serde_json::from_value::<[f64;3]>(e["position"].clone()).unwrap_or([0.;3]);let delta=std::array::from_fn::<_,3,_>(|i|(pos[i]-local[i])as f32);e["playerDistance"]=json!((delta.iter().map(|v|v*v).sum::<f32>()).sqrt());
            for s in &mut stacks{if s.as_ref().is_some_and(empty){*s=None;}}
            let used=using.and_then(|(hand,ticks)|{let stack=stacks[hand].as_ref()?;let handle=stack.object_identity();let old=previous.get(&identity).filter(|u|u.hand==hand&&ticks>=u.ticks);let value=UseRef{hand,handle:old.map_or(handle,|u|u.handle),ticks};let result=value.handle;uses.insert(identity,value);Some(result)});
            e["using"]=json!(used);e["enchantments"]=json!(stacks.iter().map(|s|s.as_ref().map_or(json!([]),|s|self.client_nametags_enchantments(s))).collect::<Vec<_>>());
            sources.insert(identity,Source{stacks,position:pos,eye});entries.push(e);
        }
        (entries,sources,uses)
    }
}
