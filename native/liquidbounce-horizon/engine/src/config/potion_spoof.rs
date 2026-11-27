// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::potion_spoof::{PotionSpoof,EFFECTS};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum PotionSpoofSetting { Enabled(usize,bool),Level(usize,i32) }
impl PotionSpoofSetting {pub fn apply(self,module:&mut PotionSpoof) {match self {Self::Enabled(i,v)=>module.effects[i].enabled=v,Self::Level(i,v)=>module.effects[i].level=v}}}
pub fn serialize_potion_spoof(module:&PotionSpoof,common:&CommonSettings,format:JsonFormat,describe:&dyn Fn(&str)->String)->Result<Value,ConfigError> {
    let tree=Tree{format,describe};let mut values=tree.common(ModuleKind::PotionSpoof,common);
    for (i,(id,name)) in EFFECTS.iter().enumerate() {
        let key=format!("liquidbounce.module.potionSpoof.{id}");let effect=module.effects[i];
        let mut group=tree.group(name,"TOGGLEABLE",vec![tree.leaf("Enabled","BOOLEAN",json!(effect.enabled),&format!("{key}.enabled"),None),tree.ranged("Level","INT",json!(effect.level),json!({"from":1,"to":10}),"",&format!("{key}.level"))]);
        if format==JsonFormat::Interop {group["key"]=json!(key);group["description"]=json!(describe(&format!("{key}.description")));}
        values.push(group);
    }
    Ok(tree.group("PotionSpoof","TOGGLEABLE",values))
}
pub(super) fn parse_potion_spoof(patch:&mut SettingsPatch,root:&Nodes<'_>) {
    for (i,(_,name)) in EFFECTS.iter().enumerate() {let path=format!("PotionSpoof.{name}");
        let Some(node)=root.get(*name) else {continue;};
        let children=match nodes(node,name,&path) {Ok(v)=>v,Err(e)=>{patch.issues.push(e);continue;}};
        leaf(patch,&children,"Enabled",&format!("{path}.Enabled"),|value,path|parse_bool(value,path).map(|v|SettingChange::PotionSpoof(PotionSpoofSetting::Enabled(i,v))));
        leaf(patch,&children,"Level",&format!("{path}.Level"),|value,path|parse_int(value,path).map(|v|SettingChange::PotionSpoof(PotionSpoofSetting::Level(i,v))));
    }
}
