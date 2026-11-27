// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete inactive-mode import/export for LiquidFiller's non-support Placer.
use super::*;
use crate::{liquid_filler_runtime::LiquidFillerRuntime,auto_walk_runtime::AutoWalkSettings};
use liquidbounce_behaviors::{liquid_filler::PlaceOrder,no_rotate_set::NoRotateSet};
use std::sync::LazyLock;
pub static SCHEMA:LazyLock<Vec<Value>>=LazyLock::new(||serde_json::from_str(include_str!("liquid_filler_schema.json")).expect("LiquidFiller schema"));
#[derive(Clone,Debug,PartialEq)]
pub enum LiquidFillerSetting{Value(String,Value),Rotation(bool,NoRotateSetSetting),Render(bool,placement_esp::PlacementSetting)}
fn join(a:&str,b:&str)->String{if a.is_empty(){b.into()}else{format!("{a}.{b}")}}
fn key(path:&str)->String{let mut k="liquidbounce.module.liquidFiller".to_owned();for p in path.split('.') {k.push('.');let mut c=p.chars();if let Some(a)=c.next(){k.extend(a.to_lowercase());k.extend(c);}}k}
fn find<'a>(ns:&'a[Value],prefix:&str,path:&str)->Option<&'a Value>{for n in ns{let p=join(prefix,n["name"].as_str()?);if p==path&&n.get("default").is_some(){return Some(n);}for child in ["children","modes"]{if let Some(a)=n[child].as_array(){if let Some(v)=find(a,&p,path){return Some(v);}}}}None}
fn validate(spec:&Value,v:&Value,p:&str,blocks:&[String])->Result<Value,ConfigError>{Ok(match spec["kind"].as_str().unwrap_or(""){
    "BOOLEAN"=>json!(parse_bool(v,p)?),"INT"=>json!(parse_int(v,p)?),"FLOAT"=>float_json(parse_float(v,p)?,p)?,
    "INT_RANGE"=>{let a=parse_int_pair(v,p)?;if a[0]>a[1]{return Err(ConfigError::new(p,"empty integer range"));}json!({"from":a[0],"to":a[1]})},
    "CHOICE"|"CHOOSE"=>{let tag=scalar_string(v).ok_or_else(||ConfigError::new(p,"expected choice"))?;spec["options"].as_array().unwrap().iter().find(|x|x.as_str().is_some_and(|x|x.eq_ignore_ascii_case(&tag))).cloned().ok_or_else(||ConfigError::new(p,"unknown choice"))?},
    "MULTI_CHOOSE"=>{let names=spec["options"].as_array().unwrap().iter().map(|v|v.as_str().unwrap()).collect::<Vec<_>>();let mask=parse_multi_mask(v,p,&names)?;let mask=if mask==0&&spec["canBeNone"]==false{(1u32<<names.len())-1}else{mask};json!(names.into_iter().enumerate().filter_map(|(i,n)|(mask&(1<<i)!=0).then_some(n)).collect::<Vec<_>>())},
    "BLOCK"=>{let a=v.as_array().ok_or_else(||ConfigError::new(p,"expected blocks array"))?;let mut out=vec![];for v in a{let id=scalar_string(v).ok_or_else(||ConfigError::new(p,"expected block id"))?;let id=if id.contains(':'){id}else{format!("minecraft:{id}")};if !blocks.is_empty()&&!blocks.contains(&id){return Err(ConfigError::new(p,"unknown block id"));}if !out.contains(&id){out.push(id);}}out.sort_by(|a,b|{let(an,ap)=a.split_once(':').unwrap();let(bn,bp)=b.split_once(':').unwrap();ap.cmp(bp).then(an.cmp(bn))});json!(out)},
    _=>return Err(ConfigError::new(p,"invalid LiquidFiller schema type"))
})}
impl LiquidFillerSetting{
    pub fn apply(self,r:&mut LiquidFillerRuntime,blocks:&[String])->Result<(),ConfigError>{match self{
        Self::Rotation(crystal,v)=>{let rotations=if crystal{&mut r.placer.crystal_rotations}else{&mut r.placer.rotations};let mut p=NoRotateSet{rotations:std::mem::take(rotations),..Default::default()};v.apply(&mut p);*rotations=p.rotations;},
        Self::Render(placed,v)=>v.apply(if placed{&mut r.placer.placed_render}else{&mut r.placer.target_render}),
        Self::Value(path,v)=>{
            let p=format!("LiquidFiller.{path}");let spec=find(&SCHEMA,"",&path).ok_or_else(||ConfigError::new(&p,"unknown LiquidFiller setting"))?;let v=validate(spec,&v,&p,blocks)?;
            let s=&mut r.placer.settings;let c=&mut r.placer.crystals;
            let boolean=||v.as_bool().unwrap();let int=||v.as_i64().unwrap()as i32;let float=||v.as_f64().unwrap()as f32;let tag=||v.as_str().unwrap().to_owned();let pair=||[v["from"].as_i64().unwrap()as i32,v["to"].as_i64().unwrap()as i32];
            match path.as_str(){
                "PlaceIn"=>r.settings.place_in=parse_multi_mask(&v,&p,&["Water","Lava"])?,"PlaceOrder"=>r.settings.order=PlaceOrder::from_tag(v.as_str().unwrap()).unwrap(),"UseSponge"=>r.settings.use_sponge=boolean(),"Filter"=>r.settings.whitelist=v=="Whitelist","Blocks"=>r.settings.blocks=serde_json::from_value(v.clone()).unwrap(),
                "Placer.Range"=>s.range=float(),"Placer.WallRange"=>s.wall_range=float(),"Placer.Cooldown"=>s.cooldown=pair(),"Placer.Swing"=>s.swing=tag(),"Placer.ConstructFailResult"=>s.construct_fail_result=boolean(),"Placer.Sneak"=>s.sneak=pair(),"Placer.Ignore"=>s.ignore=parse_multi_mask(&v,&p,&["OpenInventory","UsingItem"])?,"Placer.SlotResetDelay"=>s.slot_reset_delay=pair(),"Placer.RotationMode"=>s.rotation_mode=tag(),"Placer.RotationMode.Normal.PostMove"=>s.normal_post_move=boolean(),"Placer.RotationMode.None.PostMove"=>s.none_post_move=boolean(),"Placer.RotationMode.None.SendRotationPacket"=>s.send_rotation_packet=boolean(),"Placer.RotationMode.None.Placements"=>s.placements=int(),
                "Placer.DestroyCrystals.Enabled"=>c.enabled=boolean(),"Placer.DestroyCrystals.Range"=>c.range=float(),"Placer.DestroyCrystals.WallRange"=>c.wall_range=float(),"Placer.DestroyCrystals.Delay"=>c.delay=int(),"Placer.DestroyCrystals.SwingMode"=>c.swing_mode=tag(),"Placer.DestroyCrystals.RotationMode"=>c.rotation_mode=tag(),"Placer.DestroyCrystals.RotationMode.Normal.PostMove"=>c.normal_post_move=boolean(),"Placer.DestroyCrystals.RotationMode.Normal.Instant"=>c.normal_instant=boolean(),"Placer.DestroyCrystals.RotationMode.Normal.IgnoreOpenInventory"=>c.ignore_open_inventory=boolean(),"Placer.DestroyCrystals.RotationMode.None.PostMove"=>c.none_post_move=boolean(),"Placer.DestroyCrystals.RotationMode.None.Instant"=>c.none_instant=boolean(),"Placer.DestroyCrystals.RotationMode.None.SendRotationPacket"=>c.send_rotation_packet=boolean(),
                _=>return Err(ConfigError::new(&p,"unhandled LiquidFiller setting"))
            }
        }
    }Ok(())}
}
fn value(r:&LiquidFillerRuntime,path:&str)->Value{let s=&r.placer.settings;let c=&r.placer.crystals;match path{
    "PlaceIn"=>json!(["Water","Lava"].into_iter().enumerate().filter_map(|(i,n)|(r.settings.place_in&(1<<i)!=0).then_some(n)).collect::<Vec<_>>()),"PlaceOrder"=>json!(r.settings.order.tag()),"UseSponge"=>json!(r.settings.use_sponge),"Filter"=>json!(if r.settings.whitelist{"Whitelist"}else{"Blacklist"}),"Blocks"=>json!(r.settings.blocks),
    "Placer.Range"=>json!(s.range),"Placer.WallRange"=>json!(s.wall_range),"Placer.Cooldown"=>json!(s.cooldown),"Placer.Swing"=>json!(s.swing),"Placer.ConstructFailResult"=>json!(s.construct_fail_result),"Placer.Sneak"=>json!(s.sneak),"Placer.Ignore"=>json!(["OpenInventory","UsingItem"].into_iter().enumerate().filter_map(|(i,n)|(s.ignore&(1<<i)!=0).then_some(n)).collect::<Vec<_>>()),"Placer.SlotResetDelay"=>json!(s.slot_reset_delay),"Placer.RotationMode"=>json!(s.rotation_mode),"Placer.RotationMode.Normal.PostMove"=>json!(s.normal_post_move),"Placer.RotationMode.None.PostMove"=>json!(s.none_post_move),"Placer.RotationMode.None.SendRotationPacket"=>json!(s.send_rotation_packet),"Placer.RotationMode.None.Placements"=>json!(s.placements),
    "Placer.DestroyCrystals.Enabled"=>json!(c.enabled),"Placer.DestroyCrystals.Range"=>json!(c.range),"Placer.DestroyCrystals.WallRange"=>json!(c.wall_range),"Placer.DestroyCrystals.Delay"=>json!(c.delay),"Placer.DestroyCrystals.SwingMode"=>json!(c.swing_mode),"Placer.DestroyCrystals.RotationMode"=>json!(c.rotation_mode),"Placer.DestroyCrystals.RotationMode.Normal.PostMove"=>json!(c.normal_post_move),"Placer.DestroyCrystals.RotationMode.Normal.Instant"=>json!(c.normal_instant),"Placer.DestroyCrystals.RotationMode.Normal.IgnoreOpenInventory"=>json!(c.ignore_open_inventory),"Placer.DestroyCrystals.RotationMode.None.PostMove"=>json!(c.none_post_move),"Placer.DestroyCrystals.RotationMode.None.Instant"=>json!(c.none_instant),"Placer.DestroyCrystals.RotationMode.None.SendRotationPacket"=>json!(c.send_rotation_packet),_=>Value::Null}}
fn remap(v:&mut Value,old:&str,new:&str){match v{Value::Object(o)=>{if let Some(Value::String(k))=o.get_mut("key"){*k=k.replace(old,new);}for v in o.values_mut(){remap(v,old,new);}},Value::Array(a)=>for v in a{remap(v,old,new);},_=>{}}}
fn render(t:&Tree,n:&Value,prefix:&str,r:&LiquidFillerRuntime,c:&CommonSettings)->Result<Value,ConfigError>{
    let name=n["name"].as_str().unwrap();let path=join(prefix,name);let kind=n["kind"].as_str().unwrap();
    if kind=="PLACEMENT"{let mut v=placement_esp::placement_tree(t,if name=="PlacedRendering"{&r.placer.placed_render}else{&r.placer.target_render},"liquidFiller",&path)?;v["name"]=json!(name);return Ok(v);}
    if kind=="ROTATIONS"{
        let crystal=path.starts_with("Placer.DestroyCrystals.");let proxy=AutoWalkSettings{rotations:if crystal{r.placer.crystal_rotations.clone()}else{r.placer.rotations.clone()},..Default::default()};let new=key(&path);let d=|k:&str|(t.describe)(&k.replace("liquidbounce.module.autoWalk.rotations",&new));let mut tree=serialize_auto_walk(&proxy,c,t.format,&d)?;let mut rot=tree["value"].as_array_mut().and_then(|a|a.iter_mut().find(|v|v["name"]=="Rotations")).map(Value::take).ok_or_else(||ConfigError::new("LiquidFiller.Rotations","shared rotation tree changed"))?;remap(&mut rot,"liquidbounce.module.autoWalk.rotations",&new);return Ok(rot);
    }
    if kind=="CHOICE"{let mut v=t.group(name,"CHOICE",vec![]);v["active"]=value(r,&path);let mut modes=serde_json::Map::new();for n in n["modes"].as_array().unwrap(){modes.insert(n["name"].as_str().unwrap().into(),render(t,n,&path,r,c)?);}v["choices"]=Value::Object(modes);return Ok(v);}
    if let Some(children)=n["children"].as_array(){return Ok(t.group(name,kind,children.iter().map(|n|render(t,n,&path,r,c)).collect::<Result<_,_>>()?));}
    let mut v=value(r,&path);if kind=="FLOAT"{v=float_json(v.as_f64().unwrap()as f32,&format!("LiquidFiller.{path}"))?;}if kind=="INT_RANGE"{v=json!({"from":v[0],"to":v[1]});}
    let mut v=if let Some(range)=n.get("range"){t.ranged(name,kind,v,range.clone(),n["suffix"].as_str().unwrap_or(""),&key(&path))}else{t.leaf(name,kind,v,&key(&path),None)};
    if t.format==JsonFormat::Interop{if let Some(options)=n.get("options"){v["choices"]=options.clone();}if kind=="MULTI_CHOOSE"{v["canBeNone"]=n["canBeNone"].clone();v["isOrderSensitive"]=json!(false);}}
    Ok(v)
}
pub fn serialize_liquid_filler(r:&LiquidFillerRuntime,c:&CommonSettings,f:JsonFormat,d:&dyn Fn(&str)->String)->Result<Value,ConfigError>{let t=Tree{format:f,describe:d};let mut v=t.common(ModuleKind::LiquidFiller,c);for n in SCHEMA.iter(){v.push(render(&t,n,"",r,c)?);}Ok(t.group("LiquidFiller","TOGGLEABLE",v))}
fn push(p:&mut SettingsPatch,path:&str,v:&Value,spec:&Value){match validate(spec,v,&format!("LiquidFiller.{path}"),&[]){Ok(v)=>p.changes.push(SettingChange::LiquidFiller(LiquidFillerSetting::Value(path.into(),v))),Err(e)=>p.issues.push(e)}}
fn parse(p:&mut SettingsPatch,nodeset:&Nodes<'_>,schema:&[Value],prefix:&str){for spec in schema{let name=spec["name"].as_str().unwrap();let path=join(prefix,name);let Some(v)=nodeset.get(name)else{continue;};let kind=spec["kind"].as_str().unwrap();
    if kind=="PLACEMENT"{let change:fn(placement_esp::PlacementSetting)->SettingChange=if name=="PlacedRendering"{|v|SettingChange::LiquidFiller(LiquidFillerSetting::Render(true,v))}else{|v|SettingChange::LiquidFiller(LiquidFillerSetting::Render(false,v))};placement_esp::parse_placement(p,nodeset,name,&format!("LiquidFiller.{prefix}"),change);continue;}
    if kind=="CHOICE"{if let Some(active)=v.get("active"){push(p,&path,active,spec);}for m in spec["modes"].as_array().unwrap(){let mode=m["name"].as_str().unwrap();if let Some(v)=v["choices"].get(mode){match nodes(v,mode,&format!("LiquidFiller.{path}.{mode}")){Ok(n)=>parse(p,&n,m["children"].as_array().unwrap(),&join(&path,mode)),Err(e)=>p.issues.push(e)}}}continue;}
    if kind=="ROTATIONS"{let proxy=json!({"name":"AutoWalk","value":[v]});let root=nodes(&proxy,"AutoWalk","AutoWalk").unwrap();let mut out=SettingsPatch{module:ModuleKind::AutoWalk,changes:vec![],issues:vec![]};auto_walk::parse_auto_walk(&mut out,&root);let crystal=path.starts_with("Placer.DestroyCrystals.");for c in out.changes{if let SettingChange::AutoWalk(AutoWalkSetting::Rotation(v))=c{p.changes.push(SettingChange::LiquidFiller(LiquidFillerSetting::Rotation(crystal,v)));}}p.issues.extend(out.issues.into_iter().map(|mut e|{e.path=e.path.replace("AutoWalk.Rotations",&format!("LiquidFiller.{path}"));e}));continue;}
    if let Some(children)=spec["children"].as_array(){match nodes(v,name,&format!("LiquidFiller.{path}")){Ok(n)=>parse(p,&n,children,&path),Err(e)=>p.issues.push(e)}continue;}
    match value_of(v,&format!("LiquidFiller.{path}")){Ok(v)=>push(p,&path,v,spec),Err(e)=>p.issues.push(e)}
}}
pub(super) fn parse_liquid_filler(p:&mut SettingsPatch,root:&Nodes<'_>){parse(p,root,&SCHEMA,"")}

#[cfg(test)]
mod tests{
    use super::*;
    use liquidbounce_behaviors::no_rotate_set::SmoothMode;
    #[test]
    fn liquid_filler_config_roundtrip_keeps_inactive_modes_and_renderers(){
        let mut source=LiquidFillerRuntime::default();
        source.settings.place_in=2;source.settings.order=PlaceOrder::BottomTop;source.settings.use_sponge=true;
        source.placer.settings.rotation_mode="None".into();source.placer.settings.cooldown=[3,7];source.placer.settings.sneak=[2,5];source.placer.settings.slot_reset_delay=[7,11];
        source.placer.settings.normal_post_move=true;source.placer.settings.none_post_move=true;source.placer.settings.send_rotation_packet=true;source.placer.settings.placements=7;
        source.placer.rotations.active=SmoothMode::Sigmoid;source.placer.rotations.ticks_until_reset=11;
        source.placer.crystals.rotation_mode="None".into();source.placer.crystals.normal_post_move=true;source.placer.crystals.normal_instant=true;source.placer.crystals.ignore_open_inventory=false;source.placer.crystals.none_post_move=true;source.placer.crystals.none_instant=true;source.placer.crystals.send_rotation_packet=true;
        source.placer.crystal_rotations.active=SmoothMode::Acceleration;source.placer.crystal_rotations.reset_threshold=7.;
        source.placer.target_render.enabled=true;source.placer.target_render.in_time=315;source.placer.target_render.start_size=0.4;
        source.placer.placed_render.clump=false;source.placer.placed_render.out_time=932;source.placer.placed_render.color=0x10203040;
        let common=CommonSettings{enabled:true,hidden:false,key_bind:Value::Null};
        for format in [JsonFormat::Interop,JsonFormat::File]{
            let encoded=serialize_liquid_filler(&source,&common,format,&str::to_owned).unwrap();
            let root=nodes(&encoded,"LiquidFiller","LiquidFiller").unwrap();
            let mut patch=SettingsPatch{module:ModuleKind::LiquidFiller,changes:vec![],issues:vec![]};parse_liquid_filler(&mut patch,&root);assert!(patch.issues.is_empty(),"{:?}",patch.issues);
            let mut result=LiquidFillerRuntime::default();
            for change in patch.changes{if let SettingChange::LiquidFiller(v)=change{v.apply(&mut result,&[]).unwrap();}}
            let mut blocks=source.settings.blocks.clone();blocks.sort();let mut actual=result.settings.blocks.clone();actual.sort();assert_eq!(actual,blocks);
            assert_eq!(result.settings.place_in,2);assert_eq!(result.settings.order,PlaceOrder::BottomTop);assert!(result.settings.use_sponge);
            assert_eq!(result.placer.settings,source.placer.settings);assert_eq!(result.placer.crystals,source.placer.crystals);
            assert_eq!(result.placer.rotations,source.placer.rotations);assert_eq!(result.placer.crystal_rotations,source.placer.crystal_rotations);
            assert_eq!(result.placer.target_render,source.placer.target_render);assert_eq!(result.placer.placed_render,source.placer.placed_render);
        }
    }
    #[test]
    fn liquid_filler_config_toggle_preserves_target_and_rejects_unknown_blocks(){
        let mut r=LiquidFillerRuntime::default();r.placer.destroyer.target=Some(liquidbounce_behaviors::block_placer::Crystal{id:4,distance_squared:1.});
        LiquidFillerSetting::Value("Placer.DestroyCrystals.Enabled".into(),json!(false)).apply(&mut r,&[]).unwrap();assert!(r.placer.destroyer.target.is_some());
        assert!(LiquidFillerSetting::Value("Blocks".into(),json!(["minecraft:missing"])).apply(&mut r,&["minecraft:stone".into()]).is_err());
        assert!(LiquidFillerSetting::Value("Placer.Cooldown".into(),json!({"from":7,"to":2})).apply(&mut r,&[]).is_err());
    }
}
