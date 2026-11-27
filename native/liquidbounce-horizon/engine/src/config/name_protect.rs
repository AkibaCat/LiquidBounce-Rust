// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::name_protect_runtime::NameProtectService;
use liquidbounce_behaviors::name_protect::{ColorMode,Coloring,Kind,Settings};
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum NameProtectSetting { Replacement(String), Friends(bool), Others(bool), Mode(Kind,ColorMode), Color(Kind,u32) }
impl NameProtectSetting {
    pub fn apply(self,service:&NameProtectService){
        let mut state=service.model.lock().expect("NameProtect");let s=&mut state.settings;
        match self{Self::Replacement(v)=>s.replacement=v,Self::Friends(v)=>s.friends=v,Self::Others(v)=>s.others=v,
            Self::Mode(k,v)=>s.colors[k.index()].mode=v,Self::Color(k,v)=>s.colors[k.index()].color=v}
    }
}
fn key(path:&str)->String{
    let path=path.split('.').map(|s|s[..1].to_ascii_lowercase()+&s[1..]).collect::<Vec<_>>().join(".");
    format!("liquidbounce.module.nameProtect.{path}")
}
fn describe_group(mut v:Value,path:&str,t:&Tree<'_>)->Value{
    if t.format==JsonFormat::Interop{let k=key(path);v["key"]=json!(k);v["description"]=json!((t.describe)(&format!("{k}.description")));}v
}
fn color_mode(s:Coloring,path:&str,t:&Tree<'_>)->Value{
    let mut choices=Map::new();
    choices.insert("Static".into(),describe_group(t.group("Static","CONFIGURABLE",vec![t.leaf("Color","COLOR",json!(s.color as i32),&key(&format!("{path}.Static.Color")),None)]),&format!("{path}.Static"),t));
    choices.insert("Rainbow".into(),describe_group(t.group("Rainbow","CONFIGURABLE",vec![]),&format!("{path}.Rainbow"),t));
    let mut v=t.mode(s.mode.tag(),choices);v["name"]=json!("ColorMode");describe_group(v,path,t)
}
pub fn serialize_name_protect(s:&Settings,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
    let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::NameProtect,common);
    values.push(tree.leaf("Replacement","TEXT",json!(s.replacement),&key("Replacement"),None));
    values.push(color_mode(s.colors[0],"ColorMode",&tree));
    for (name,enabled,i) in [("ObfuscateFriends",s.friends,1),("ObfuscateOthers",s.others,2)]{
        values.push(describe_group(tree.group(name,"TOGGLEABLE",vec![
            tree.leaf("Enabled","BOOLEAN",json!(enabled),&key(&format!("{name}.Enabled")),None),
            color_mode(s.colors[i],&format!("{name}.ColorMode"),&tree),
        ]),name,&tree));
    }
    // ModuleNameProtect.doNotIncludeAlways: local FILE and INTEROP include it.
    // A PUBLIC serializer must exclude the entire module, like TargetLock/GUICloser.
    Ok(tree.group("NameProtect","TOGGLEABLE",values))
}
fn color(v:&Value,p:&str)->Result<u32,ConfigError>{
    if let Some(s)=v.as_str(){
        let h=s.strip_prefix('#').unwrap_or(s);
        if !matches!(h.len(),6|8){return Err(ConfigError::new(p,"expected RRGGBB or AARRGGBB hex color"));}
        return u32::from_str_radix(h,16).map(|n|n|if h.len()==6{0xff000000}else{0}).map_err(|_|ConfigError::new(p,"invalid hex color"));
    }
    v.as_i64().map(|v|v as u32).or_else(||v.as_u64().map(|v|v as u32)).ok_or_else(||ConfigError::new(p,"expected color integer or hex string"))
}
fn parse_mode(patch:&mut SettingsPatch,v:&Value,path:&str,kind:Kind){
    let p=format!("NameProtect.{path}");
    match v.get("active").and_then(scalar_string).and_then(|s|ColorMode::parse(&s)){
        Some(mode)=>patch.changes.push(SettingChange::NameProtect(NameProtectSetting::Mode(kind,mode))),
        None=>patch.issues.push(ConfigError::new(&format!("{p}.active"),"missing or unknown case-sensitive color mode")),
    }
    if let Some(choices)=v.get("choices").and_then(Value::as_object){
        for name in ["Static","Rainbow"]{
            let p=format!("{p}.{name}");
            match choices.get(name).ok_or_else(||ConfigError::new(&p,"choice not found")).and_then(|v|nodes(v,name,&p)){
                Ok(choice) if name=="Static"=>leaf(patch,&choice,"Color",&format!("{p}.Color"),|v,p|color(v,p).map(|v|SettingChange::NameProtect(NameProtectSetting::Color(kind,v)))),
                Ok(_)=>{},Err(e)=>patch.issues.push(e),
            }
        }
    }else{patch.issues.push(ConfigError::new(&format!("{p}.choices"),"expected choices object"));}
    if let Err(e)=nodes(v,"ColorMode",&p){patch.issues.push(e);}
}
pub(super) fn parse_name_protect(patch:&mut SettingsPatch,root:&Nodes<'_>){
    leaf(patch,root,"Replacement","NameProtect.Replacement",|v,p|scalar_string(v).map(|v|SettingChange::NameProtect(NameProtectSetting::Replacement(v))).ok_or_else(||ConfigError::new(p,"expected text primitive")));
    if let Some(v)=root.get("ColorMode"){parse_mode(patch,v,"ColorMode",Kind::User);}
    for (name,kind) in [("ObfuscateFriends",Kind::Friend),("ObfuscateOthers",Kind::Other)]{
        let p=format!("NameProtect.{name}");
        if let Some(v)=root.get(name){match nodes(v,name,&p){
            Ok(group)=>{
                leaf(patch,&group,"Enabled",&format!("{p}.Enabled"),|v,p|parse_bool(v,p).map(|v|SettingChange::NameProtect(if kind==Kind::Friend{NameProtectSetting::Friends(v)}else{NameProtectSetting::Others(v)})));
                if let Some(v)=group.get("ColorMode"){parse_mode(patch,v,&format!("{name}.ColorMode"),kind);}
            },Err(e)=>patch.issues.push(e),
        }}
    }
}
#[cfg(test)]
mod name_protect_config_tests{
    use super::*;
    #[test]fn all_inactive_colors_and_toggles_roundtrip(){
        for format in [JsonFormat::File,JsonFormat::Interop]{
            let mut expected=Settings::default();expected.replacement="你😀".into();expected.friends=false;expected.others=true;
            for (i,c) in expected.colors.iter_mut().enumerate(){c.mode=ColorMode::Rainbow;c.color=0xff112233+i as u32;}
            let value=serialize_name_protect(&expected,&CommonSettings{enabled:false,hidden:false,key_bind:json!({"boundKey":"key.keyboard.unknown","action":"Toggle"})},format,&str::to_owned).unwrap();
            let patch=parse_settings(ModuleKind::NameProtect,&value).unwrap();assert!(patch.issues.is_empty(),"{:?}",patch.issues);
            let service=NameProtectService::default();for v in patch.changes{if let SettingChange::NameProtect(v)=v{v.apply(&service);}}
            assert_eq!(service.settings(),expected);
            let root=nodes(&value,"NameProtect","").unwrap();assert_eq!(root["ColorMode"]["name"],"ColorMode");
            assert_eq!(root["ObfuscateFriends"]["value"][1]["choices"]["Static"]["value"][0]["value"],json!(expected.colors[1].color as i32));
        }
    }
    #[test]fn malformed_mode_preserves_later_valid_values_and_unsigned_colors(){
        let value=json!({"name":"NameProtect","value":[{"name":"Replacement","value":"新名字"},
            {"name":"ColorMode","active":"rainbow","value":[],"choices":{"Static":{"name":"Static","value":[{"name":"Color","value":4294967295u64}]},"Rainbow":{"name":"Rainbow","value":[]}}},
            {"name":"ObfuscateOthers","value":[{"name":"Enabled","value":true}]}]});
        let patch=parse_settings(ModuleKind::NameProtect,&value).unwrap();assert_eq!(patch.issues.len(),1);
        let service=NameProtectService::default();for v in patch.changes{if let SettingChange::NameProtect(v)=v{v.apply(&service);}}
        let s=service.settings();assert_eq!(s.replacement,"新名字");assert!(s.others);assert_eq!(s.colors[0].mode,ColorMode::Static);assert_eq!(s.colors[0].color,0xffffffff);
        assert_eq!(color(&json!("#80442211"),"c").unwrap(),0x80442211);assert_eq!(color(&json!("abcdef"),"c").unwrap(),0xffabcdef);
    }
}
