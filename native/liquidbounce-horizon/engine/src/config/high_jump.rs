// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::high_jump::{HighJump,Mode,MODES};
#[derive(Clone,Copy,Debug,PartialEq)]pub enum HighJumpSetting{Mode(Mode),Motion(f32),Glide(bool)}
impl HighJumpSetting{pub fn apply(self,s:&mut HighJump){match self{Self::Mode(v)=>s.set_mode(v),Self::Motion(v)=>s.settings.motion=v,Self::Glide(v)=>s.settings.glide=v}}}
pub fn serialize_high_jump(s:&HighJump,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
    let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::HighJump,common);
    let mut choices=Map::new();
    choices.insert("Vanilla".into(),tree.group("Vanilla","CONFIGURABLE",vec![]));
    choices.insert("Vulcan-2.7.5".into(),tree.group("Vulcan-2.7.5","CONFIGURABLE",vec![tree.leaf("Glide","BOOLEAN",json!(s.settings.glide),"liquidbounce.module.highJump.mode.vulcan-2.7.5.glide",None)]));
    values.push(tree.mode(s.settings.mode.tag(),choices));
    values.push(tree.ranged("Motion","FLOAT",float_json(s.settings.motion,"HighJump.Motion")?,json!({"from":0.2,"to":10.0}),"","liquidbounce.module.highJump.motion"));
    Ok(tree.group("HighJump","TOGGLEABLE",values))
}
pub(super) fn parse_high_jump(patch:&mut SettingsPatch,root:&Nodes<'_>){
    leaf(patch,root,"Motion","HighJump.Motion",|v,p|parse_float(v,p).map(|v|SettingChange::HighJump(HighJumpSetting::Motion(v))));
    let Some(mode)=root.get("Mode")else{return;};
    match mode.get("active").and_then(scalar_string).and_then(|s|Mode::parse(&s)){
        Some(v)=>patch.changes.push(SettingChange::HighJump(HighJumpSetting::Mode(v))),
        None=>patch.issues.push(ConfigError::new("HighJump.Mode.active","missing or unknown case-sensitive mode")),
    }
    if let Err(e)=nodes(mode,"Mode","HighJump.Mode"){patch.issues.push(e);}
    let Some(choices)=mode.get("choices").and_then(Value::as_object)else{patch.issues.push(ConfigError::new("HighJump.Mode.choices","expected choices object"));return;};
    for tag in MODES{let path=format!("HighJump.Mode.{tag}");match choices.get(tag){
        Some(v)=>match nodes(v,tag,&path){Ok(v)if tag=="Vulcan-2.7.5"=>leaf(patch,&v,"Glide",&format!("{path}.Glide"),|v,p|parse_bool(v,p).map(|v|SettingChange::HighJump(HighJumpSetting::Glide(v)))),Ok(_)=>{},Err(e)=>patch.issues.push(e)},
        None=>patch.issues.push(ConfigError::new(&path,"choice not found")),
    }}
}
