// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of Minecraft Agent world/Input.kt and navigation/Keystrokes.kt,
//! commit 0abf69ac35157f33348310326017f4e3fea6b95f, GPL-3.0-or-later.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Input {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
}
impl Input {
    pub const EMPTY: Self = Self {
        forward: false, backward: false, left: false, right: false,
        jump: false, shift: false, sprint: false,
    };
    pub fn is_moving(self) -> bool {self.forward || self.backward || self.left || self.right}
    pub fn with_keys(self,jump:bool,shift:bool,sprint:bool)->Self {Self{jump,shift,sprint,..self}}
    pub fn java_hash(self)->i32 {
        [self.forward,self.backward,self.left,self.right,self.jump,self.shift,self.sprint]
            .into_iter().fold(0,|hash,key|hash*2+i32::from(key))
    }
    pub fn from_mask(mask:i32)->Self {
        Self{forward:mask&64!=0,backward:mask&32!=0,left:mask&16!=0,right:mask&8!=0,jump:mask&4!=0,shift:mask&2!=0,sprint:mask&1!=0}
    }
}

const LONG_AGO:i32=1000;
const JUMP:usize=5;
#[derive(Clone,Debug)]
pub struct Keystrokes {down:[bool;6],ticks:[i32;6]}
impl Default for Keystrokes {
    fn default()->Self {Self{down:[false;6],ticks:[LONG_AGO;6]}}
}
impl Keystrokes {
    pub fn reset(&mut self){self.down.fill(false);self.ticks.fill(LONG_AGO);}
    pub fn press(&mut self,input:Input,exact:bool,min_press:i32,min_release:i32)->Input {
        let wanted=[input.forward,input.backward,input.left,input.right,input.shift,input.jump];
        let mut keys=wanted;
        if !exact&&(min_press>0||min_release>0) {
            for key in 0..6 {
                keys[key]=if self.down[key]&&!wanted[key] {self.ticks[key]<min_press}
                    else if !self.down[key]&&wanted[key] {key==JUMP||self.ticks[key]>=min_release}
                    else {wanted[key]};
            }
            for(a,b)in[(0,1),(2,3)] {
                if keys[a]&&keys[b] {
                    if self.down[a]&&!wanted[a] {keys[b]=false;}
                    else if self.down[b]&&!wanted[b] {keys[a]=false;}
                }
            }
        }
        for (key, pressed) in keys.into_iter().enumerate() {
            if pressed==self.down[key] {self.ticks[key]=self.ticks[key].wrapping_add(1).min(LONG_AGO);}
            else {self.down[key]=pressed;self.ticks[key]=1;}
        }
        Input{forward:keys[0],backward:keys[1],left:keys[2],right:keys[3],jump:keys[5],shift:keys[4],sprint:input.sprint}
    }
}

#[derive(Clone,Copy,Debug,Default)]
pub struct MovementKeys {pub directional_input:Input,pub jump:bool,pub sneak:bool}
impl MovementKeys {pub fn to_input(self,sprint:bool)->Input{self.directional_input.with_keys(self.jump,self.sneak,sprint)}}
