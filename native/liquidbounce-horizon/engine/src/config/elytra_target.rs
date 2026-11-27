// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::elytra_target_runtime::ElytraTargetRuntime;
use liquidbounce_behaviors::{auto_bow::PRIORITIES,elytra_target::Settings,target_renderer::TargetRendererSetting};
#[derive(Clone,Debug,PartialEq)]
pub enum ElytraTargetSetting{Value(String,Value),Rendering(TargetRendererSetting)}
const SPECS:&[(&str,&str,f64,f64,&str)]=&[
 ("Target.FOV","FLOAT",0.,180.,""),("Target.HurtTime","INT",0.,10.,""),("Target.Priority","MULTI_CHOOSE",0.,0.,""),
 ("Rotations.Sharp","BOOLEAN",0.,0.,""),("Rotations.IgnoreKillAuraRotation","BOOLEAN",0.,0.,""),("Rotations.Look","BOOLEAN",0.,0.,""),("Rotations.AutoDistance","BOOLEAN",0.,0.,""),
 ("Rotations.Prediction.Enabled","BOOLEAN",0.,0.,""),("Rotations.Prediction.Mode","CHOOSE",0.,0.,""),("Rotations.Prediction.GlidingOnly","BOOLEAN",0.,0.,""),("Rotations.Prediction.Multiplier","FLOAT_RANGE",0.5,3.,""),("Rotations.RotateAt","CHOOSE",0.,0.,""),
 ("AutoFirework.Enabled","BOOLEAN",0.,0.,""),("AutoFirework.UseMode","CHOOSE",0.,0.,""),("AutoFirework.ExtraDistance","FLOAT",5.,100.,"m"),("AutoFirework.SlotResetDelay","INT_RANGE",0.,20.,"ticks"),("AutoFirework.SyncCooldownWithKillAura","BOOLEAN",0.,0.,""),("AutoFirework.Cooldown","INT_RANGE",1.,50.,"ticks"),
 ("Safe","BOOLEAN",0.,0.,""),("AlwaysGlide","BOOLEAN",0.,0.,"")];
fn key(path:&str)->String{format!("liquidbounce.module.elytraTarget.{}",path.split('.').map(|p|{if p=="FOV"{return "fov".to_owned();}let mut s=p.to_owned();s[..1].make_ascii_lowercase();s}).collect::<Vec<_>>().join("."))}
fn choices(path:&str)->Vec<&'static str>{match path{"Target.Priority"=>PRIORITIES.map(|p|p.name()).to_vec(),"Rotations.Prediction.Mode"=>vec!["Simple","WithGravity"],"Rotations.RotateAt"=>vec!["Eyes","Center"],"AutoFirework.UseMode"=>vec!["Normal","Packet"],_=>vec![]}}
fn read(s:&Settings,path:&str)->Value{match path{
 "Safe"=>json!(s.safe),"AlwaysGlide"=>json!(s.always_glide),"Target.FOV"=>json!(s.fov),"Target.HurtTime"=>json!(s.hurt_time),"Target.Priority"=>json!(s.priority.iter().map(|p|p.name()).collect::<Vec<_>>()),
 "Rotations.Sharp"=>json!(s.sharp),"Rotations.IgnoreKillAuraRotation"=>json!(s.ignore_kill_aura),"Rotations.Look"=>json!(s.look),"Rotations.AutoDistance"=>json!(s.auto_distance),"Rotations.Prediction.Enabled"=>json!(s.prediction),"Rotations.Prediction.Mode"=>json!(if s.gravity{"WithGravity"}else{"Simple"}),"Rotations.Prediction.GlidingOnly"=>json!(s.gliding_only),"Rotations.Prediction.Multiplier"=>json!({"from":s.multiplier[0],"to":s.multiplier[1]}),"Rotations.RotateAt"=>json!(if s.center{"Center"}else{"Eyes"}),
 "AutoFirework.Enabled"=>json!(s.firework),"AutoFirework.UseMode"=>json!(if s.packet{"Packet"}else{"Normal"}),"AutoFirework.ExtraDistance"=>json!(s.extra_distance),"AutoFirework.SlotResetDelay"=>json!({"from":s.slot_reset[0],"to":s.slot_reset[1]}),"AutoFirework.SyncCooldownWithKillAura"=>json!(s.sync),"AutoFirework.Cooldown"=>json!({"from":s.cooldown[0],"to":s.cooldown[1]}),_=>Value::Null}}
fn validate(path:&str,v:&Value)->Result<(),ConfigError>{
 let p=format!("ElytraTarget.{path}");let Some((_,kind,lo,hi,_))=SPECS.iter().find(|s|s.0==path)else{return Err(ConfigError::new(&p,"unknown setting"));};
 let check=|a:f64,b:f64|{if a.is_finite()&&b.is_finite()&&a<=b&&a>=*lo&&b<=*hi{Ok(())}else{Err(ConfigError::new(&p,"value outside supported range"))}};
 match *kind{
 "BOOLEAN"=>{parse_bool(v,&p)?;},"FLOAT"=>{let a=parse_float(v,&p)?;check(f64::from(a),f64::from(a))?;},"INT"=>{let a=parse_int(v,&p)?;check(f64::from(a),f64::from(a))?;},"FLOAT_RANGE"=>{let a=parse_float_pair(v,&p)?;check(f64::from(a[0]),f64::from(a[1]))?;},"INT_RANGE"=>{let a=parse_int_pair(v,&p)?;check(f64::from(a[0]),f64::from(a[1]))?;},
 "CHOOSE"=>{let c=scalar_string(v).ok_or_else(||ConfigError::new(&p,"expected choice"))?;if !choices(path).iter().any(|s|s.eq_ignore_ascii_case(&c)){return Err(ConfigError::new(&p,"unknown choice"));}},
 "MULTI_CHOOSE"=>{let a=v.as_array().ok_or_else(||ConfigError::new(&p,"expected priority array"))?;if a.is_empty()||a.iter().any(|v|v.as_str().is_none_or(|s|!choices(path).iter().any(|c|c.eq_ignore_ascii_case(s)))){return Err(ConfigError::new(&p,"invalid priority"));}},_=>{}
 }Ok(())
}
impl ElytraTargetSetting{pub fn apply(self,r:&mut ElytraTargetRuntime)->Result<(),ConfigError>{match self{Self::Rendering(v)=>v.apply(&mut r.target_renderer),Self::Value(path,v)=>{
 validate(&path,&v)?;let p=format!("ElytraTarget.{path}");let s=&mut r.behavior.settings;let choice=||scalar_string(&v).unwrap_or_default();
 match path.as_str(){
 "Safe"=>s.safe=parse_bool(&v,&p)?,"AlwaysGlide"=>s.always_glide=parse_bool(&v,&p)?,"Target.FOV"=>s.fov=parse_float(&v,&p)?,"Target.HurtTime"=>s.hurt_time=parse_int(&v,&p)?,"Target.Priority"=>{s.priority.clear();for v in v.as_array().unwrap(){let c=PRIORITIES.into_iter().find(|p|p.name().eq_ignore_ascii_case(v.as_str().unwrap())).unwrap();if !s.priority.contains(&c){s.priority.push(c);}}},
 "Rotations.Sharp"=>s.sharp=parse_bool(&v,&p)?,"Rotations.IgnoreKillAuraRotation"=>s.ignore_kill_aura=parse_bool(&v,&p)?,"Rotations.Look"=>s.look=parse_bool(&v,&p)?,"Rotations.AutoDistance"=>s.auto_distance=parse_bool(&v,&p)?,"Rotations.Prediction.Enabled"=>s.prediction=parse_bool(&v,&p)?,"Rotations.Prediction.Mode"=>s.gravity=choice().eq_ignore_ascii_case("WithGravity"),"Rotations.Prediction.GlidingOnly"=>s.gliding_only=parse_bool(&v,&p)?,"Rotations.Prediction.Multiplier"=>s.multiplier=parse_float_pair(&v,&p)?,"Rotations.RotateAt"=>s.center=choice().eq_ignore_ascii_case("Center"),
 "AutoFirework.Enabled"=>s.firework=parse_bool(&v,&p)?,"AutoFirework.UseMode"=>s.packet=choice().eq_ignore_ascii_case("Packet"),"AutoFirework.ExtraDistance"=>s.extra_distance=parse_float(&v,&p)?,"AutoFirework.SlotResetDelay"=>s.slot_reset=parse_int_pair(&v,&p)?,"AutoFirework.SyncCooldownWithKillAura"=>s.sync=parse_bool(&v,&p)?,"AutoFirework.Cooldown"=>s.cooldown=parse_int_pair(&v,&p)?,_=>{}
 }
 }}Ok(())}}
fn children(t:&Tree<'_>,s:&Settings,prefix:&str)->Vec<Value>{
 let mut result=vec![];let mut groups=std::collections::BTreeSet::new();
 for &(path,kind,lo,hi,suffix)in SPECS{let Some(rest)=path.strip_prefix(prefix)else{continue;};if let Some((group,_))=rest.split_once('.'){
  if groups.insert(group){let next=format!("{prefix}{group}.");result.push(t.group(group,if matches!(group,"Prediction"|"AutoFirework"){"TOGGLEABLE"}else{"CONFIGURABLE"},children(t,s,&next)));}continue;
 }
 let mut value=if matches!(kind,"INT"|"FLOAT"|"INT_RANGE"|"FLOAT_RANGE"){t.ranged(rest,kind,read(s,path),json!({"from":lo,"to":hi}),suffix,&key(path))}else{t.leaf(rest,kind,read(s,path),&key(path),None)};
 if t.format==JsonFormat::Interop{let c=choices(path);if !c.is_empty(){value["choices"]=json!(c);}if kind=="MULTI_CHOOSE"{value["canBeNone"]=json!(false);value["isOrderSensitive"]=json!(true);}}
 result.push(value);
 }result
}
pub fn serialize_elytra_target(r:&ElytraTargetRuntime,c:&CommonSettings,f:JsonFormat,d:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
 let t=Tree{format:f,describe:d};let mut values=t.common(ModuleKind::ElytraTarget,c);values.extend(children(&t,&r.behavior.settings,""));let rendering=target_renderer::serialize_target_renderer(&r.target_renderer,f,d,"liquidbounce.module.elytraTarget.targetRendering")?;values.insert(values.len().saturating_sub(2),rendering);Ok(t.group("ElytraTarget","TOGGLEABLE",values))
}
fn descend(p:&mut SettingsPatch,n:&Nodes<'_>,prefix:&str){
 let mut groups=std::collections::BTreeSet::new();for &(path,_,_,_,_)in SPECS{let Some(rest)=path.strip_prefix(prefix)else{continue;};if let Some((group,_))=rest.split_once('.'){
  if groups.insert(group){if let Some(v)=n.get(group){match nodes(v,group,&format!("ElytraTarget.{prefix}{group}")){Ok(n)=>descend(p,&n,&format!("{prefix}{group}.")),Err(e)=>p.issues.push(e)}}}continue;
 }
 leaf(p,n,rest,&format!("ElytraTarget.{path}"),|v,_|{validate(path,v)?;Ok(SettingChange::ElytraTarget(ElytraTargetSetting::Value(path.into(),v.clone())))});
 }
}
pub(super)fn parse_elytra_target(p:&mut SettingsPatch,root:&Nodes<'_>){descend(p,root,"");if let Some(v)=root.get("TargetRendering"){let(settings,issues)=target_renderer::parse_target_renderer(v,"ElytraTarget.TargetRendering");p.changes.extend(settings.into_iter().map(|s|SettingChange::ElytraTarget(ElytraTargetSetting::Rendering(s))));p.issues.extend(issues);}}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_settings_and_renderer_roundtrip_in_both_formats(){
  let mut expected=ElytraTargetRuntime::default();let s=&mut expected.behavior.settings;
  s.safe=false;s.always_glide=true;s.sharp=true;s.ignore_kill_aura=false;s.look=true;s.auto_distance=false;
  s.prediction=false;s.gravity=true;s.gliding_only=false;s.multiplier=[0.75,2.5];s.center=true;
  s.fov=75.;s.hurt_time=3;s.priority=vec![PRIORITIES[2],PRIORITIES[1]];
  s.firework=false;s.packet=true;s.extra_distance=20.;s.slot_reset=[2,6];s.sync=true;s.cooldown=[10,20];
  TargetRendererSetting("Enabled".into(),json!(false)).apply(&mut expected.target_renderer);
  for f in[JsonFormat::File,JsonFormat::Interop]{
   let tree=serialize_elytra_target(&expected,&CommonSettings{enabled:false,hidden:false,key_bind:json!({"boundKey":"key.keyboard.unknown","action":"Toggle"})},f,&str::to_owned).unwrap();
   let patch=parse_settings(ModuleKind::ElytraTarget,&tree).unwrap();assert!(patch.issues.is_empty(),"{:?}",patch.issues);
   let mut restored=ElytraTargetRuntime::default();for c in patch.changes{if let SettingChange::ElytraTarget(c)=c{c.apply(&mut restored).unwrap();}}
   assert_eq!(expected.behavior.settings,restored.behavior.settings);assert!(!restored.target_renderer.boolean("Enabled"));
  }
  assert_eq!(key("Target.FOV"),"liquidbounce.module.elytraTarget.target.fov");
 }
 #[test]fn invalid_ranges_and_priorities_do_not_mutate_state(){
  let mut r=ElytraTargetRuntime::default();let before=r.behavior.settings.clone();
  for(path,v)in[("Target.Priority",json!([])),("Target.Priority",json!(["unknown"])),("Target.HurtTime",json!(11)),("AutoFirework.Cooldown",json!({"from":20,"to":10})),("Rotations.Prediction.Multiplier",json!({"from":0.1,"to":1.}))]{
   assert!(ElytraTargetSetting::Value(path.into(),v).apply(&mut r).is_err());assert_eq!(before,r.behavior.settings);
  }
 }
}
