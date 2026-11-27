// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::debug::{Debug,Filter,Settings};
#[derive(Clone,Debug,PartialEq)]pub struct DebugSetting(pub String,pub Value);
fn key(p:&str)->String{format!("liquidbounce.module.debug.{}",p.split('.').map(|s|s[..1].to_ascii_lowercase()+&s[1..]).collect::<Vec<_>>().join("."))}
impl DebugSetting{
 pub fn apply(self,m:&mut Debug)->Result<(),ConfigError>{let Self(p,v)=self;let path=format!("Debug.{p}");match p.as_str(){
  "Parameters"=>{let v=parse_bool(&v,&path)?;if m.settings.parameters!=v{m.settings.parameters=v;m.parameters_changed();}},
  "Global"=>{let v=parse_bool(&v,&path)?;if m.settings.global!=v{m.settings.global=v;m.global_changed();}},
  "Geometry"=>{let v=parse_bool(&v,&path)?;if m.settings.geometry!=v{m.settings.geometry=v;m.geometry_changed();}},
  "TitleFilter"=>{let text=scalar_string(&v).ok_or_else(||ConfigError::new(&path,"expected regex text"))?;let v=Filter::new(&text).map_err(|e|ConfigError::new(&path,e.to_string()))?;if m.settings.title_filter!=v{m.settings.title_filter=v;m.filter_changed();}},
  "DebugLog"=>m.settings.debug_log=parse_bool(&v,&path)?,"SimulatedPlayer.Enabled"=>m.settings.simulated_player=parse_bool(&v,&path)?,"Graph.Enabled"=>m.settings.graph=parse_bool(&v,&path)?,
  "Expires"|"Width"|"SimulatedPlayer.TicksToPredict"=>{let value=parse_int(&v,&path)?;let(lo,hi)=match p.as_str(){"Expires"=>(1,30),"Width"=>(120,400),_=>(5,100)};if !(lo..=hi).contains(&value){return Err(ConfigError::new(&path,format!("expected integer in {lo}..{hi}")));}match p.as_str(){"Expires"=>m.settings.expires=value,"Width"=>m.settings.width=value,_=>m.settings.ticks_to_predict=value}},
  "Graph.Curve"=>{let points=v.as_array().ok_or_else(||ConfigError::new(&path,"expected curve points"))?.iter().map(|p|{let x=parse_float(&p["x"],&path)?;let y=parse_float(&p["y"],&path)?;if !x.is_finite()||!y.is_finite(){return Err(ConfigError::new(&path,"expected finite curve point"));}Ok([x,y])}).collect::<Result<Vec<_>,ConfigError>>()?;m.settings.curve=points;},
  _=>return Err(ConfigError::new(&path,"unknown setting")),
 }Ok(())}
}
pub fn serialize_debug(s:&Settings,c:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
 let t=Tree{format,describe};let mut values=t.common(ModuleKind::Debug,c);let leaf=|n:&str,ty:&str,v:Value,p:&str|t.leaf(n,ty,v,&key(p),None);
 for(n,v)in[("Parameters",s.parameters),("Global",s.global),("Geometry",s.geometry)]{values.push(leaf(n,"BOOLEAN",json!(v),n));}
 for(n,v,lo,hi,suffix)in[("Expires",s.expires,1,30,"secs"),("Width",s.width,120,400,"px")]{values.push(t.ranged(n,"INT",json!(v),json!({"from":lo,"to":hi}),suffix,&key(n)));}
 values.push(leaf("TitleFilter","TEXT",json!(s.title_filter.pattern()),"TitleFilter"));values.push(leaf("DebugLog","BOOLEAN",json!(s.debug_log),"DebugLog"));
 let mut simulation=t.group("SimulatedPlayer","TOGGLEABLE",vec![leaf("Enabled","BOOLEAN",json!(s.simulated_player),"SimulatedPlayer.Enabled"),t.ranged("TicksToPredict","INT",json!(s.ticks_to_predict),json!({"from":5,"to":100}),"",&key("SimulatedPlayer.TicksToPredict"))]);
 let points=s.curve.iter().map(|p|Ok(json!({"x":float_json(p[0],"Debug.Graph.Curve.x")?,"y":float_json(p[1],"Debug.Graph.Curve.y")?}))).collect::<Result<Vec<_>,ConfigError>>()?;let mut curve=leaf("Curve","CURVE",json!(points),"Graph.Curve");if format==JsonFormat::Interop{curve["innerType"]=json!("VECTOR2_F");curve["xAxis"]=json!({"label":"X Axis","range":{"from":0,"to":180}});curve["yAxis"]=json!({"label":"Y Axis","range":{"from":40,"to":120}});curve["tension"]=json!(0.4);}
 let mut graph=t.group("Graph","TOGGLEABLE",vec![leaf("Enabled","BOOLEAN",json!(s.graph),"Graph.Enabled"),curve]);if format==JsonFormat::Interop{for(v,p)in[(&mut simulation,"SimulatedPlayer"),(&mut graph,"Graph")]{v["key"]=json!(key(p));v["description"]=json!(describe(&format!("{}.description",key(p))));}}values.extend([simulation,graph]);Ok(t.group("Debug","TOGGLEABLE",values))
}
fn add(patch:&mut SettingsPatch,n:&Nodes<'_>,name:&str,p:&str){if let Some(v)=n.get(name).and_then(|v|v.get("value")){let setting=DebugSetting(p.into(),v.clone());match setting.clone().apply(&mut Debug::default()){Ok(())=>patch.changes.push(SettingChange::Debug(setting)),Err(e)=>patch.issues.push(e)}}}
pub(super) fn parse_debug(patch:&mut SettingsPatch,root:&Nodes<'_>){for name in["Parameters","Global","Geometry","Expires","Width","TitleFilter","DebugLog"]{add(patch,root,name,name);}for(name,fields)in[("SimulatedPlayer",vec!["Enabled","TicksToPredict"]),("Graph",vec!["Enabled","Curve"]) ]{if let Some(v)=root.get(name){match nodes(v,name,&format!("Debug.{name}")){Ok(n)=>for field in fields{add(patch,&n,field,&format!("{name}.{field}"));},Err(e)=>patch.issues.push(e)}}}}
#[cfg(test)]mod debug_config_tests{use super::*;
 #[test]fn file_and_interop_roundtrip_inactive_subtrees(){let mut expected=Debug::default();expected.settings.global=true;expected.settings.expires=17;expected.settings.width=251;expected.settings.title_filter=Filter::new("(?i)debug|scaffold").unwrap();expected.settings.ticks_to_predict=73;expected.settings.curve=vec![[0.,85.],[180.,100.]];for format in[JsonFormat::File,JsonFormat::Interop]{let c=CommonSettings{enabled:false,hidden:false,key_bind:json!({"boundKey":"key.keyboard.unknown","action":"Toggle"})};let v=serialize_debug(&expected.settings,&c,format,&str::to_owned).unwrap();let p=parse_settings(ModuleKind::Debug,&v).unwrap();assert!(p.issues.is_empty(),"{:?}",p.issues);let mut actual=Debug::default();for change in p.changes{if let SettingChange::Debug(v)=change{v.apply(&mut actual).unwrap();}}assert_eq!(actual.settings,expected.settings);}}
 #[test]fn invalid_regex_and_range_are_isolated(){let input=json!({"name":"Debug","value":[{"name":"TitleFilter","value":"["},{"name":"Width","value":401},{"name":"Global","value":true}]});let p=parse_settings(ModuleKind::Debug,&input).unwrap();assert_eq!(p.issues.len(),2);let mut d=Debug::default();for change in p.changes{if let SettingChange::Debug(v)=change{v.apply(&mut d).unwrap();}}assert!(d.settings.global);assert_eq!(d.settings.width,180);assert_eq!(d.settings.title_filter.pattern(),".*");}
}
