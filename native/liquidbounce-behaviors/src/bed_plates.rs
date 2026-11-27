// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleBedPlates, BedState and the four IsSelfBedMode implementations.
use serde_json::{Value,json};
use std::collections::BTreeSet;
#[path="bed_plates_scan.rs"]pub mod scan;
pub use crate::item_tags::{Background,Filter};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum FilterMode{Predefined,Custom}
impl FilterMode{pub fn tag(self)->&'static str{match self{Self::Predefined=>"Predefined",Self::Custom=>"Custom"}}pub fn parse(s:&str)->Option<Self>{match s{"Predefined"=>Some(Self::Predefined),"Custom"=>Some(Self::Custom),_=>None}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum SelfMode{None,Color,SpawnLocation,Manual}
impl SelfMode{pub const ALL:[Self;4]=[Self::None,Self::Color,Self::SpawnLocation,Self::Manual];pub fn tag(self)->&'static str{match self{Self::None=>"None",Self::Color=>"Color",Self::SpawnLocation=>"SpawnLocation",Self::Manual=>"Manual"}}pub fn parse(s:&str)->Option<Self>{Self::ALL.into_iter().find(|v|v.tag()==s)}}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]pub enum Slot{Feet,Legs,Chest,Head}
impl Slot{pub const ALL:[Self;4]=[Self::Feet,Self::Legs,Self::Chest,Self::Head];pub fn tag(self)->&'static str{match self{Self::Feet=>"Feet",Self::Legs=>"Legs",Self::Chest=>"Chest",Self::Head=>"Head"}}pub fn parse(s:&str)->Option<Self>{match s.to_ascii_lowercase().as_str(){"feet"|"boots"=>Some(Self::Feet),"legs"|"pants"=>Some(Self::Legs),"chest"|"chestplate"=>Some(Self::Chest),"head"|"helmet"=>Some(Self::Head),_=>None}}pub fn armor_index(self)->usize{match self{Self::Head=>0,Self::Chest=>1,Self::Legs=>2,Self::Feet=>3}}}
#[derive(Clone,Debug,PartialEq)]pub struct Settings{
    pub background:Background,pub color:u32,pub outline_color:u32,pub margin:f32,pub radius:f32,pub gradient:f32,pub glow:f32,
    pub outline:bool,pub max_layers:i32,pub show_bed:bool,pub text_shadow:bool,pub render_offset:[f64;3],pub scale:Vec<[f32;2]>,pub max_count:i32,
    pub highlight_unbreakable:bool,pub compact:bool,pub prevent_overlap:bool,pub filter_mode:FilterMode,pub blocks:BTreeSet<String>,pub filter:Filter,
    pub self_mode:SelfMode,pub slots:BTreeSet<Slot>,pub loose:bool,pub bed_distance:f32,pub track:String,pub untrack:String,pub ignore_adjacent:bool,
}
impl Default for Settings{fn default()->Self{Self{background:Background::Rect,color:0x80000000,outline_color:0,margin:2.,radius:4.,gradient:0.3,glow:0.3,outline:false,max_layers:5,show_bed:true,text_shadow:true,render_offset:[0.;3],scale:vec![[0.,1.],[200.,1.]],max_count:8,highlight_unbreakable:true,compact:true,prevent_overlap:true,filter_mode:FilterMode::Predefined,blocks:BTreeSet::new(),filter:Filter::Blacklist,self_mode:SelfMode::None,slots:BTreeSet::from([Slot::Head]),loose:false,bed_distance:24.,track:"key.keyboard.keypad.add".into(),untrack:"key.keyboard.keypad.subtract".into(),ignore_adjacent:false}}}
impl Settings{
    pub fn draw_settings(&self)->Value{json!({"background":self.background.tag(),"color":self.color,"outlineColor":self.outline_color,"margin":self.margin,"radius":self.radius,"gradient":self.gradient,"glow":self.glow,"showBed":self.show_bed,"textShadow":self.text_shadow,"compact":self.compact,"highlightUnbreakable":self.highlight_unbreakable,"preventOverlap":self.prevent_overlap,"maxCount":self.max_count})}
    pub fn options(&self)->Value{json!({"maxLayers":self.max_layers,"mode":self.self_mode.tag(),"track":self.track,"untrack":self.untrack})}
}
#[derive(Clone,Debug,PartialEq)]pub struct Surrounding{pub block:String,pub item:String,pub description:String,pub count:i32,pub layer:i32,pub hardness:f32,pub predefined:bool,pub requires_tool:bool}
impl Surrounding{
    pub fn from_json(v:&Value)->Option<Self>{Some(Self{block:v["block"].as_str()?.into(),item:v["item"].as_str()?.into(),description:v["description"].as_str()?.into(),count:v["count"].as_i64()? as i32,layer:v["layer"].as_i64()? as i32,hardness:v["hardness"].as_f64()? as f32,predefined:v["predefined"].as_bool()?,requires_tool:v["requiresTool"].as_bool()?})}
    fn json(&self)->Value{json!({"block":self.block,"item":self.item,"count":self.count,"layer":self.layer,"requiresTool":self.requires_tool})}
}
pub fn sort_surrounding(v:&mut[Surrounding]){v.sort_by(|a,b|a.layer.cmp(&b.layer).then_with(||b.count.cmp(&a.count)).then_with(||{let h=|s:&Surrounding|if s.hardness == -1.{f32::INFINITY}else{s.hardness};h(b).total_cmp(&h(a))}).then_with(||a.description.encode_utf16().cmp(b.description.encode_utf16())));}
pub fn compact(v:&[Surrounding])->Vec<Surrounding>{
    let mut result:Vec<Surrounding>=Vec::new();
    for s in v{if let Some(old)=result.iter_mut().find(|old|old.block==s.block){old.count=old.count.wrapping_add(s.count);old.layer=old.layer.min(s.layer);}else{let mut copy=s.clone();copy.layer=0;result.push(copy);}}
    // Upstream starts its count/layer pair at (0,0), so compact layers remain zero.
    // Its reference hash iteration order is process-specific; retain first input order.
    result
}
#[derive(Clone,Debug)]pub struct Bed{pub pos:[i32;3],pub facing:[i32;3],pub center:[f64;3],pub item:String,pub dye:usize,pub outline:u32,pub surrounding:Vec<Surrounding>,pub distance:f64}
impl Bed{pub fn from_json(v:&Value)->Option<Self>{let pos:[i32;3]=serde_json::from_value(v["pos"].clone()).ok()?;let facing:[i32;3]=serde_json::from_value(v["facing"].clone()).ok()?;Some(Self{pos,facing,center:[f64::from(pos[0])-f64::from(facing[0])*0.5+0.5,f64::from(pos[1])+1.,f64::from(pos[2])-f64::from(facing[2])*0.5+0.5],item:v["item"].as_str()?.into(),dye:v["dye"].as_u64()? as usize,outline:v["outline"].as_u64()? as u32,surrounding:v["surrounding"].as_array()?.iter().filter_map(Surrounding::from_json).collect(),distance:0.})}}
pub const DYE_RGB:[u32;16]=[0xF9FFFE,0xF9801D,0xC74EBD,0x3AB3DA,0xFED83D,0x80C71F,0xF38BAA,0x474F52,0x9D9D97,0x169C9C,0x8932B8,0x3C44AA,0x835432,0x5E7C16,0xB02E26,0x1D1D21];
pub fn closest_dye(rgb:u32)->usize{DYE_RGB.iter().enumerate().min_by_key(|(_,dye)|(0..3).map(|i|{let shift=i*8;let d=((rgb>>shift)&255)as i32-((**dye>>shift)&255)as i32;d*d}).sum::<i32>()).map_or(0,|(i,_)|i)}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Notice{pub kind:&'static str,pub pos:[i32;3]}
impl Notice{pub fn severity(&self)->&'static str{match self.kind{"missing"=>"ERROR","tracked"=>"SUCCESS",_=>"INFO"}}pub fn english(&self)->String{let[x,y,z]=self.pos;match self.kind{"missing"=>"Cannot find any bed around you! Please get close to your bed.".into(),"tracked"=>format!("Tracked bed position ({x}, {y}, {z})."),"worldChange"=>format!("Bed position ({x}, {y}, {z}) has been untracked due to world change."),_=>format!("Bed position ({x}, {y}, {z}) has been untracked.")}}}
pub struct BedPlates{pub settings:Settings,pub beds:Vec<Bed>,pub spawn:[f64;3],pub manual:[i32;3]}
impl Default for BedPlates{fn default()->Self{Self{settings:Default::default(),beds:Vec::new(),spawn:[f64::MAX;3],manual:[0;3]}}}
impl BedPlates{
    pub fn disable(&mut self){self.beds.clear();self.spawn=[f64::MAX;3];self.manual=[0;3];}
    pub fn set_mode(&mut self,mode:SelfMode){if self.settings.self_mode!=mode{match self.settings.self_mode{SelfMode::SpawnLocation=>self.spawn=[f64::MAX;3],SelfMode::Manual=>self.manual=[0;3],_=>{}}self.settings.self_mode=mode;}}
    pub fn world_change(&mut self)->Option<Notice>{self.beds.clear();if self.settings.self_mode==SelfMode::Manual{let n=Notice{kind:"worldChange",pos:self.manual};self.manual=[0;3];Some(n)}else{None}}
    pub fn packet_position(&mut self,old:[f64;3],raw:[f64;3]){if self.settings.self_mode==SelfMode::SpawnLocation&&(0..3).map(|i|(old[i]-raw[i]).powi(2)).sum::<f64>()<256.{self.spawn=raw;}}
    pub fn key(&mut self,key:&str,eye:[f64;3],beds:&[[i32;3]])->Option<Notice>{
        if self.settings.self_mode!=SelfMode::Manual{return None;}
        if key==self.settings.track{
            let nearest=beds.iter().filter(|p|(0..3).all(|i|p[i]>=((eye[i]-16.).floor()as i32)&&p[i]<=((eye[i]+16.).floor()as i32))).min_by(|a,b|{let d=|p:&&[i32;3]|(0..3).map(|i|(f64::from(p[i])+0.5-eye[i]).powi(2)).sum::<f64>();d(a).total_cmp(&d(b))});
            if let Some(pos)=nearest{self.manual=*pos;Some(Notice{kind:"tracked",pos:*pos})}else{Some(Notice{kind:"missing",pos:[0;3]})}
        }else if key==self.settings.untrack&&self.manual!=[0;3]{let n=Notice{kind:"untracked",pos:self.manual};self.manual=[0;3];Some(n)}else{None}
    }
    pub fn tick(&mut self,camera:[f64;3],mut beds:Vec<Bed>){for bed in &mut beds{bed.distance=(0..3).map(|i|(bed.center[i]-camera[i]).powi(2)).sum::<f64>().sqrt();}beds.sort_by(|a,b|a.distance.total_cmp(&b.distance));self.beds=beds;}
    pub fn is_self(&self,bed:&Bed,armor:&[Option<u32>;4])->bool{match self.settings.self_mode{
        SelfMode::None=>false,SelfMode::Manual=>bed.pos==self.manual||std::array::from_fn::<_,3,_>(|i|bed.pos[i]-bed.facing[i])==self.manual,
        SelfMode::SpawnLocation=>(0..3).map(|i|(self.spawn[i]-f64::from(bed.pos[i])).powi(2)).sum::<f64>()<=f64::from(self.settings.bed_distance*self.settings.bed_distance),
        SelfMode::Color=>self.settings.slots.iter().any(|slot|armor[slot.armor_index()].is_some_and(|rgb|if self.settings.loose{closest_dye(rgb)==bed.dye}else{DYE_RGB.get(bed.dye).is_some_and(|dye|rgb==(*dye|0xff000000))})),
    }}
    pub fn frame(&self,armor:&[Option<u32>;4])->Value{
        let s=&self.settings;let groups=self.beds.iter().filter_map(|bed|{
            let scale=crate::tracers::curve::sample(&s.scale,bed.distance as f32);
            if scale<0.01||self.is_self(bed,armor)||s.ignore_adjacent&&self.beds.iter().any(|other|other.pos!=bed.pos&&(0..3).map(|i|(i64::from(other.pos[i])-i64::from(bed.pos[i])).abs()).sum::<i64>()<=1){return None;}
            let surrounding=if s.compact{compact(&bed.surrounding)}else{bed.surrounding.clone()};
            let surrounding=surrounding.into_iter().filter(|b|match s.filter_mode{FilterMode::Predefined=>b.predefined,FilterMode::Custom=>s.blocks.contains(&b.block)==(s.filter==Filter::Whitelist)}).collect::<Vec<_>>();
            Some(json!({"position":bed.center,"distance":bed.distance,"scale":scale,"bed":bed.item,"outline":if s.outline{Some(bed.outline)}else{None},"surrounding":surrounding.iter().map(Surrounding::json).collect::<Vec<_>>()}))
        }).collect::<Vec<_>>();json!({"settings":s.draw_settings(),"offset":s.render_offset,"groups":groups})
    }
}
#[cfg(test)]mod tests{
    use super::*;
    fn bed(pos:[i32;3])->Bed{Bed{pos,facing:[0,0,1],center:pos.map(f64::from),item:"minecraft:red_bed".into(),dye:14,outline:0xff993333,surrounding:vec![],distance:0.}}
    #[test]fn independent_layers_never_walk_down_and_mirror_both_halves(){let scan=scan::layers([0;3],[1,0,0],2);assert_eq!(scan.len(),26);assert!(scan.contains(&([2,0,0],2)));assert!(scan.contains(&([-3,0,0],2)));assert!(!scan.iter().any(|(p,_)|p[1]<0));let first=scan::layers([0;3],[0,0,1],1);assert_eq!(first,vec![([0,0,1],1),([0,1,0],1),([-1,0,0],1),([1,0,0],1),([0,0,-2],1),([0,1,-1],1),([-1,0,-1],1),([1,0,-1],1)]);}
    #[test]fn self_modes_raw_position_strict_threshold_reset_and_manual_halves(){let mut m=BedPlates::default();m.set_mode(SelfMode::SpawnLocation);m.packet_position([0.;3],[16.,0.,0.]);assert_eq!(m.spawn,[f64::MAX;3]);m.packet_position([0.;3],[15.99,0.,0.]);assert_eq!(m.spawn,[15.99,0.,0.]);assert!(m.is_self(&bed([16,0,0]),&[None;4]));m.set_mode(SelfMode::Manual);assert_eq!(m.spawn,[f64::MAX;3]);assert!(m.is_self(&bed([0;3]),&[None;4]));assert_eq!(m.key("key.keyboard.keypad.add",[0.;3],&[[15,15,15],[-1,0,0]]).unwrap().pos,[-1,0,0]);assert!(m.is_self(&bed([-1,0,1]),&[None;4]));assert_eq!(m.world_change().unwrap().pos,[-1,0,0]);assert_eq!(m.manual,[0;3]);assert_eq!(m.world_change().unwrap().pos,[0;3]);m.manual=[4;3];m.set_mode(SelfMode::None);assert_eq!(m.manual,[0;3]);}
    #[test]fn dye_armor_slots_and_adjacent_unfiltered_beds(){let mut m=BedPlates::default();m.set_mode(SelfMode::Color);let b=bed([5;3]);assert!(!m.is_self(&b,&[Some(0xffb02e27),None,None,None]));m.settings.loose=true;assert!(m.is_self(&b,&[Some(0xffb02e27),None,None,None]));assert!(!m.is_self(&b,&[None,None,None,Some(0xffb02e26)]));m.settings.slots=BTreeSet::from([Slot::Feet]);assert!(m.is_self(&b,&[None,None,None,Some(0xffb02e26)]));m.set_mode(SelfMode::None);m.settings.ignore_adjacent=true;m.tick([0.;3],vec![bed([1,0,0]),bed([2,0,0]),bed([5,0,0])]);assert_eq!(m.frame(&[None;4])["groups"].as_array().unwrap().len(),1);}
    #[test]fn surrounding_sort_compact_bug_filter_and_empty_lists(){let make=|name:&str,count,layer,hardness|Surrounding{block:name.into(),item:name.into(),description:name.into(),count,layer,hardness,predefined:true,requires_tool:false};let mut v=vec![make("weak",2,1,1.),make("hard",2,1,5.),make("bedrock",2,1,-1.),make("weak",8,2,1.)];sort_surrounding(&mut v);assert_eq!(v.iter().map(|v|v.block.as_str()).collect::<Vec<_>>(),["bedrock","hard","weak","weak"]);let merged=compact(&v);assert!(merged.iter().all(|v|v.layer==0));assert_eq!(merged.iter().find(|v|v.block=="weak").unwrap().count,10);let mut m=BedPlates::default();m.settings.show_bed=false;m.settings.filter_mode=FilterMode::Custom;m.settings.filter=Filter::Whitelist;m.tick([0.;3],vec![bed([1;3])]);assert_eq!(m.frame(&[None;4])["groups"].as_array().unwrap().len(),1);}
}
