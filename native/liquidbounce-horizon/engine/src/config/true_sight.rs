// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::true_sight::TrueSight;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum TrueSightSetting{Sight(u8),EntityColor(i32),EntityFeatureLayerColor(i32)}
impl TrueSightSetting {pub fn apply(self,module:&mut TrueSight){match self{Self::Sight(v)=>module.sight=v,Self::EntityColor(v)=>module.entity_color=v,Self::EntityFeatureLayerColor(v)=>module.entity_feature_layer_color=v}}}
pub fn serialize_true_sight(module:&TrueSight,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
 let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::TrueSight,common);let names=["Barriers","Lights","Entities"];
 let selected=names.iter().enumerate().filter(|(i,_)|module.sight&(1<<i)!=0).map(|(_,n)|*n).collect::<Vec<_>>();
 let mut sight=tree.leaf("Sight","MULTI_CHOOSE",json!(selected),"liquidbounce.module.trueSight.sight",None);
 if format==JsonFormat::Interop{sight["choices"]=json!(names);sight["canBeNone"]=json!(true);sight["isOrderSensitive"]=json!(false);}values.push(sight);
 for (name,key,color) in [("EntityColor","entityColor",module.entity_color),("EntityFeatureLayerColor","entityFeatureLayerColor",module.entity_feature_layer_color)]{values.push(tree.leaf(name,"COLOR",json!(color),&format!("liquidbounce.module.trueSight.{key}"),None));}
 Ok(tree.group("TrueSight","TOGGLEABLE",values))
}
fn parse_color(value:&Value,path:&str)->Result<i32,ConfigError>{
 let invalid=||ConfigError::new(path,"expected ARGB number or 6/8 digit hex string");
 match value{Value::Number(n)=>decimal_int(&n.to_string()).ok_or_else(invalid),Value::String(text)=>{let hex=text.strip_prefix('#').unwrap_or(text);if !matches!(hex.len(),6|8){return Err(invalid());}let v=i64::from_str_radix(hex,16).map_err(|_|invalid())?;Ok(if hex.len()==6{v as i32|0xff000000u32 as i32}else{v as i32})},_=>Err(invalid())}
}
pub(super) fn parse_true_sight(patch:&mut SettingsPatch,root:&Nodes<'_>){
 if let Some(node)=root.get("Sight") {if let Ok(value)=value_of(node,"TrueSight.Sight"){
  let mut bits=0u8;patch.changes.push(SettingChange::TrueSight(TrueSightSetting::Sight(bits)));
  let entries=match value{Value::Array(a)=>a.iter().collect::<Vec<_>>(),Value::String(_)|Value::Bool(_)|Value::Number(_)=>vec![value],_=>vec![]};
  for value in entries {let Some(tag)=scalar_string(value) else {patch.issues.push(ConfigError::new("TrueSight.Sight","invalid Gson string entry"));break;};
   for (i,name) in ["Barriers","Lights","Entities"].iter().enumerate(){if liquidbounce_behaviors::true_sight::sight_tag(&tag,name){bits|=1<<i;}}
   patch.changes.push(SettingChange::TrueSight(TrueSightSetting::Sight(bits)));
  }
 }}
 leaf(patch,root,"EntityColor","TrueSight.EntityColor",|v,p|parse_color(v,p).map(|v|SettingChange::TrueSight(TrueSightSetting::EntityColor(v))));
 leaf(patch,root,"EntityFeatureLayerColor","TrueSight.EntityFeatureLayerColor",|v,p|parse_color(v,p).map(|v|SettingChange::TrueSight(TrueSightSetting::EntityFeatureLayerColor(v))));
}
