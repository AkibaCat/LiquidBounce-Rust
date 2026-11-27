// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::spider::{Spider,Mode,MODES};
#[derive(Clone,Copy,Debug,PartialEq)]pub enum SpiderSetting{Mode(Mode),Motion(f32),JumpHeight(f32)}
impl SpiderSetting{pub fn apply(self,s:&mut Spider){match self{Self::Mode(v)=>s.set_mode(v),Self::Motion(v)=>s.settings.motion=v,Self::JumpHeight(v)=>s.settings.jump_height=v}}}
pub fn serialize_spider(s:&Spider,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
    let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::Spider,common);let mut choices=Map::new();
    choices.insert(MODES[0].into(),tree.group(MODES[0],"CONFIGURABLE",vec![tree.ranged("Motion","FLOAT",float_json(s.settings.motion,"Spider.Mode.Vanilla.Motion")?,json!({"from":0.05,"to":0.8}),"","liquidbounce.module.spider.mode.vanilla.motion")]));
    choices.insert(MODES[1].into(),tree.group(MODES[1],"CONFIGURABLE",vec![tree.ranged("JumpHeight","FLOAT",float_json(s.settings.jump_height,"Spider.Mode.Polar-2025-03-29.JumpHeight")?,json!({"from":0.42,"to":0.6}),"","liquidbounce.module.spider.mode.polar-2025-03-29.jumpHeight")]));
    choices.insert(MODES[2].into(),tree.group(MODES[2],"CONFIGURABLE",vec![]));values.push(tree.mode(s.settings.mode.tag(),choices));Ok(tree.group("Spider","TOGGLEABLE",values))
}
pub(super) fn parse_spider(patch:&mut SettingsPatch,root:&Nodes<'_>){
    let Some(mode)=root.get("Mode")else{return;};
    match mode.get("active").and_then(scalar_string).and_then(|s|Mode::parse(&s)){Some(v)=>patch.changes.push(SettingChange::Spider(SpiderSetting::Mode(v))),None=>patch.issues.push(ConfigError::new("Spider.Mode.active","unknown Spider mode"))}
    if let Err(e)=nodes(mode,"Mode","Spider.Mode"){patch.issues.push(e);}
    let Some(choices)=mode.get("choices").and_then(Value::as_object)else{patch.issues.push(ConfigError::new("Spider.Mode.choices","expected choices object"));return;};
    for tag in MODES{let path=format!("Spider.Mode.{tag}");match choices.get(tag){Some(v)=>match nodes(v,tag,&path){
        Ok(v)if tag==MODES[0]=>leaf(patch,&v,"Motion",&format!("{path}.Motion"),|v,p|parse_float(v,p).map(|v|SettingChange::Spider(SpiderSetting::Motion(v)))),
        Ok(v)if tag==MODES[1]=>leaf(patch,&v,"JumpHeight",&format!("{path}.JumpHeight"),|v,p|parse_float(v,p).map(|v|SettingChange::Spider(SpiderSetting::JumpHeight(v)))),Ok(_)=>{},Err(e)=>patch.issues.push(e)},None=>patch.issues.push(ConfigError::new(&path,"choice not found"))}}
}
