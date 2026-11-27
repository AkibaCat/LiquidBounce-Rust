// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleItemTags and ItemStackListLayout/GuiOverlapRearranger, ae0eb396.
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashSet};
#[path="item_tags_layout.rs"]pub mod layout;
pub use layout::{bounds,dimensions,rearrange};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter { Whitelist, Blacklist }
impl Filter {
    pub fn tag(self)->&'static str {match self{Self::Whitelist=>"Whitelist",Self::Blacklist=>"Blacklist"}}
    pub fn parse(s:&str)->Option<Self>{[Self::Whitelist,Self::Blacklist].into_iter().find(|v|v.tag().eq_ignore_ascii_case(s))}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeMode { None, ByItem, ByComponents }
impl MergeMode {
    pub const ALL:[Self;3]=[Self::None,Self::ByItem,Self::ByComponents];
    pub fn tag(self)->&'static str {match self{Self::None=>"None",Self::ByItem=>"ByItem",Self::ByComponents=>"ByComponents"}}
    pub fn parse(s:&str)->Option<Self>{Self::ALL.into_iter().find(|v|v.tag().eq_ignore_ascii_case(s))}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background { Rect, Texture }
impl Background {
    pub fn tag(self)->&'static str{match self{Self::Rect=>"Rect",Self::Texture=>"Texture"}}
    pub fn parse(s:&str)->Option<Self>{match s{"Rect"=>Some(Self::Rect),"Texture"=>Some(Self::Texture),_=>None}}
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub filter:Filter,pub items:BTreeSet<String>,pub background:Background,
    pub color:u32,pub outline_color:u32,pub margin:f32,pub radius:f32,pub gradient:f32,pub glow:f32,
    pub scale:Vec<[f32;2]>,pub render_offset:[f64;3],pub row_length:i32,pub prevent_overlap:bool,
    pub cluster_entities:Vec<[f32;2]>,pub merge_mode:MergeMode,
    pub shulker:bool,pub merge_stacks:bool,pub show_title:bool,
}
impl Default for Settings {
    fn default()->Self{Self{
        filter:Filter::Blacklist,items:BTreeSet::new(),background:Background::Rect,
        color:0x80000000,outline_color:0,margin:2.,radius:4.,gradient:0.3,glow:0.3,
        scale:vec![[0.,1.],[200.,1.]],render_offset:[0.;3],row_length:100,prevent_overlap:true,
        cluster_entities:vec![[0.,2.],[64.,16.],[128.,16.],[200.,24.]],merge_mode:MergeMode::ByComponents,
        shulker:false,merge_stacks:true,show_title:true,
    }}
}
impl Settings {
    pub fn padding(&self)->f32{if self.background==Background::Rect{self.margin+if self.glow>0.{3.}else{0.}}else{0.}}
    pub fn accepts(&self,item:&str)->bool{self.items.contains(item)==(self.filter==Filter::Whitelist)}
    pub fn draw_settings(&self)->Value{json!({"background":self.background.tag(),"color":self.color,"outlineColor":self.outline_color,"margin":self.margin,"radius":self.radius,"gradient":self.gradient,"glow":self.glow,"rowLength":self.row_length,"preventOverlap":self.prevent_overlap,"shulker":self.shulker,"mergeStacks":self.merge_stacks,"showTitle":self.show_title,"mergeMode":self.merge_mode.tag()})}
}
/// A source handle is the native stack's exact received component patch.
/// `bare` asks the renderer for a new ItemStack(item,count), as ByItem does.
#[derive(Clone, Debug, PartialEq)]
pub struct Stack {pub handle:u64,pub item:String,pub components:String,pub description:String,pub count:i32,pub bare:bool,pub live:bool}
impl Stack {
    pub fn from_json(v:&Value)->Option<Self>{Some(Self{handle:v["handle"].as_u64()?,item:v["item"].as_str()?.into(),components:v["components"].as_str()?.into(),description:v["description"].as_str()?.into(),count:v["count"].as_i64()? as i32,bare:false,live:true})}
    pub fn json(&self)->Value{json!({"handle":self.handle,"item":self.item,"count":self.count,"bare":self.bare,"live":self.live})}
}
pub fn merge(stacks:&[Stack],mode:MergeMode)->Vec<Stack>{
    let mut out:Vec<Stack>=Vec::new();
    for stack in stacks {
        if mode==MergeMode::None{out.push(stack.clone());continue;}
        let existing=out.iter_mut().find(|s|s.item==stack.item&&(mode==MergeMode::ByItem||s.components==stack.components));
        if let Some(existing)=existing{
            existing.count=existing.count.wrapping_add(stack.count);
            if mode==MergeMode::ByItem{existing.bare=true;existing.live=false;existing.components.clear();}
        }else{let mut copy=stack.clone();if mode==MergeMode::ByComponents{copy.live=false;}out.push(copy);}
    }
    // Java compares UTF-16 descriptionIds; counts are signed JVM Int sums.
    out.sort_by(|a,b|b.count.cmp(&a.count).then_with(||a.description.encode_utf16().cmp(b.description.encode_utf16())));
    out
}
#[derive(Clone, Debug)]
pub struct Entity {pub identity:u64,pub position:[f64;3],pub stack:Stack}
#[derive(Clone, Debug)]
pub struct Cluster {pub entities:Vec<u64>,pub stacks:Vec<Stack>,pub scale:f32}
#[derive(Default)]
pub struct ItemTags {pub settings:Settings,pub groups:Vec<Cluster>}
impl ItemTags {
    pub fn clear(&mut self){self.groups.clear();}
    pub fn tick(&mut self,camera:[f64;3],entities:&[Entity]){
        let s=&self.settings;let entities=entities.iter().filter(|e|s.accepts(&e.stack.item)).collect::<Vec<_>>();
        self.groups.clear();let mut visited=HashSet::new();
        for entity in &entities {
            if visited.contains(&entity.identity){continue;}
            let distance=(0..3).map(|i|(entity.position[i]-camera[i]).powi(2)).sum::<f64>().sqrt() as f32;
            let scale=crate::tracers::curve::sample(&s.scale,distance);
            if scale<0.01{continue;}
            let radius=crate::tracers::curve::sample(&s.cluster_entities,distance);let radius_squared=radius*radius;
            let group=entities.iter().filter(|other|!visited.contains(&other.identity)&&(0..3).map(|i|(entity.position[i]-other.position[i]).powi(2)).sum::<f64>()<f64::from(radius_squared)).copied().collect::<Vec<_>>();
            visited.extend(group.iter().map(|e|e.identity));
            let stacks=merge(&group.iter().map(|e|e.stack.clone()).collect::<Vec<_>>(),s.merge_mode);
            self.groups.push(Cluster{entities:group.iter().map(|e|e.identity).collect(),stacks,scale});
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stack(handle:u64,item:&str,components:&str,count:i32)->Stack{Stack{handle,item:item.into(),components:components.into(),description:item.into(),count,bare:false,live:true}}
    #[test]fn three_merges_preserve_components_and_only_by_item_multiple_loses_them(){
        let stacks=vec![stack(1,"stone","a",5),stack(2,"stone","b",7),stack(3,"stone","a",2),stack(4,"apple","x",10)];
        assert_eq!(merge(&stacks,MergeMode::None).iter().map(|s|s.count).collect::<Vec<_>>(),[10,7,5,2]);
        let components=merge(&stacks,MergeMode::ByComponents);assert_eq!(components.len(),3);assert!(!components.iter().any(|s|s.bare));assert!(components.iter().any(|s|s.components=="a"&&s.count==7));
        let by_item=merge(&stacks,MergeMode::ByItem);assert_eq!(by_item[0].count,14);assert!(by_item[0].bare);assert!(!by_item[1].bare);assert_eq!(by_item[1].components,"x");
        let overflow=merge(&[stack(1,"stone","",i32::MAX),stack(2,"stone","",1)],MergeMode::ByComponents);assert_eq!(overflow[0].count,i32::MIN);
    }
    #[test]fn clustering_uses_seed_strict_radius_not_transitive_reach_and_filter(){
        let mut model=ItemTags::default();model.settings.cluster_entities=vec![[0.,2.],[200.,2.]];
        let entities=(0..4).map(|i|Entity{identity:i+1,position:[i as f64*1.5,0.,0.],stack:stack(i+1,"stone","",1)}).collect::<Vec<_>>();
        model.tick([0.;3],&entities);assert_eq!(model.groups.iter().map(|g|g.entities.clone()).collect::<Vec<_>>(),[vec![1,2],vec![3,4]]);
        model.settings.cluster_entities=vec![[0.,1.5],[200.,1.5]];model.tick([0.;3],&entities);assert_eq!(model.groups.len(),4);
        model.settings.items.insert("stone".into());model.tick([0.;3],&entities);assert!(model.groups.is_empty());model.settings.filter=Filter::Whitelist;model.tick([0.;3],&entities);assert_eq!(model.groups.len(),4);model.clear();assert!(model.groups.is_empty());
    }
    #[test]fn layouts_keep_title_width_glow_padding_order_and_edge_touching(){
        assert_eq!(dimensions(10,9,None),[162,36]);assert_eq!(dimensions(1,100,Some(40)),[40,29]);assert_eq!(dimensions(0,100,Some(40)),[40,9]);
        assert_eq!(bounds([100.,100.],[18,18],5.,2.),[72.,72.,128.,128.]);
        let mut boxes=[[0.,0.,20.,20.],[0.,0.,20.,20.],[0.,10.,20.,30.],[20.,0.,40.,20.]];rearrange(&mut boxes);assert_eq!(boxes,[[0.,0.,20.,20.],[0.,20.,20.,40.],[0.,40.,20.,60.],[20.,0.,40.,20.]]);
    }
}
