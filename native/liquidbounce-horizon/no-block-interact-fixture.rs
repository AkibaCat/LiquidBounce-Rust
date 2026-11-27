#![allow(dead_code)]
#![allow(clippy::wrong_self_convention)] // Unmodified upstream ComponentsView::to_map.
extern crate self as horizon_client;
use std::{collections::BTreeMap,sync::{LazyLock,Mutex,OnceLock}};
use serde_json::Value;
use horizon_items::{blocks::BlockFacts,items::{ItemStack,Items}};
use horizon_world::{nbt::Tag,registry::{BlockRegistry,BlockStateId}};
type Result<T> = std::result::Result<T,()>;
pub mod items { pub use horizon_items::items::*; }
pub mod item_data { pub use horizon_items::item_data::*; }
pub mod interaction_input { pub use horizon_client_input::interaction_input::*; }
#[path="__ENTITY_RULES_PATH__"] mod no_entity_interact;
#[path="__RULES_PATH__"] mod no_block_interact;
__RULES__
static SCENE:Mutex<()>=Mutex::new(());
struct ModuleState { running:bool, module:no_block_interact::NoBlockInteract }
static MODULE:LazyLock<Mutex<ModuleState>>=LazyLock::new(||Mutex::new(ModuleState{running:true,module:Default::default()}));
mod mod_api {
    use super::*;
    pub trait Host {}
    pub trait RenderHost {}
    pub trait RenderHooks:Send+Sync {}
    pub struct ModHandle;
    pub type Mod = dyn Hooks;
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks> {mods.iter().map(Box::as_ref)}
    mod generated {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}
    pub use generated::*;
    struct Adapter;
    impl Hooks for Adapter {
        fn client_block_interact(&self,_:&dyn Host,event:&mut ClientBlockInteractEvent) {
            let mut state=MODULE.lock().unwrap();
            if state.running && event.should_sneak { state.module.start_sneaking(); event.canceled=true; }
        }
    }
    pub fn mods()->&'static [Box<dyn Hooks>] {static MODS:OnceLock<Vec<Box<dyn Hooks>>>=OnceLock::new();MODS.get_or_init(||vec![Box::new(Adapter)]).as_slice()}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]enum Hand {Main,Off}
impl Hand {const BOTH:[Self;2]=[Self::Main,Self::Off];}
#[derive(Clone,Copy)]struct InteractTarget;
#[derive(Clone,Copy)]enum GuestUse {OnBlock(Hand),Item(Hand),Entity(Hand,InteractTarget)}
#[derive(Clone,Copy)]struct Hit {pos:[i32;3],face:[i32;3]}
#[derive(Default)]struct Remote {is_destroying:bool}
#[derive(Default)]struct Food {food:i32}
#[derive(Default)]struct Health {food:Food}
#[derive(Default)]struct Player {shift_key_down:bool,health:Health,server_copy:bool,replaying:bool}
impl Player {fn native_client_reach_queries_allowed(&self)->bool {!self.server_copy && !self.replaying}}
#[derive(Default)]struct Destroying {active:bool}
#[derive(Default)]struct Clicks {destroying:Destroying}
#[derive(Default)]struct Heights {used:Vec<Hand>}
impl Heights {fn item_used(&mut self,hand:Hand) {self.used.push(hand);}}
struct SwingAnimation;impl SwingAnimation {const DEFAULT:Self=Self;}
struct World {block:BlockStateId}
impl World {fn block(&self,_:i32,_:i32,_:i32)->BlockStateId {self.block}}
struct Resources {facts:BlockFacts,registry:BlockRegistry,items:Items}
impl Resources {fn facts(&self)->&BlockFacts {&self.facts}}
fn resources()->&'static Resources {
    static RES:OnceLock<Resources>=OnceLock::new();RES.get_or_init(|| {
        let root=std::path::Path::new("__RESOURCE_ROOT__");
        let registry=BlockRegistry::from_facts_json(std::fs::File::open(root.join("facts.json")).unwrap()).unwrap();
        let facts=BlockFacts::load(root,&registry).unwrap();
        let items=Items::catalog(root,&horizon_world::loot::PackOverlay::NONE).unwrap();
        Resources {facts,registry,items}
    })
}
type Components=BTreeMap<String,Value>;
pub struct StackView {item:String,count:i32,bundle_selected:i32,patch:BTreeMap<String,Option<Value>>}
impl StackView {fn new(item:&str,count:i32)->Self {Self {item:item.into(),count,bundle_selected:-1,patch:BTreeMap::new()}}}
#[derive(Clone,Copy)]struct ComponentsView<'a> {prototype:&'a Components,patch:&'a BTreeMap<String,Option<Value>>}
__COMPONENTS_VIEW__
struct Models {prototypes:BTreeMap<String,Components>}
impl Models {
    fn components_view<'a>(&'a self,stack:&'a StackView)->ComponentsView<'a> {static NONE:Components=BTreeMap::new();ComponentsView{prototype:self.prototypes.get(&stack.item).unwrap_or(&NONE),patch:&stack.patch}}
}
struct Renderer {models:Models}
struct Assets {registry:&'static BlockRegistry}
struct Pack {item_renderer:Renderer,assets:Assets}
fn pack()->Pack {
    let prototypes=serde_json::from_slice(&std::fs::read("__RESOURCE_ROOT__/items.json").unwrap()).unwrap();
    Pack {item_renderer:Renderer{models:Models{prototypes}},assets:Assets{registry:&resources().registry}}
}
pub mod item_render {
    use super::*;
    __TAG_JSON__
    __CUSTOM_JSON__
    __STACK_VIEW__
}
struct Game {
    window:Option<()>,lan_acting:Option<u32>,lan_acting_uuid:Option<u32>,player:Option<Player>,
    remote:Option<Remote>,inventory:Vec<Option<ItemStack>>,offhand:Option<ItemStack>,armor:[Option<ItemStack>;4],
    selected:usize,silent:Option<usize>,creative:bool,master:bool,world:World,res:&'static Resources,pack:Pack,
    guest_use:Option<GuestUse>,riding:Option<u32>,hand:Hand,clicks:Clicks,using:Option<()>,swings_started:u32,
    heights:Heights,use_delay:u32,busy:bool,entity:bool,entity_consumes:bool,item_consumes:[bool;2],block_consumes:[bool;2],trace:Vec<String>,has_hit:bool,
}
impl mod_api::Host for Game {}
impl Game {
    fn new()->Self {
        let res=resources();Self {window:Some(()),lan_acting:None,lan_acting_uuid:None,player:Some(Player{health:Health{food:Food{food:20}},..Default::default()}),remote:Some(Remote::default()),inventory:vec![None;36],offhand:None,armor:std::array::from_fn(|_|None),selected:0,silent:None,creative:false,master:false,world:World{block:res.registry.default_state("minecraft:chest").unwrap()},res,pack:pack(),guest_use:None,riding:None,hand:Hand::Main,clicks:Default::default(),using:None,swings_started:0,heights:Default::default(),use_delay:0,busy:false,entity:false,entity_consumes:false,item_consumes:[false;2],block_consumes:[false;2],trace:Vec::new(),has_hit:true}
    }
    fn items(&self)->&Items {&self.res.items}
    fn client_hotbar_slot(&self)->usize {self.silent.unwrap_or(self.selected)}
    fn stack_in(&self,slot:usize)->Option<&ItemStack> {self.inventory.get(slot)?.as_ref()}
    fn held(&self)->Option<&ItemStack> {if self.hand==Hand::Main {self.stack_in(self.client_hotbar_slot())} else {self.offhand.as_ref()}}
    fn with_hand<T>(&mut self,hand:Hand,action:impl FnOnce(&mut Self)->T)->T {let previous=self.hand;self.hand=hand;let result=action(self);self.hand=previous;result}
    fn hands_busy(&self)->bool {self.busy}
    fn spectator(&self)->bool {false}
    fn spectator_open_menu(&mut self,_:[i32;3]) {}
    fn pick(&self)->Option<Hit> {self.has_hit.then_some(Hit{pos:[0,0,0],face:[0,1,0]})}
    fn can_use_game_master_blocks(&self)->bool {self.master}
    fn entity_targeted(&self)->bool {self.entity}
    fn remote_use_targets_entity(&self)->bool {self.entity}
    fn use_steering_stick(&mut self,_:&str)->bool {false}
    fn interact_entity(&mut self,_:Option<InteractTarget>)->Result<bool> {self.trace.push(format!("entity:{:?}",self.hand));Ok(self.entity && self.entity_consumes)}
    fn interact_knot(&mut self)->bool {false}
    fn use_hand(&mut self,_:Option<&Hit>)->Result<bool> {self.trace.push(format!("local:{:?}",self.hand));Ok(self.block_consumes[self.hand as usize])}
    fn use_item(&mut self)->Result<Option<Hand>> {self.use_item_as_player()}
    fn native_interaction(&mut self,_:u8,_:Hand)->Option<bool> {None}
    fn mod_use_start(&mut self)->bool {false}
    fn swing_hand(&mut self,_:Hand,_:SwingAnimation) {}
    fn remote_use_entity(&mut self,hand:Hand)->bool {self.trace.push(format!("entity:{hand:?}"));self.entity_consumes}
    fn remote_use_item_on(&mut self,hand:Hand)->bool {self.trace.push(format!("block:{hand:?}"));self.block_consumes[hand as usize]}
    fn remote_use_item(&mut self,hand:Hand)->bool {self.trace.push(format!("item:{hand:?}"));self.item_consumes[hand as usize]}
    __NATIVE__
    __USE_ITEM_AS_PLAYER__
    __USE_WITH_HAND__
    __REMOTE_USE__
    __REMOTE_USE_BLOCK__
    __REMOTE_USE_HANDS__
}
mod spans {pub fn time<T>(_:&str,f:impl FnOnce()->T)->T {f()}}
struct RemoteUseWorld<'a>(&'a mut Game);
__REMOTE_USE_WORLD__

#[cfg(test)] mod tests {
    use super::*;
    fn reset() {*MODULE.lock().unwrap()=ModuleState{running:true,module:Default::default()};}
    fn put(game:&mut Game,item:&str) {game.inventory[0]=Some(ItemStack::new(item,1));}
    fn patched(item:&str,key:&str,value:Value)->ItemStack {let mut stack=ItemStack::new(item,1);stack.data_mut().set_loot_component(key,&value);stack}
    fn tick_head(game:&mut Game) {
        let retry={let state=MODULE.lock().unwrap();state.module.should_retry(state.running)};
        if retry {
            game.native_client_start_use_item();
            {
                let state = MODULE.lock().unwrap();
                assert!(state.module.sneaking && state.module.interacting);
            }
            MODULE.lock().unwrap().module.finish_retry();
        }
    }
    #[test]fn remote_cancels_entire_click_then_input_then_next_tick_retries() {
        let _lock=SCENE.lock().unwrap();reset();let mut game=Game::new();put(&mut game,"minecraft:stone");
        game.remote_use();assert_eq!(game.use_delay,4);assert!(game.trace.is_empty());
        assert!(MODULE.lock().unwrap().module.sneaking);assert!(!MODULE.lock().unwrap().module.interacting);
        tick_head(&mut game);assert!(game.trace.is_empty());
        let mut sneak=false;MODULE.lock().unwrap().module.movement_input(true,&mut sneak);game.player.as_mut().unwrap().shift_key_down=sneak;
        tick_head(&mut game);assert_eq!(game.trace,["block:Main","item:Main","block:Off","item:Off"]);
        assert!(!MODULE.lock().unwrap().module.sneaking);assert!(!MODULE.lock().unwrap().module.interacting);
    }
    #[test]fn local_loop_and_remote_offhand_cancel_at_use_item_on_invocation() {
        let _lock=SCENE.lock().unwrap();reset();let mut game=Game::new();game.remote=None;put(&mut game,"minecraft:stone");
        assert_eq!(game.use_item_as_player().unwrap(),None);assert_eq!(game.trace,["entity:Main"]);
        reset();game.trace.clear();game.guest_use=Some(GuestUse::OnBlock(Hand::Off));
        game.use_item_as_player().unwrap();assert_eq!(game.trace,["local:Off"]);
        reset();game.trace.clear();game.guest_use=None;game.remote=Some(Remote::default());
        // Direct offhand invocation must consume the complete remaining click.
        assert!(game.remote_use_block(Hand::Off));assert!(game.trace.is_empty());
    }
    #[test]fn breaking_busy_entity_and_disabled_paths_keep_the_original_order() {
        let _lock=SCENE.lock().unwrap();reset();let mut game=Game::new();put(&mut game,"minecraft:stone");
        game.remote.as_mut().unwrap().is_destroying=true;game.remote_use();assert_eq!(game.use_delay,0);assert!(!MODULE.lock().unwrap().module.sneaking);
        game.remote.as_mut().unwrap().is_destroying=false;game.busy=true;game.remote_use();assert_eq!(game.use_delay,4);assert!(!MODULE.lock().unwrap().module.sneaking);
        game.busy=false;game.entity=true;game.entity_consumes=true;game.remote_use();assert_eq!(game.trace,["entity:Main"]);assert!(!MODULE.lock().unwrap().module.sneaking);
        game.entity=false;game.trace.clear();MODULE.lock().unwrap().running=false;game.remote_use();assert_eq!(game.trace,["block:Main","item:Main","block:Off","item:Off"]);
    }
    #[test]fn actual_component_conversion_handles_removal_boolean_defaults_and_equipment() {
        let _lock=SCENE.lock().unwrap();reset();let mut game=Game::new();
        let apple=ItemStack::new("minecraft:apple",1);assert!(game.client_no_block_item(Some(&apple)));
        let mut no_food=apple.clone();no_food.data_mut().set_other("!minecraft:consumable",Some(Tag::Compound(Default::default())));assert!(!game.client_no_block_item(Some(&no_food)));
        for (value,want) in [(serde_json::json!({}),false),(serde_json::json!({"can_sprint":true}),true),(serde_json::json!({"interact_vibrations":false}),true),(serde_json::json!({"speed_multiplier":0.3}),true)] {
            let item=patched("minecraft:stick","minecraft:use_effects",value);assert_eq!(game.client_no_block_item(Some(&item)),want);
        }
        let helmet=ItemStack::new("minecraft:diamond_helmet",1);assert!(game.client_no_block_item(Some(&helmet)));game.armor[0]=Some(helmet.clone());assert!(!game.client_no_block_item(Some(&helmet)));
        let boots=patched("minecraft:stick","minecraft:equippable",serde_json::json!({"slot":"feet","swappable":false}));assert!(!game.client_no_block_item(Some(&boots)));
        let restricted=patched("minecraft:stick","minecraft:equippable",serde_json::json!({"slot":"feet","allowed_entities":["minecraft:pig"]}));assert!(!game.client_no_block_item(Some(&restricted)));
        let allowed=patched("minecraft:stick","minecraft:equippable",serde_json::json!({"slot":"feet","allowed_entities":["minecraft:player"]}));assert!(game.client_no_block_item(Some(&allowed)));
        game.armor[3]=Some(ItemStack::new("minecraft:iron_boots",1));game.armor[3].as_mut().unwrap().enchant("minecraft:binding_curse",1);assert!(!game.client_no_block_item(Some(&allowed)));game.creative=true;assert!(game.client_no_block_item(Some(&allowed)));
    }
    #[test]fn arrow_inventory_armor_and_hands_match_exact_class_oracle() {
        let _lock=SCENE.lock().unwrap();reset();let mut game=Game::new();let bow=ItemStack::new("minecraft:bow",1);let crossbow=ItemStack::new("minecraft:crossbow",1);
        assert!(!game.client_no_block_item(Some(&bow)));assert!(!game.client_no_block_item(Some(&crossbow)));
        game.offhand=Some(ItemStack::new("minecraft:firework_rocket",1));assert!(game.client_no_block_item(Some(&crossbow)));assert!(!game.client_no_block_item(Some(&bow)));
        game.offhand=None;game.armor[2]=Some(ItemStack::new("minecraft:spectral_arrow",1));assert!(game.client_no_block_item(Some(&bow)));
        game.armor[2]=None;game.inventory[35]=Some(ItemStack::new("minecraft:tipped_arrow",1));assert!(game.client_no_block_item(Some(&bow)));
        let oracle:Value=serde_json::from_str(include_str!("../oracle.json")).unwrap();
        let classes:std::collections::BTreeSet<_>=oracle["items"].as_array().unwrap().iter().flat_map(|item|item["classes"].as_array().unwrap().iter().map(|value|value.as_str().unwrap())).collect();
        for item in oracle["items"].as_array().unwrap() {for class in &classes {assert_eq!(client_no_block_rules::item_facts::item_is(item["id"].as_str().unwrap(),class),item["classes"].as_array().unwrap().iter().any(|name|name==class));}}
    }
    #[test]fn block_class_interface_and_state_predicates_use_actual_registry() {
        let _lock=SCENE.lock().unwrap();reset();let mut game=Game::new();put(&mut game,"minecraft:stone");
        for (name,expected) in [("minecraft:chest",true),("minecraft:stone",false),("minecraft:cake",false),("minecraft:jukebox",false),("minecraft:command_block",false)] {
            game.world.block=game.res.registry.default_state(name).unwrap();assert_eq!(game.client_cancel_block_interact(&game.pick().unwrap()),expected,"{name}");
        }
        game.master=true;game.world.block=game.res.registry.default_state("minecraft:command_block").unwrap();assert!(game.client_cancel_block_interact(&game.pick().unwrap()));
        game.world.block=game.res.registry.default_state("minecraft:cake").unwrap();game.player.as_mut().unwrap().health.food.food=19;assert!(game.client_cancel_block_interact(&game.pick().unwrap()));
        game.player.as_mut().unwrap().shift_key_down=true;assert!(!game.client_cancel_block_interact(&game.pick().unwrap()));
    }
    #[test]fn all_real_block_states_and_all_class_interfaces_match_oracle_and_predicates() {
        let _lock=SCENE.lock().unwrap();reset();let mut game=Game::new();put(&mut game,"minecraft:stone");
        let oracle:Value=serde_json::from_str(include_str!("../oracle.json")).unwrap();
        let blocks:BTreeMap<_,_>=oracle["blocks"].as_array().unwrap().iter().map(|row|(row["id"].as_str().unwrap(),row)).collect();
        let classes:std::collections::BTreeSet<_>=blocks.values().flat_map(|row|row["classes"].as_array().unwrap().iter().map(|class|class.as_str().unwrap())).collect();
        for row in blocks.values() {for class in &classes {assert_eq!(client_no_block_rules::item_facts::block_is(row["id"].as_str().unwrap(),class),row["classes"].as_array().unwrap().iter().any(|name|name==class));}}
        for id in 0..game.res.registry.len() {
            let id=BlockStateId(id as u32);let state=game.res.registry.state(id).unwrap();
            let classes=blocks[state.name.as_str()]["classes"].as_array().unwrap();
            for (hungry,master) in [(false,false),(true,true)] {
                game.player.as_mut().unwrap().health.food.food=if hungry {19}else{20};game.master=master;game.world.block=id;
                let expected=no_block_interact::block_is_interactable(|class|classes.iter().any(|name|name==class),|name|state.properties.get(name).map(String::as_str),hungry,master);
                assert_eq!(game.client_cancel_block_interact(&game.pick().unwrap()),expected,"{} {:?}",state.name,state.properties);
            }
        }
    }
    #[test]fn offhand_and_silent_main_hand_drive_predicate_with_native_identity_guards() {
        let _lock=SCENE.lock().unwrap();reset();let mut game=Game::new();put(&mut game,"minecraft:diamond_sword");
        assert!(!game.client_cancel_block_interact(&game.pick().unwrap()));
        game.offhand=Some(ItemStack::new("minecraft:stone",1));assert!(game.client_cancel_block_interact(&game.pick().unwrap()));
        game.offhand=None;game.inventory[1]=Some(ItemStack::new("minecraft:glass_bottle",1));game.silent=Some(1);assert!(game.client_cancel_block_interact(&game.pick().unwrap()));
        for guard in 0..5 {
            game.window=Some(());game.lan_acting=None;game.lan_acting_uuid=None;game.player.as_mut().unwrap().server_copy=false;game.player.as_mut().unwrap().replaying=false;
            match guard {0=>game.window=None,1=>game.lan_acting=Some(2),2=>game.lan_acting_uuid=Some(9),3=>game.player.as_mut().unwrap().server_copy=true,_=>game.player.as_mut().unwrap().replaying=true}
            assert!(!game.client_cancel_block_interact(&game.pick().unwrap()));
        }
    }
}
