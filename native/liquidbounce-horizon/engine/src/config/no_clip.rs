// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::no_clip::NoClip;
#[derive(Clone,Copy,Debug,PartialEq)]pub enum NoClipSetting{Speed(f32),OnlyInVehicle(bool),DisableOnSetback(bool)}
impl NoClipSetting {pub fn apply(self,module:&mut NoClip){match self{Self::Speed(v)=>module.speed=v,Self::OnlyInVehicle(v)=>module.only_in_vehicle=v,Self::DisableOnSetback(v)=>module.disable_on_setback=v}}}
pub fn serialize_no_clip(module:&NoClip,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
 let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::NoClip,common);
 values.push(tree.ranged("Speed","FLOAT",float_json(module.speed,"NoClip.Speed")?,json!({"from":0.1,"to":0.4}),"","liquidbounce.module.noClip.speed"));
 for (name,key,value) in [("OnlyInVehicle","onlyInVehicle",module.only_in_vehicle),("DisableOnSetback","disableOnSetback",module.disable_on_setback)] {values.push(tree.leaf(name,"BOOLEAN",json!(value),&format!("liquidbounce.module.noClip.{key}"),None));}
 Ok(tree.group("NoClip","TOGGLEABLE",values))
}
pub(super) fn parse_no_clip(patch:&mut SettingsPatch,root:&Nodes<'_>){
 leaf(patch,root,"Speed","NoClip.Speed",|v,p|parse_float(v,p).map(|v|SettingChange::NoClip(NoClipSetting::Speed(v))));
 leaf(patch,root,"OnlyInVehicle","NoClip.OnlyInVehicle",|v,p|parse_bool(v,p).map(|v|SettingChange::NoClip(NoClipSetting::OnlyInVehicle(v))));
 leaf(patch,root,"DisableOnSetback","NoClip.DisableOnSetback",|v,p|parse_bool(v,p).map(|v|SettingChange::NoClip(NoClipSetting::DisableOnSetback(v))));
}
