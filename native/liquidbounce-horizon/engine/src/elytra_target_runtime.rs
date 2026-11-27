// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{auto_bow::Candidate,auto_bow_math::V,elytra_target::ElytraTarget,rotations::{DefaultRotationTarget,MovementCorrection},target_renderer::{TargetRendererRuntime,TargetRendererSettings}};
use serde_json::{Value,json};
pub const ROTATION_PROVIDER:u64=31;
pub const PROCESSOR:u64=6000;
pub const REQUESTER:u64=0x454c_5954_5241_5447;
pub struct ElytraTargetRuntime{pub behavior:ElytraTarget,pub target_renderer:TargetRendererSettings,pub target_render_runtime:TargetRendererRuntime,pub previous_raw:bool}
impl Default for ElytraTargetRuntime{fn default()->Self{Self{behavior:ElytraTarget::new(crate::behaviors::current_time_millis()),target_renderer:Default::default(),target_render_runtime:Default::default(),previous_raw:false}}}
fn query(h:&mut dyn Host,v:Value)->Value{serde_json::from_str(&h.client_elytra_target_query(&v.to_string())).unwrap_or(Value::Null)}
fn vector(v:&Value)->Option<V>{let a=serde_json::from_value::<[f64;3]>(v.clone()).ok()?;Some(V(a[0],a[1],a[2]))}
impl LiquidBounceHooks{
 pub(crate) fn elytra_target_prepare(&self,host:&mut dyn Host){
  let (active,id)={let s=self.state();let b=s.behaviors.borrow();(b.query_running(&s.registry,"ElytraTarget"),b.elytra_target.behavior.target)};
  if !active{return;}let Some(id)=id else{return;};let facts=query(host,json!({"op":"facts"}));
  if facts["gliding"]!=true{return;}let Some(player)=vector(&facts["position"])else{return;};
  let Some(e)=facts["candidates"].as_array().and_then(|a|a.iter().find(|e|e["id"].as_i64()==Some(i64::from(id))))else{return;};
  let (Some(pos),Some(eye),Some(velocity))=(vector(&e["position"]),vector(&e["eyes"]),vector(&e["velocity"]))else{return;};
  let s=self.state();let b=s.behaviors.borrow();let r=&b.elytra_target.behavior;
  let point=r.point(player,pos,eye,e["height"].as_f64().unwrap_or(0.),velocity,e["gliding"]==true,crate::behaviors::current_time_millis(),&mut||f64::from(liquidbounce_behaviors::random::random_unit_f32()));
  b.rotations.lock().expect("RotationManager").request_processed(ROTATION_PROVIDER,60,DefaultRotationTarget{rotation:r.rotation(player,point),horizontal_turn_speed:180.,vertical_turn_speed:180.,ticks_until_reset:1,reset_threshold:1.,consider_inventory:true,movement_correction:if r.settings.look{MovementCorrection::ChangeLook}else{MovementCorrection::Strict}},PROCESSOR,true);
 }
 pub(crate) fn elytra_target_tick(&self,host:&mut dyn Host){
  let facts=query(host,json!({"op":"facts"}));let raw=facts["gliding"]==true;
  let restart={let s=self.state();let mut b=s.behaviors.borrow_mut();let enabled=b.query_ids.get("ElytraTarget").is_some_and(|id|s.registry.running(*id));let r=&mut b.elytra_target;
   let restart=r.previous_raw&&!raw&&enabled&&r.behavior.settings.always_glide&&r.behavior.target.is_some()&&facts["flying"]!=true;
   r.previous_raw=raw;r.behavior.gliding=raw||restart;if !enabled||!r.behavior.gliding{r.behavior.target=None;}restart};
  if restart{query(host,json!({"op":"restartGlide"}));}
  let active={let s=self.state();let b=s.behaviors.borrow();b.query_running(&s.registry,"ElytraTarget")};if !active{return;}
  let mut candidates=vec![];let mut visible=vec![];
  for e in facts["candidates"].as_array().into_iter().flatten(){
   let Some(id)=e["id"].as_i64().map(|v|v as i32)else{continue;};
   let Some(target)=crate::targeting::snapshot(&e["target"].to_string())else{continue;};
   let valid={let mut s=self.state();let info=s.target_info(id,&target);target.entity.kind.living&&s.global_targets.combat.should_be_attacked(&target.entity,info)};if !valid{continue;}
   candidates.push(Candidate{id,type_weight:e["type"].as_i64().unwrap_or(i64::from(i32::MAX))as i32,health:e["health"].as_f64().unwrap_or(0.)as f32,distance:e["distance"].as_f64().unwrap_or(f64::INFINITY),cosine:e["cosine"].as_f64().unwrap_or(-1.),hurt:e["hurt"].as_i64().unwrap_or(0)as i32,age:e["age"].as_i64().unwrap_or(0)as i32});if e["visible"]==true{visible.push(id);}
  }
  let (id,safe)={let s=self.state();let mut b=s.behaviors.borrow_mut();let r=&mut b.elytra_target.behavior;r.select(candidates,|id|visible.contains(&id));(r.target,r.settings.safe)};
  if safe{query(host,json!({"op":"safe"}));}let Some(id)=id else{return;};
  let distance=facts["candidates"].as_array().and_then(|a|a.iter().find(|e|e["id"].as_i64()==Some(i64::from(id)))).and_then(|e|e["distance"].as_f64()).unwrap_or(f64::INFINITY);
  let click=self.crosshair_cooldown(host).is_some_and(|(c,_)|self.state().behaviors.borrow_mut().kill_aura.clicker.amount_at(0,c)>0);
  let now=crate::behaviors::current_time_millis();
  let (ready,packet,reset)={let s=self.state();let mut b=s.behaviors.borrow_mut();let aura=b.query_running(&s.registry,"KillAura");let a=&b.kill_aura;let outside=a.behavior.target.and_then(|id|facts["candidates"].as_array().and_then(|v|v.iter().find(|e|e["id"].as_i64()==Some(i64::from(id))))).and_then(|e|e["distance"].as_f64()).is_some_and(|distance|distance>=f64::from(a.behavior.interaction(a.facts["attributeRange"].as_f64().unwrap_or(3.)as f32)).powi(2));let r=&mut b.elytra_target.behavior;let ready=r.firework_ready(now,distance,aura,click,outside);let range=r.settings.slot_reset;let reset=range[0]+(liquidbounce_behaviors::random::random_unit_f32()*((range[1]-range[0]+1)as f32)).floor()as i32;(ready,r.settings.packet,reset)};
  if !ready{return;}
  let Some(slot)=facts["rockets"].as_array().and_then(|a|a.first()).and_then(Value::as_i64)else{return;};
  let used=if packet{query(host,json!({"op":"packetUse","slot":slot}))==true}else{
   if slot!=40{let _=host.client_select_slot_silently(REQUESTER,slot as i32,reset);}
   let rotation=host.client_player_rotation();host.client_use_item(if slot==40{1}else{0},rotation[0],rotation[1])
  };
  if used{self.state().behaviors.borrow_mut().elytra_target.behavior.used(now);}
 }
}
