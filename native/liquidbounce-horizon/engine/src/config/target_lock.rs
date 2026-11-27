// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::target_lock::{TargetLock,Mode,FilterType,WhenNoLock};
#[derive(Clone,Debug,PartialEq)]
pub enum TargetLockSetting { Mode(Mode), Combat(bool), MaximumTime(i32), MaximumRange(f32), WhenNoLock(WhenNoLock), Usernames(Vec<Option<String>>), FilterType(FilterType) }
impl TargetLockSetting {
    pub fn apply(self,state:&mut TargetLock) {
        match self { Self::Mode(v)=>state.settings.mode=v,Self::Combat(v)=>state.settings.combat_only=v,
            Self::MaximumTime(v)=>state.settings.maximum_time=v,Self::MaximumRange(v)=>state.settings.maximum_range=v,
            Self::WhenNoLock(v)=>state.settings.when_no_lock=v,Self::Usernames(v)=>state.settings.usernames=v,Self::FilterType(v)=>state.settings.filter_type=v }
    }
}
pub fn serialize_target_lock(state:&TargetLock,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError> {
    let tree=Tree{format,describe};let s=&state.settings;
    let mut when=tree.leaf("WhenNoLock","CHOOSE",json!(s.when_no_lock.tag()),"liquidbounce.module.targetLock.mode.temporary.whenNoLock",None);
    let mut filter=tree.leaf("FilterType","CHOOSE",json!(s.filter_type.tag()),"liquidbounce.module.targetLock.mode.filter.filterType",None);
    let mut names=tree.leaf("Usernames","MUTABLE_LIST",json!(s.usernames),"liquidbounce.module.targetLock.mode.filter.usernames",None);
    if format==JsonFormat::Interop { when["choices"]=json!(["AllowAll","AllowNone"]);filter["choices"]=json!(["Whitelist","Blacklist"]);names["innerValueType"]=json!("TEXT"); }
    let mut choices=Map::new();
    choices.insert("Temporary".into(),tree.group("Temporary","CONFIGURABLE",vec![
        tree.ranged("MaximumTime","INT",json!(s.maximum_time),json!({"from":0,"to":120}),"s","liquidbounce.module.targetLock.mode.temporary.maximumTime"),
        tree.ranged("MaximumRange","FLOAT",float_json(s.maximum_range,"TargetLock.Mode.Temporary.MaximumRange")?,json!({"from":8.0,"to":40.0}),"","liquidbounce.module.targetLock.mode.temporary.maximumRange"),when]));
    choices.insert("Filter".into(),tree.group("Filter","CONFIGURABLE",vec![names,filter]));
    let mut values=tree.common(ModuleKind::TargetLock,common);
    values.extend([tree.mode(s.mode.tag(),choices),tree.leaf("Combat","BOOLEAN",json!(s.combat_only),"liquidbounce.module.targetLock.combat",None)]);
    // The whole module is doNotIncludeAlways: root excludes it from PUBLIC;
    // this function serves FILE and INTEROP, which explicitly include it.
    Ok(tree.group("TargetLock","TOGGLEABLE",values))
}
fn usernames(value:&Value,path:&str)->Result<Vec<Option<String>>,ConfigError> {
    fn one(value:&Value,path:&str)->Result<Option<String>,ConfigError> {
        if value.is_null(){return Ok(None);}
        scalar_string(value).map(Some).ok_or_else(||ConfigError::new(path,"expected string primitive or null"))
    }
    match value {Value::Array(values)=>values.iter().map(|v|one(v,path)).collect(),value=>one(value,path).map(|v|vec![v])}
}
fn change(value:TargetLockSetting)->SettingChange{SettingChange::TargetLock(value)}
pub(super) fn parse_target_lock(patch:&mut SettingsPatch,root:&Nodes<'_>) {
    if let Some(mode)=root.get("Mode") {
        match mode.get("active").and_then(scalar_string).and_then(|v|Mode::parse(&v)) {
            Some(value)=>patch.changes.push(change(TargetLockSetting::Mode(value))),
            None=>patch.issues.push(ConfigError::new("TargetLock.Mode.active","missing or unknown case-sensitive mode")),
        }
        if let Err(error)=nodes(mode,"Mode","TargetLock.Mode"){patch.issues.push(error);}
        if let Some(choices)=mode.get("choices").and_then(Value::as_object) {
            for name in ["Temporary","Filter"] {
                let path=format!("TargetLock.Mode.{name}");
                match choices.get(name).ok_or_else(||ConfigError::new(&path,"choice not found")).and_then(|v|nodes(v,name,&path)) {
                    Ok(choice) if name=="Temporary"=>{
                        leaf(patch,&choice,"MaximumTime",&format!("{path}.MaximumTime"),|v,p|parse_int(v,p).map(|v|change(TargetLockSetting::MaximumTime(v))));
                        leaf(patch,&choice,"MaximumRange",&format!("{path}.MaximumRange"),|v,p|parse_float(v,p).map(|v|change(TargetLockSetting::MaximumRange(v))));
                        leaf(patch,&choice,"WhenNoLock",&format!("{path}.WhenNoLock"),|v,p|scalar_string(v).and_then(|v|WhenNoLock::parse(&v)).map(|v|change(TargetLockSetting::WhenNoLock(v))).ok_or_else(||ConfigError::new(p,"unknown choice")));
                    }
                    Ok(choice)=>{
                        let p=format!("{path}.Usernames");
                        if let Some(node)=choice.get("Usernames") {
                            match node.get("value").ok_or_else(||ConfigError::new(&p,"missing value")).and_then(|v|usernames(v,&p)) {
                                Ok(v)=>patch.changes.push(change(TargetLockSetting::Usernames(v))),Err(error)=>patch.issues.push(error),
                            }
                        }
                        leaf(patch,&choice,"FilterType",&format!("{path}.FilterType"),|v,p|scalar_string(v).and_then(|v|FilterType::parse(&v)).map(|v|change(TargetLockSetting::FilterType(v))).ok_or_else(||ConfigError::new(p,"unknown choice")));
                    }
                    Err(error)=>patch.issues.push(error),
                }
            }
        } else {patch.issues.push(ConfigError::new("TargetLock.Mode.choices","expected choices object"));}
    }
    leaf(patch,root,"Combat","TargetLock.Combat",|v,p|parse_bool(v,p).map(|v|change(TargetLockSetting::Combat(v))));
}
