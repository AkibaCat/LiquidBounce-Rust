use std::{cell::{Cell,RefCell},collections::{HashMap,BTreeMap},sync::{Mutex,OnceLock}};
use horizon_items::{blocks::BlockFacts,items::{ItemStack,Items}};
use horizon_world::registry::{BlockRegistry,BlockStateId};
use horizon_protocol::packets::{Empty,play::Serverbound as S};
use serde_json::Value;
use horizon_world::nbt::Tag;
pub mod stations {use super::*;__MAX_DAMAGE__ __COMPONENT_INT__}
pub mod menu {#[derive(Clone,Copy)]pub enum Input{Swap}}
pub mod spear {pub struct Kinetic;impl Kinetic{pub fn of(_:&str)->Option<Self>{None}}}
pub mod winit {pub mod dpi {pub struct PhysicalPosition;impl PhysicalPosition{pub fn new(_:f32,_:f32)->Self{Self}}}}
struct Window;impl Window{fn set_cursor_position(&self,_:winit::dpi::PhysicalPosition)->Result<(),()>{Ok(())}}
struct Renderer{config:RendererConfig}struct RendererConfig{width:u32,height:u32}
struct HudBatch{scale:f32}impl HudBatch{fn new(_:[u32;2])->Self{Self{scale:1.}}}
__FIRST_PERSON__
static TRACE:Mutex<Vec<String>>=Mutex::new(Vec::new());
mod mod_api {
    use super::*;
    pub trait Host{}pub trait RenderHost{}pub trait RenderHooks:Send+Sync{}pub struct ModHandle;
    pub type Mod=dyn Hooks;
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(Box::as_ref)}
    mod generated{include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}pub use generated::*;
    struct Adapter;impl Hooks for Adapter{
        fn client_block_breaking_progress(&self,_:&dyn Host,event:&mut ClientBlockBreakingProgressEvent){TRACE.lock().unwrap().push(format!("progress:{:?}",event.pos));}
        fn client_silent_hotbar_render(&self,_:&dyn Host,event:&mut ClientSilentHotbarRenderEvent){event.visible_slot=2;event.swap_scale=1.;}
    }
    pub fn mods()->&'static [Box<dyn Hooks>]{static MODS:OnceLock<Vec<Box<dyn Hooks>>>=OnceLock::new();MODS.get_or_init(||vec![Box::new(Adapter)]).as_slice()}
}
struct Resources{facts:BlockFacts,registry:BlockRegistry,items:Items}
impl Resources{fn facts(&self)->&BlockFacts{&self.facts}}
fn resources()->&'static Resources{static RES:OnceLock<Resources>=OnceLock::new();RES.get_or_init(||{let root=std::path::Path::new("__RESOURCE_ROOT__");let registry=BlockRegistry::from_facts_json(std::fs::File::open(root.join("facts.json")).unwrap()).unwrap();let facts=BlockFacts::load(root,&registry).unwrap();let items=Items::load(root,&registry,&facts,&horizon_world::loot::PackOverlay::NONE).unwrap();Resources{facts,registry,items}})}
type Components=BTreeMap<String,Value>;
pub struct StackView{item:String,count:i32,bundle_selected:i32,patch:BTreeMap<String,Option<Value>>}
impl StackView{fn new(item:&str,count:i32)->Self{Self{item:item.into(),count,bundle_selected:-1,patch:BTreeMap::new()}}}
#[derive(Clone,Copy)]struct ComponentsView<'a>{prototype:&'a Components,patch:&'a BTreeMap<String,Option<Value>>}
#[allow(clippy::wrong_self_convention)]
__COMPONENTS_VIEW__
struct Models{prototypes:BTreeMap<String,Components>}
impl Models{fn components_view<'a>(&'a self,stack:&'a StackView)->ComponentsView<'a>{static NONE:Components=BTreeMap::new();ComponentsView{prototype:self.prototypes.get(&stack.item).unwrap_or(&NONE),patch:&stack.patch}}}
struct ItemRenderer{models:Models}
struct Assets{registry:&'static BlockRegistry}struct Pack{assets:Assets,item_renderer:ItemRenderer}
pub mod item_render{use super::*;__TAG_JSON__ __CUSTOM_JSON__ __STACK_VIEW__}
struct Blocks(Vec<u32>);impl Blocks{fn maybe_has(&self,f:&impl Fn(u32)->bool)->bool{self.0.iter().copied().any(f)}}
struct Section{blocks:Blocks}impl Section{fn get(&self,n:usize)->u32{self.blocks.0[n]}}
struct Chunk{sections:Vec<Section>}impl Chunk{fn min_y(&self)->i32{0}}
struct World{block:BlockStateId,chunks:HashMap<(i32,i32),Chunk>}impl World{fn block(&self,_:i32,_:i32,_:i32)->BlockStateId{self.block}}
#[derive(Default)]struct Health{tick_count:i32}
#[derive(Default)]struct Player{health:Health,shift_key_down:bool,move_vector:[f32;2],jumping:bool,yaw:f32,pitch:f32,server:bool,replay:bool}
impl Player{fn native_client_reach_queries_allowed(&self)->bool{!self.server&&!self.replay}}
#[derive(Default)]struct Remote{is_destroying:bool,destroying:Option<()>}
#[derive(Clone,Copy,PartialEq,Eq)]enum Screen{Playing,Inventory}
#[derive(Clone,Copy)]struct BlockHit{pos:[i32;3]}
struct Game {
    window:Option<Window>,renderer:Option<Renderer>,lan_acting:Option<u32>,lan_acting_uuid:Option<u32>,player:Option<Player>,
    res:&'static Resources,pack:Pack,world:World,inventory:Vec<Option<ItemStack>>,armor:[Option<ItemStack>;4],offhand:Option<ItemStack>,
    craft_grid:Vec<Option<ItemStack>>,carried:Option<ItemStack>,container:Option<()>,station:Option<()>,craft_size:usize,
    remote:Option<Remote>,creative:bool,selected:usize,silent:Option<usize>,screen:Screen,captured:bool,cursor:(f32,f32),
    using:Option<()>,clicks:Clicks,client_inventory_previous_rotation:Cell<Option<[f32;2]>>,mount:bool,chest_boat:bool,chest_blocked:bool,
    eye:[f64;3],hit:Option<BlockHit>,mining:Option<()>,trace:RefCell<Vec<String>>,
}
impl mod_api::Host for Game{}
impl Game {
    fn new()->Self{let res=resources();Self{window:Some(Window),renderer:None,lan_acting:None,lan_acting_uuid:None,player:Some(Player::default()),res,pack:Pack{assets:Assets{registry:&res.registry},item_renderer:ItemRenderer{models:Models{prototypes:serde_json::from_slice(&std::fs::read("__RESOURCE_ROOT__/items.json").unwrap()).unwrap()}}},world:World{block:res.registry.default_state("minecraft:stone").unwrap(),chunks:HashMap::new()},inventory:vec![None;36],armor:std::array::from_fn(|_|None),offhand:None,craft_grid:vec![None;4],carried:None,container:None,station:None,craft_size:2,remote:Some(Default::default()),creative:false,selected:0,silent:None,screen:Screen::Playing,captured:true,cursor:(0.,0.),using:None,clicks:Default::default(),client_inventory_previous_rotation:Cell::new(None),mount:false,chest_boat:false,chest_blocked:false,eye:[0.;3],hit:Some(BlockHit{pos:[1,2,3]}),mining:None,trace:RefCell::new(vec![])}}
    fn native_window(&self)->bool{self.window.is_some()}
    fn items(&self)->&Items{&self.res.items}
    fn craft_result(&self)->Option<ItemStack>{None}
    fn eye_position(&self)->Option<[f64;3]>{Some(self.eye)}
    fn remote_menu(&self)->(i32,String){(if self.container.is_some()||self.station.is_some(){1}else{0},"inventory".into())}
    fn mount_active(&self)->bool{false}
    fn riding_mount_with_menu(&self)->bool{self.mount}
    fn riding_chest_boat(&self)->bool{self.chest_boat}
    fn chest_menu_blocked(&self,_:(i32,i32,i32),_:&str)->bool{self.chest_blocked}
    fn gui_origin(&self)->Result<(f32,f32),()>{Ok((10.,20.))}
    fn client_hotbar_slot(&self)->usize{self.silent.unwrap_or(self.selected)}
    fn remote_menu_ops(&mut self,ops:&[(Option<usize>,i32,menu::Input)])->bool{for &(source,target,_)in ops{let Some(source)=source else{return false;};self.inventory.swap(source,target as usize);self.trace.borrow_mut().push(format!("swap:{source}:{target}"));}true}
    fn menu_click(&mut self,source:Option<usize>,target:i32,input:menu::Input)->bool{self.remote_menu_ops(&[(source,target,input)])}
    fn send(&self,packet:S){self.trace.borrow_mut().push(match packet{S::ContainerClose(_)=>"close",_=>"packet"}.into());}
    fn held(&self)->Option<&ItemStack>{self.inventory.get(self.client_hotbar_slot())?.as_ref()}
    fn entity_targeted(&self)->bool{false}fn pick(&self)->Option<BlockHit>{self.hit}
    fn client_multi_actions(&self)->(bool,bool,bool){(false,false,false)}
    fn native_interaction(&mut self,_:u8,_:Hand)->Option<bool>{None}
    fn remote_send_carried(&mut self){self.trace.borrow_mut().push("carried".into());}
    fn swing_arm(&mut self){}fn remote_break_restricted(&self,_:[i32;3])->bool{false}
    fn remote_mining(&mut self,_:bool){self.trace.borrow_mut().push("mine".into());}
    fn stop_remote_mining(&mut self)->bool{false}
    __NATIVE__
    __REMOTE_CONTINUE__
}
struct GameHands<'a>{game:&'a Game}impl GameHands<'_>{__HAND_FORWARD__}
struct LocalWorld{called:Cell<i32>,creative:bool}
impl HandWorld for LocalWorld {
    fn hit(&mut self)->Hit{Hit::Miss}fn client_block_breaking_progress(&self,_:[i32;3]){self.called.set(self.called.get()+1);}
    fn client_attack_queries(&self)->(bool,bool){(false,false)}fn client_abort_breaking(&self)->bool{false}
    fn is_air(&mut self,_:[i32;3])->bool{false}fn creative(&self)->bool{self.creative}fn spectator(&self)->bool{false}fn instabuild(&self)->bool{self.creative}
    fn hands_busy(&self)->bool{false}fn using_item(&self)->bool{false}fn item_enabled(&mut self,_:Hand)->bool{true}fn held_empty(&mut self,_:Hand)->bool{false}fn held_count(&mut self,_:Hand)->i32{1}
    fn infinite_materials(&self)->bool{false}fn cannot_attack(&mut self)->bool{false}fn piercing_weapon(&mut self)->bool{false}
    fn attack_animation(&mut self)->SwingAnimation{SwingAnimation::DEFAULT}fn interact_animation(&mut self,_:Hand)->SwingAnimation{SwingAnimation::DEFAULT}
    fn piercing_attack(&mut self,_:SwingAnimation){}fn attack_entity(&mut self)->bool{false}fn block_action_restricted(&mut self,_:[i32;3])->bool{false}fn within_border(&mut self,_:[i32;3])->bool{true}
    fn attack_block(&mut self,_:[i32;3]){}fn destroy_progress(&mut self,_:[i32;3])->f32{0.2}fn destroy_block(&mut self,_:[i32;3])->bool{true}fn main_hand_key(&mut self)->String{String::new()}
    fn destroy_stage(&mut self,_:[i32;3],_:Option<i32>){}fn swing(&mut self,_:Hand,_:SwingAnimation){}fn reset_attack_strength(&mut self){}
    fn entity_within_border(&mut self)->bool{true}fn use_on_entity(&mut self,_:Hand)->Option<UseResult>{None}fn use_on_block(&mut self,_:Hand)->UseResult{UseResult::Pass}fn use_item(&mut self,_:Hand)->UseResult{UseResult::Pass}
    fn item_used(&mut self,_:Hand){}fn release_using(&mut self){}
}
#[cfg(test)]mod tests {
use super::*;static SCENE:Mutex<()>=Mutex::new(());
#[test]fn actual_continue_destroy_head_precedes_delay_and_creative(){let mut w=LocalWorld{called:Cell::new(0),creative:false};let mut d=Destroying{delay:5,..Default::default()};d.continue_destroy(&mut w,[1;3],1);assert_eq!(w.called.get(),1);assert_eq!(d.delay,4);w.creative=true;d.continue_destroy(&mut w,[2;3],1);assert_eq!(w.called.get(),2);}
#[test]fn actual_remote_head_and_local_forward_dispatch_generated_hook(){let _lock=SCENE.lock().unwrap();TRACE.lock().unwrap().clear();let mut g=Game::new();g.remote_continue_attack(true);assert_eq!(TRACE.lock().unwrap().as_slice(),["progress:[1, 2, 3]"]);assert_eq!(g.trace.borrow().as_slice(),["carried","mine","packet"]);TRACE.lock().unwrap().clear();GameHands{game:&g}.client_block_breaking_progress([9;3]);assert_eq!(TRACE.lock().unwrap().as_slice(),["progress:[9, 9, 9]"]);for i in 0..6{let mut g=Game::new();match i{0=>g.window=None,1=>g.lan_acting=Some(1),2=>g.lan_acting_uuid=Some(1),3=>g.player=None,4=>g.player.as_mut().unwrap().server=true,_=>g.player.as_mut().unwrap().replay=true}assert!(!g.client_auto_tool_player());}}
#[test]fn actual_snapshot_get_max_damage_and_clamped_damage_match_minecraft(){let mut g=Game::new();let facts:Value=serde_json::from_str(include_str!("oracle.json")).unwrap();for row in facts["stacks"].as_array().unwrap(){let name=row["name"].as_str().unwrap();let mut s=ItemStack::new(if name.starts_with("custom"){ "minecraft:stick"}else{"minecraft:diamond_pickaxe"},1);if name=="unbreakable"{s.damage=1559;s.data_mut().set_other("minecraft:unbreakable",Some(horizon_world::nbt::Tag::Compound(Default::default())));}
if name=="removedDamage"{s.data_mut().set_other("!minecraft:damage",Some(horizon_world::nbt::Tag::Compound(Default::default())));}
if name=="removedMaxDamage"{s.data_mut().set_other("!minecraft:max_damage",Some(horizon_world::nbt::Tag::Compound(Default::default())));}
if name.starts_with("custom"){s.data_mut().set_other("minecraft:max_damage",Some(horizon_world::nbt::Tag::Int(20)));if name=="customDamageClamped"{s.damage=99;}}g.inventory[0]=Some(s);let snap=crate::actor::Snapshot::parse(&g.native_client_auto_tool_snapshot([0;3],false)).unwrap().unwrap();assert_eq!(snap.stacks[0].max_damage,row["maxDamage"].as_i64().unwrap() as i32,"{name}");assert_eq!(snap.stacks[0].damage,row["damage"].as_i64().unwrap() as i32,"{name}");}}
#[test]fn actual_snapshot_server_controlled_mounts_and_native_menu_invoker(){let mut g=Game::new();g.inventory[9]=Some(ItemStack::new("minecraft:diamond_pickaxe",1));g.mount=true;assert!(!g.native_client_auto_tool_swap(9,0));g.mount=false;g.chest_boat=true;assert!(!g.native_client_auto_tool_swap(9,0));g.chest_boat=false;assert!(g.native_client_auto_tool_swap(9,0));assert_eq!(&*g.inventory[0].as_ref().unwrap().item,"minecraft:diamond_pickaxe");assert!(g.inventory[9].is_none());g.container=Some(());assert!(!g.native_client_auto_tool_swap(9,0));}
#[test]fn menu_provider_class_oracle_excludes_end_chest_and_nonmenu_entities(){let mut g=Game::new();for (name,expected) in [("minecraft:ender_chest",false),("minecraft:jukebox",false),("minecraft:crafting_table",true),("minecraft:anvil",true),("minecraft:beacon",true),("minecraft:chest",true),("minecraft:lectern",false)]{g.world.block=g.res.registry.default_state(name).unwrap();assert_eq!(g.native_client_auto_tool_menu_provider([0;3]),expected,"{name}");}g.world.block=g.res.registry.default_state("minecraft:chest").unwrap();g.chest_blocked=true;assert!(!g.native_client_auto_tool_menu_provider([0;3]));}
#[test]fn actual_bed_scan_is_head_only_and_uses_eye_center_distance(){let mut g=Game::new();let mut state=g.res.registry.state(g.res.registry.default_state("minecraft:red_bed").unwrap()).unwrap().clone();state.properties.insert("part".into(),"head".into());let head=g.res.registry.resolve(&state).unwrap();let mut states=vec![0;4096];states[0]=head.0;g.world.chunks.insert((0,0),Chunk{sections:vec![Section{blocks:Blocks(states)}]});let snapshot=crate::actor::Snapshot::parse(&g.native_client_auto_tool_snapshot([0;3],true)).unwrap().unwrap();assert_eq!(snapshot.beds,vec![0.75]);}
#[test]fn actual_silent_render_dispatch_sets_visible_slot_and_scale(){let g=Game::new();let mut slot=5;let mut scale=0.2;g.native_client_silent_hotbar_render(&mut slot,&mut scale);assert_eq!((slot,scale),(2,1.));}
#[test]fn effective_component_identity_ignores_redundant_prototype_patch(){let g=Game::new();let base=ItemStack::new("minecraft:diamond_pickaxe",1);let mut redundant=base.clone();redundant.data_mut().set_other("minecraft:max_damage",Some(Tag::Int(1561)));assert_eq!(g.client_inventory_stack_record(Some(&base)),g.client_inventory_stack_record(Some(&redundant)));let mut changed=base.clone();changed.data_mut().set_other("minecraft:max_damage",Some(Tag::Int(20)));assert_ne!(g.client_inventory_stack_record(Some(&base)),g.client_inventory_stack_record(Some(&changed)));}
}
