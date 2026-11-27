// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNoClip, ae0eb396: pause is independent of enabled state.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct NoClip {pub speed:f32,pub only_in_vehicle:bool,pub disable_on_setback:bool,pub no_clip_set:bool}
impl Default for NoClip {fn default()->Self {Self{speed:0.32,only_in_vehicle:false,disable_on_setback:true,no_clip_set:false}}}
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Tick {None,Clear,Apply{vehicle:bool,velocity:Option<[f64;3]>}}
impl NoClip {
 pub fn paused(self,controlled_vehicle:bool)->bool {self.only_in_vehicle&&!controlled_vehicle}
 pub fn tick(&mut self,running:bool,vehicle:bool,vehicle_control:bool,input:u8,jump:bool,sneak:bool,sin_cos:[f64;2])->Tick {
  if !running {return Tick::None;}
  if self.paused(vehicle) {return if self.no_clip_set {self.no_clip_set=false;Tick::Clear}else{Tick::None};}
  self.no_clip_set=true;let speed=f64::from(self.speed);
  let moving=(input&1!=0)!=(input&2!=0)||(input&4!=0)!=(input&8!=0);
  let [sin,cos]=sin_cos;
  // Preserve withStrafe's 0*old components: NaN handling lives in native host.
  let velocity=if vehicle&&vehicle_control {None}else{Some([if moving {-sin*speed}else{0.0},if jump {speed}else if sneak {-speed}else{0.0},if moving {cos*speed}else{0.0}])};
  Tick::Apply{vehicle,velocity}
 }
 pub fn setback(self,running:bool,vehicle:bool)->bool {running&&self.disable_on_setback&&!self.paused(vehicle)}
 pub fn on_disabled(&mut self) {self.no_clip_set=false;}
}
