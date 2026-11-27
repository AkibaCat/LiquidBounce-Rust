// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::item_tags::{Background,Filter,MergeMode,Settings};
use std::collections::BTreeSet;
#[derive(Clone,Debug,PartialEq)]
pub enum ItemTagsSetting {Filter(Filter),Items(Value),Background(Background),Color(bool,u32),Panel(&'static str,f32),Scale(Vec<[f32;2]>),Cluster(Vec<[f32;2]>),Offset([f64;3]),RowLength(i32),PreventOverlap(bool),MergeMode(MergeMode),Shulker(&'static str,bool)}
impl ItemTagsSetting {
    pub fn apply(self,s:&mut Settings,registry:&[String])->Result<(),ConfigError>{
        match self {
            Self::Filter(v)=>s.filter=v,Self::Background(v)=>s.background=v,
            Self::Color(false,v)=>s.color=v,Self::Color(true,v)=>s.outline_color=v,
            Self::Panel(p,v)=>match p{"Margin"=>s.margin=v,"Radius"=>s.radius=v,"Gradient"=>s.gradient=v,"Glow"=>s.glow=v,_=>unreachable!()},
            Self::Scale(v)=>s.scale=v,Self::Cluster(v)=>s.cluster_entities=v,Self::Offset(v)=>s.render_offset=v,
            Self::RowLength(v)=>s.row_length=v,Self::PreventOverlap(v)=>s.prevent_overlap=v,Self::MergeMode(v)=>s.merge_mode=v,
            Self::Shulker(p,v)=>match p{"Enabled"=>s.shulker=v,"MergeStacks"=>s.merge_stacks=v,"ShowTitle"=>s.show_title=v,_=>unreachable!()},
            Self::Items(v)=>{
                let values=match &v{Value::Array(a)=>a.as_slice(),_=>std::slice::from_ref(&v)};let mut items=BTreeSet::new();
                for value in values{
                    let raw=match value{Value::String(s)=>s.clone(),Value::Number(n)=>n.to_string(),_=>return Err(ConfigError::new("ItemTags.Items","expected registry primitive"))};
                    let id=match raw.split_once(':'){None=>format!("minecraft:{raw}"),Some(("",path))=>format!("minecraft:{path}"),Some(_)=>raw};
                    items.insert(if registry.contains(&id){id}else{"minecraft:air".into()});
                }s.items=items;
            }
        }Ok(())
    }
}
fn key(path:&str)->String{format!("liquidbounce.module.itemTags.{}",path.split('.').map(|s|s[..1].to_ascii_lowercase()+&s[1..]).collect::<Vec<_>>().join("."))}
fn desc(mut v:Value,path:&str,t:&Tree<'_>)->Value{if t.format==JsonFormat::Interop{let k=key(path);v["key"]=json!(k);v["description"]=json!((t.describe)(&format!("{k}.description")));}v}
fn curve(t:&Tree<'_>,name:&str,points:&[[f32;2]],ylabel:&str,yfrom:f32,yto:f32)->Value{
    let mut v=t.leaf(name,"CURVE",json!(points.iter().map(|p|json!({"x":p[0],"y":p[1]})).collect::<Vec<_>>()),&key(name),None);
    if t.format==JsonFormat::Interop{v["innerType"]=json!("VECTOR2_F");v["xAxis"]=json!({"label":"Distance","range":{"from":0,"to":200}});v["yAxis"]=json!({"label":ylabel,"range":{"from":yfrom,"to":yto}});v["tension"]=json!(0.4);}v
}
pub fn serialize_item_tags(s:&Settings,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
    let t=Tree{format,describe};let mut values=t.common(ModuleKind::ItemTags,common);
    let choice=|name:&str,value:&str,choices:Value|{let mut v=t.leaf(name,"CHOOSE",json!(value),&key(name),None);if format==JsonFormat::Interop{v["choices"]=choices;}v};
    values.push(choice("Filter",s.filter.tag(),json!(["Whitelist","Blacklist"])));
    let mut items=s.items.iter().collect::<Vec<_>>();items.sort_by(|a,b|{let(an,ap)=a.split_once(':').unwrap_or(("minecraft",a));let(bn,bp)=b.split_once(':').unwrap_or(("minecraft",b));ap.cmp(bp).then_with(||an.cmp(bn))});
    let mut item_leaf=t.leaf("Items","REGISTRY_LIST",json!(items),&key("Items"),None);if format==JsonFormat::Interop{item_leaf["registry"]=json!("item");}values.push(item_leaf);
    let mut panel=vec![];
    for(name,color)in[("Color",s.color),("OutlineColor",s.outline_color)]{panel.push(t.leaf(name,"COLOR",json!(color as i32),&key(&format!("Background.Rect.{name}")),None));}
    for(name,value,to)in[("Margin",s.margin,100.),("Radius",s.radius,16.),("Gradient",s.gradient,1.),("Glow",s.glow,1.)]{panel.push(t.ranged(name,"FLOAT",float_json(value,name)?,json!({"from":0,"to":to}),"",&key(&format!("Background.Rect.{name}"))));}
    let mut choices=Map::new();choices.insert("Rect".into(),desc(t.group("Rect","CONFIGURABLE",panel),"Background.Rect",&t));choices.insert("Texture".into(),desc(t.group("Texture","CONFIGURABLE",vec![]),"Background.Texture",&t));
    let mut background=t.mode(s.background.tag(),choices);background["name"]=json!("Background");values.push(desc(background,"Background",&t));
    values.push(curve(&t,"Scale",&s.scale,"Scale",0.25,4.));
    let mut offset=t.leaf("RenderOffset","VECTOR3_D",json!({"x":s.render_offset[0],"y":s.render_offset[1],"z":s.render_offset[2]}),&key("RenderOffset"),None);if format==JsonFormat::Interop{offset["useLocateButton"]=json!(false);}values.push(offset);
    values.push(t.ranged("RowLength","INT",json!(s.row_length),json!({"from":1,"to":100}),"",&key("RowLength")));
    values.push(t.leaf("PreventOverlap","BOOLEAN",json!(s.prevent_overlap),&key("PreventOverlap"),None));
    values.push(curve(&t,"ClusterEntities",&s.cluster_entities,"Size",0.1,32.));
    values.push(choice("MergeMode",s.merge_mode.tag(),json!(MergeMode::ALL.map(MergeMode::tag))));
    values.push(desc(t.group("Shulker","TOGGLEABLE",[("Enabled",s.shulker),("MergeStacks",s.merge_stacks),("ShowTitle",s.show_title)].into_iter().map(|(name,value)|t.leaf(name,"BOOLEAN",json!(value),&key(&format!("Shulker.{name}")),None)).collect()),"Shulker",&t));
    Ok(t.group("ItemTags","TOGGLEABLE",values))
}
fn change(v:ItemTagsSetting)->SettingChange{SettingChange::ItemTags(v)}
fn color(v:&Value,p:&str)->Result<u32,ConfigError>{
    if let Some(s)=v.as_str(){let h=s.strip_prefix('#').unwrap_or(s);if !matches!(h.len(),6|8){return Err(ConfigError::new(p,"expected hex RRGGBB or AARRGGBB"));}return u32::from_str_radix(h,16).map(|v|v|if h.len()==6{0xff000000}else{0}).map_err(|_|ConfigError::new(p,"invalid color"));}
    let number=v.as_i64().map(|v|v as u32).or_else(||v.as_u64().map(|v|v as u32));number.ok_or_else(||ConfigError::new(p,"expected ARGB integer"))
}
fn points(v:&Value,p:&str)->Result<Vec<[f32;2]>,ConfigError>{v.as_array().ok_or_else(||ConfigError::new(p,"expected curve points"))?.iter().map(|point|Ok([parse_float(&point["x"],p)?,parse_float(&point["y"],p)?])).collect()}
fn double(v:&Value,p:&str)->Result<f64,ConfigError>{scalar_string(v).and_then(|s|s.parse().ok()).filter(|v:&f64|v.is_finite()).ok_or_else(||ConfigError::new(p,"expected finite double"))}
pub(super) fn parse_item_tags(patch:&mut SettingsPatch,root:&Nodes<'_>){
    leaf(patch,root,"Filter","ItemTags.Filter",|v,p|scalar_string(v).and_then(|s|Filter::parse(&s)).map(|v|change(ItemTagsSetting::Filter(v))).ok_or_else(||ConfigError::new(p,"unknown filter")));
    leaf(patch,root,"Items","ItemTags.Items",|v,_|Ok(change(ItemTagsSetting::Items(v.clone()))));
    leaf(patch,root,"MergeMode","ItemTags.MergeMode",|v,p|scalar_string(v).and_then(|s|MergeMode::parse(&s)).map(|v|change(ItemTagsSetting::MergeMode(v))).ok_or_else(||ConfigError::new(p,"unknown merge mode")));
    leaf(patch,root,"RowLength","ItemTags.RowLength",|v,p|parse_int(v,p).map(|v|change(ItemTagsSetting::RowLength(v))));
    leaf(patch,root,"PreventOverlap","ItemTags.PreventOverlap",|v,p|parse_bool(v,p).map(|v|change(ItemTagsSetting::PreventOverlap(v))));
    for(name,make)in[("Scale",ItemTagsSetting::Scale as fn(Vec<[f32;2]>)->ItemTagsSetting),("ClusterEntities",ItemTagsSetting::Cluster)]{leaf(patch,root,name,&format!("ItemTags.{name}"),|v,p|points(v,p).map(|v|change(make(v))));}
    leaf(patch,root,"RenderOffset","ItemTags.RenderOffset",|v,p|Ok(change(ItemTagsSetting::Offset([double(&v["x"],p)?,double(&v["y"],p)?,double(&v["z"],p)?]))));
    if let Some(v)=root.get("Background"){
        match v.get("active").and_then(scalar_string).and_then(|s|Background::parse(&s)){Some(v)=>patch.changes.push(change(ItemTagsSetting::Background(v))),None=>patch.issues.push(ConfigError::new("ItemTags.Background.active","unknown case-sensitive mode"))}
        if let Err(e)=nodes(v,"Background","ItemTags.Background"){patch.issues.push(e);}
        if let Some(choices)=v.get("choices").and_then(Value::as_object){for name in ["Rect","Texture"]{
            let path=format!("ItemTags.Background.{name}");match choices.get(name).ok_or_else(||ConfigError::new(&path,"choice not found")).and_then(|v|nodes(v,name,&path)){
                Ok(n) if name=="Rect"=>{
                    for(name,outline)in[("Color",false),("OutlineColor",true)]{leaf(patch,&n,name,&format!("{path}.{name}"),|v,p|color(v,p).map(|v|change(ItemTagsSetting::Color(outline,v))));}
                    for name in ["Margin","Radius","Gradient","Glow"]{leaf(patch,&n,name,&format!("{path}.{name}"),|v,p|parse_float(v,p).map(|v|change(ItemTagsSetting::Panel(name,v))));}
                },Ok(_)=>{},Err(e)=>patch.issues.push(e)
            }
        }}else{patch.issues.push(ConfigError::new("ItemTags.Background.choices","expected choices object"));}
    }
    if let Some(v)=root.get("Shulker"){match nodes(v,"Shulker","ItemTags.Shulker"){
        Ok(n)=>for name in ["Enabled","MergeStacks","ShowTitle"]{leaf(patch,&n,name,&format!("ItemTags.Shulker.{name}"),|v,p|parse_bool(v,p).map(|v|change(ItemTagsSetting::Shulker(name,v))));},Err(e)=>patch.issues.push(e)
    }}
}
#[cfg(test)]
mod item_tags_config_tests{
    use super::*;
    #[test]fn all_inactive_panel_values_curves_offset_and_shulker_roundtrip(){
        for format in [JsonFormat::File,JsonFormat::Interop]{
            let mut expected=Settings::default();expected.items.insert("minecraft:diamond".into());expected.filter=Filter::Whitelist;expected.background=Background::Texture;expected.color=0xffabcdef;expected.outline_color=0x80112233;expected.margin=12.25;expected.radius=8.5;expected.gradient=0.7;expected.glow=0.9;expected.scale=vec![[0.,4.],[12.,2.],[200.,0.25]];expected.cluster_entities=vec![[0.,0.1],[200.,32.]];expected.render_offset=[0.1234567890123,-12.5,3.];expected.row_length=4;expected.prevent_overlap=false;expected.merge_mode=MergeMode::ByItem;expected.shulker=true;expected.merge_stacks=false;expected.show_title=false;
            let common=CommonSettings{enabled:false,hidden:false,key_bind:json!({"boundKey":"key.keyboard.unknown","action":"Toggle"})};let value=serialize_item_tags(&expected,&common,format,&str::to_owned).unwrap();let patch=parse_settings(ModuleKind::ItemTags,&value).unwrap();assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut actual=Settings::default();for c in patch.changes{if let SettingChange::ItemTags(v)=c{v.apply(&mut actual,&["minecraft:diamond".into()]).unwrap();}}assert_eq!(actual,expected);
            let n=nodes(&value,"ItemTags","").unwrap();if format==JsonFormat::Interop{assert_eq!(n["Scale"]["innerType"],"VECTOR2_F");assert_eq!(n["RenderOffset"]["useLocateButton"],false);assert_eq!(n["Items"]["registry"],"item");}
        }
    }
    #[test]fn invalid_active_mode_keeps_valid_inactive_fields_and_registry_defaults(){
        let value=json!({"name":"ItemTags","value":[{"name":"Background","active":"texture","value":[],"choices":{"Rect":{"name":"Rect","value":[{"name":"Color","value":"#80123456"},{"name":"Glow","value":"0.5"}]},"Texture":{"name":"Texture","value":[]}}},{"name":"Items","value":["diamond","unknown"]},{"name":"Filter","value":"WHITELIST"},{"name":"RowLength","value":3}]});
        let patch=parse_settings(ModuleKind::ItemTags,&value).unwrap();assert_eq!(patch.issues.len(),1);let mut s=Settings::default();for c in patch.changes{if let SettingChange::ItemTags(v)=c{v.apply(&mut s,&["minecraft:diamond".into()]).unwrap();}}assert_eq!(s.background,Background::Rect);assert_eq!(s.color,0x80123456);assert_eq!(s.glow,0.5);assert!(s.items.contains("minecraft:air"));assert!(s.items.contains("minecraft:diamond"));assert_eq!(s.filter,Filter::Whitelist);assert_eq!(s.row_length,3);
    }
}
