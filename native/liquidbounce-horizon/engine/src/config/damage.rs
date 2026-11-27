// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::damage::{Damage,Mode};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum DamageSetting {Mode(Mode),Damage(i32)}
impl DamageSetting {pub fn apply(self,state:&mut Damage) {match self {Self::Mode(v)=>state.mode=v,Self::Damage(v)=>state.damage=v}}}
pub fn serialize_damage(state:&Damage,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError> {
    let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::Damage,common);
    let mut mode=tree.leaf("Mode","CHOOSE",json!(state.mode.tag()),"liquidbounce.module.damage.mode",None);
    if format==JsonFormat::Interop {mode["choices"]=json!(Mode::ALL.map(Mode::tag));}
    values.extend([mode,tree.ranged("Damage","INT",json!(state.damage),json!({"from":0,"to":20}),"HP","liquidbounce.module.damage.damage")]);
    Ok(tree.group("Damage","TOGGLEABLE",values))
}
pub(super) fn parse_damage(patch:&mut SettingsPatch,root:&Nodes<'_>) {
    leaf(patch,root,"Mode","Damage.Mode",|v,p|scalar_string(v).and_then(|v|Mode::parse(&v)).map(|v|SettingChange::Damage(DamageSetting::Mode(v))).ok_or_else(||ConfigError::new(p,"unknown Damage mode")));
    leaf(patch,root,"Damage","Damage.Damage",|v,p|parse_int(v,p).map(|v|SettingChange::Damage(DamageSetting::Damage(v))));
}
