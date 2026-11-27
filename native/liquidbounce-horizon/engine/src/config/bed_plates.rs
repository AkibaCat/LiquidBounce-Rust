// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{bed_plates::{BedPlates,Settings,Background,Filter,FilterMode,SelfMode,Slot},camera_clip::ModifierKey};
use std::collections::BTreeSet;
#[derive(Clone,Debug,PartialEq)]pub struct BedPlatesSetting(pub String,pub Value);
fn error(p:&str,m:&str)->ConfigError{ConfigError::new(&format!("BedPlates.{p}"),m)}
fn text(v:&Value,p:&str)->Result<String,ConfigError>{scalar_string(v).ok_or_else(||error(p,"expected string"))}
fn double(v:&Value,p:&str)->Result<f64,ConfigError>{text(v,p)?.parse::<f64>().ok().filter(|v|v.is_finite()).ok_or_else(||error(p,"expected finite double"))}
fn color(v:&Value,p:&str)->Result<u32,ConfigError>{if let Some(raw)=v.as_str(){let s=raw.strip_prefix('#').unwrap_or(raw);if !matches!(s.len(),6|8){return Err(error(p,"expected hex RRGGBB or AARRGGBB"));}u32::from_str_radix(s,16).map(|v|v|if s.len()==6{0xff000000}else{0}).map_err(|_|error(p,"invalid color"))}else{v.as_i64().map(|v|v as u32).or_else(||v.as_u64().map(|v|v as u32)).ok_or_else(||error(p,"expected ARGB integer"))}}
impl BedPlatesSetting{
    pub fn apply(self,m:&mut BedPlates,registry:&[String])->Result<(),ConfigError>{
        let Self(p,v)=self;let s=&mut m.settings;let path=format!("BedPlates.{p}");match p.as_str(){
            "Background"=>s.background=Background::parse(&text(&v,&p)?).ok_or_else(||error(&p,"unknown case-sensitive mode"))?,
            "Background.Rect.Color"=>s.color=color(&v,&p)?,"Background.Rect.OutlineColor"=>s.outline_color=color(&v,&p)?,
            "Background.Rect.Margin"=>s.margin=parse_float(&v,&path)?,"Background.Rect.Radius"=>s.radius=parse_float(&v,&path)?,"Background.Rect.Gradient"=>s.gradient=parse_float(&v,&path)?,"Background.Rect.Glow"=>s.glow=parse_float(&v,&path)?,
            "Outline"=>s.outline=parse_bool(&v,&path)?,"MaxLayers"=>s.max_layers=parse_int(&v,&path)?,"ShowBed"=>s.show_bed=parse_bool(&v,&path)?,"TextShadow"=>s.text_shadow=parse_bool(&v,&path)?,
            "RenderOffset"=>s.render_offset=[double(&v["x"],&p)?,double(&v["y"],&p)?,double(&v["z"],&p)?],
            "Scale"=>s.scale=v.as_array().ok_or_else(||error(&p,"expected curve points"))?.iter().map(|point|Ok([parse_float(&point["x"],&path)?,parse_float(&point["y"],&path)?])).collect::<Result<_,ConfigError>>()?,
            "MaxCount"=>s.max_count=parse_int(&v,&path)?,"HighlightUnbreakable"=>s.highlight_unbreakable=parse_bool(&v,&path)?,"Compact"=>s.compact=parse_bool(&v,&path)?,"PreventOverlap"=>s.prevent_overlap=parse_bool(&v,&path)?,"IgnoreAdjacent"=>s.ignore_adjacent=parse_bool(&v,&path)?,
            "FilterMode"=>s.filter_mode=FilterMode::parse(&text(&v,&p)?).ok_or_else(||error(&p,"unknown case-sensitive mode"))?,
            "FilterMode.Custom.Filter"=>s.filter=Filter::parse(&text(&v,&p)?).ok_or_else(||error(&p,"unknown filter"))?,
            "FilterMode.Custom.Blocks"=>{let list=match &v{Value::Array(a)=>a.as_slice(),_=>std::slice::from_ref(&v)};let mut result=BTreeSet::new();for v in list{let raw=match v{Value::String(s)=>s.clone(),Value::Number(n)=>n.to_string(),_=>return Err(error(&p,"expected registry primitive"))};let id=match raw.split_once(':'){None=>format!("minecraft:{raw}"),Some(("",name))=>format!("minecraft:{name}"),Some(_)=>raw};result.insert(if registry.contains(&id){id}else{"minecraft:air".into()});}s.blocks=result;},
            "IgnoreSelfBed"=>{let mode=SelfMode::parse(&text(&v,&p)?).ok_or_else(||error(&p,"unknown case-sensitive mode"))?;m.set_mode(mode);},
            "IgnoreSelfBed.Color.Slots"=>{let list=match &v{Value::Array(a)=>a.as_slice(),Value::String(_)|Value::Number(_)|Value::Bool(_)=>std::slice::from_ref(&v),_=>&[]};let mut result=BTreeSet::new();for v in list{if let Some(slot)=Slot::parse(&text(v,&p)?){result.insert(slot);}}if result.is_empty(){result.extend(Slot::ALL);}s.slots=result;},
            "IgnoreSelfBed.Color.Loose"=>s.loose=parse_bool(&v,&path)?,"IgnoreSelfBed.SpawnLocation.BedDistance"=>s.bed_distance=parse_float(&v,&path)?,
            "IgnoreSelfBed.Manual.Track"|"IgnoreSelfBed.Manual.Untrack"=>{let key=ModifierKey::from_name(&text(&v,&p)?).ok_or_else(||error(&p,"unknown InputConstants key"))?.name;if p.ends_with(".Track"){s.track=key;}else{s.untrack=key;}},
            _=>return Err(error(&p,"unknown setting")),
        }Ok(())
    }
}
fn key(p:&str)->String{format!("liquidbounce.module.bedPlates.{}",p.split('.').map(|s|s[..1].to_ascii_lowercase()+&s[1..]).collect::<Vec<_>>().join("."))}
fn desc(mut value:Value,p:&str,t:&Tree<'_>)->Value{if t.format==JsonFormat::Interop{let k=key(p);value["key"]=json!(k);value["description"]=json!((t.describe)(&format!("{k}.description")));}value}
pub fn serialize_bed_plates(s:&Settings,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
    let t=Tree{format,describe};let leaf=|name:&str,ty:&str,value:Value,p:&str|t.leaf(name,ty,value,&key(p),None);
    let ranged=|name:&str,ty:&str,value:Value,from:f32,to:f32,p:&str|t.ranged(name,ty,value,if ty=="INT"{json!({"from":from as i32,"to":to as i32})}else{json!({"from":from,"to":to})},"",&key(p));
    let choice=|name:&str,value:&str,choices:Value,p:&str|{let mut v=leaf(name,"CHOOSE",json!(value),p);if format==JsonFormat::Interop{v["choices"]=choices;}v};
    let mode=|name:&str,active:&str,entries:Vec<(&str,Vec<Value>)>|{let mut choices=Map::new();for(n,values)in entries{choices.insert(n.into(),desc(t.group(n,"CONFIGURABLE",values),&format!("{name}.{n}"),&t));}let mut v=t.mode(active,choices);v["name"]=json!(name);desc(v,name,&t)};
    let mut values=t.common(ModuleKind::BedPlates,common);let mut panel=Vec::new();
    for(n,v)in[("Color",s.color),("OutlineColor",s.outline_color)]{panel.push(leaf(n,"COLOR",json!(v as i32),&format!("Background.Rect.{n}")));}
    for(n,v,to)in[("Margin",s.margin,100.),("Radius",s.radius,16.),("Gradient",s.gradient,1.),("Glow",s.glow,1.)]{panel.push(ranged(n,"FLOAT",float_json(v,n)?,0.,to,&format!("Background.Rect.{n}")));}
    values.push(mode("Background",s.background.tag(),vec![("Rect",panel),("Texture",vec![])]));
    values.push(leaf("Outline","BOOLEAN",json!(s.outline),"Outline"));values.push(ranged("MaxLayers","INT",json!(s.max_layers),1.,5.,"MaxLayers"));
    values.push(leaf("ShowBed","BOOLEAN",json!(s.show_bed),"ShowBed"));values.push(leaf("TextShadow","BOOLEAN",json!(s.text_shadow),"TextShadow"));
    let mut offset=leaf("RenderOffset","VECTOR3_D",json!({"x":s.render_offset[0],"y":s.render_offset[1],"z":s.render_offset[2]}),"RenderOffset");if format==JsonFormat::Interop{offset["useLocateButton"]=json!(false);}values.push(offset);
    let mut curve=leaf("Scale","CURVE",json!(s.scale.iter().map(|p|json!({"x":p[0],"y":p[1]})).collect::<Vec<_>>()),"Scale");if format==JsonFormat::Interop{curve["innerType"]=json!("VECTOR2_F");curve["xAxis"]=json!({"label":"Distance","range":{"from":0,"to":200}});curve["yAxis"]=json!({"label":"Scale","range":{"from":0.25,"to":4}});curve["tension"]=json!(0.4);}values.push(curve);
    values.push(ranged("MaxCount","INT",json!(s.max_count),1.,64.,"MaxCount"));
    for(n,v)in[("HighlightUnbreakable",s.highlight_unbreakable),("Compact",s.compact),("PreventOverlap",s.prevent_overlap)]{values.push(leaf(n,"BOOLEAN",json!(v),n));}
    let mut blocks=s.blocks.iter().collect::<Vec<_>>();blocks.sort_by(|a,b|{let(an,ap)=a.split_once(':').unwrap_or(("minecraft",a));let(bn,bp)=b.split_once(':').unwrap_or(("minecraft",b));ap.cmp(bp).then_with(||an.cmp(bn))});
    let mut registry=leaf("Blocks","REGISTRY_LIST",json!(blocks),"FilterMode.Custom.Blocks");if format==JsonFormat::Interop{registry["registry"]=json!("block");}
    values.push(mode("FilterMode",s.filter_mode.tag(),vec![("Predefined",vec![]),("Custom",vec![registry,choice("Filter",s.filter.tag(),json!(["Whitelist","Blacklist"]),"FilterMode.Custom.Filter")])]));
    let mut slots=leaf("Slots","MULTI_CHOOSE",json!(s.slots.iter().map(|v|v.tag()).collect::<Vec<_>>()),"IgnoreSelfBed.Color.Slots");if format==JsonFormat::Interop{slots["choices"]=json!(Slot::ALL.map(Slot::tag));slots["canBeNone"]=json!(false);slots["isOrderSensitive"]=json!(false);}
    values.push(mode("IgnoreSelfBed",s.self_mode.tag(),vec![("None",vec![]),("Color",vec![slots,leaf("Loose","BOOLEAN",json!(s.loose),"IgnoreSelfBed.Color.Loose")]),("SpawnLocation",vec![ranged("BedDistance","FLOAT",float_json(s.bed_distance,"BedDistance")?,16.,48.,"IgnoreSelfBed.SpawnLocation.BedDistance")]),("Manual",vec![leaf("Track","KEY",json!(s.track),"IgnoreSelfBed.Manual.Track"),leaf("Untrack","KEY",json!(s.untrack),"IgnoreSelfBed.Manual.Untrack")])]));
    values.push(leaf("IgnoreAdjacent","BOOLEAN",json!(s.ignore_adjacent),"IgnoreAdjacent"));Ok(t.group("BedPlates","TOGGLEABLE",values))
}
fn push(patch:&mut SettingsPatch,p:&str,v:&Value){let change=BedPlatesSetting(p.into(),v.clone());match change.clone().apply(&mut BedPlates::default(),&[]){Ok(())=>patch.changes.push(SettingChange::BedPlates(change)),Err(e)=>patch.issues.push(e)}}
pub(super) fn parse_bed_plates(patch:&mut SettingsPatch,root:&Nodes<'_>){
    for name in["Outline","MaxLayers","ShowBed","TextShadow","RenderOffset","Scale","MaxCount","HighlightUnbreakable","Compact","PreventOverlap","IgnoreAdjacent"]{if let Some(value)=root.get(name){if let Some(value)=value.get("value"){push(patch,name,value);}}}
    for(name,modes)in[("Background",vec![("Rect",vec!["Color","OutlineColor","Margin","Radius","Gradient","Glow"]),("Texture",vec![])]),("FilterMode",vec![("Predefined",vec![]),("Custom",vec!["Blocks","Filter"])]),("IgnoreSelfBed",vec![("None",vec![]),("Color",vec!["Slots","Loose"]),("SpawnLocation",vec!["BedDistance"]),("Manual",vec!["Track","Untrack"])])]{
        let Some(value)=root.get(name)else{continue;};if let Some(active)=value.get("active"){push(patch,name,active);}else{patch.issues.push(error(name,"missing active mode"));}
        if let Err(e)=nodes(value,name,&format!("BedPlates.{name}")){patch.issues.push(e);}
        let Some(choices)=value.get("choices").and_then(Value::as_object)else{patch.issues.push(error(name,"expected choices object"));continue;};
        for(mode,fields)in modes{let p=format!("{name}.{mode}");match choices.get(mode).ok_or_else(||error(&p,"choice not found")).and_then(|v|nodes(v,mode,&format!("BedPlates.{p}"))){Ok(n)=>for field in fields{if let Some(v)=n.get(field).and_then(|v|v.get("value")){push(patch,&format!("{p}.{field}"),v);}},Err(e)=>patch.issues.push(e)}}
    }
}
#[cfg(test)]mod bed_plates_config_tests{
    use super::*;
    #[test]fn every_inactive_mode_roundtrips_with_curves_and_registry_metadata(){for format in[JsonFormat::File,JsonFormat::Interop]{let mut expected=Settings::default();expected.background=Background::Texture;expected.color=0x80123456;expected.outline_color=0xffabcdef;expected.margin=7.5;expected.radius=6.;expected.gradient=0.5;expected.glow=0.7;expected.outline=true;expected.max_layers=3;expected.show_bed=false;expected.text_shadow=false;expected.render_offset=[0.123456789012,-2.,3.];expected.scale=vec![[0.,4.],[200.,0.25]];expected.max_count=20;expected.highlight_unbreakable=false;expected.compact=false;expected.prevent_overlap=false;expected.filter_mode=FilterMode::Custom;expected.blocks.insert("minecraft:stone".into());expected.filter=Filter::Whitelist;expected.self_mode=SelfMode::Manual;expected.slots=BTreeSet::from([Slot::Feet,Slot::Chest]);expected.loose=true;expected.bed_distance=41.;expected.track="key.keyboard.g".into();expected.untrack="key.mouse.99".into();expected.ignore_adjacent=true;
        let common=CommonSettings{enabled:false,hidden:false,key_bind:json!({"boundKey":"key.keyboard.unknown","action":"Toggle"})};let v=serialize_bed_plates(&expected,&common,format,&str::to_owned).unwrap();let patch=parse_settings(ModuleKind::BedPlates,&v).unwrap();assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut actual=BedPlates::default();for c in patch.changes{if let SettingChange::BedPlates(v)=c{v.apply(&mut actual,&["minecraft:stone".into()]).unwrap();}}assert_eq!(actual.settings,expected);if format==JsonFormat::Interop{let n=nodes(&v,"BedPlates","").unwrap();assert_eq!(n["RenderOffset"]["useLocateButton"],false);assert_eq!(n["Scale"]["tension"],0.4);}}
    }
    #[test]fn mode_transitions_empty_slots_aliases_and_invalid_values(){let mut m=BedPlates::default();m.set_mode(SelfMode::Manual);m.manual=[1;3];BedPlatesSetting("IgnoreSelfBed".into(),json!("Color")).apply(&mut m,&[]).unwrap();assert_eq!(m.manual,[0;3]);BedPlatesSetting("IgnoreSelfBed.Color.Slots".into(),json!(["Boots","Helmet"])).apply(&mut m,&[]).unwrap();assert_eq!(m.settings.slots,BTreeSet::from([Slot::Feet,Slot::Head]));BedPlatesSetting("IgnoreSelfBed.Color.Slots".into(),json!([])).apply(&mut m,&[]).unwrap();assert_eq!(m.settings.slots.len(),4);BedPlatesSetting("FilterMode.Custom.Blocks".into(),json!(["stone","unknown"])).apply(&mut m,&["minecraft:stone".into()]).unwrap();assert_eq!(m.settings.blocks,BTreeSet::from(["minecraft:stone".into(),"minecraft:air".into()]));assert!(BedPlatesSetting("Background".into(),json!("texture")).apply(&mut m,&[]).is_err());assert!(BedPlatesSetting("TextShadow".into(),json!({})).apply(&mut m,&[]).is_err());BedPlatesSetting("IgnoreSelfBed.Manual.Track".into(),json!("key.keyboard.87")).apply(&mut m,&[]).unwrap();assert_eq!(m.settings.track,"key.keyboard.keypad.add");}
}
