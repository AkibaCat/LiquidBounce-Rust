// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleHoleFiller target collection, inventory budget and movement-angle rules.
use crate::hole_tracker::{Hole,Pos,Region};
use std::cmp::Ordering;
pub const FEATURES:[&str;5]=["Smart","PreventSelfFill","OnlyWhenSelfInHole","CheckMovement","Only1x1"];
pub const SMART:u32=1;pub const PREVENT_SELF_FILL:u32=2;pub const ONLY_WHEN_SELF_IN_HOLE:u32=4;pub const CHECK_MOVEMENT:u32=8;pub const ONLY_ONE_BY_ONE:u32=16;
#[derive(Clone,Debug,PartialEq)]pub struct Settings{pub features:u32,pub area:i32,pub whitelist:bool,pub blocks:Vec<String>}
impl Default for Settings{fn default()->Self{Self{features:SMART|PREVENT_SELF_FILL|CHECK_MOVEMENT,area:2,whitelist:true,blocks:vec!["minecraft:obsidian".into()]}}}
impl Settings{pub fn has(&self,f:u32)->bool{self.features&f!=0}pub fn allows(&self,b:&str)->bool{self.blocks.iter().any(|v|v==b)==self.whitelist}}
#[derive(Clone,Debug,PartialEq)]pub struct Slot{pub index:u8,pub count:i32,pub block:Option<String>}
pub fn slot(s:&Settings,slots:&[Slot],selected:u8)->Option<u8>{slots.iter().filter(|v|v.block.as_deref().is_some_and(|b|s.allows(b))).min_by_key(|v|crate::liquid_filler::nearby(v.index,selected)).map(|v|v.index)}
pub fn available(s:&Settings,slots:&[Slot])->i32{slots.iter().filter(|v|v.block.as_deref().is_some_and(|b|s.allows(b))).fold(0i32,|n,v|n.wrapping_add(v.count))}
#[derive(Clone,Copy,Debug,PartialEq)]pub struct Entity{pub position:[f64;3],pub previous:[f64;3]}
fn block_pos(p:[f64;3])->Pos{p.map(|n|n.floor()as i32)}
fn allows_self(s:&Settings,h:Hole,player:[f64;3],inside:bool,region:Region)->bool{!s.has(PREVENT_SELF_FILL)||f64::from(h.position[1])+1.>player[1]||inside||!h.region().intersects(region)}
pub fn movement_angle(h:Hole,e:Entity)->f64{
    // BoundingBox.getCenter is an integer BlockPos. JOML normalize deliberately
    // carries NaN for zero vectors, and the source compares radians to 0.866.
    let center=[f64::from(h.position[0])+f64::from(h.size[0])*0.5,f64::from(h.position[2])+f64::from(h.size[1])*0.5];
    let velocity=[e.position[0]-e.previous[0],e.position[2]-e.previous[2]];
    let delta=[center[0]-e.position[0],center[1]-e.position[2]];
    let vi=(velocity[0]*velocity[0]+velocity[1]*velocity[1]).sqrt().recip();
    let di=(delta[0]*delta[0]+delta[1]*delta[1]).sqrt().recip();
    ((delta[0]*di)*(velocity[0]*vi)+(delta[1]*di)*(velocity[1]*vi)).acos()
}
fn java_compare(a:f64,b:f64)->Ordering{if a<b{return Ordering::Less;}if a>b{return Ordering::Greater;}let bits=|v:f64|if v.is_nan(){0x7ff8_0000_0000_0000i64}else{v.to_bits()as i64};bits(a).cmp(&bits(b))}
fn append(out:&mut Vec<Pos>,p:Pos){if !out.contains(&p){out.push(p);}}
/// None is the source's zero-items early return: retain the previous queue.
/// Hole order is HoleTracker order; entities retain world rendering order.
pub fn targets(s:&Settings,holes:&[Hole],player:[f64;3],creative:bool,range:i32,items:i32,entities:&[Entity])->Option<Vec<Pos>>{
    let holes=holes.iter().copied().filter(|h|!s.has(ONLY_ONE_BY_ONE)||h.size==[1,1]).collect::<Vec<_>>();
    let at=block_pos(player);let inside=holes.iter().any(|h|h.region().contains(at));
    if s.has(ONLY_WHEN_SELF_IN_HOLE)&&!inside{return Some(vec![]);}
    let self_region=Region::around(at,s.area,s.area);let mut out=vec![];
    if !s.has(SMART){for h in holes{if allows_self(s,h,player,inside,self_region){for p in h.cells(){append(&mut out,p);}}}return Some(out);}
    if items==0{return None;}
    let mut checked=vec![false;holes.len()];let mut remaining=items;
    let range_squared=f64::from(range.wrapping_mul(range))+10.;
    for e in entities{
        if (0..3).map(|i|(e.position[i]-player[i]).powi(2)).sum::<f64>()>range_squared{continue;}
        let region=Region::around(block_pos(e.position),s.area,s.area);let mut found:Vec<(f64,Pos)>=vec![];
        for (i,&h) in holes.iter().enumerate(){
            if checked[i]{continue;}let angle=movement_angle(h,*e);
            if s.has(CHECK_MOVEMENT)&&!(angle>=0.866)||!h.region().intersects(region)||f64::from(h.position[1])+1.>e.position[1]||!allows_self(s,h,player,inside,self_region){continue;}
            let size=h.size[0].wrapping_mul(h.size[1]);remaining=remaining.wrapping_sub(size);
            if remaining<0&&!creative{remaining=remaining.wrapping_add(size);continue;}
            checked[i]=true;for p in h.cells(){if !found.iter().any(|(a,q)|java_compare(*a,angle)==Ordering::Equal&&*q==p){found.push((angle,p));}}
            if remaining==0&&!creative{break;}
        }
        // Source's HashSet traversal breaks exact angle ties; insertion order is
        // retained here. All unequal angles and Java NaN ordering are preserved.
        found.sort_by(|a,b|java_compare(b.0,a.0));for (_,p) in found{append(&mut out,p);}
        if remaining<=0&&!creative{break;}
    }
    Some(out)
}
#[cfg(test)]mod tests{
    use super::*;
    fn hole(p:Pos,size:[i32;2])->Hole{Hole{position:p,size,kind:if size==[1,1]{1}else{3}}}
    fn entity(p:[f64;3],previous:[f64;3])->Entity{Entity{position:p,previous}}
    #[test]fn hole_filler_simple_self_fill_and_filtered_self_hole(){let mut s=Settings{features:PREVENT_SELF_FILL,area:2,..Default::default()};let holes=[hole([0,0,0],[1,1]),hole([5,0,0],[2,2])];assert_eq!(targets(&s,&holes,[0.,1.,0.],false,5,0,&[]),Some(vec![[5,0,0],[6,0,0],[5,0,1],[6,0,1]]));assert_eq!(targets(&s,&holes,[0.,0.,0.],false,5,0,&[]).unwrap().len(),5);s.features|=ONLY_WHEN_SELF_IN_HOLE|ONLY_ONE_BY_ONE;assert_eq!(targets(&s,&holes,[5.,0.,0.],false,5,0,&[]),Some(vec![]));}
    #[test]fn hole_filler_slot_filter_effective_hotbar_and_wrapping_count(){let mut s=Settings::default();let slots=[Slot{index:40,count:i32::MAX,block:Some("minecraft:obsidian".into())},Slot{index:4,count:2,block:Some("minecraft:obsidian".into())},Slot{index:2,count:64,block:None}];assert_eq!(slot(&s,&slots,4),Some(4));assert_eq!(slot(&s,&slots,5),Some(40));assert_eq!(available(&s,&slots),i32::MIN+1);s.whitelist=false;assert_eq!(available(&s,&slots),0);}
    #[test]fn hole_filler_zero_items_retains_queue_but_self_gate_clears(){let mut s=Settings::default();assert_eq!(targets(&s,&[],[0.;3],false,5,0,&[]),None);s.features|=ONLY_WHEN_SELF_IN_HOLE;assert_eq!(targets(&s,&[],[0.;3],false,5,0,&[]),Some(vec![]));}
    #[test]fn hole_filler_source_angle_is_radians_and_nan_is_rejected(){let h=hole([0,0,0],[1,1]);let toward=entity([1.,1.,0.],[2.,1.,0.]);let away=entity([1.,1.,0.],[0.,1.,0.]);assert_eq!(movement_angle(h,toward),0.);assert_eq!(movement_angle(h,away),std::f64::consts::PI);let mut s=Settings{features:SMART|CHECK_MOVEMENT,area:5,..Default::default()};assert_eq!(targets(&s,&[h],[0.,1.,0.],false,5,1,&[toward]),Some(vec![]));assert_eq!(targets(&s,&[h],[0.,1.,0.],false,5,1,&[away]),Some(vec![[0;3]]));let still=entity([1.,1.,0.],[1.,1.,0.]);assert_eq!(targets(&s,&[h],[0.,1.,0.],false,5,1,&[still]),Some(vec![]));s.features=SMART;assert_eq!(targets(&s,&[h],[0.,1.,0.],false,5,1,&[still]),Some(vec![[0;3]]));assert_eq!(java_compare(f64::NAN,1.),Ordering::Greater);}
    #[test]fn hole_filler_budget_skips_large_holes_and_creative_ignores_exhaustion(){let s=Settings{features:SMART,area:5,..Default::default()};let holes=[hole([0,0,0],[2,2]),hole([4,0,0],[1,1])];let e=entity([2.,1.,0.],[2.,1.,0.]);assert_eq!(targets(&s,&holes,[0.,1.,0.],false,5,1,&[e]),Some(vec![[4,0,0]]));assert_eq!(targets(&s,&holes,[0.,1.,0.],true,5,1,&[e]).unwrap().len(),5);}
}
