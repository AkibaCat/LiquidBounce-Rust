use std::{collections::BTreeMap,sync::{Mutex,OnceLock},cell::RefCell};
use horizon_items::{blocks::BlockFacts,items::{ItemStack,Items}};
use horizon_world::{registry::BlockRegistry,consumable::UseAnimation};
use serde_json::Value;
use horizon_world::nbt::Tag;
pub mod brewing {pub use horizon_client_core::potions::potion;}
pub mod first_person {pub use horizon_items::use_animation::consumable_animation;}
pub mod items {pub use horizon_items::items::*;}
pub mod item_data {pub use horizon_items::item_data::*;}
type Components=BTreeMap<String,Value>;
pub struct StackView {item:String,count:i32,bundle_selected:i32,patch:BTreeMap<String,Option<Value>>}
impl StackView {fn new(item:&str,count:i32)->Self {Self {item:item.into(),count,bundle_selected:-1,patch:BTreeMap::new()}}}
#[derive(Clone,Copy)]struct ComponentsView<'a> {prototype:&'a Components,patch:&'a BTreeMap<String,Option<Value>>}
#[allow(clippy::wrong_self_convention)]
__COMPONENTS_VIEW__
struct Models {prototypes:BTreeMap<String,Components>}
impl Models {fn components_view<'a>(&'a self,stack:&'a StackView)->ComponentsView<'a> {static NONE:Components=BTreeMap::new();ComponentsView {prototype:self.prototypes.get(&stack.item).unwrap_or(&NONE),patch:&stack.patch}}}
struct ItemRenderer {models:Models}struct Pack {item_renderer:ItemRenderer}
pub mod item_render {use super::*;__TAG_JSON__ __CUSTOM_JSON__ __STACK_VIEW__}
struct Resources {items:Items}
fn resources()->&'static Resources {static RES:OnceLock<Resources>=OnceLock::new();RES.get_or_init(||{let root=std::path::Path::new("__RESOURCE_ROOT__");let registry=BlockRegistry::from_facts_json(std::fs::File::open(root.join("facts.json")).unwrap()).unwrap();let facts=BlockFacts::load(root,&registry).unwrap();Resources {items:Items::load(root,&registry,&facts,&horizon_world::loot::PackOverlay::NONE).unwrap()}})}
struct FoodHealth {food:i32}struct Health {food:FoodHealth,value:f32}impl Health {fn health(&self)->f32 {self.value}}
struct Player {health:Health,invulnerable:bool,server:bool,replay:bool}impl Player {fn native_client_queries_allowed(&self)->bool {!self.server&&!self.replay}}
struct HudBatch {width:f32,height:f32}
static TRACE:Mutex<Vec<String>>=Mutex::new(Vec::new());
#[allow(dead_code)] // Minimal generated-API scaffold, not product code.
mod mod_api {
    use super::*;pub trait Host {}pub trait RenderHost {}pub trait RenderHooks:Send+Sync {}pub struct ModHandle;
    pub type Mod=dyn Hooks;pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks> {mods.iter().map(Box::as_ref)}
    mod generated {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}pub use generated::*;
    struct Adapter;impl Hooks for Adapter {
        fn client_smart_eat_wait(&self,_:&mut dyn Host,_:&mut ClientSmartEatWaitEvent) {TRACE.lock().unwrap().push("wait".into());}
        fn client_use_key_pressed(&self,_:&dyn Host,e:&mut ClientUseKeyPressedEvent) {TRACE.lock().unwrap().push(format!("useKey:{}",e.pressed));e.pressed=true;}
        fn client_item_interacted(&self,_:&mut dyn Host,e:&mut ClientItemInteractedEvent) {TRACE.lock().unwrap().push(format!("item:{}",e.pass));}
    }
    pub fn mods()->&'static [Box<dyn Hooks>] {static MODS:OnceLock<Vec<Box<dyn Hooks>>>=OnceLock::new();MODS.get_or_init(||vec![Box::new(Adapter)]).as_slice()}
}
struct Game {window:bool,lan_acting:Option<u32>,lan_acting_uuid:Option<u32>,player:Option<Player>,right:bool,
    inventory:Vec<Option<ItemStack>>,offhand:Option<ItemStack>,selected:usize,silent:Option<usize>,using:Option<(usize,u32)>,pack:Pack,trace:RefCell<Vec<String>>}
impl mod_api::Host for Game {}
impl Game {
    fn new()->Self {Self {window:true,lan_acting:None,lan_acting_uuid:None,player:Some(Player {health:Health {food:FoodHealth {food:14},value:12.},invulnerable:false,server:false,replay:false}),right:false,inventory:vec![None;9],offhand:None,selected:0,silent:None,using:None,pack:Pack {item_renderer:ItemRenderer {models:Models {prototypes:serde_json::from_slice(&std::fs::read("__RESOURCE_ROOT__/items.json").unwrap()).unwrap()}}},trace:RefCell::new(vec![])}}
    fn native_window(&self)->bool {self.window}fn items(&self)->&Items {&resources().items}fn client_hotbar_slot(&self)->usize {self.silent.unwrap_or(self.selected)}
    fn stack_in(&self,slot:usize)->Option<&ItemStack> {if slot==40 {self.offhand.as_ref()}else {self.inventory.get(slot).and_then(Option::as_ref)}}
    fn native_use_animation(&self,_:&ItemStack,_:&mut UseAnimation) {}
    fn draw_item_decorations(&self,_:&mut HudBatch,_:&ItemStack,x:f32,y:f32,_:Option<String>) {self.trace.borrow_mut().push(format!("decorations:{x},{y}"));}
    fn draw_item_model(&self,_:&mut HudBatch,_:&ItemStack,x:f32,y:f32) {self.trace.borrow_mut().push(format!("model:{x},{y}"));}
    fn native_sprite(&self,_:&mut HudBatch,name:&str,src:[f32;4],dst:[f32;4])->bool {self.trace.borrow_mut().push(format!("sprite:{name}:{src:?}:{dst:?}"));true}
    __NATIVE__
}
#[test]fn native_snapshot_uses_actual_food_potion_and_mining_item_components() {let mut g=Game::new();g.inventory[0]=Some(ItemStack::new("minecraft:diamond_pickaxe",1));g.inventory[2]=Some(ItemStack::new("minecraft:bread",1));let mut potion=ItemStack::new("minecraft:potion",1);potion.data_mut().potion=Some("healing".into());g.offhand=Some(potion);let c=crate::actor::parse_snapshot(&g.native_client_smart_eat_snapshot()).unwrap().unwrap();assert_eq!(c.stacks[0].slot,crate::smart_eat::Slot::Offhand);assert!(c.stacks[0].instant_health);assert!(c.stacks[0].food.is_none());assert!(c.main_mining_tool);assert_eq!(c.stacks[3].food.unwrap().nutrition,5);assert_eq!((c.health,c.hunger),(12.,14));}

#[test]fn native_snapshot_can_eat_reads_invulnerable_ability_and_hunger() {
    let mut g=Game::new();g.player.as_mut().unwrap().health.food.food=20;
    assert!(!crate::actor::parse_snapshot(&g.native_client_smart_eat_snapshot()).unwrap().unwrap().can_eat);
    g.player.as_mut().unwrap().invulnerable=true;
    assert!(crate::actor::parse_snapshot(&g.native_client_smart_eat_snapshot()).unwrap().unwrap().can_eat);
    g.player.as_mut().unwrap().invulnerable=false;g.player.as_mut().unwrap().health.food.food=19;
    assert!(crate::actor::parse_snapshot(&g.native_client_smart_eat_snapshot()).unwrap().unwrap().can_eat);
}
#[test]fn snapshot_resolves_component_removal_custom_food_and_potion_effects() {let mut g=Game::new();let mut s=ItemStack::new("minecraft:bread",1);s.data_mut().set_other("!minecraft:food",Some(Tag::compound([] as [(&str,Tag);0])));g.inventory[1]=Some(s);let mut s=ItemStack::new("minecraft:stick",1);s.data_mut().set_other("minecraft:food",Some(Tag::compound([("nutrition",Tag::Int(30)),("can_always_eat",Tag::Byte(1))])));g.inventory[2]=Some(s);let mut s=ItemStack::new("minecraft:potion",1);s.data_mut().potion_extra=vec![("custom_effects".into(),Tag::List(vec![Tag::compound([("id",Tag::String("minecraft:instant_health".into()))])]))];g.offhand=Some(s);let c=crate::actor::parse_snapshot(&g.native_client_smart_eat_snapshot()).unwrap().unwrap();assert!(c.stacks[2].food.is_none());assert_eq!(c.stacks[3].food,Some(crate::smart_eat::Food {nutrition:30,can_always_eat:true}));assert!(c.stacks[0].instant_health);}
#[test]fn native_use_animation_respects_actual_java_subclass_dispatch() {let mut g=Game::new();for (item,expected) in [("minecraft:bread",true),("minecraft:potion",true),("minecraft:bow",false)] {let mut s=ItemStack::new(item,1);s.data_mut().set_other("minecraft:consumable",Some(Tag::compound([("animation",Tag::String("drink".into()))])));g.inventory[0]=Some(s);g.using=Some((0,1));let c=crate::actor::parse_snapshot(&g.native_client_smart_eat_snapshot()).unwrap().unwrap();assert_eq!(c.using_food_or_drink,expected,"{item}");}g.using=None;assert!(!crate::actor::parse_snapshot(&g.native_client_smart_eat_snapshot()).unwrap().unwrap().using_food_or_drink);}
#[test]fn native_key_query_is_ephemeral_and_rejects_guest_replay_server() {TRACE.lock().unwrap().clear();let mut g=Game::new();g.native_client_smart_eat_wait();assert!(g.client_use_key_down());assert!(!g.right);g.native_client_item_interacted(true);assert_eq!(TRACE.lock().unwrap().as_slice(),["wait","useKey:false","item:true"]);for i in 0..6 {let mut g=Game::new();match i {0=>g.window=false,1=>g.lan_acting=Some(1),2=>g.lan_acting_uuid=Some(1),3=>g.player=None,4=>g.player.as_mut().unwrap().server=true,_=>g.player.as_mut().unwrap().replay=true};assert!(!g.client_use_key_down());assert_eq!(g.native_client_smart_eat_snapshot(),"null");}}
#[test]fn native_renderer_keeps_decorations_model_sprite_order_and_integer_gui_center() {let mut g=Game::new();g.inventory[1]=Some(ItemStack::new("minecraft:bread",1));g.native_client_smart_eat_draw(&mut HudBatch {width:321.,height:240.},1,40);assert_eq!(g.trace.borrow().as_slice(),["decorations:3,221","model:3,221","sprite:hud/hotbar_offhand_left:[0.0, 0.0, 29.0, 24.0]:[0.0, 217.0, 29.0, 24.0]"]);}
#[test]fn actual_component_view_methods_and_stack_count_are_preserved() {let components=BTreeMap::from([("minecraft:food".into(),serde_json::json!({"nutrition":5}))]);let view=ComponentsView::of(&components);assert!(view.contains_key("minecraft:food"));assert_eq!(view.to_map(),components);assert_eq!(item_render::stack_view(&ItemStack::new("minecraft:bread",2)).count,2);}
