// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_core::geometry::Vec3;
use horizon_protocol::packets::play::{self,Serverbound as S};
use serde_json::{Value,json};
impl Game<'_>{
 pub(super) fn elytra_target_query(&mut self,text:&str)->String{
  let Ok(q)=serde_json::from_str::<Value>(text)else{return "null".into();};
  if q["op"]=="safe"{
   let Some(player)=self.player.as_ref()else{return "false".into();};
   let bounds=player.bounds().move_by(player.velocity);let entities=self.player_entity_collisions();
   let mut motion=horizon_client::motion::MotionWorld{entities:&entities,player:self.player.as_mut().unwrap(),scene:&mut self.world,assets:&self.pack.assets,collision_binding:self.collision,resources:self.motion,health:&self.health_resources};
   let blocked=motion.no_player_collision(bounds).is_ok_and(|v|!v);
   if blocked{motion.player.velocity.y+=0.1;}
   return json!(blocked).to_string();
  }
  if q["op"]=="restartGlide"{
   if let Some(p)=self.player.as_mut(){if p.flying{return "false".into();}p.fall_flying=true;}
   if let Some(r)=self.remote.as_ref(){r.session.send(S::PlayerCommand(play::PlayerCommand{id:r.entity_id,action:6,data:0}));}
   return "true".into();
  }
  if q["op"]=="packetUse"{
   let slot=q["slot"].as_u64().unwrap_or(255)as usize;
   if !(slot<9||slot==40)||self.stack_in(slot).is_none_or(|s|s.count==0||s.item.as_ref()!="minecraft:firework_rocket"){return "false".into();}
   let previous=self.selected;let change=slot<9&&slot!=previous;
   if change{self.selected=slot;self.send(S::SetCarriedItem(play::SetCarriedItem{slot:slot as i16}));}
   if self.remote.is_some(){let sequence=self.next_sequence();self.send(S::UseItem(play::UseItem{hand:if slot==40{1}else{0},sequence,yaw:self.yaw,pitch:self.pitch}));}
   else{let _=self.native_client_use_item(if slot==40{1}else{0},self.yaw,self.pitch);}
   if change{self.selected=previous;self.send(S::SetCarriedItem(play::SetCarriedItem{slot:previous as i16}));}
   return "true".into();
  }
  if q["op"]!="facts"{return "null".into();}
  let Ok(mut facts)=serde_json::from_str::<Value>(&self.auto_rod_query("{\"op\":\"facts\"}"))else{return "null".into();};
  let Some(player)=self.player.as_ref()else{return "null".into();};
  let position=player.position;
  facts["position"]=json!([position.x,position.y,position.z]);facts["gliding"]=json!(player.fall_flying);facts["flying"]=json!(player.flying);
  facts["rockets"]=json!((0..9).chain(std::iter::once(40)).filter(|&slot|self.stack_in(slot).is_some_and(|s|s.count>0&&s.item.as_ref()=="minecraft:firework_rocket")).collect::<Vec<_>>());
  let entities=super::auto_bow::entities(self);
  if let Some(rows)=facts["candidates"].as_array_mut(){for row in rows{
   let Some(id)=row["id"].as_i64().map(|v|v as i32)else{continue;};let Some(e)=entities.iter().find(|e|e.id==id)else{continue;};
   let (eye,gliding)=if let Some(m)=self.mobs.iter().find(|m|m.id as i32==id){(Vec3::new(m.position.x,m.position.y+m.eye_height(),m.position.z),m.extra.fall_flying)}
    else if let Some(a)=self.armor_stands.iter().find(|a|a.id as i32==id){(Vec3::new(a.position.x,a.position.y+if a.marker{0.}else{a.probe_dimensions().2},a.position.z),false)}
    else if self.dragon.as_ref().is_some_and(|d|d.id as i32==id){(Vec3::new(e.position.x,e.position.y+6.8,e.position.z),false)}
    else if let Some(g)=self.lan.iter().flat_map(|l|l.hub.guests.iter()).find(|g|g.entity_id==id){(Vec3::new(e.position.x,e.position.y+g.motion.as_ref().map_or(1.62,|p|f64::from(p.dimensions.eye_height)),e.position.z),g.fall_flying)}
    else{continue;};
   row["position"]=json!([e.position.x,e.position.y,e.position.z]);row["velocity"]=json!([e.velocity.x,e.velocity.y,e.velocity.z]);row["eyes"]=json!([eye.x,eye.y,eye.z]);row["height"]=json!(e.bounds.max_position().y-e.bounds.min_position().y);row["gliding"]=json!(gliding);
  }}
  facts.to_string()
 }
}
