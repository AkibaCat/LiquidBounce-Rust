// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNametags, ordered text/equipment and original enchantment abbreviation rules.
use serde_json::{json,Value};
use std::collections::{BTreeMap,BTreeSet,VecDeque};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Part{Distance,Ping,Name,Health,GameMode,BotMark,TotemPops}
impl Part{
    pub const ALL:[Self;7]=[Self::Distance,Self::Ping,Self::Name,Self::Health,Self::GameMode,Self::BotMark,Self::TotemPops];
    pub fn tag(self)->&'static str{match self{Self::Distance=>"Distance",Self::Ping=>"Ping",Self::Name=>"Name",Self::Health=>"Health",Self::GameMode=>"GameMode",Self::BotMark=>"BotMark",Self::TotemPops=>"TotemPops"}}
    pub fn parse(s:&str)->Option<Self>{Self::ALL.into_iter().find(|p|p.tag().eq_ignore_ascii_case(s))}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Slot{Mainhand,Offhand,Feet,Legs,Chest,Head,Body,Saddle}
impl Slot{
    pub const ALL:[Self;8]=[Self::Mainhand,Self::Offhand,Self::Feet,Self::Legs,Self::Chest,Self::Head,Self::Body,Self::Saddle];
    pub fn tag(self)->&'static str{match self{Self::Mainhand=>"Mainhand",Self::Offhand=>"Offhand",Self::Feet=>"Feet",Self::Legs=>"Legs",Self::Chest=>"Chest",Self::Head=>"Head",Self::Body=>"Body",Self::Saddle=>"Saddle"}}
    pub fn parse(s:&str)->Option<Self>{match s.to_ascii_lowercase().as_str(){"boots"=>Some(Self::Feet),"pants"=>Some(Self::Legs),"chestplate"=>Some(Self::Chest),"helmet"=>Some(Self::Head),_=>Self::ALL.into_iter().find(|p|p.tag().eq_ignore_ascii_case(s))}}
    pub fn ordinal(self)->usize{Self::ALL.iter().position(|s|*s==self).unwrap()}
}
#[derive(Clone,Debug,PartialEq)]
pub struct Settings{
    pub parts:Vec<Part>,pub slots:Vec<Slot>,pub skip_empty:bool,pub show_info:bool,
    pub highlight:bool,pub fill:u32,pub outline:u32,pub enchantment:bool,
    pub enchantment_scale:f32,pub max_count:i32,pub enchantment_radius:f32,
    pub enchantments:BTreeSet<String>,pub whitelist:bool,
    pub border_width:f32,pub background_radius:f32,pub scale:Vec<[f32;2]>,
}
impl Default for Settings{
    fn default()->Self{Self{parts:Part::ALL[..6].to_vec(),slots:vec![Slot::Mainhand,Slot::Head,Slot::Chest,Slot::Legs,Slot::Feet,Slot::Offhand],skip_empty:true,show_info:true,highlight:false,fill:0x64ff0000,outline:0,enchantment:true,enchantment_scale:0.8,max_count:4,enchantment_radius:1.,enchantments:BTreeSet::new(),whitelist:false,border_width:1.,background_radius:2.,scale:vec![[0.,1.],[200.,1.]]}}
}
impl Settings{
    pub fn draw(&self)->Value{json!({"slots":self.slots.iter().map(|s|s.ordinal()).collect::<Vec<_>>(),"skipEmpty":self.skip_empty,"showInfo":self.show_info,"highlight":self.highlight,"fill":self.fill,"outline":self.outline,"enchantment":self.enchantment,"enchantmentScale":self.enchantment_scale,"enchantmentRadius":self.enchantment_radius,"borderWidth":self.border_width,"backgroundRadius":self.background_radius})}
}
#[derive(Clone,Debug)]pub struct Selected{pub identity:u64,pub distance:f32,pub scale:f32,pub count:i32,pub facts:Value}
#[derive(Default)]pub struct Nametags{pub settings:Settings,pub selected:Vec<Selected>,pub perspective:Option<i32>,pub locale:String,pub cache:Abbreviations}
impl Nametags{
    pub fn clear(&mut self){self.selected.clear();self.perspective=None;}
    pub fn collect(&mut self,camera:[f64;3],entries:Vec<(Value,i32)>,perspective:i32){
        self.selected=entries.into_iter().filter_map(|(facts,count)|{
            let identity=facts["identity"].as_u64()?;let position=serde_json::from_value::<[f64;3]>(facts["position"].clone()).ok()?;
            let distance=(0..3).map(|i|(position[i]-camera[i]).powi(2)).sum::<f64>().sqrt()as f32;
            let scale=crate::tracers::curve::sample(&self.settings.scale,distance);
            (scale>0.01).then_some(Selected{identity,distance,scale,count,facts})
        }).collect();
        self.selected.sort_by(|a,b|a.distance.total_cmp(&b.distance));self.perspective=Some(perspective);
    }
}
/// Java strings are UTF-16 sequences. Keep those units through the comparator.
pub fn abbreviation(name:&str)->Vec<u16>{
    let chars=name.encode_utf16().collect::<Vec<_>>();if chars.len()<=3{return chars;}
    let words=chars.split(|c|*c==32).filter(|s|!s.is_empty()).collect::<Vec<_>>();
    if words.len()>=3{return words.iter().map(|w|w[0]).collect();}
    if words.len()==2{let mut result=words[0].iter().take(3).copied().collect::<Vec<_>>();result.extend(words[1].iter().take(3-result.len()).copied());return result;}
    words.first().map_or_else(Vec::new,|w|w.iter().take(3).copied().collect())
}
#[derive(Default)]pub struct Abbreviations{values:BTreeMap<(String,String),Vec<u16>>,order:VecDeque<(String,String)>}
impl Abbreviations{
    pub fn get(&mut self,id:&str,name:&str)->Vec<u16>{
        let key=(id.into(),name.into());self.order.retain(|k|k!=&key);self.order.push_back(key.clone());
        let result=self.values.entry(key).or_insert_with(||abbreviation(name)).clone();
        if self.order.len()>64&&let Some(old)=self.order.pop_front(){self.values.remove(&old);}result
    }
}
pub fn enchantment_cells(settings:&Settings,cache:&mut Abbreviations,enchantments:&Value)->Vec<Value>{
    if !settings.enchantment{return vec![];}
    let mut entries=enchantments.as_array().into_iter().flatten().filter_map(|e|{
        let id=e["id"].as_str()?;let level=e["level"].as_i64()?as i32;
        if level<=0||settings.enchantments.contains(id)!=settings.whitelist{return None;}
        let name=cache.get(id,e["name"].as_str()?);Some((level,name,e["curse"]==true))
    }).collect::<Vec<_>>();
    entries.sort_by(|a,b|b.0.cmp(&a.0).then_with(||a.1.cmp(&b.1)));
    let limit=settings.max_count.max(0)as usize;let more=entries.len()>limit;entries.truncate(limit);
    let mut cells=entries.into_iter().map(|(level,name,curse)|{
        let color=if curse{0xffff5555}else if level>=4{0xffffaa00}else if level==3{0xffffff55}else if level==2{0xff55ff55}else{u32::MAX};
        json!({"text":format!("{}{level}",String::from_utf16_lossy(&name)),"color":color,"curse":curse})
    }).collect::<Vec<_>>();if more&&let Some(last)=cells.last_mut(){*last=json!({"text":"...","color":0xffaaaaaau32,"curse":false});}cells
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn original_defaults_aliases_and_utf16_abbreviations(){let s=Settings::default();assert_eq!(s.parts.len(),6);assert_eq!(s.slots.iter().map(|s|s.ordinal()).collect::<Vec<_>>(),[0,5,4,3,2,1]);assert_eq!(Slot::parse("Chestplate"),Some(Slot::Chest));for(name,want)in[("Unbreaking","Unb"),("Aqua Affinity","Aqu"),("of Doom","ofD"),("Curse of Vanishing","CoV"),("a b c d","abcd"),("锋利","锋利")]{assert_eq!(abbreviation(name),want.encode_utf16().collect::<Vec<_>>());}assert_eq!(abbreviation("😀ab"),vec![0xd83d,0xde00,97]);}
    #[test]fn filter_level_order_ellipsis_and_cache_capacity(){let mut s=Settings::default();let mut c=Abbreviations::default();let e=json!([{"id":"x:a","name":"Aaa","level":1,"curse":false},{"id":"x:z","name":"Zzz","level":4,"curse":false},{"id":"x:b","name":"Bbb","level":4,"curse":true}]);s.max_count=2;let cells=enchantment_cells(&s,&mut c,&e);assert_eq!(cells[0]["text"],"Bbb4");assert_eq!(cells[1]["text"],"...");s.whitelist=true;assert!(enchantment_cells(&s,&mut c,&e).is_empty());s.enchantments.insert("x:z".into());assert_eq!(enchantment_cells(&s,&mut c,&e)[0]["color"],0xffffaa00u32);for i in 0..65{c.get(&i.to_string(),"abc");}assert_eq!(c.order.len(),64);assert_eq!(c.values.len(),64);}
    #[test]fn collection_keeps_stable_nearest_order_strict_scale_and_identity(){let mut m=Nametags::default();m.collect([0.;3],vec![(json!({"identity":4,"position":[2,0,0]}),3),(json!({"identity":7,"position":[1,0,0]}),1),(json!({"identity":9,"position":[1,0,0]}),1)],0);assert_eq!(m.selected.iter().map(|s|s.identity).collect::<Vec<_>>(),[7,9,4]);assert_eq!(m.selected[2].count,3);m.settings.scale=vec![[0.,0.01],[200.,0.01]];m.collect([0.;3],vec![(json!({"identity":1,"position":[0,0,0]}),1)],0);assert!(m.selected.is_empty());m.clear();assert!(m.perspective.is_none());}
}
