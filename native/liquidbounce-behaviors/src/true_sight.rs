// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTrueSight, LiquidBounce ae0eb396.
pub const BARRIERS:u8=1;pub const LIGHTS:u8=2;pub const ENTITIES:u8=4;
#[path="navigation/java_case.rs"]mod java_case;
pub fn sight_tag(tag:&str,name:&str)->bool {java_case::equals_ignore_case(tag,name)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct TrueSight {pub sight:u8,pub entity_color:i32,pub entity_feature_layer_color:i32}
impl Default for TrueSight {fn default()->Self {Self{sight:7,entity_color:0x64ffffff,entity_feature_layer_color:0x78ffffff}}}
pub fn abgr(argb:i32)->u32 {let c=argb as u32;(c&0xff00ff00)|((c&0xff)<<16)|((c>>16)&0xff)}
impl TrueSight {
 pub fn markers(self,running:bool)->u8 {if running {self.sight&3}else{0}}
 /// ESP supplies its real requiresTrueSight result when that module is ported.
 pub fn colors(self,running:bool,living:bool,invisible:bool,esp_requires:bool)->Option<[u32;2]> {
  if !living||!invisible {return None;}let enabled=running&&self.sight&ENTITIES!=0;
  if enabled {Some([abgr(self.entity_color),abgr(self.entity_feature_layer_color)])}else if esp_requires {Some([0x78ffffff;2])}else {None}
 }
}
