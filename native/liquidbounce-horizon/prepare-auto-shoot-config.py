"""Build the owned AutoShoot adapter from the pinned shared AimPoint/Rotations tree.

No production registration or existing module is mutated by this preparation.
"""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
rod=(HERE/'engine/src/config/auto_rod.rs').read_text(encoding='utf8')
def renamed(s):return s.replace('AutoRod','AutoShoot').replace('autoRod','autoShoot')
text='''// SPDX-License-Identifier: GPL-3.0-or-later
// AimPoint tree and noncombat rotations derived verbatim from the pinned adapter.
use super::*;
use crate::{auto_shoot_runtime::AutoShootRuntime,auto_rod_runtime::NativeSamples};
use liquidbounce_behaviors::{auto_bow::PRIORITIES,clicking::Technique,no_rotate_set::NoRotateSet,target_renderer::TargetRendererSetting};
pub const PARTS:[&str;3]=["Head","Body","Feet"];
const OLD:&str="liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW:&str="liquidbounce.module.autoShoot.rotations";
#[derive(Clone,Debug,PartialEq)]pub enum AutoShootSetting{Value(String,Value),Point(String,Value),Rotation(NoRotateSetSetting),Rendering(TargetRendererSetting)}
'''
for anchor in ['fn choice(','fn multi<','fn floats(','fn ints(','fn remap(','fn keys(','fn key(','fn pair_f(','fn pair_i(','fn selected<']:
 text+=renamed(body(rod,anchor))+'\n'
point=rod[rod.index('            Self::Point(path, v) => {'):rod.index('            Self::Value(path, v) => {')]
text+='''impl AutoShootSetting{pub fn apply(self,r:&mut AutoShootRuntime,items:&[String])->Result<(),ConfigError>{match self{
Self::Rotation(v)=>{let mut proxy=NoRotateSet{rotations:std::mem::take(&mut r.rotations),..Default::default()};v.apply(&mut proxy);r.rotations=proxy.rotations;},
Self::Rendering(v)=>v.apply(&mut r.target_renderer),
'''+renamed(point)+'''Self::Value(path,v)=>{
let p=format!("AutoShoot.{path}");let s=&mut r.behavior.settings;
match path.as_str(){
"ThrowableType"=>s.custom=choice(&v,&["EggAndSnowball","Custom","Anything"],&p)?!=0,
"ThrowableType.Custom.Filter"=>s.whitelist=choice(&v,&["Whitelist","Blacklist"],&p)?==0,
"ThrowableType.Custom.Items"=>{
let values=if let Some(a)=v.as_array(){a.clone()}else{vec![v.clone()]};
let mut parsed=std::collections::BTreeSet::new();
for value in values{let name=scalar_string(&value).ok_or_else(||ConfigError::new(&p,"expected item identifier"))?;let id=match name.find(':'){None=>format!("minecraft:{name}"),Some(0)=>format!("minecraft{name}"),_=>name};parsed.insert(if items.contains(&id){id}else{"minecraft:air".into()});}s.items=parsed;
},
"GravityType"=>s.gravity=choice(&v,&["Auto","Linear","Projectile"],&p)? as u8,
"Clicker.Technique"=>r.clicker.settings.technique=if choice(&v,&["Human","Constant"],&p)?==0{Technique::Human}else{Technique::Constant},
"Clicker.CPS"=>r.clicker.settings.cps=ints(&v,&p)?,
"Clicker.MaxPerTick"=>r.clicker.settings.max_per_tick=parse_int(&v,&p)?,
"Target.Range"=>s.range=floats(&v,&p)?,
"Target.FOV"=>s.fov=parse_float(&v,&p)?,
"Target.HurtTime"=>s.hurt_time=parse_int(&v,&p)?,
"Target.Priority"=>{
let values=if let Some(a)=v.as_array(){a.clone()}else{vec![v.clone()]};let mut parsed=vec![];
for value in values{let index=choice(&value,&PRIORITIES.map(|v|v.name()),&p)?;if !parsed.contains(&PRIORITIES[index]){parsed.push(PRIORITIES[index]);}}
if parsed.is_empty(){return Err(ConfigError::new(&p,"priority cannot be empty"));}s.priority=parsed;
},
"AimOffThreshold"=>s.aim_threshold=parse_float(&v,&p)?,
"SwingMode"=>s.hide_swing=choice(&v,&["DoNotHide","HideForClient"],&p)?==1,
"SelectSlotAutomatically"=>s.select_slot=parse_bool(&v,&p)?,
"TicksUntilSlotReset"=>s.slot_reset=parse_int(&v,&p)?,
"ConsiderInventory"=>s.consider_inventory=parse_bool(&v,&p)?,
"RequiresKillAura"=>s.requires_aura=parse_bool(&v,&p)?,
"NotDuringCombat"=>s.not_during_combat=parse_bool(&v,&p)?,
"ConstantLag"=>s.constant_lag=parse_bool(&v,&p)?,
_=>return Err(ConfigError::new(&p,"unknown AutoShoot setting")),
}}}Ok(())}}
pub fn serialize_auto_shoot(r:&AutoShootRuntime,c:&CommonSettings,f:JsonFormat,d:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
let t=Tree{format:f,describe:d};let s=&r.behavior.settings;let a=&r.point_tracker.settings;
let b=|n:&str,v:bool,p:&str|t.leaf(n,"BOOLEAN",json!(v),&key(p),None);
let range=|n:&str,k:&str,v:Value,from:f64,to:f64,suffix:&str,p:&str|t.ranged(n,k,v,json!({"from":from,"to":to}),suffix,&key(p));
let choose=|n:&str,v:&str,options:&[&str],p:&str|{let mut value=t.leaf(n,"CHOOSE",json!(v),&key(p),None);if f==JsonFormat::Interop{value["choices"]=json!(options);}value};
let multi=|n:&str,v:Value,options:&[&str],p:&str|{let mut value=t.leaf(n,"MULTI_CHOOSE",v,&key(p),None);if f==JsonFormat::Interop{value["choices"]=json!(options);value["canBeNone"]=json!(true);}value};
let mut values=t.common(ModuleKind::AutoShoot,c);
let mut items=s.items.iter().collect::<Vec<_>>();items.sort_by(|a,b|{let(an,ap)=a.split_once(':').unwrap_or(("minecraft",a));let(bn,bp)=b.split_once(':').unwrap_or(("minecraft",b));ap.cmp(bp).then(an.cmp(bn))});
let custom=t.group("Custom","CONFIGURABLE",vec![choose("Filter",if s.whitelist{"Whitelist"}else{"Blacklist"},&["Whitelist","Blacklist"],"throwableType.custom.filter"),t.leaf("Items","ITEM",json!(items),&key("throwableType.custom.items"),None)]);
let mut throwable=t.group("ThrowableType","CHOICE",vec![]);throwable["active"]=json!(if s.custom{"Custom"}else{"EggAndSnowball"});throwable["choices"]=json!({"EggAndSnowball":t.group("EggAndSnowball","CONFIGURABLE",vec![]),"Custom":custom});values.push(throwable);
values.push(choose("GravityType",["Auto","Linear","Projectile"][usize::from(s.gravity.min(2))],&["Auto","Linear","Projectile"],"GravityType"));
values.push(crosshair_clicker::click_tree(&t,r.clicker.settings,"liquidbounce.module.autoShoot.clicker")?);
let mut priority=multi("Priority",json!(s.priority.iter().map(|p|p.name()).collect::<Vec<_>>()),&PRIORITIES.map(|p|p.name()),"Target.Priority");if f==JsonFormat::Interop{priority["canBeNone"]=json!(false);priority["isOrderSensitive"]=json!(true);}
values.push(t.group("Target","CONFIGURABLE",vec![range("Range","FLOAT_RANGE",pair_f(s.range),1.,256.,"m","Target.Range"),range("FOV","FLOAT",json!(s.fov),0.,180.,"","Target.FOV"),range("HurtTime","INT",json!(s.hurt_time),0.,10.,"","Target.HurtTime"),priority]));
'''
segment=rod[rod.index('    let dynamic = t.group('):rod.index('    let proxy = NoRotateSet {',rod.index('pub fn serialize_auto_rod('))]
text+=renamed(segment)
start=rod.index('    let proxy = NoRotateSet {',rod.index('pub fn serialize_auto_rod('))
text+=renamed(rod[start:rod.index('    values.push(range(',start)])
text+='''values.push(range("AimOffThreshold","FLOAT",json!(s.aim_threshold),0.5,10.,"","AimOffThreshold"));
values.push(choose("SwingMode",if s.hide_swing{"HideForClient"}else{"DoNotHide"},&["DoNotHide","HideForClient"],"SwingMode"));
values.push(target_renderer::serialize_target_renderer(&r.target_renderer,f,d,"liquidbounce.module.autoShoot.targetRendering")?);
values.push(b("SelectSlotAutomatically",s.select_slot,"SelectSlotAutomatically"));
values.push(range("TicksUntilSlotReset","INT",json!(s.slot_reset),0.,20.,"ticks","TicksUntilSlotReset"));
for(n,v)in[("ConsiderInventory",s.consider_inventory),("RequiresKillAura",s.requires_aura),("NotDuringCombat",s.not_during_combat),("ConstantLag",s.constant_lag)]{values.push(b(n,v,n));}
Ok(t.group("AutoShoot","TOGGLEABLE",values))}
fn add(p:&mut SettingsPatch,path:&str,v:&Value,is_point:bool){let s=if is_point{AutoShootSetting::Point(path.into(),v.clone())}else{AutoShootSetting::Value(path.into(),v.clone())};match s.clone().apply(&mut AutoShootRuntime::default(),&[]){Ok(())=>p.changes.push(SettingChange::AutoShoot(s)),Err(e)=>p.issues.push(e)}}
'''
for anchor in ['fn rotations(','fn point(']:text+=renamed(body(rod,anchor))+'\n'
text+='''pub(super)fn parse_auto_shoot(p:&mut SettingsPatch,root:&Nodes<'_>){
for n in ["GravityType","AimOffThreshold","SwingMode","SelectSlotAutomatically","TicksUntilSlotReset","ConsiderInventory","RequiresKillAura","NotDuringCombat","ConstantLag"]{leaf(p,root,n,&format!("AutoShoot.{n}"),|v,_|Ok(SettingChange::AutoShoot(AutoShootSetting::Value(n.into(),v.clone()))));}
if let Some(v)=root.get("ThrowableType"){
if let Some(active)=v.get("active"){add(p,"ThrowableType",active,false);}
if let Some(custom)=v["choices"].get("Custom").or_else(||v["choices"].get("Anything")){match nodes(custom,custom["name"].as_str().unwrap_or("Custom"),"AutoShoot.ThrowableType.Custom"){Ok(n)=>for name in ["Filter","Items"]{if let Some(v)=n.get(name){match value_of(v,name){Ok(v)=>add(p,&format!("ThrowableType.Custom.{name}"),v,false),Err(e)=>p.issues.push(e)}}},Err(e)=>p.issues.push(e)}}
}
for(group,names)in[("Clicker",&["Technique","CPS","MaxPerTick"][..]),("Target",&["Range","FOV","HurtTime","Priority"][..])]{if let Some(v)=root.get(group).or_else(||(group=="Clicker").then(||root.get("ClickScheduler")).flatten()){match nodes(v,v["name"].as_str().unwrap_or(group),&format!("AutoShoot.{group}")){Ok(n)=>for name in names{if let Some(v)=n.get(*name){match value_of(v,name){Ok(v)=>add(p,&format!("{group}.{name}"),v,false),Err(e)=>p.issues.push(e)}}},Err(e)=>p.issues.push(e)}}}
if let Some(v)=root.get("AimPoint"){point(p,v);}
if let Some(v)=root.get("Rotations"){rotations(p,v);}
if let Some(v)=root.get("TargetRendering"){let(values,errors)=target_renderer::parse_target_renderer(v,"AutoShoot.TargetRendering");p.changes.extend(values.into_iter().map(|v|SettingChange::AutoShoot(AutoShootSetting::Rendering(v))));p.issues.extend(errors);}
}
'''
(HERE/'engine/src/config/auto_shoot.rs').write_text(text,encoding='utf8')
print('Prepared owned AutoShoot config with the complete shared AimPoint and Rotations schemas; no registry changes')
