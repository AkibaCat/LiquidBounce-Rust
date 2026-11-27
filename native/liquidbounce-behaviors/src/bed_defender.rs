// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleBedDefender's slot order and ordered bed-layer target planner.
use crate::bed_plates::{self, Bed, BedPlates, SelfMode};
pub type Pos=[i32;3];
#[derive(Clone,Debug,PartialEq)]pub struct Settings{pub max_layers:i32,pub allow_chests:bool,pub requires_sneak:bool}
impl Default for Settings{fn default()->Self{Self{max_layers:1,allow_chests:false,requires_sneak:false}}}
#[derive(Clone,Debug,PartialEq)]pub struct Slot{pub index:u8,pub count:i32,pub hardness:f32,pub full:bool,pub chest:bool}
fn nearby(slot:u8,selected:u8)->i32{if slot==selected{i32::MIN}else if slot==40{i32::MIN+1}else{(i32::from(slot)-i32::from(selected)).abs()}}
pub fn slot(settings:&Settings,slots:&[Slot],selected:u8)->Option<u8>{
    let hardness=|s:&Slot|if s.hardness== -1.{f64::INFINITY}else{f64::from(s.hardness)};
    slots.iter().filter(|s|s.full||settings.allow_chests&&s.chest).min_by(|a,b|hardness(b).total_cmp(&hardness(a)).then_with(||b.count.cmp(&a.count)).then_with(||nearby(a.index,selected).cmp(&nearby(b.index,selected)))).map(|s|s.index)
}
fn center_distance(p:Pos,eye:[f64;3])->f64{(0..3).map(|i|(f64::from(p[i])+0.5-eye[i]).powi(2)).sum()}
fn box_distance(p:Pos,eye:[f64;3])->f64{(0..3).map(|i|{let lo=f64::from(p[i]);let d=if eye[i]<lo{lo-eye[i]}else if eye[i]>lo+1.{eye[i]-lo-1.}else{0.};d*d}).sum()}
/// Input beds retain section scanner order; minimum-distance and target ties
/// remain stable. Both bed parts are eligible, even if the opposite is missing.
pub fn targets(settings:&Settings,self_bed:&BedPlates,beds:&[Bed],armor:&[Option<u32>;4],eye:[f64;3],range:f64)->Vec<Pos>{
    let bed=beds.iter().filter(|b|self_bed.settings.self_mode==SelfMode::None||self_bed.is_self(b,armor)).min_by(|a,b|center_distance(a.pos,eye).total_cmp(&center_distance(b.pos,eye)));
    let Some(bed)=bed else{return vec![];};
    let mut positions=bed_plates::scan::layers(bed.pos,bed.facing,settings.max_layers).into_iter().filter(|(p,_)|box_distance(*p,eye)<=range*range).collect::<Vec<_>>();
    positions.sort_by(|(a,la),(b,lb)|la.cmp(lb).then_with(||center_distance(*b,eye).total_cmp(&center_distance(*a,eye))));
    let mut unique=vec![];for(p,_)in positions{if !unique.contains(&p){unique.push(p);}}unique
}
#[cfg(test)]mod tests{
    use super::*;
    fn bed(pos:Pos)->Bed{Bed{pos,facing:[0,0,1],center:pos.map(f64::from),item:"minecraft:red_bed".into(),dye:14,outline:0,surrounding:vec![],distance:0.}}
    fn item(index:u8,count:i32,hardness:f32,full:bool,chest:bool)->Slot{Slot{index,count,hardness,full,chest}}
    #[test]fn bed_defender_slots_use_unbreakable_then_count_then_effective_hotbar(){let mut s=Settings::default();let slots=[item(40,64,1.,true,false),item(2,1,-1.,true,false),item(6,64,50.,true,false)];assert_eq!(slot(&s,&slots,6),Some(2));let tied=[item(40,32,2.,true,false),item(2,32,2.,true,false),item(6,32,2.,true,false)];assert_eq!(slot(&s,&tied,6),Some(6));assert_eq!(slot(&s,&tied,5),Some(40));let chest=[item(40,64,5.,false,true)];assert_eq!(slot(&s,&chest,0),None);s.allow_chests=true;assert_eq!(slot(&s,&chest,0),Some(40));assert_eq!(slot(&s,&[item(0,10,1.,true,false),item(8,11,1.,true,false)],0),Some(8));}
    #[test]fn bed_defender_layer_order_is_farther_first_and_does_not_walk_down(){let s=Settings::default();let m=BedPlates::default();let actual=targets(&s,&m,&[bed([0;3])],&[None;4],[0.5;3],6.);assert_eq!(actual,vec![[0,0,-2],[0,1,-1],[-1,0,-1],[1,0,-1],[0,0,1],[0,1,0],[-1,0,0],[1,0,0]]);let mut s=s;s.max_layers=5;let actual=targets(&s,&m,&[bed([0;3])],&[None;4],[0.5;3],20.);assert!(actual.iter().all(|p|p[1]>=0));let unique=actual.iter().copied().collect::<std::collections::BTreeSet<_>>();assert_eq!(actual.len(),unique.len());}
    #[test]fn bed_defender_nearest_ties_retain_scan_order_and_none_defends_every_bed(){let s=Settings::default();let mut m=BedPlates::default();let beds=[bed([-3,0,0]),bed([3,0,0])];let eye=[0.5;3];assert_eq!(targets(&s,&m,&beds,&[None;4],eye,20.),targets(&s,&m,&beds[..1],&[None;4],eye,20.));m.set_mode(SelfMode::Manual);m.manual=[3,0,0];assert_eq!(targets(&s,&m,&beds,&[None;4],eye,20.),targets(&s,&m,&beds[1..],&[None;4],eye,20.));m.set_mode(SelfMode::Color);assert!(targets(&s,&m,&beds,&[None;4],eye,20.).is_empty());assert!(!targets(&s,&m,&beds,&[Some(0xffb02e26),None,None,None],eye,20.).is_empty());}
    #[test]fn bed_defender_range_uses_inclusive_unit_box_distance(){let s=Settings::default();let m=BedPlates::default();assert!(targets(&s,&m,&[bed([0;3])],&[None;4],[0.5;3],0.49).is_empty());assert_eq!(targets(&s,&m,&[bed([0;3])],&[None;4],[0.5;3],0.5),vec![[0,0,1],[0,1,0],[-1,0,0],[1,0,0]]);}
}
