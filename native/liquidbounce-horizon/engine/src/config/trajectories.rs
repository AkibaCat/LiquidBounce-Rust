// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::trajectories::{Trajectories,Spec,specs};
pub use liquidbounce_behaviors::trajectories::TrajectoriesSetting;
fn key(p:&str)->String{format!("liquidbounce.module.trajectories.{}",p.split('.').map(|s|s[..1].to_ascii_lowercase()+&s[1..]).collect::<Vec<_>>().join("."))}
fn groups<'a>(all:&'a[Spec],prefix:&str)->Vec<&'a str>{let mut names=vec![];for s in all{if let Some(r)=s.path.strip_prefix(prefix){let name=r.split('.').next().unwrap();if !names.contains(&name){names.push(name);}}}names}
pub fn serialize_trajectories(s:&Trajectories,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError>{
    fn build(t:&Tree,s:&Trajectories,all:&[Spec],p:&str,name:&str)->Result<Value,ConfigError>{
        let mut v=if let Some(sp)=all.iter().find(|sp|sp.path==p){
            let value=s.value(p);let mut v=if let Some([a,b])=sp.range{
                let range=if sp.kind=="INT"{json!({"from":a as i32,"to":b as i32})}else{json!({"from":float_json(a,p)?,"to":float_json(b,p)?})};
                let value=if sp.kind=="FLOAT"{float_json(value.as_f64().unwrap_or(0.)as f32,p)?}else{value};
                t.ranged(name,sp.kind,value,range,sp.suffix,&key(p))
            }else{t.leaf(name,sp.kind,value,&key(p),None)};
            if t.format==JsonFormat::Interop{
                if !sp.choices.is_empty(){v["choices"]=json!(sp.choices);if sp.kind=="MULTI_CHOOSE"{v["canBeNone"]=json!(p!="TrajectoryTypes");}}
                if sp.kind=="CURVE"{v["innerType"]=json!("VECTOR2_F");v["xAxis"]=json!({"label":"Distance","range":{"from":0,"to":200}});v["yAxis"]=json!({"label":"Scale","range":{"from":0.25,"to":4}});v["tension"]=json!(0.4);}
                if sp.kind=="VECTOR3_D"{v["showLocateButton"]=json!(false);}
                if p=="ShowDetailedInfo.TimeUnit"{v["aliases"]=json!(["DurationUnit"]);}
            }
            v
        }else{let prefix=format!("{p}.");let children=groups(all,&prefix).into_iter().map(|n|build(t,s,all,&format!("{prefix}{n}"),n)).collect::<Result<Vec<_>,_>>()?;t.group(name,"TOGGLEABLE",children)};
        if t.format==JsonFormat::Interop{v["key"]=json!(key(p));v["description"]=json!((t.describe)(&format!("{}.description",key(p))));}Ok(v)
    }
    let all=specs();let t=Tree{format,describe};let mut children=t.common(ModuleKind::Trajectories,common);
    for name in groups(&all,""){children.push(build(&t,s,&all,name,name)?);}
    Ok(t.group("Trajectories","TOGGLEABLE",children))
}
fn parsed(sp:&Spec,v:&Value,p:&str)->Result<Value,ConfigError>{
    fn vector(v:&Value,axes:&[&str],p:&str)->Result<Value,ConfigError>{
        let mut o=Map::new();for(i,k)in axes.iter().enumerate(){let x=v.get(*k).or_else(||v.get(i)).and_then(Value::as_f64).filter(|x|x.is_finite()).ok_or_else(||ConfigError::new(p,"expected finite vector component"))?;o.insert((*k).into(),json!(x));}Ok(Value::Object(o))
    }
    let result=match sp.kind{
        "BOOLEAN"=>json!(parse_bool(v,p)?),"INT"=>json!(parse_int(v,p)?),"FLOAT"=>json!(parse_float(v,p)?),
        "COLOR"=>{if let Some(hex)=v.as_str().map(|s|s.trim_start_matches('#')).filter(|s|matches!(s.len(),6|8)){let n=u32::from_str_radix(hex,16).map_err(|_|ConfigError::new(p,"expected ARGB color"))?;json!((n|if hex.len()==6{0xff000000}else{0})as i32)}else{json!(parse_int(v,p)?)}}
        "CHOOSE"=>{let s=scalar_string(v).ok_or_else(||ConfigError::new(p,"expected trajectory choice"))?;if !sp.choices.contains(&s.as_str()){return Err(ConfigError::new(p,"unknown trajectory choice"));}json!(s)}
        "MULTI_CHOOSE"=>{let a=v.as_array().ok_or_else(||ConfigError::new(p,"expected trajectory collection"))?;if sp.path=="TrajectoryTypes"&&a.is_empty(){return Err(ConfigError::new(p,"at least one trajectory type is required"));}let mut names=vec![];for x in a{let n=x.as_str().filter(|n|sp.choices.contains(n)).ok_or_else(||ConfigError::new(p,"unknown trajectory tag"))?;if !names.contains(&n){names.push(n);}}json!(names)}
        "VECTOR3_D"=>vector(v,&["x","y","z"],p)?,
        "CURVE"=>{let a=v.as_array().ok_or_else(||ConfigError::new(p,"expected scale curve"))?;let points=a.iter().map(|v|vector(v,&["x","y"],p)).collect::<Result<Vec<_>,_>>()?;json!(points)}
        _=>return Err(ConfigError::new(p,"unsupported trajectory value")),
    };
    if let Some([a,b])=sp.range{let n=result.as_f64().unwrap_or(f64::NAN);if !n.is_finite()||n<f64::from(a)||n>f64::from(b){return Err(ConfigError::new(p,"value outside trajectory range"));}}
    Ok(result)
}
pub(super) fn parse_trajectories(patch:&mut SettingsPatch,root:&Nodes<'_>){
    fn walk(patch:&mut SettingsPatch,root:&Nodes<'_>,all:&[Spec],prefix:&str){
        for name in groups(all,prefix){let p=format!("{prefix}{name}");let full=format!("Trajectories.{p}");
            if let Some(sp)=all.iter().find(|sp|sp.path==p){
                let alias=if p=="ShowDetailedInfo.TimeUnit"&&!root.contains_key(name){"DurationUnit"}else{name};
                leaf(patch,root,alias,&full,|v,path|parsed(sp,v,path).map(|v|SettingChange::Trajectories(TrajectoriesSetting(p.clone(),v))));
            }else if let Some(n)=root.get(name){match nodes(n,name,&full){Ok(n)=>walk(patch,&n,all,&format!("{p}.")),Err(e)=>patch.issues.push(e)}}
        }
    }
    walk(patch,root,&specs(),"");
}
