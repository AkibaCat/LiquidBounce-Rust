// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::{LiquidBounceHooks,raw_options};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{no_clip::Tick,strafe::movement_direction};
impl LiquidBounceHooks {
 pub(crate) fn no_clip_drain(&self,host:&mut dyn Host) {
  let pending=self.state().behaviors.borrow().no_clip_packets.take_pending();
  for _ in 0..pending {self.no_clip_setback(host);}
 }
 pub(crate) fn no_clip_tick(&self,host:&mut dyn Host) {
  let Some([yaw,vehicle,jump,sneak])=host.client_no_clip_state() else {return;};
  let (running,input,vehicle_control)={let state=self.state();let modules=state.behaviors.borrow();let i=modules.directional_input;
   (modules.query_running(&state.registry,"NoClip"),u8::from(i.forwards)|(u8::from(i.backwards)<<1)|(u8::from(i.left)<<2)|(u8::from(i.right)<<3),false)};
  // VehicleControl can supply its real running flag here when it is ported.
  let angle=f64::from(movement_direction(yaw as f32,input))*0.017453292519943295;
  let sin_cos=host.client_java_sin_cos(angle);
  let effect=self.state().behaviors.borrow_mut().no_clip.tick(running,vehicle!=0.0,vehicle_control,input,jump!=0.0,sneak!=0.0,sin_cos);
  let moving=(input&1!=0)!=(input&2!=0)||(input&4!=0)!=(input&8!=0);
  match effect {Tick::None=>{},Tick::Clear=>host.client_no_clip_flags(false),Tick::Apply{vehicle,velocity}=>{host.client_no_clip_flags(true);if let Some(v)=velocity {host.client_no_clip_velocity(vehicle,moving,v);}}}
 }
 pub(crate) fn no_clip_disabled(&self,host:&mut dyn Host) {self.state().behaviors.borrow_mut().no_clip.on_disabled();host.client_no_clip_flags(false);}
 /// Call at this module's PacketEvent position for every inbound PlayerPosition,
 /// including an event cancelled by an earlier listener.
 pub(crate) fn no_clip_setback(&self,host:&mut dyn Host) {
  let Some([_,vehicle,_,_])=host.client_no_clip_state() else {return;};
  let disable={let state=self.state();let modules=state.behaviors.borrow();modules.no_clip.setback(modules.query_running(&state.registry,"NoClip"),vehicle!=0.0)};
  if !disable {return;}
  let lang=horizon_client::lang::code().to_ascii_lowercase();
  host.chat(match lang.as_str(){"zh_cn"|"zh_sg"|"zh_hans"=>"§c检测到回弹，请确保你已经站在一个方块里面。","zh_tw"|"zh_hk"|"zh_hant"=>"§c檢測到回彈，請確保你已經站在一個方塊裡面。",_=>"§cYou have been set back by the server! Make sure you are already standing inside of a block."});
  let options=raw_options(host);
  let effects={let mut state=self.state();let id=state.behaviors.borrow().query_ids["NoClip"];
   if let Err(e)=state.with_registry(options,|r,c|r.set_enabled_sync(id,false,c)){eprintln!("[LiquidBounce] NoClip disable: {e:?}");}
   state.take_effects()};
  crate::runtime::apply_effects(self,host,effects);
 }
}
