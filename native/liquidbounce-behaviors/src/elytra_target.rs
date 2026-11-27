// SPDX-License-Identifier: GPL-3.0-or-later
//! ElytraTarget arithmetic and state. Host owns entity, collision and item actions.
use crate::{auto_bow::{self, Candidate, Priority},auto_bow_math::{self as math,V},rotations::{Rotation,wrap_degrees}};
#[derive(Clone,Debug,PartialEq)]
pub struct Settings{
 pub safe:bool,pub always_glide:bool,pub sharp:bool,pub ignore_kill_aura:bool,pub look:bool,pub auto_distance:bool,
 pub prediction:bool,pub gravity:bool,pub gliding_only:bool,pub multiplier:[f32;2],pub center:bool,
 pub fov:f32,pub hurt_time:i32,pub priority:Vec<Priority>,
 pub firework:bool,pub packet:bool,pub extra_distance:f32,pub slot_reset:[i32;2],pub sync:bool,pub cooldown:[i32;2],
}
impl Default for Settings{fn default()->Self{Self{safe:true,always_glide:false,sharp:false,ignore_kill_aura:true,look:false,auto_distance:true,prediction:true,gravity:false,gliding_only:true,multiplier:[1.8,2.],center:false,fov:180.,hurt_time:10,priority:vec![Priority::Type,Priority::Health],firework:true,packet:false,extra_distance:50.,slot_reset:[0,0],sync:false,cooldown:[8,10]}}}
#[derive(Clone,Debug)]
pub struct ElytraTarget{pub settings:Settings,pub target:Option<i32>,pub gliding:bool,pub firework_cooldown:i32,pub last_firework:i64,pub waiting:bool}
impl Default for ElytraTarget{fn default()->Self{Self::new(0)}}
impl ElytraTarget{
 pub fn new(now:i64)->Self{Self{settings:Default::default(),target:None,gliding:false,firework_cooldown:750,last_firework:now,waiting:false}}
 pub fn disabled(&mut self){self.target=None;self.gliding=false;self.waiting=false;}
 pub fn select(&mut self,rows:Vec<Candidate>,visible:impl Fn(i32)->bool){let s=auto_bow::Settings{fov:self.settings.fov,hurt_time:self.settings.hurt_time,priority:self.settings.priority.clone(),..Default::default()};self.target=auto_bow::targets(&s,rows).into_iter().find(|e|visible(e.id)).map(|e|e.id);}
 pub fn point(&self,player:V,position:V,eyes:V,height:f64,velocity:V,gliding:bool,now:i64,random:&mut impl FnMut()->f64)->V{
  let mut target=if self.settings.center{position.add(V(0.,height/2.,0.))}else{eyes};
  if self.settings.prediction&&(!self.settings.gliding_only||gliding){let m=f64::from(self.settings.multiplier[0])+random()*f64::from(self.settings.multiplier[1]-self.settings.multiplier[0]);target=target.fma(m,velocity);if self.settings.gravity{target=target.sub(V(0.,0.5*0.05*m*m,0.));}}
  let t=now as f64/1000.;let noise=V((t*1.8).sin()*0.04+(random()-0.5)*0.02,(t*2.2).sin()*0.03+(random()-0.5)*0.015,(t*1.8).cos()*0.04+(random()-0.5)*0.02);
  target=target.fma(4.,noise);
  if self.settings.auto_distance{let direction=target.sub(player).normalize();let distance=player.sub(direction).len2();if distance<100.{target=target.fma(distance-10.,direction);}}
  target
 }
 pub fn rotation(&self,player:V,target:V)->Rotation{let a=math::rotation(target.sub(player));Rotation::new(a[0],a[1])}
 pub fn process(&self,from:Rotation,to:Rotation,now:i64)->Rotation{
  let yaw=wrap_degrees(to.yaw-from.yaw);let pitch=to.pitch-from.pitch;let difference=(yaw*yaw+pitch*pitch).sqrt();let time=now as f64;
  let boost=(time/300.).sin()>0.8;
  let speed=if boost{2.*(((now%360)as f32/300.*std::f32::consts::PI).sin()*0.8+1.2)}else{1.2*1.2};
  let back=if yaw.abs()>90.{(2.2*(time/150.).sin()*0.2+1.)as f32}else{1.2};
  let sharp=if self.settings.sharp{1.5}else{1.};
  let mut dy=yaw.clamp(-45.*sharp*speed*back,45.*sharp*speed*back);let mut dp=pitch.clamp(-35.*sharp*speed,35.*sharp*speed);
  if difference<5.{let micro=((time/80.).sin()*0.08+(time/120.).cos()*0.05)as f32;dy+=micro*0.2;dp+=micro*0.8;}
  Rotation::new(from.yaw+dy,(from.pitch+dp).clamp(-90.,90.))
 }
 /// Readiness uses the previous distance's cooldown, matching source handler order.
 pub fn firework_ready(&mut self,now:i64,distance:f64,aura:bool,click:bool,outside:bool)->bool{
  if !self.settings.firework||self.target.is_none(){self.waiting=false;return false;}
  let elapsed=now.wrapping_sub(self.last_firework)>=(i64::from(self.firework_cooldown)*50);
  let next=self.waiting;self.waiting=false;
  let allowed=if next||!aura||!self.settings.sync||click&&outside{true}else if click{self.waiting=elapsed;false}else{false};
  self.firework_cooldown=if distance>f64::from(self.settings.extra_distance).powi(2){self.settings.cooldown[1]}else{self.settings.cooldown[0]};
  elapsed&&allowed
 }
 pub fn used(&mut self,now:i64){self.last_firework=now;}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn source_defaults_and_disable_preserve_settings(){let mut b=ElytraTarget::new(10);assert_eq!(b.firework_cooldown,750);b.settings.sharp=true;b.target=Some(1);b.disabled();assert!(b.settings.sharp);assert_eq!(b.target,None);}
 #[test]fn gravity_and_gliding_gate(){let mut b=ElytraTarget::default();b.settings.auto_distance=false;b.settings.multiplier=[2.,2.];let p=V(1.,2.,3.);let v=V(1.,1.,1.);let a=b.point(V::default(),p,p,2.,v,true,0,&mut||0.5);b.settings.gravity=true;let c=b.point(V::default(),p,p,2.,v,true,0,&mut||0.5);assert!((a.1-c.1-0.1).abs()<1e-12);let c=b.point(V::default(),p,p,2.,v,false,0,&mut||0.5);assert_eq!(c.1,2.);}
 #[test]fn source_auto_distance_uses_player_distance_to_direction(){let b=ElytraTarget::default();let p=b.point(V::default(),V(20.,0.,0.),V(20.,0.,0.),2.,V::default(),false,0,&mut||0.5);let direct=V(20.,0.,0.16);let direction=direct.normalize();let expected=direct.fma(-9.,direction);assert_eq!(p,expected);}
 #[test]fn cooldown_previous_distance_and_one_tick_wait(){let mut b=ElytraTarget::new(0);b.target=Some(1);b.settings.sync=true;assert!(!b.firework_ready(1000,1.,true,true,false));assert_eq!(b.firework_cooldown,8);assert!(!b.firework_ready(1000,1.,true,true,false));assert!(b.firework_ready(1050,1.,true,false,false));b.used(1050);assert!(!b.firework_ready(1100,5000.,false,false,false));assert_eq!(b.firework_cooldown,10);}
 #[test]fn processor_wraps_and_caps_pitch(){let b=ElytraTarget::default();let p=b.process(Rotation::new(179.,89.),Rotation::new(-179.,90.),0);assert!(p.yaw>179.);assert!(p.pitch<=90.);}
}
