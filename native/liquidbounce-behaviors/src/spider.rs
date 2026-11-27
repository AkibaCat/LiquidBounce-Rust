// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSpider's three complete modes, ae0eb396.
#[path="spider_geometry.rs"]pub mod geometry;
pub const MODES:[&str;3]=["Vanilla","Polar-2025-03-29","Vulcan-2.8.8"];
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub enum Mode{#[default]Vanilla,Polar,Vulcan}
impl Mode{
    pub fn tag(self)->&'static str{MODES[self as usize]}
    pub fn parse(v:&str)->Option<Self>{match v{"Vanilla"=>Some(Self::Vanilla),"Polar-2025-03-29"|"Polar-29.03.2025"=>Some(Self::Polar),"Vulcan-2.8.8"|"Vulcan288"=>Some(Self::Vulcan),_=>None}}
}
#[derive(Clone,Copy,Debug,PartialEq)]pub struct Settings{pub mode:Mode,pub motion:f32,pub jump_height:f32}
impl Default for Settings{fn default()->Self{Self{mode:Mode::Vanilla,motion:0.3,jump_height:0.55}}}
#[derive(Clone,Copy,Debug,PartialEq)]pub enum Velocity{Vertical(f64),All([f64;3])}
#[derive(Clone,Copy,Debug,Default,PartialEq)]pub struct Step{pub velocity:Option<Velocity>,pub wait:Option<u64>}
#[derive(Clone,Copy,Debug)]struct Wait{id:u64,phase:u8,elapsed:u8}
#[derive(Debug,Default)]pub struct Spider{pub settings:Settings,pub requires_stop:bool,next:u64,wait:Option<Wait>}
impl Spider{
    pub fn set_mode(&mut self,value:Mode){if self.settings.mode!=value{self.cancel();self.settings.mode=value;}}
    /// Mode disable cancels its sequence, but the original object field survives.
    pub fn cancel(&mut self){self.wait=None;}
    fn wait(&mut self,phase:u8)->u64{self.next=self.next.checked_add(1).expect("Spider waiter exhausted");self.wait=Some(Wait{id:self.next,phase,elapsed:0});self.next}
    pub fn game_tick(&mut self,running:bool,collision:bool,on_climbable:bool)->Step{
        if !running{return Step::default();}
        match self.settings.mode{
            Mode::Vanilla=>Step{velocity:collision.then_some(Velocity::Vertical(f64::from(self.settings.motion))),wait:None},
            Mode::Polar=>Step::default(),
            Mode::Vulcan=>{
                if self.wait.is_some(){return Step::default();}
                if collision&&!on_climbable{self.requires_stop=true;return Step{velocity:None,wait:Some(self.wait(0))};}
                if !collision&&self.requires_stop{self.requires_stop=false;return Step{velocity:Some(Velocity::Vertical(0.0)),wait:None};}
                Step::default()
            }
        }
    }
    /// Each waitTicks(2) is a distinct shared FIFO registration.
    pub fn callback(&mut self,id:u64,running:bool)->(bool,Step){
        let Some(wait)=self.wait.as_mut().filter(|w|w.id==id)else{return(false,Step::default());};
        if !running||self.settings.mode!=Mode::Vulcan{self.cancel();return(false,Step::default());}
        wait.elapsed+=1;if wait.elapsed<2{return(true,Step::default());}
        let phase=wait.phase;self.wait=None;
        (false,if phase==0{Step{velocity:Some(Velocity::Vertical(9.6599696)),wait:Some(self.wait(1))}}else{Step{velocity:Some(Velocity::All([0.0,0.0001,0.0])),wait:None}})
    }
    pub fn jump(&self,running:bool,horizontal_collision:bool,motion:&mut f32){if running&&self.settings.mode==Mode::Polar&&horizontal_collision&&self.settings.jump_height>0.42{*motion=self.settings.jump_height;}}
}
