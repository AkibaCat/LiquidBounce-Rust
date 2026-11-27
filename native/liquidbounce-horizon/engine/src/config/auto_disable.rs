// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::auto_disable::{AutoDisable,Reason};
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum AutoDisableSetting {Modules(std::collections::BTreeSet<String>),On(u32)}
impl AutoDisableSetting {pub fn apply(self,state:&mut AutoDisable){match self{Self::Modules(v)=>state.module_names=v,Self::On(v)=>state.on=v}}}
pub fn serialize_auto_disable(state:&AutoDisable,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
    let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::AutoDisable,common);
    let mut modules=tree.leaf("Modules","REGISTRY_LIST",json!(state.module_names),"liquidbounce.module.autoDisable.modules",None);
    let mut on=tree.multi("On",state.on,&Reason::TAGS,"liquidbounce.module.autoDisable.on");
    if format==JsonFormat::Interop {modules["registry"]=json!("client_module");on["canBeNone"]=json!(false);}
    values.extend([modules,on]);Ok(tree.group("AutoDisable","TOGGLEABLE",values))
}
fn names(value:&Value,path:&str)->Result<std::collections::BTreeSet<String>,ConfigError>{
    let values=match value {Value::Array(values)=>values.clone(),Value::String(_)|Value::Number(_)|Value::Bool(_)=>vec![value.clone()],_=>return Ok(Default::default())};
    values.into_iter().map(|value|scalar_string(&value).ok_or_else(||ConfigError::new(path,"module names must be primitives"))).collect()
}
pub(super) fn parse_auto_disable(patch:&mut SettingsPatch,root:&Nodes<'_>){
    leaf(patch,root,"Modules","AutoDisable.Modules",|v,p|names(v,p).map(|v|SettingChange::AutoDisable(AutoDisableSetting::Modules(v))));
    leaf(patch,root,"On","AutoDisable.On",|v,p|parse_multi_mask(v,p,&Reason::TAGS).map(|v|SettingChange::AutoDisable(AutoDisableSetting::On(if v==0{15}else{v}))));
}
