//! Compile the actual native result, ray and candidate-loop bodies against a
//! small world. Physical shapes and render picks are controllable fixtures.
use super::{Pack,ItemRenderer,Models,Components,ItemStack};
use crate::client_smart_eat_use;
use horizon_core::geometry::{Aabb,Vec3};
use horizon_world::{registry::BlockStateId,nbt::Tag};
use std::{collections::{BTreeMap,BTreeSet},cell::RefCell,marker::PhantomData};

pub mod projectiles {
    #[derive(Clone)]pub enum ProjectileKind {ShulkerBullet {},Other}
}
pub mod mobs {pub mod leash {
    use horizon_core::geometry::Vec3;
    pub struct Knot {pub position:Vec3,pub width:f64,pub height:f64}
    pub fn knot_state(block:[i32;3])->Knot {Knot {position:Vec3::new(f64::from(block[0])+0.5,f64::from(block[1])+0.375,f64::from(block[2])+0.5),width:0.375,height:0.5}}
}}
mod entity {use super::*;pub trait Entity {fn bounds(&self)->Aabb;}}
struct UsePlayer {fall_flying:bool,bounds:Aabb}
impl UsePlayer {fn bounds(&self)->Aabb {self.bounds}}
#[derive(Clone)]struct BoxEntity {id:u32,bounds:Aabb,marker:bool,in_ground:bool}
impl entity::Entity for BoxEntity {fn bounds(&self)->Aabb {self.bounds}}
#[derive(Clone)]struct ExtentsEntity {id:u32,bounds:Aabb}
impl ExtentsEntity {fn bounds(&self)->([f64;3],[f64;3]) {(self.bounds.min_position().components(),self.bounds.max_position().components())}}
#[derive(Clone)]struct Kind {id:String}
#[derive(Clone)]struct Extra {spectator:bool}
#[derive(Clone)]struct Mob {id:u32,bounds:Aabb,removed:bool,kind:Kind,extra:Extra}
impl entity::Entity for Mob {fn bounds(&self)->Aabb {self.bounds}}
struct Guest {entity_id:i32,bounds:Aabb,placed:bool,game_mode:i32}
impl Guest {fn bounds(&self)->Aabb {self.bounds}}
struct Hub {guests:Vec<Guest>}struct Lan {hub:Hub}
struct Creatures {projectiles:Vec<Projectile>}
struct Arrow {in_ground:bool}
struct Projectile {kind:projectiles::ProjectileKind,bounds:Aabb,entity_type:String,arrow:Option<Arrow>}
impl Projectile {fn entity_type(&self)->&str {&self.entity_type}}
impl entity::Entity for Projectile {fn bounds(&self)->Aabb {self.bounds}}
struct Dragon {id:u32,parts:[Aabb;8]}impl Dragon {fn part_box(&self,i:usize)->Aabb {self.parts[i]}}
struct Cooldowns {groups:BTreeSet<String>}impl Cooldowns {fn is_on_cooldown(&self,group:&str)->bool {self.groups.contains(group)}}
struct Enchantments {entity_tags:BTreeMap<String,BTreeSet<String>>}struct Stations {enchantments:Enchantments}
struct Remote {entity_id:i32,registries:horizon_protocol::registry::Dynamic,builtin:horizon_protocol::registry::Builtin}
struct World {blocks:BTreeMap<(i32,i32,i32),BlockStateId>}
impl World {fn block(&self,x:i32,y:i32,z:i32)->BlockStateId {self.blocks.get(&(x,y,z)).copied().unwrap_or(BlockStateId(0))}}
struct UseResources {facts:horizon_items::blocks::BlockFacts,registry:horizon_world::registry::BlockRegistry}
impl UseResources {fn facts(&self)->&horizon_items::blocks::BlockFacts {&self.facts}}
fn use_resources()->&'static UseResources {
    static RES:std::sync::OnceLock<UseResources>=std::sync::OnceLock::new();
    RES.get_or_init(|| {
        let path=std::path::Path::new("__RESOURCE_ROOT__");
        let registry=horizon_world::registry::BlockRegistry::from_facts_json(std::fs::File::open(path.join("facts.json")).unwrap()).unwrap();
        UseResources {facts:horizon_items::blocks::BlockFacts::load(path,&registry).unwrap(),registry}
    })
}
struct Game<'a> {
    marker:PhantomData<&'a ()>,player:Option<UsePlayer>,eye:Option<[f64;3]>,yaw:f32,pitch:f32,reach:f64,
    spectator:bool,owner:bool,cooldowns:Cooldowns,pack:Pack,stations:Stations,remote:Option<Remote>,world:World,res:&'static UseResources,
    outlines:BTreeMap<(i32,i32,i32),Vec<[f64;6]>>,fluids:BTreeMap<(i32,i32,i32),(f32,bool)>,
    margins:BTreeMap<i32,f64>,margin_queries:RefCell<Vec<i32>>,
    mobs:Vec<Mob>,lan:Vec<Lan>,armor_stands:Vec<BoxEntity>,paintings:Vec<ExtentsEntity>,item_frames:Vec<ExtentsEntity>,
    cushions:Vec<ExtentsEntity>,boats:Vec<BoxEntity>,carts:Vec<BoxEntity>,tnts:Vec<BoxEntity>,falling:Vec<BoxEntity>,
    interactions:BTreeMap<i32,BoxEntity>,crystals:Vec<BoxEntity>,dragon:Option<Dragon>,remote_knots:BTreeMap<u32,[i32;3]>,natural_knots:BTreeMap<u32,[i32;3]>,
    creatures:Creatures,rockets:Vec<BoxEntity>,tridents:Vec<BoxEntity>,
}
fn bounds(lo:[f64;3],hi:[f64;3])->Aabb {Aabb::new(Vec3::new(lo[0],lo[1],lo[2]),Vec3::new(hi[0],hi[1],hi[2]))}
impl Game<'_> {
    fn new()->Self {
        let prototypes: BTreeMap<String,Components>=serde_json::from_slice(&std::fs::read("__RESOURCE_ROOT__/items.json").unwrap()).unwrap();
        Self {marker:PhantomData,player:Some(UsePlayer {fall_flying:false,bounds:bounds([-0.3,0.,-0.3],[0.3,1.8,0.3])}),eye:Some([0.,1.62,0.]),yaw:0.,pitch:0.,reach:5.,spectator:false,owner:true,
            cooldowns:Cooldowns {groups:BTreeSet::new()},pack:Pack {item_renderer:ItemRenderer {models:Models {prototypes}}},stations:Stations {enchantments:Enchantments {entity_tags:BTreeMap::new()}},remote:None,world:World {blocks:BTreeMap::new()},res:use_resources(),outlines:BTreeMap::new(),fluids:BTreeMap::new(),margins:BTreeMap::new(),margin_queries:RefCell::new(vec![]),
            mobs:vec![],lan:vec![],armor_stands:vec![],paintings:vec![],item_frames:vec![],cushions:vec![],boats:vec![],carts:vec![],tnts:vec![],falling:vec![],interactions:BTreeMap::new(),crystals:vec![],dragon:None,remote_knots:BTreeMap::new(),natural_knots:BTreeMap::new(),creatures:Creatures {projectiles:vec![]},rockets:vec![],tridents:vec![]}
    }
    fn client_smart_eat_player(&self)->bool {self.owner&&self.player.is_some()}
    fn spectator(&self)->bool {self.spectator}
    fn stations(&self)->&Stations {&self.stations}
    fn eye_position(&self)->Option<[f64;3]> {self.eye}
    fn reach(&self)->f64 {self.reach}
    fn block_name(&self,p:[i32;3])->std::sync::Arc<str> {self.res.facts().get(self.world.block(p[0],p[1],p[2])).name.clone()}
    fn fluid_height_of(&self,x:i32,y:i32,z:i32,_:Option<()>)->Option<(f32,bool)> {self.fluids.get(&(x,y,z)).copied()}
    fn outline_boxes(&self,p:[i32;3])->Vec<[f64;6]> {self.outlines.get(&(p[0],p[1],p[2])).cloned().unwrap_or_default()}
    fn client_pick_margin(&self,id:i32,_:f32)->f64 {self.margin_queries.borrow_mut().push(id);self.margins.get(&id).copied().unwrap_or(0.)}
    fn natural_knot_ids(&self)->Vec<u32> {self.natural_knots.keys().copied().collect()}
    fn natural_knot(&self,id:u32)->Option<[i32;3]> {self.natural_knots.get(&id).copied()}
}
mod native_hooks {
    use super::*;
    mod client_smart_eat_ray {__ITEM_RAY__}
    impl Game<'_> {
        __USE_RESULT__
        __PICKABLE__
    }
}

fn component(stack:&mut ItemStack,name:&str,value:Tag) {
    let opposite=name.strip_prefix('!').map_or_else(||format!("!{name}"),str::to_owned);
    stack.data_mut().set_other(&opposite,None);
    stack.data_mut().set_other(name,Some(value));
}

#[test]fn actual_native_entity_tags_follow_configuration_and_live_replacement(){
    use horizon_protocol::{registry::{Builtin,Dynamic,Ids},packets::configuration::{UpdateTags,RegistryTags,TagEntry}};
    let mut g=Game::new();
    g.stations.enchantments.entity_tags.insert("test:equip".into(),BTreeSet::from(["minecraft:player".into()]));
    assert!(g.client_item_entity_tag_contains("test:equip","minecraft:player"));
    let mut builtin=Builtin::default();
    builtin.registries.insert("minecraft:entity_type".into(),Ids::from_names(vec!["minecraft:player".into(),"minecraft:arrow".into()]));
    let mut registries=Dynamic::default();
    registries.add_tags(&UpdateTags {registries:vec![RegistryTags {registry:"minecraft:entity_type".into(),tags:vec![TagEntry {name:"test:equip".into(),ids:vec![1]}]}]});
    g.remote=Some(Remote {entity_id:100,registries,builtin});
    assert!(!g.client_item_entity_tag_contains("test:equip","minecraft:player"));
    assert!(g.client_item_entity_tag_contains("test:equip","minecraft:arrow"));
    g.remote.as_mut().unwrap().registries.add_tags(&UpdateTags {registries:vec![RegistryTags {registry:"minecraft:entity_type".into(),tags:vec![]}]});
    assert!(!g.client_item_entity_tag_contains("test:equip","minecraft:player")); // empty synced registry cannot resurrect local tags
}

#[test]fn actual_native_generic_dispatch_uses_effective_components_and_entity_tags(){
    let mut g=Game::new();
    let mut bread=ItemStack::new("minecraft:bread",1);
    assert_eq!(g.client_item_use_pass(&bread),Some(false));
    component(&mut bread,"!minecraft:consumable",Tag::compound([] as [(&str,Tag);0]));
    assert_eq!(g.client_item_use_pass(&bread),Some(true));
    let mut pick=ItemStack::new("minecraft:diamond_pickaxe",1);
    assert_eq!(g.client_item_use_pass(&pick),Some(true));
    component(&mut pick,"minecraft:equippable",Tag::compound([("slot",Tag::String("head".into())),("allowed_entities",Tag::String("#test:animals".into()))]));
    component(&mut pick,"minecraft:blocks_attacks",Tag::compound([] as [(&str,Tag);0]));
    assert_eq!(g.client_item_use_pass(&pick),Some(true));
    g.stations.enchantments.entity_tags.insert("test:animals".into(),BTreeSet::from(["minecraft:player".into()]));
    assert_eq!(g.client_item_use_pass(&pick),Some(false));
    component(&mut pick,"minecraft:equippable",Tag::compound([("slot",Tag::String("head".into())),("swappable",Tag::Byte(0)),("allowed_entities",Tag::String("minecraft:pig".into()))]));
    assert_eq!(g.client_item_use_pass(&pick),Some(false));
    component(&mut pick,"!minecraft:blocks_attacks",Tag::compound([] as [(&str,Tag);0]));
    assert_eq!(g.client_item_use_pass(&pick),Some(true));
    assert_eq!(g.client_item_use_pass(&ItemStack::new("mod:unknown_item",1)),None);
}

#[test]fn actual_native_cooldown_group_honors_removed_defaults_and_override(){
    let mut g=Game::new();
    g.pack.item_renderer.models.prototypes.get_mut("minecraft:diamond_pickaxe").unwrap().insert("minecraft:use_cooldown".into(),serde_json::json!({"seconds":1,"cooldown_group":"test:default"}));
    g.cooldowns.groups.insert("test:default".into());
    let mut pick=ItemStack::new("minecraft:diamond_pickaxe",1);
    component(&mut pick,"minecraft:consumable",Tag::compound([("consume_seconds",Tag::Float(1_f32.to_bits()))]));
    assert_eq!(g.client_item_use_pass(&pick),Some(true));
    component(&mut pick,"!minecraft:use_cooldown",Tag::compound([] as [(&str,Tag);0]));
    assert_eq!(g.client_item_use_pass(&pick),Some(false));
    g.cooldowns.groups.insert("minecraft:diamond_pickaxe".into());
    assert_eq!(g.client_item_use_pass(&pick),Some(true));
    component(&mut pick,"minecraft:use_cooldown",Tag::compound([("seconds",Tag::Float(1_f32.to_bits())),("cooldown_group",Tag::String("test:new".into()))]));
    assert_eq!(g.client_item_use_pass(&pick),Some(false));
    g.cooldowns.groups.insert("test:new".into());
    assert_eq!(g.client_item_use_pass(&pick),Some(true));
}

#[test]fn actual_item_ray_is_independent_of_render_pick_and_uses_raw_player_angles(){
    let mut g=Game::new();
    let p=(0,1,3);
    g.outlines.insert(p,vec![[0.,1.,3.,1.,2.,4.]]);
    let boat=ItemStack::new("minecraft:oak_boat",1);
    assert_eq!(g.client_item_use_pass(&boat),Some(false));
    g.yaw=180.;
    assert_eq!(g.client_item_use_pass(&boat),Some(true));
    g.yaw=0.;g.reach=2.;
    assert_eq!(g.client_item_use_pass(&boat),Some(true));
    g.reach=5.;
    let egg=ItemStack::new("minecraft:pig_spawn_egg",1);
    assert_eq!(g.client_item_use_pass(&egg),Some(false)); // any block, not only liquid
    assert_eq!(g.client_item_use_pass(&ItemStack::new("minecraft:bucket",1)),Some(false)); // failed pickup != PASS
}

#[test]fn actual_native_bottle_and_ender_eye_distinguish_source_water_and_block_hits(){
    let mut g=Game::new();
    let p=(0,1,3);
    let bottle=ItemStack::new("minecraft:glass_bottle",1);
    let eye=ItemStack::new("minecraft:ender_eye",1);
    assert_eq!(g.client_item_use_pass(&bottle),Some(true));
    g.world.blocks.insert(p,g.res.registry.default_state("minecraft:water").unwrap());
    g.fluids.insert(p,(1.,true));
    assert_eq!(g.client_item_use_pass(&bottle),Some(false));
    assert_eq!(g.client_item_use_pass(&eye),Some(false)); // NONE ignores fluid
    g.fluids.insert(p,(1.,false));
    assert_eq!(g.client_item_use_pass(&bottle),Some(true)); // SOURCE_ONLY ignores flowing water
    g.world.blocks.insert(p,g.res.registry.default_state("minecraft:lava").unwrap());
    g.fluids.insert(p,(1.,true));
    assert_eq!(g.client_item_use_pass(&bottle),Some(true)); // fluid hit alone is insufficient
    g.fluids.clear();
    g.world.blocks.insert(p,g.res.registry.default_state("minecraft:stone").unwrap());
    g.outlines.insert(p,vec![[0.,1.,3.,1.,2.,4.]]);
    assert_eq!(g.client_item_use_pass(&bottle),Some(true));
    assert_eq!(g.client_item_use_pass(&eye),Some(false));
    g.world.blocks.insert(p,g.res.registry.default_state("minecraft:end_portal_frame").unwrap());
    assert_eq!(g.client_item_use_pass(&eye),Some(true));
}

#[test]fn actual_boat_query_covers_each_nonliving_category_and_strict_box_boundaries(){
    let contained=bounds([-0.1,1.5,-0.1],[0.1,1.7,0.1]);
    let b=BoxEntity {id:9,bounds:contained,marker:false,in_ground:false};
    for kind in 0..13 {
        let mut g=Game::new();
        match kind {
            0=>g.armor_stands.push(b.clone()),1=>g.boats.push(b.clone()),2=>g.carts.push(b.clone()),
            3=>g.tnts.push(b.clone()),4=>g.falling.push(b.clone()),5=>{g.interactions.insert(9,b.clone());},
            6=>g.crystals.push(b.clone()),7=>g.paintings.push(ExtentsEntity {id:9,bounds:contained}),
            8=>g.item_frames.push(ExtentsEntity {id:9,bounds:contained}),9=>g.cushions.push(ExtentsEntity {id:9,bounds:contained}),
            10=>g.dragon=Some(Dragon {id:8,parts:[contained;8]}),
            11=>g.lan.push(Lan {hub:Hub {guests:vec![Guest {entity_id:9,bounds:contained,placed:true,game_mode:0}]}}),
            _=>{g.remote_knots.insert(9,[0,1,0]);g.eye=Some([0.5,1.62,0.5]);}
        }
        assert_eq!(g.client_boat_eye_inside_pickable(),Some(true),"kind {kind}");
    }
    let mut g=Game::new();g.armor_stands.push(BoxEntity {marker:true,..b.clone()});
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(false));
    g.armor_stands[0].marker=false;g.armor_stands[0].bounds=bounds([-0.1,1.5,-0.1],[0.,1.7,0.1]);
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(false)); // max is exclusive
    g.margins.insert(9,0.01);
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(true));
}

#[test]fn actual_boat_query_preserves_living_removal_and_player_spectator_rules(){
    let mut g=Game::new();
    let m=Mob {id:9,bounds:bounds([-0.1,1.5,-0.1],[0.1,1.7,0.1]),removed:false,kind:Kind {id:"minecraft:pig".into()},extra:Extra {spectator:true}};
    g.mobs.push(m);
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(true)); // no alive/health or arbitrary spectator filter
    g.mobs[0].kind.id="minecraft:player".into();
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(false));
    g.mobs[0].extra.spectator=false;
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(true));
    g.mobs[0].removed=true;
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(false));
}

#[test]fn actual_boat_query_includes_local_natural_knots_and_excludes_local_player(){
    let mut g=Game::new();
    g.eye=Some([0.5,1.62,0.5]);
    g.natural_knots.insert(22,[0,1,0]);
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(true));
    assert_eq!(*g.margin_queries.borrow(),vec![22]);
    g.natural_knots.clear();
    let mut builtin=horizon_protocol::registry::Builtin::default();
    builtin.registries.insert("minecraft:entity_type".into(),horizon_protocol::registry::Ids::from_names(vec!["minecraft:player".into()]));
    g.remote=Some(Remote {entity_id:9,registries:horizon_protocol::registry::Dynamic::default(),builtin});
    g.mobs.push(Mob {id:9,bounds:bounds([0.4,1.5,0.4],[0.6,1.7,0.6]),removed:false,kind:Kind {id:"minecraft:player".into()},extra:Extra {spectator:false}});
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(false));
    g.mobs[0].id=10;
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(true));
}

#[test]fn actual_boat_projectiles_use_dynamic_tags_in_ground_and_shulker_radius_one(){
    let mut g=Game::new();
    let outside=bounds([0.7,1.5,-0.1],[0.9,1.7,0.1]);
    g.creatures.projectiles.push(Projectile {kind:projectiles::ProjectileKind::ShulkerBullet {},bounds:outside,entity_type:"minecraft:shulker_bullet".into(),arrow:None});
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(true));
    assert!(g.margin_queries.borrow().is_empty()); // Projectile.getPickRadius bypasses Entity's margin mixin
    g.creatures.projectiles[0].kind=projectiles::ProjectileKind::Other;
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(false));
    g.stations.enchantments.entity_tags.insert("minecraft:redirectable_projectile".into(),BTreeSet::from(["minecraft:shulker_bullet".into()]));
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(true));
    g.creatures.projectiles[0].arrow=Some(Arrow {in_ground:true});
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(false));
    g.creatures.projectiles.clear();
    g.tridents.push(BoxEntity {id:8,bounds:outside,marker:false,in_ground:false});
    g.stations.enchantments.entity_tags.get_mut("minecraft:redirectable_projectile").unwrap().insert("minecraft:trident".into());
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(true));
    g.tridents[0].in_ground=true;
    assert_eq!(g.client_boat_eye_inside_pickable(),Some(false));
}
