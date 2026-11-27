// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoDisable, LiquidBounce ae0eb396.
use std::collections::BTreeSet;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)] pub enum Reason {Flag=0,Death=1,WorldChange=2,Disconnect=3}
impl Reason {
    pub const TAGS:[&'static str;4]=["Flag","Death","WorldChange","Disconnect"];
    pub const fn description(self)->&'static str {match self {Self::Flag=>"flag",Self::Death=>"your death",Self::WorldChange=>"world change",Self::Disconnect=>"disconnection"}}
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct AutoDisable {
    pub on:u32,
    /// The original registry-list value and reference set are distinct. Loading
    /// this value alone does not call add/remove and must not change membership.
    pub module_names:BTreeSet<String>,
    selected:Vec<String>,
}
impl Default for AutoDisable {
    fn default()->Self {let selected=["Fly","Speed","NoClip","KillAura"].map(str::to_owned).to_vec();Self{on:15,module_names:selected.iter().cloned().collect(),selected}}
}
impl AutoDisable {
    pub fn handles(&self,running:bool,reason:Reason)->bool {running&&self.on&(1<<(reason as u8))!=0}
    pub fn selected(&self)->&[String] {&self.selected}
    pub fn clear(&mut self) {self.selected.clear();self.module_names.clear();}
    /// `name` must be the canonical name of a resolved, identity-stable module.
    pub fn add(&mut self,name:&str)->bool {if self.selected.iter().any(|v|v==name){return false;}self.selected.push(name.to_owned());self.module_names.insert(name.to_owned());true}
    pub fn remove(&mut self,name:&str)->bool {let Some(index)=self.selected.iter().position(|v|v==name)else{return false;};self.selected.remove(index);self.module_names.remove(name);true}
}
