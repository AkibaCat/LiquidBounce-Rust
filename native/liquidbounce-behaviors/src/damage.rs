// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleDamage, LiquidBounce ae0eb396. Each send remains an ordinary packet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode { #[default] Ncp, Aac, Verus }
impl Mode {
    pub const ALL: [Self;3] = [Self::Ncp,Self::Aac,Self::Verus];
    pub const fn tag(self)->&'static str { match self {Self::Ncp=>"NCP-2021-04-13",Self::Aac=>"AAC-2020-07-10",Self::Verus=>"Verus-b3896"} }
    pub fn parse(value:&str)->Option<Self> { Self::ALL.into_iter().find(|mode|mode.tag().eq_ignore_ascii_case(value)||match mode {Self::Ncp=>value.eq_ignore_ascii_case("NCP"),Self::Aac=>value.eq_ignore_ascii_case("AAC"),Self::Verus=>value.eq_ignore_ascii_case("Verus")}) }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Damage {pub mode:Mode,pub damage:i32}
impl Default for Damage {fn default()->Self {Self{mode:Mode::Ncp,damage:1}}}
pub trait Host {
    fn position(&mut self)->[f64;3];
    fn horizontal_collision(&mut self)->bool;
    fn send_position(&mut self,position:[f64;3],on_ground:bool,horizontal_collision:bool);
    fn set_vertical_velocity(&mut self,value:f64);
}
impl Damage {
    pub fn run(self,host:&mut impl Host) {
        match self.mode {
            Mode::Ncp=>{
                let [x,y,z]=host.position();
                // Kotlin Int multiplication wraps; a non-positive repeat is empty.
                let count=65_i32.wrapping_mul(self.damage);
                for _ in 0..count {
                    let collision=host.horizontal_collision();
                    host.send_position([x,y+0.049,z],false,collision);
                    let collision=host.horizontal_collision();
                    host.send_position([x,y,z],false,collision);
                }
                let collision=host.horizontal_collision();
                host.send_position([x,y,z],true,collision);
            }
            Mode::Aac=>host.set_vertical_velocity(5.0*f64::from(self.damage)),
            Mode::Verus=>{
                for (offset,on_ground) in [(0.0,false),(3.25,false),(0.0,false),(0.0,true)] {
                    // Source re-reads live player coordinates for every packet.
                    let [x,y,z]=host.position();
                    let collision=host.horizontal_collision();
                    host.send_position([x,y+offset,z],on_ground,collision);
                }
            }
        }
    }
}
