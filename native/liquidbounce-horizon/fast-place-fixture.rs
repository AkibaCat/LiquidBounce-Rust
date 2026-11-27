#![cfg(test)]
#![allow(dead_code, unused_imports)]
extern crate self as horizon_client;
pub use horizon_items::items;
mod interaction_input;
#[path="projectile_items.rs"] mod fast_place_items;
use std::sync::{Mutex,MutexGuard,OnceLock};
use std::sync::atomic::{AtomicI64,AtomicU8,Ordering};
static NOW:AtomicI64=AtomicI64::new(100);
static SILENT:AtomicU8=AtomicU8::new(u8::MAX);
static TRACE:Mutex<Vec<&str>>=Mutex::new(Vec::new());
static EVENTS:Mutex<Vec<(u8,u8,i32,i32)>>=Mutex::new(Vec::new());
fn trace(event:&'static str){TRACE.lock().unwrap().push(event);}
mod behaviors{pub fn current_time_millis()->i64{crate::NOW.load(crate::Ordering::Acquire)}}
mod slot_bridge{pub fn serverside_slot(real:Option<u8>)->u8{let slot=crate::SILENT.load(crate::Ordering::Acquire);if slot==u8::MAX{real.unwrap_or(0)}else{slot}}}
struct Modules{fast_place:liquidbounce_behaviors::fast_place::FastPlace,press_times:liquidbounce_behaviors::fast_place::PressTimes,running:bool}
impl Default for Modules{fn default()->Self{Self{fast_place:Default::default(),press_times:Default::default(),running:true}}}
impl Modules{fn query_running(&self,_:&(),_:&str)->bool{self.running}}
#[derive(Default)]struct State{behaviors:std::cell::RefCell<Modules>,registry:()}
fn state()->MutexGuard<'static,State>{static STATE:OnceLock<Mutex<State>>=OnceLock::new();STATE.get_or_init(Default::default).lock().unwrap()}
fn reset(){*state()=State::default();NOW.store(100,Ordering::Release);SILENT.store(u8::MAX,Ordering::Release);TRACE.lock().unwrap().clear();EVENTS.lock().unwrap().clear();}
fn catalog()->&'static items::Items{
    static ITEMS:OnceLock<items::Items>=OnceLock::new();ITEMS.get_or_init(||{
        let root=std::path::Path::new(__RESOURCE_ROOT__);
        let (registry,_)=horizon_world::registry::BlockRegistry::from_facts_json_with_flags(std::fs::File::open(root.join("facts.json")).unwrap()).unwrap();
        let facts=horizon_items::blocks::BlockFacts::load(root,&registry).unwrap();
        items::Items::load(root,&registry,&facts,&horizon_world::loot::PackOverlay::NONE).unwrap()
    })
}
mod mod_api{
    use super::*;
    pub trait RenderHooks:Send+Sync{}
    mod host{include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}pub use host::*;
    mod generated{include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}pub use generated::*;
    pub fn entered<'a>(mods:&'a[Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(|m|&**m)}
    pub fn mods()->&'static[Box<dyn Hooks>]{static MODS:OnceLock<Vec<Box<dyn Hooks>>>=OnceLock::new();MODS.get_or_init(||vec![Box::new(Recorder),Box::new(Runtime)])}
    struct Recorder;impl Hooks for Recorder{
        fn client_use_cooldown(&self,_:&dyn Host,event:&mut ClientUseCooldownEvent){
            trace("event");EVENTS.lock().unwrap().push((event.main_kind,event.off_kind,event.mouse_button,event.cooldown));
        }
    }
    struct Runtime;impl Runtime{fn state(&self)->MutexGuard<'static,State>{state()}}
    impl Hooks for Runtime{__RUNTIME_METHODS__}
}
mod game{
    use super::*;
    use items::{ItemStack,Items};
    type Result<T>=std::result::Result<T,&'static str>;
    #[derive(Clone,Copy,Default,Debug,PartialEq,Eq)]enum Hand{#[default] Main,Off}
    mod shield{pub const OFFHAND_SLOT:usize=40;}
    const OFFHAND_SLOT:usize=shield::OFFHAND_SLOT;
    struct Player{allowed:bool}impl Player{fn native_client_queries_allowed(&self)->bool{self.allowed}}
    #[derive(Default)]struct Destroying{active:bool}#[derive(Default)]struct Clicks{destroying:Destroying}
    #[derive(Default)]struct Heights{calls:usize}impl Heights{fn item_used(&mut self,_:Hand){self.calls+=1;}}
    struct Remote{is_destroying:bool}
    struct Settings{keys:Keys}struct Keys{use_key:String}impl Keys{fn get(&self,_:&str)->&str{&self.use_key}}
    #[derive(Clone,Copy,PartialEq,Eq)]enum ElementState{Pressed,Released}
    enum WindowEvent{MouseInput{state:ElementState,button:i32},Other}
    fn glfw_button(button:i32)->i32{button}fn glfw_modifiers(modifiers:u32)->u32{modifiers}
    struct SwingAnimation;impl SwingAnimation{const DEFAULT:Self=Self;}
    pub(super) struct Game<'a>{
        inventory:Vec<Option<ItemStack>>,selected:usize,offhand:Option<ItemStack>,hand:Hand,
        use_delay:i32,player:Option<Player>,lan_acting_uuid:Option<()>,native:bool,
        clicks:Clicks,busy:bool,using:Option<()>,swings_started:u32,creative:bool,heights:Heights,
        remote:Option<Remote>,settings:Settings,interaction:Option<bool>,mod_used:bool,
        fail:bool,item_hand:Option<Hand>,synthetic:bool,modifiers:u32,marker:std::marker::PhantomData<&'a()>}
    impl Default for Game<'_>{fn default()->Self{Self{inventory:vec![None;9],selected:0,offhand:None,hand:Hand::Main,
        use_delay:0,player:Some(Player{allowed:true}),lan_acting_uuid:None,native:true,clicks:Default::default(),busy:false,
        using:None,swings_started:0,creative:false,heights:Default::default(),remote:Some(Remote{is_destroying:false}),
        settings:Settings{keys:Keys{use_key:"key.mouse.right".into()}},interaction:None,mod_used:false,fail:false,item_hand:None,
        synthetic:false,modifiers:0,marker:Default::default()}}}
    fn stack(id:&str)->ItemStack{ItemStack{item:id.into(),count:1,damage:0,extra:None}}
    impl Game<'_>{
        fn native_window(&self)->bool{self.native}
        fn items(&self)->&Items{catalog()}
        fn hands_busy(&self)->bool{trace("hands_busy");self.busy}
        fn use_item(&mut self)->Result<Option<Hand>>{trace("local_use");if self.fail{Err("use failed")}else{Ok(self.item_hand)}}
        fn native_interaction(&mut self,_:i32,_:Hand)->Option<bool>{trace("interaction");self.interaction}
        fn swing_hand(&mut self,_:Hand,_:SwingAnimation){trace("swing");self.swings_started+=1;}
        fn mod_use_start(&mut self)->bool{trace("mod_use");self.mod_used}
        fn remote_use_targets_entity(&self)->bool{trace("target");false}
        fn pad_synthetic_input(&self)->bool{self.synthetic}
        fn handle_window_event_inner(&mut self,_:(),_:WindowEvent){trace("mapping");}
        fn native_mouse_button_post(&mut self,_:i32,_:bool,_:u32){trace("post");}
        __SLOT_METHODS__
        __STACK_METHOD__
        __HELD_METHODS__
        __LOCAL_METHOD__
        __REMOTE_METHOD__
        fn fixture_window(&mut self,event:WindowEvent){let event_loop=();__WINDOW_FRAGMENT__}
    }
    mod native_hooks{use super::*;impl Game<'_>{__NATIVE_METHODS__}}
    mod mod_api_host{include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}
    struct RemoteUseWorld<'g,'a>(&'g mut Game<'a>);
    impl interaction_input::TargetUseWorld for RemoteUseWorld<'_,'_>{
        fn use_entity(&mut self)->bool{trace("entity");false}
        fn use_block(&mut self)->bool{trace("block");false}
        fn use_item(&mut self){trace("remote_item");}
    }
    impl interaction_input::InteractionWorld for RemoteUseWorld<'_,'_>{
        fn pick_hit_result(&mut self){}fn release_hit_result(&mut self){}fn using_item(&self)->bool{self.0.using.is_some()}
        fn release_using(&mut self){self.0.using=None;}fn start_attack(&mut self)->bool{false}
        fn start_use(&mut self){self.0.remote_use();}fn pick(&mut self){}
        __REMOTE_DELAY__
        fn continue_attack(&mut self,_:bool){}
    }
    #[test]fn actual_items_load_and_projectile_membership_match_all_1658_official_item_classes(){
        let oracle:serde_json::Value=serde_json::from_str(include_str!(__ORACLE_PATH__)).unwrap();
        let rows=oracle["items"].as_array().unwrap();assert_eq!(rows.len(),1658);
        let mut projectiles=0;
        for row in rows{let id=row["id"].as_str().unwrap();let projectile=row["projectile"].as_bool().unwrap();
            let block=row["block"].as_bool().unwrap();projectiles+=usize::from(projectile);
            assert_eq!(fast_place_items::is_projectile_item(id),projectile,"ProjectileItem {id}");
            assert_eq!(catalog().get(id).unwrap().block.is_some(),block,"BlockItem {id}");
        }assert_eq!(projectiles,14);
    }
    #[test]fn actual_generated_event_preserves_signed_i32_and_both_hands_before_any_interaction(){
        reset();state().behaviors.borrow_mut().fast_place.cooldown=(-2,-2);
        let mut game=Game::default();game.inventory[0]=Some(stack("minecraft:ender_pearl"));game.offhand=Some(stack("minecraft:snowball"));
        game.use_with_hand().unwrap();assert_eq!(game.use_delay,-2);
        assert_eq!(*EVENTS.lock().unwrap(),vec![(0,1,1,4)]);
        assert_eq!(*TRACE.lock().unwrap(),vec!["event","hands_busy","local_use"]);
        TRACE.lock().unwrap().clear();game.remote_use();assert_eq!(game.use_delay,-2);
        assert_eq!(*TRACE.lock().unwrap(),vec!["event","hands_busy","interaction","mod_use","target","block","remote_item"]);
        let _:i32=game.use_delay;
    }
    #[test]fn actual_mainhand_classification_obeys_silent_hotbar_and_does_not_follow_current_offhand(){
        reset();let mut game=Game::default();game.inventory[0]=Some(stack("minecraft:stick"));game.inventory[3]=Some(stack("minecraft:stone"));
        game.offhand=Some(stack("minecraft:ender_pearl"));game.hand=Hand::Off;SILENT.store(3,Ordering::Release);
        game.native_begin_use_cooldown();assert_eq!(game.use_delay,0);assert_eq!(EVENTS.lock().unwrap().last(),Some(&(2,0,1,4)));
        game.lan_acting_uuid=Some(());assert_eq!(game.client_hotbar_slot(),0);game.native_begin_use_cooldown();assert_eq!(game.use_delay,4);
    }
    #[test]fn actual_local_and_remote_break_guard_precedes_event_but_boat_mod_and_failed_use_follow_it(){
        reset();let mut game=Game::default();game.inventory[0]=Some(stack("minecraft:stone"));
        game.use_delay=7;game.clicks.destroying.active=true;game.use_with_hand().unwrap();assert_eq!(game.use_delay,7);assert!(TRACE.lock().unwrap().is_empty());
        game.remote.as_mut().unwrap().is_destroying=true;game.remote_use();assert_eq!(game.use_delay,7);assert!(TRACE.lock().unwrap().is_empty());
        game.clicks.destroying.active=false;game.remote.as_mut().unwrap().is_destroying=false;game.busy=true;
        game.use_with_hand().unwrap();assert_eq!(game.use_delay,0);assert_eq!(*TRACE.lock().unwrap(),vec!["event","hands_busy"]);
        TRACE.lock().unwrap().clear();game.remote_use();assert_eq!(*TRACE.lock().unwrap(),vec!["event","hands_busy"]);
        game.busy=false;game.interaction=Some(false);TRACE.lock().unwrap().clear();game.remote_use();assert_eq!(*TRACE.lock().unwrap(),vec!["event","hands_busy","interaction"]);
        game.interaction=None;game.mod_used=true;TRACE.lock().unwrap().clear();game.remote_use();assert_eq!(*TRACE.lock().unwrap(),vec!["event","hands_busy","interaction","mod_use"]);
        game.fail=true;assert_eq!(game.use_with_hand(),Err("use failed"));assert_eq!(game.use_delay,0);
    }
    #[test]fn actual_mouse_pre_sets_press_time_before_mapping_and_release_keeps_time(){
        reset();let mut game=Game::default();game.inventory[0]=Some(stack("minecraft:stone"));state().behaviors.borrow_mut().fast_place.start_delay=100;
        game.fixture_window(WindowEvent::MouseInput{state:ElementState::Pressed,button:1});
        assert_eq!(state().behaviors.borrow().press_times.since_use_press(1,100),0);
        assert_eq!(*TRACE.lock().unwrap(),vec!["mapping","post"]);
        game.use_with_hand().unwrap();assert_eq!(game.use_delay,4);
        NOW.store(199,Ordering::Release);game.use_with_hand().unwrap();assert_eq!(game.use_delay,4);
        game.fixture_window(WindowEvent::MouseInput{state:ElementState::Released,button:1});
        NOW.store(200,Ordering::Release);game.use_with_hand().unwrap();assert_eq!(game.use_delay,0);
        game.synthetic=true;NOW.store(250,Ordering::Release);game.fixture_window(WindowEvent::MouseInput{state:ElementState::Pressed,button:1});
        assert_eq!(state().behaviors.borrow().press_times.since_use_press(1,250),150);
    }
    #[test]fn rebound_mouse_buttons_keyboard_and_disabled_owner_match_original_tracker_guards(){
        reset();let mut game=Game{offhand:Some(stack("minecraft:stone")),..Default::default()};state().behaviors.borrow_mut().fast_place.start_delay=100;
        for (key,button) in [("key.mouse.left",0),("key.mouse.middle",2),("key.mouse.4",3),("key.mouse.32",31)]{
            game.settings.keys.use_key=key.into();NOW.store(100,Ordering::Release);game.fixture_window(WindowEvent::MouseInput{state:ElementState::Pressed,button});
            game.native_begin_use_cooldown();assert_eq!(game.use_delay,4,"{key}");
            NOW.store(200,Ordering::Release);game.native_begin_use_cooldown();assert_eq!(game.use_delay,0,"{key}");
            assert_eq!(EVENTS.lock().unwrap().last().unwrap().2,button);
        }
        game.settings.keys.use_key="key.keyboard.r".into();game.native_begin_use_cooldown();assert_eq!(game.use_delay,0);
        assert_eq!(EVENTS.lock().unwrap().last().unwrap().2,-1);
        state().behaviors.borrow_mut().running=false;game.native_begin_use_cooldown();assert_eq!(game.use_delay,4);
        state().behaviors.borrow_mut().running=true;game.native=false;game.native_begin_use_cooldown();assert_eq!(game.use_delay,4);
        game.native=true;game.player.as_mut().unwrap().allowed=false;game.native_begin_use_cooldown();assert_eq!(game.use_delay,4);
    }
    #[test]fn actual_interaction_input_allows_same_tick_zero_repeat_but_negative_import_waits_for_explicit_click(){
        reset();let mut game=Game::default();game.inventory[0]=Some(stack("minecraft:stone"));
        let mut input=interaction_input::InteractionInput::default();input.press(interaction_input::Button::Use);
        input.tick(&mut RemoteUseWorld(&mut game),false,true);assert_eq!(EVENTS.lock().unwrap().len(),2);
        state().behaviors.borrow_mut().fast_place.cooldown=(-3,-3);EVENTS.lock().unwrap().clear();input.press(interaction_input::Button::Use);
        input.tick(&mut RemoteUseWorld(&mut game),false,true);assert_eq!(game.use_delay,-3);assert_eq!(EVENTS.lock().unwrap().len(),1);
        input.tick(&mut RemoteUseWorld(&mut game),false,true);assert_eq!(EVENTS.lock().unwrap().len(),1);
    }
}
