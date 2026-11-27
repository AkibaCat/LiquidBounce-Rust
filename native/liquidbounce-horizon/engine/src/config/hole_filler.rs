// SPDX-License-Identifier: GPL-3.0-or-later
//! HoleFiller settings with the complete production non-support BlockPlacer tree.
use super::*;
use crate::{hole_filler_runtime::HoleFillerRuntime,liquid_filler_runtime::LiquidFillerRuntime};
use liquidbounce_behaviors::hole_filler::FEATURES;
#[derive(Clone,Debug,PartialEq)]pub enum HoleFillerSetting{Value(String,Value),Placing(LiquidFillerSetting)}
impl HoleFillerSetting{
    pub fn apply(self,r:&mut HoleFillerRuntime,blocks:&[String])->Result<(),ConfigError>{match self{
        Self::Placing(v)=>{let mut proxy=LiquidFillerRuntime{settings:Default::default(),placer:std::mem::take(&mut r.placer)};let result=v.apply(&mut proxy,blocks);r.placer=proxy.placer;result.map_err(|mut e|{e.path=e.path.replace("LiquidFiller.Placer","HoleFiller.Placing");e})},
        Self::Value(p,v)=>{let path=format!("HoleFiller.{p}");match p.as_str(){
            "Features"=>r.settings.features=parse_multi_mask(&v,&path,&FEATURES)?,
            "Area"=>r.settings.area=parse_int(&v,&path)?,
            "Filter"|"Blocks"=>{let mut proxy=LiquidFillerRuntime::default();proxy.settings.whitelist=r.settings.whitelist;proxy.settings.blocks=r.settings.blocks.clone();LiquidFillerSetting::Value(p,v).apply(&mut proxy,blocks).map_err(|mut e|{e.path=e.path.replace("LiquidFiller","HoleFiller");e})?;r.settings.whitelist=proxy.settings.whitelist;r.settings.blocks=proxy.settings.blocks;},
            _=>return Err(ConfigError::new(&path,"unknown HoleFiller setting"))
        }Ok(())}
    }}
}
fn remap(v:&mut Value){match v{Value::Object(o)=>{if let Some(Value::String(k))=o.get_mut("key"){*k=k.replace("liquidbounce.module.liquidFiller.placer","liquidbounce.module.holeFiller.placing");}for v in o.values_mut(){remap(v);}},Value::Array(a)=>for v in a{remap(v);},_=>{}}}
pub fn serialize_hole_filler(r:&HoleFillerRuntime,c:&CommonSettings,f:JsonFormat,d:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
    let t=Tree{format:f,describe:d};let mut values=t.common(ModuleKind::HoleFiller,c);let key=|p:&str|format!("liquidbounce.module.holeFiller.{p}");
    let names=FEATURES.into_iter().enumerate().filter_map(|(i,n)|(r.settings.features&(1<<i)!=0).then_some(n)).collect::<Vec<_>>();
    let mut features=t.leaf("Features","MULTI_CHOOSE",json!(names),&key("features"),None);if f==JsonFormat::Interop{features["choices"]=json!(FEATURES);features["canBeNone"]=json!(true);features["isOrderSensitive"]=json!(false);}values.push(features);
    values.push(t.ranged("Area","INT",json!(r.settings.area),json!({"from":1,"to":5}),"",&key("area")));
    let mut filter=t.leaf("Filter","CHOOSE",json!(if r.settings.whitelist{"Whitelist"}else{"Blacklist"}),&key("filter"),None);if f==JsonFormat::Interop{filter["choices"]=json!(["Whitelist","Blacklist"]);}values.push(filter);
    values.push(t.leaf("Blocks","BLOCK",json!(r.settings.blocks),&key("blocks"),None));
    let proxy=LiquidFillerRuntime{settings:Default::default(),placer:r.placer.clone()};let describe=|k:&str|d(&k.replace("liquidbounce.module.liquidFiller.placer","liquidbounce.module.holeFiller.placing"));let mut tree=serialize_liquid_filler(&proxy,c,f,&describe)?;
    let mut placing=tree["value"].as_array_mut().and_then(|a|a.iter_mut().find(|v|v["name"]=="Placer")).map(Value::take).ok_or_else(||ConfigError::new("HoleFiller.Placing","shared placement tree changed"))?;placing["name"]=json!("Placing");remap(&mut placing);values.push(placing);Ok(t.group("HoleFiller","TOGGLEABLE",values))
}
pub(super) fn parse_hole_filler(p:&mut SettingsPatch,root:&Nodes<'_>){
    for name in["Features","Area","Filter","Blocks"]{if let Some(v)=root.get(name){match value_of(v,&format!("HoleFiller.{name}")){Ok(v)=>{let change=HoleFillerSetting::Value(name.into(),v.clone());match change.clone().apply(&mut HoleFillerRuntime::default(),&[]){Ok(())=>p.changes.push(SettingChange::HoleFiller(change)),Err(e)=>p.issues.push(e)}},Err(e)=>p.issues.push(e)}}}
    if let Some(v)=root.get("Placing"){let mut v=(*v).clone();v["name"]=json!("Placer");let proxy=json!({"name":"LiquidFiller","value":[v]});let root=nodes(&proxy,"LiquidFiller","LiquidFiller").unwrap();let mut patch=SettingsPatch{module:ModuleKind::LiquidFiller,changes:vec![],issues:vec![]};liquid_filler::parse_liquid_filler(&mut patch,&root);for c in patch.changes{if let SettingChange::LiquidFiller(v)=c{p.changes.push(SettingChange::HoleFiller(HoleFillerSetting::Placing(v)));}}p.issues.extend(patch.issues.into_iter().map(|mut e|{e.path=e.path.replace("LiquidFiller.Placer","HoleFiller.Placing");e}));}
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn hole_filler_config_complete_inactive_placing_roundtrip(){let mut source=HoleFillerRuntime::default();source.settings.features=0;source.settings.area=5;source.settings.whitelist=false;source.settings.blocks=vec!["minecraft:stone".into()];source.placer.settings.rotation_mode="None".into();source.placer.settings.normal_post_move=true;source.placer.settings.placements=6;source.placer.crystals.none_instant=true;source.placer.crystals.normal_instant=true;source.placer.target_render.enabled=true;source.placer.placed_render.out_time=614;let common=CommonSettings{enabled:true,hidden:false,key_bind:Value::Null};for f in[JsonFormat::File,JsonFormat::Interop]{let encoded=serialize_hole_filler(&source,&common,f,&str::to_owned).unwrap();let root=nodes(&encoded,"HoleFiller","HoleFiller").unwrap();let mut patch=SettingsPatch{module:ModuleKind::HoleFiller,changes:vec![],issues:vec![]};parse_hole_filler(&mut patch,&root);assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut result=HoleFillerRuntime::default();for c in patch.changes{if let SettingChange::HoleFiller(v)=c{v.apply(&mut result,&[]).unwrap();}}assert_eq!(result.settings,source.settings);assert_eq!(result.placer.settings,source.placer.settings);assert_eq!(result.placer.crystals,source.placer.crystals);assert_eq!(result.placer.rotations,source.placer.rotations);assert_eq!(result.placer.crystal_rotations,source.placer.crystal_rotations);assert_eq!(result.placer.target_render,source.placer.target_render);assert_eq!(result.placer.placed_render,source.placer.placed_render);}}
    #[test]fn hole_filler_bad_settings_preserve_placing_and_reject_unknown_blocks(){let mut r=HoleFillerRuntime::default();let before=r.placer.settings.clone();assert!(HoleFillerSetting::Placing(LiquidFillerSetting::Value("Placer.Cooldown".into(),json!({"from":8,"to":2}))).apply(&mut r,&[]).is_err());assert_eq!(r.placer.settings,before);assert!(HoleFillerSetting::Value("Blocks".into(),json!(["minecraft:missing"])).apply(&mut r,&["minecraft:stone".into()]).is_err());assert_eq!(r.settings.blocks,vec!["minecraft:obsidian".to_owned()]);}
}
