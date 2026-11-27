// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleHighJump ae0eb396. Waiters belong to the shared CoroutineTicker.
use std::collections::BTreeMap;

pub const MODES: [&str; 2] = ["Vanilla", "Vulcan-2.7.5"];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode { #[default] Vanilla, Vulcan }
impl Mode {
    pub fn tag(self)->&'static str{MODES[usize::from(self==Self::Vulcan)]}
    pub fn parse(tag:&str)->Option<Self>{match tag{"Vanilla"=>Some(Self::Vanilla),"Vulcan-2.7.5"|"Vulcan"=>Some(Self::Vulcan),_=>None}}
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {pub mode:Mode,pub motion:f32,pub glide:bool}
impl Default for Settings{fn default()->Self{Self{mode:Mode::Vanilla,motion:0.8,glide:false}}}
#[derive(Clone, Copy, Debug)]
pub struct TickContext{pub on_ground:bool,pub fall_distance:f64,pub tick_count:i32}
#[derive(Default,Debug)]
pub struct HighJump {
    pub settings:Settings,
    /// Kotlin object field survives disable, mode changes and world changes.
    pub should_glide:bool,
    next:u64,waiters:BTreeMap<u64,u8>,
}
impl HighJump {
    /// ModeValueGroup removes the old scope even if the listener is not running.
    pub fn set_mode(&mut self,mode:Mode){if self.settings.mode!=mode{self.waiters.clear();self.settings.mode=mode;}}
    /// Real module/mode disable removes its scope. Losing world.running alone
    /// is observed by each waiter's guard at its next CoroutineTicker position.
    pub fn disabled(&mut self){self.waiters.clear();}
    /// SequenceHandler starts UNDISPATCHED at PlayerJumpEvent HEAD. Every
    /// Vulcan jump registers an independent waiter, including rejected powers.
    pub fn jump(&mut self,running:bool,motion:&mut f32)->Option<u64>{
        if !running{return None;}
        *motion=self.settings.motion;
        if self.settings.mode!=Mode::Vulcan{return None;}
        self.next=self.next.checked_add(1).expect("HighJump waiter exhausted");
        self.waiters.insert(self.next,0);Some(self.next)
    }
    /// Return (retain callback, write vertical velocity). Called once at this
    /// callback's FIFO slot, before GameTickEvent, only on outer Minecraft ticks.
    pub fn advance_callback(&mut self,id:u64,running:bool)->(bool,Option<f64>){
        let Some(elapsed)=self.waiters.get_mut(&id)else{return(false,None)};
        if !running||self.settings.mode!=Mode::Vulcan{self.waiters.remove(&id);return(false,None);}
        *elapsed+=1;
        if *elapsed<100{return(true,None);}
        self.waiters.remove(&id);self.should_glide=true;(false,Some(0.0))
    }
    /// Reads the same pre-entity-tick tickCount as the original GameTickEvent.
    pub fn game_tick(&mut self,running:bool,c:TickContext)->Option<f64>{
        if !running||self.settings.mode!=Mode::Vulcan||!self.settings.glide||!self.should_glide{return None;}
        if c.on_ground{self.should_glide=false;return None;}
        if c.fall_distance>0.0{(c.tick_count%2==0).then_some(-0.155)}else{Some(-0.1)}
    }
    pub fn pending(&self)->usize{self.waiters.len()}
}
