#![allow(dead_code)]
extern crate self as horizon_client;
extern crate self as horizon_world;
use horizon_core::geometry::{Aabb, Vec3};
use std::{collections::{BTreeMap, HashMap},sync::{LazyLock,Mutex}};
#[path="__REACH_PATH__"] mod reach;
#[path="__VIEW_DIRECTION_PATH__"] mod view_direction;
__PICKER__
__RAY_BOX__
__PICK_VIEW_VECTOR__
__VIEW_VECTOR__
fn interact_location(point:Vec3,feet:Vec3)->[f64;3] {[point.x-feet.x,point.y-feet.y,point.z-feet.z]}
__ATTACK_REACH__
__REACH_PICK__
fn v3(v:[f64;3])->Vec3 {Vec3::new(v[0],v[1],v[2])}
struct Dimensions {eye_height:f32}
struct Player {position:Vec3,dimensions:Dimensions,server_copy:bool,replaying:bool,attributes:BTreeMap<String,f64>,bounds:Aabb}
impl Player {
    __PLAYER_GUARD__
    fn bounds(&self)->Aabb {self.bounds}
    fn attribute(&mut self,name:&str)->Result<f64,()> {self.attributes.get(name).copied().ok_or(())}
}
struct Camera {position:[f64;3],eye_height:f32,y_rot:f32,x_rot:f32}
#[derive(Default)] struct Extra {spectator:bool,mount:Option<u32>}
struct Mob {id:u32,position:Vec3,removed:bool,extra:Extra,bounds:Aabb}
impl Mob {fn bounds(&self)->Aabb {self.bounds} fn alive(&self)->bool {true}}
struct BoxEntity {id:u32,position:Vec3,bounds:Aabb,marker:bool,vehicle:Option<u32>}
impl BoxEntity {fn bounds(&self)->Aabb {self.bounds}}
struct FlatEntity {id:u32,position:Vec3,pos:[i32;3],bounds:([f64;3],[f64;3])}
impl FlatEntity {fn bounds(&self)->([f64;3],[f64;3]) {self.bounds}}
struct Interaction {position:Vec3,bounds:Aabb,vehicle:Option<u32>}
impl Interaction {fn bounds(&self)->Aabb {self.bounds}}
mod projectiles {
    use super::*;
    pub enum ProjectileKind {ShulkerBullet{},WindCharge,Other}
    pub struct Projectile {pub id:u32,pub position:Vec3,pub kind:ProjectileKind,pub bounds:Aabb,pub pickable:bool}
    impl Projectile {pub fn bounds(&self)->Aabb {self.bounds}}
}
mod mobs {pub mod leash {
    use super::super::*;
    pub struct Knot {pub position:Vec3,pub width:f64,pub height:f64}
    pub fn knot_state(pos:[i32;3])->Knot {Knot {position:Vec3::new(f64::from(pos[0]),f64::from(pos[1]),f64::from(pos[2])),width:0.5,height:0.5}}
}}
#[derive(Default)] struct Creatures {projectiles:Vec<projectiles::Projectile>}
struct Dragon {id:u32,parts:[(Vec3,Vec3);8],boxes:[Aabb;8]}
impl Dragon {fn alive(&self)->bool {true} fn part_box(&self,i:usize)->Aabb {self.boxes[i]}}
#[derive(Default)]struct Remote {passenger_vehicles:HashMap<u32,u32>}
#[derive(Clone)]struct Stack {range:Option<serde_json::Value>}
#[derive(Clone,Copy,Debug)]struct Hit {pos:[i32;3],face:[i32;3],point:[f64;3]}
struct ObjectHit {id:i32,distance:f64,location:[f64;3]}
struct Game {
    player:Option<Player>,remote:Option<Remote>,window:bool,lan_acting:Option<u32>,lan_acting_uuid:Option<u32>,creative:bool,
    yaw:f32,pitch:f32,camera:Option<Camera>,riding:Option<u32>,entity_reach:f64,block_reach:f64,
    mobs:Vec<Mob>,paintings:Vec<FlatEntity>,item_frames:Vec<FlatEntity>,armor_stands:Vec<BoxEntity>,
    cushions:Vec<FlatEntity>,boats:Vec<BoxEntity>,carts:Vec<BoxEntity>,interactions:BTreeMap<i32,Interaction>,
    creatures:Creatures,remote_knots:BTreeMap<u32,[i32;3]>,crystals:Vec<BoxEntity>,dragon:Option<Dragon>,
    using:Option<(usize,i32)>,inventory:Vec<Option<Stack>>,selected:usize,movement:Vec3,
    forced_hit:Option<Hit>,tick_hit:Option<Option<Hit>>,tick_mob:Option<Option<(usize,f64,[f64;3])>>,wall:Option<Aabb>,
}
fn bounds(z:f64)->Aabb {Aabb::new(Vec3::new(-0.25,-0.25,z),Vec3::new(0.25,0.25,z+0.5))}
fn box_entity(id:u32,z:f64)->BoxEntity {BoxEntity {id,position:Vec3::new(0.0,-0.25,z),bounds:bounds(z),marker:false,vehicle:None}}
fn flat_entity(id:u32,z:f64)->FlatEntity {FlatEntity {id,position:Vec3::new(0.0,-0.25,z),pos:[0,0,z as i32],bounds:([-0.25,-0.25,z],[0.25,0.25,z+0.5])}}
fn aabb((lo,hi):([f64;3],[f64;3]))->Aabb {Aabb::new(v3(lo),v3(hi))}
impl Game {
    fn new()->Self {Self {
        player:Some(Player {position:Vec3::ZERO,dimensions:Dimensions {eye_height:0.0},server_copy:false,replaying:false,
            attributes:BTreeMap::from([("minecraft:entity_interaction_range".into(),3.0),("minecraft:block_interaction_range".into(),4.5)]),
            bounds:Aabb::new(Vec3::new(-0.3,-1.6,-0.3),Vec3::new(0.3,0.2,0.3))}),
        remote:Some(Remote::default()),window:true,lan_acting:None,lan_acting_uuid:None,creative:false,yaw:0.0,pitch:0.0,camera:None,riding:None,
        entity_reach:3.0,block_reach:4.5,mobs:Vec::new(),paintings:Vec::new(),item_frames:Vec::new(),armor_stands:Vec::new(),
        cushions:Vec::new(),boats:Vec::new(),carts:Vec::new(),interactions:BTreeMap::new(),creatures:Creatures::default(),
        remote_knots:BTreeMap::new(),crystals:Vec::new(),dragon:None,using:None,inventory:vec![None],selected:0,movement:Vec3::ZERO,
        forced_hit:None,tick_hit:None,tick_mob:None,wall:Some(Aabb::new(Vec3::new(-1.0,-1.0,1.0),Vec3::new(1.0,1.0,1.1))),
    }}
    fn add(&mut self,kind:u8,id:u32,z:f64) {match kind {
        0=>self.mobs.push(Mob {id,position:Vec3::new(0.0,-0.25,z),removed:false,extra:Extra::default(),bounds:bounds(z)}),
        1=>self.paintings.push(flat_entity(id,z)),2=>self.item_frames.push(flat_entity(id,z)),
        3=>self.armor_stands.push(box_entity(id,z)),4=>self.cushions.push(flat_entity(id,z)),
        5=>self.boats.push(box_entity(id,z)),6=>self.carts.push(box_entity(id,z)),
        7=>{self.interactions.insert(id as i32,Interaction {position:Vec3::new(0.0,-0.25,z),bounds:bounds(z),vehicle:None});},
        8=>self.creatures.projectiles.push(projectiles::Projectile {id,position:Vec3::new(0.0,-0.25,z),kind:projectiles::ProjectileKind::ShulkerBullet{},bounds:bounds(z),pickable:true}),
        9=>{self.remote_knots.insert(id,[0,0,z as i32]);},10=>self.crystals.push(box_entity(id,z+1.0)),
        11=>{self.dragon=Some(Dragon {id:id-1,parts:[(Vec3::new(0.0,-0.25,z),Vec3::ZERO);8],boxes:[bounds(z);8]});},_=>panic!("unknown kind"),
    }}
    fn native_window(&self)->bool {self.window}
    fn camera_entity(&self)->Option<&Camera> {self.camera.as_ref()}
    fn remote_camera_id(&self)->Option<u32> {self.camera.as_ref().map(|_|999)}
    fn held(&self)->Option<&Stack> {self.inventory[self.selected].as_ref()}
    fn stack_attack_range(&self,s:&Stack)->Option<serde_json::Value> {s.range.clone()}
    fn known_player_movement(&self)->Vec3 {self.movement}
    fn clip_ray(&self,origin:[f64;3],dir:[f64;3],reach:f64)->Option<Hit> {
        self.wall?.clip(v3(origin),v3(origin).add_vector(v3(dir).scale(reach)))
            .map(|p|Hit {pos:[0,0,1],face:[0,0,-1],point:[p.x,p.y,p.z]})
    }
    __EYE_POSITION__
    __AIM__
    __PICK__
    __PICK_UNFILTERED__
    __MOB_HIT__
    __MOB_WITHIN__
    __INTERACTION_RANGE__
    __HELD_RANGE__
    __ACTIVE_RANGE__
    __ATTACK_RANGE_PICK__
    __CROSSHAIR_LIVING__
    __HITTABLE_MOBS__
}
mod host_methods {use super::*;impl Game {__HOOKS__}}
mod painting_methods {use super::*;impl Game {__PAINTING_HIT__}}
mod frame_methods {use super::*;impl Game {__FRAME_HIT__ __BEFORE_BLOCK__}}
mod stand_methods {use super::*;impl Game {__STAND_HIT__}}
mod cushion_methods {use super::*;impl Game {__CUSHION_HIT__}}
mod boat_methods {use super::*;impl Game {__BOAT_HIT__}}
mod cart_methods {use super::*;impl Game {__CART_HIT__}}
mod interaction_methods {use super::*;impl Game {__INTERACTION_HIT__}}
mod remote_methods {use super::*;impl Game {__KNOT_HIT__ __OBJECT_HIT__}}
mod deflect {
    use super::*;
    pub fn attackable(p:&projectiles::Projectile)->bool {p.pickable}
    fn pick_box(p:&projectiles::Projectile)->Aabb {
        let r=if matches!(p.kind,projectiles::ProjectileKind::ShulkerBullet{}) {0.0} else {1.0};
        p.bounds().inflate(Vec3::new(r,r,r))
    }
    impl Game {__PROJECTILE_HIT__}
}
mod dragon {
    use super::*;
    pub const SPECIAL:u32=u32::MAX-4096;
    pub const CRYSTAL_IDS:u32=SPECIAL+16;
    impl Game {__DRAGON_HIT__ __REMOTE_DRAGON_HIT__ __SPECIAL_TARGETS__}
}
mod spear {
    use super::*;
    type Vec3d=Vec3;
    __HITS_ALONG__
    __HITS_IN_REACH__
}
struct ModuleState {enabled:bool,settings:reach::Reach,rotation:Option<(f32,f32)>}
static MODULE:Mutex<ModuleState>=Mutex::new(ModuleState {enabled:true,settings:reach::Reach {range_increase:1.0,through_walls_range:4.0,block_range_increase:0.5},rotation:None});
static SCENE:Mutex<()>=Mutex::new(());
impl mod_api::Host for Game {}
mod mod_api {
    use super::*;
    pub trait Host {}
    pub trait RenderHooks:Send+Sync {}
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks> {mods.iter().map(Box::as_ref)}
    mod generated {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}
    pub use generated::*;
    struct Module;
    impl Hooks for Module {
        fn client_interaction_range(&self,_:&dyn Host,event:&mut ClientInteractionRangeEvent) {
            let state=MODULE.lock().unwrap();if state.enabled {event.range=state.settings.query_range(event.attribute,event.original);}
        }
        fn client_attack_range(&self,_:&dyn Host,event:&mut ClientAttackRangeEvent) {
            let state=MODULE.lock().unwrap();if state.enabled {state.settings.adjust_attack_range(&mut event.min_reach,&mut event.max_reach,&mut event.min_creative_reach,&mut event.max_creative_reach);}
        }
        fn client_reach_policy(&self,_:&dyn Host,event:&mut ClientReachPolicyEvent) {
            let state=MODULE.lock().unwrap();if state.enabled {event.through_walls=state.settings.through_walls_range;}
        }
        fn client_rotation_aim(&self,_:&dyn Host,event:&mut ClientRotationAimEvent) {
            if let Some((yaw,pitch))=MODULE.lock().unwrap().rotation {event.yaw=yaw;event.pitch=pitch;event.managed=true;}
        }
    }
    pub fn mods()->&'static [Box<dyn Hooks>] {static MODS:LazyLock<Vec<Box<dyn Hooks>>>=LazyLock::new(||vec![Box::new(Module)]);&MODS}
}
#[cfg(test)]
mod tests {
    use super::*;
    fn reset() {*MODULE.lock().unwrap()=ModuleState {enabled:true,settings:reach::Reach {through_walls_range:4.0,..Default::default()},rotation:None};}
    #[test]
    fn all_twelve_pickable_categories_replace_wall_and_feed_existing_actions() {
        let _scene=SCENE.lock().unwrap();reset();
        for kind in 0..12 {
            let mut game=Game::new();game.add(kind,40,2.0);
            let hit=game.client_reach_target().unwrap();assert_eq!(hit.kind,kind,"kind {kind}");
            assert!(hit.distance<game.block_reach);
            assert!(game.pick().is_none());assert!(game.pick_unfiltered().is_none());assert!(game.before_block(hit.distance));
            let present=[game.mob_hit().is_some(),game.painting_hit().is_some(),game.frame_hit().is_some(),game.armor_stand_hit().is_some(),
                game.cushion_hit().is_some(),game.boat_hit().is_some(),game.cart_hit().is_some(),game.interaction_hit().is_some(),
                game.redirectable_hit().is_some(),game.remote_knot_hit().is_some()];
            for (i,p) in present.into_iter().enumerate() {assert_eq!(p,usize::from(kind)==i,"candidate {kind}, query {i}");}
            assert_eq!(game.dragon_hit().is_some(),kind>=10);
            assert_eq!(game.remote_object_hit(true).is_some(),kind!=0&&kind!=8);
            assert_eq!(game.remote_object_hit(false).is_some(),kind!=0&&kind!=8);
            if matches!(kind,0|3) {assert_eq!(game.crosshair_living(),Some(true));}
        }
    }
    #[test]
    fn nearest_entity_across_categories_owns_the_pick_and_interaction_location() {
        let _scene=SCENE.lock().unwrap();reset();
        let mut game=Game::new();game.add(0,50,3.0);game.add(2,60,2.0);
        let hit=game.client_reach_target().unwrap();assert_eq!(hit.kind,2);assert_eq!(hit.id,60);
        assert!(game.mob_hit().is_none());assert!(game.frame_hit().is_some());
        let object=game.remote_object_hit(false).unwrap();assert_eq!(object.id,60);assert_eq!(object.location,hit.location);
        assert_eq!(game.crosshair_living(),None);
    }
    #[test]
    fn through_wall_hit_uses_strict_block_range_filter_and_retains_miss() {
        let _scene=SCENE.lock().unwrap();reset();let mut game=Game::new();game.add(0,50,3.0);
        game.block_reach=3.0;assert!(game.client_reach_target().is_some());
        assert!(game.mob_hit().is_none());assert!(game.pick().is_none());
        game.block_reach=3.01;assert!(game.mob_hit().is_some());
        MODULE.lock().unwrap().settings.through_walls_range=3.0;
        assert!(game.client_reach_target().is_none());assert!(game.pick().is_some());
    }
    #[test]
    fn active_item_nonmiss_precedes_through_walls_and_its_component_is_unchanged() {
        let _scene=SCENE.lock().unwrap();reset();let mut game=Game::new();game.add(0,50,2.0);
        let range=serde_json::json!({"min_reach":0.0,"max_reach":4.5,"hitbox_margin":0.3,"mob_factor":2.0});
        game.inventory[0]=Some(Stack {range:Some(range.clone())});
        assert!(game.attack_range_pick(&range).is_some()); // the wall is a nonmiss
        assert!(game.client_reach_target().is_none());assert!(game.pick().is_some());
        assert_eq!(game.stack_attack_range(game.held().unwrap()),Some(range.clone()));
        game.wall=None;assert!(game.attack_range_pick(&range).flatten().is_some());
        assert!(game.client_reach_target().is_none());assert_eq!(game.active_attack_range(),Some(range));
    }
    #[test]
    fn receiver_adjustment_preserves_component_and_margin_for_both_game_modes() {
        let _scene=SCENE.lock().unwrap();reset();let mut game=Game::new();
        let range=serde_json::json!({"min_reach":-1.0,"max_reach":4.5,"min_creative_reach":-2.0,"max_creative_reach":6.0,"hitbox_margin":0.25,"mob_factor":7.0});
        game.inventory[0]=Some(Stack {range:Some(range.clone())});
        assert!(game.held_range_allows(5.75));assert!(!game.held_range_allows(5.75001));
        assert!(!game.held_range_allows(-0.5));game.creative=true;
        assert!(game.held_range_allows(7.25));assert!(!game.held_range_allows(7.25001));
        MODULE.lock().unwrap().enabled=false;assert!(game.held_range_allows(-0.5));assert!(!game.held_range_allows(7.0));
        assert_eq!(game.active_attack_range(),Some(range));
    }
    #[test]
    fn actual_attribute_getters_use_the_source_float_and_double_orders() {
        let _scene=SCENE.lock().unwrap();reset();let mut game=Game::new();let raw=16_777_217.25;
        game.player.as_mut().unwrap().attributes.insert("minecraft:entity_interaction_range".into(),raw);
        game.player.as_mut().unwrap().attributes.insert("minecraft:block_interaction_range".into(),raw);
        assert_eq!(game.interaction_range("minecraft:entity_interaction_range",3.0,5.0),f64::from(raw as f32+1.0));
        assert_eq!(game.interaction_range("minecraft:block_interaction_range",4.5,5.0),raw+0.5);
        game.camera=Some(Camera {position:[9.0;3],eye_height:1.0,y_rot:20.0,x_rot:0.0});
        assert_eq!(game.interaction_range("minecraft:block_interaction_range",4.5,5.0),raw+0.5);
    }
    #[test]
    fn disabled_guest_server_replay_and_other_camera_preserve_unmodified_queries() {
        let _scene=SCENE.lock().unwrap();
        for mode in 0..7 {
            reset();let mut game=Game::new();game.add(0,50,2.0);
            match mode {0=>MODULE.lock().unwrap().enabled=false,1=>game.window=false,2=>game.lan_acting=Some(9),
                3=>game.player.as_mut().unwrap().server_copy=true,4=>game.player.as_mut().unwrap().replaying=true,
                5=>game.camera=Some(Camera {position:[0.0;3],eye_height:0.0,y_rot:0.0,x_rot:0.0}),
                _=>game.lan_acting_uuid=Some(91)}
            assert!(game.client_reach_target().is_none());assert!(game.pick().is_some());
            if mode!=5 {assert_eq!(game.interaction_range("minecraft:entity_interaction_range",3.0,5.0),3.0);}
        }
    }
    #[test]
    fn spectator_marker_and_shared_root_are_rejected_unless_eye_is_inside() {
        let _scene=SCENE.lock().unwrap();reset();let mut game=Game::new();game.add(0,50,2.0);
        game.mobs[0].extra.spectator=true;assert!(game.client_reach_target().is_none());
        game.mobs[0].extra.spectator=false;game.mobs[0].extra.mount=Some(70);game.riding=Some(70);
        assert!(game.client_reach_target().is_none());game.mobs[0].bounds=bounds(-0.1);
        assert_eq!(game.client_reach_target().unwrap().distance,0.0);
        game.mobs.clear();game.add(3,60,2.0);game.armor_stands[0].marker=true;
        assert!(game.client_reach_target().is_none());
    }
    #[test]
    fn block_rotation_always_uses_direction_from_rotation_and_entity_ray_only_when_managed() {
        let _scene=SCENE.lock().unwrap();reset();let mut game=Game::new();game.yaw=37.123;game.pitch = -21.456;
        assert_eq!(game.aim().unwrap().1,pick_view_vector(game.yaw,game.pitch));
        assert_eq!(game.client_block_aim().unwrap().1,client_rotation_direction(game.yaw,game.pitch));
        MODULE.lock().unwrap().rotation=Some((123.456,-54.321));
        assert_eq!(game.aim().unwrap().1,client_rotation_direction(123.456,-54.321));
        assert_eq!(game.client_block_aim().unwrap().1,client_rotation_direction(123.456,-54.321));
    }
    #[test]
    fn picker_and_rotation_bits_match_actual_minecraft_26_3_geometry_oracle() {
        let oracle:serde_json::Value=serde_json::from_str(include_str!("../java-vectors.json")).unwrap();
        for item in oracle["directions"].as_array().unwrap() {
            let direction=client_rotation_direction(item["yaw"].as_f64().unwrap() as f32,item["pitch"].as_f64().unwrap() as f32);
            for (axis,value) in direction.into_iter().enumerate() {assert_eq!(value.to_bits(),item["bits"][axis].as_u64().unwrap());}
        }
        for case in oracle["cases"].as_array().unwrap() {
            let candidates=case["candidates"].as_array().unwrap().iter().map(|v| {
                let n=|i:usize|v[i].as_f64().unwrap();
                ClientReachCandidate {kind:0,index:0,id:v[0].as_i64().unwrap() as i32,local_id:0,position:[0.0;3],
                    bounds:Aabb::new(Vec3::new(n(1),n(2),n(3)),Vec3::new(n(4),n(5),n(6))),pick_radius:n(7),
                    same_root_vehicle:v[8].as_bool().unwrap(),can_pick_inside:v[9].as_bool().unwrap()}
            });
            let picked=client_reach_pick(candidates,Vec3::ZERO,Vec3::new(0.0,0.0,1.0),Aabb::new(Vec3::new(-0.3,-1.6,-0.3),Vec3::new(0.3,0.2,0.3)),4.0);
            assert_eq!(picked.map_or(-1,|p|p.id),case["target"].as_i64().unwrap() as i32);
            if let Some(hit)=picked {for (axis,v) in hit.point.into_iter().enumerate() {assert_eq!(v.to_bits(),case["pointBits"][axis].as_u64().unwrap());}}
        }
    }
}
