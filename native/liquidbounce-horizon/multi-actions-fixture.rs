#![allow(dead_code)]
extern crate self as horizon_client;
extern crate self as liquidbounce_behaviors;
use std::sync::{LazyLock, Mutex, OnceLock};
#[cfg(test)] use serde_json::Value;
type Result<T> = std::result::Result<T, ()>;
#[path="__POLICY_PATH__"] pub mod multi_actions;
#[path="interaction_input.rs"] pub mod interaction_input;
#[path="config-fixture.rs"] mod config;
__FIRST_PERSON__

static SCENE: Mutex<()> = Mutex::new(());
static MODULE: LazyLock<Mutex<(bool, multi_actions::MultiActions)>> = LazyLock::new(||Mutex::new((true, Default::default())));
mod mod_api {
    use super::*;
    pub trait Host {}
    pub trait RenderHost {}
    pub trait RenderHooks: Send + Sync {}
    pub struct ModHandle;
    pub type Mod = dyn Hooks;
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks> {mods.iter().map(Box::as_ref)}
    mod generated {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}
    pub use generated::*;
    struct Adapter;
    impl Hooks for Adapter {
        fn client_multi_actions(&self,_:&dyn Host,event:&mut ClientMultiActionsEvent) {
            let state=MODULE.lock().unwrap();
            (event.place_while_breaking,event.attack_while_using,event.break_while_using)=state.1.policy(state.0);
        }
    }
    pub fn mods()->&'static [Box<dyn Hooks>] {static MODS:OnceLock<Vec<Box<dyn Hooks>>>=OnceLock::new();MODS.get_or_init(||vec![Box::new(Adapter)]).as_slice()}
}
#[derive(Clone,Default)]struct Stack {item:String,count:u32}
struct Heights;impl Heights {fn item_used(&mut self,_:Hand){}}
#[derive(Default)]struct Player {server_copy:bool,replaying:bool}
impl Player {fn native_client_reach_queries_allowed(&self)->bool{!self.server_copy&&!self.replaying}}
#[derive(Default)]struct Remote {is_destroying:bool,destroying:Option<()>,input:interaction_input::InteractionInput}
#[derive(Clone,Copy,PartialEq,Eq)]enum Screen {Playing,Other}
#[derive(Clone,Copy,PartialEq,Eq)]enum MouseButton {Left,Right,Middle,Other}
#[derive(Default)]struct Toggles([bool;4]);impl Toggles {fn button(&mut self,n:usize,p:bool,_:bool,_:bool){self.0[n]=p;}fn is_down(&self,n:usize)->bool{self.0[n]}}
#[derive(Default)]struct Settings {toggle_attack:bool,toggle_use:bool}
#[derive(Clone,Copy)]struct BlockHit {pos:[i32;3],face:[i32;3]}
#[derive(Clone)]struct Sounds {hit:String,volume:f32,pitch:f32}
struct Facts;struct Fact {sounds:Sounds}
impl Facts {fn is_air(&self,_:u32)->bool{false}fn get(&self,_:u32)->Fact {Fact{sounds:Sounds{hit:String::new(),volume:1.,pitch:1.}}}}
struct Resources;impl Resources {fn facts(&self)->Facts {Facts}}
struct World;impl World {fn block(&self,_:i32,_:i32,_:i32)->u32{0}}
struct Audio;impl Audio {fn play(&mut self,_:&str,_:Option<[f64;3]>,_:f32,_:f32){}}
struct Mining {pos:[i32;3],progress:f32,per_tick:f32,ticks:u32}
struct Empty;enum S {Punch(Empty)}
mod spear {pub struct Kinetic;impl Kinetic{pub fn of(_: &str)->Option<Self>{None}}}
mod spans {pub fn time<T>(_:&str,f:impl FnOnce()->T)->T {f()}}
mod pick_item {use super::*;pub fn pick(inventory:&mut [Option<Stack>],selected:&mut usize,stack:&Stack,_:bool){inventory[8]=Some(stack.clone());*selected=8;}}
struct Game {
    window:Option<()>,lan_acting:Option<u32>,lan_acting_uuid:Option<u32>,player:Option<Player>,
    clicks:Clicks,remote:Option<Remote>,using:Option<(usize,u32)>,inventory:Vec<Option<Stack>>,offhand:Option<Stack>,selected:usize,
    heights:Heights,swings_started:u32,use_delay:i32,busy:bool,creative:bool,hit:Hit,trace:Vec<&'static str>,
    attack_clicks:u32,use_clicks:u32,pick_clicks:u32,left:bool,right:bool,screen:Screen,title:Option<()>,sleeping:Option<()>,
    world:World,res:Resources,audio:Audio,mining:Option<Mining>,hotbar_presses:Vec<()>,toggles:Toggles,settings:Settings,
    complete_on_tick:bool,keep_using:bool,instant:bool,begin_using:bool,
}
impl mod_api::Host for Game {}
impl Game {
    fn new()->Self {Self {window:Some(()),lan_acting:None,lan_acting_uuid:None,player:Some(Default::default()),clicks:Default::default(),remote:None,using:None,inventory:vec![Some(Stack{item:"minecraft:stone".into(),count:1});36],offhand:None,selected:0,heights:Heights,swings_started:0,use_delay:0,busy:false,creative:false,hit:Hit::Entity,trace:vec![],attack_clicks:0,use_clicks:0,pick_clicks:0,left:false,right:false,screen:Screen::Playing,title:None,sleeping:None,world:World,res:Resources,audio:Audio,mining:None,hotbar_presses:vec![],toggles:Default::default(),settings:Default::default(),complete_on_tick:false,keep_using:false,instant:false,begin_using:false}}
    fn stack_in(&self,n:usize)->Option<&Stack>{self.inventory.get(n)?.as_ref()}
    fn held(&self)->Option<&Stack>{self.stack_in(self.selected)}
    fn hands_busy(&self)->bool{self.busy}
    fn native_begin_use_cooldown(&mut self){self.trace.push("cooldown");self.use_delay=4;}
    fn use_item(&mut self)->Result<Option<Hand>>{self.trace.push("use");if self.begin_using {self.using=Some((0,0));}Ok(Some(Hand::Main))}
    fn pvp_click(&mut self,_:MouseButton){}
    fn spectator(&self)->bool{false}
    fn spectator_action_key(&mut self){}
    fn select_queued_hotbar(&mut self){}
    fn pick_entity_hit(&self)->Option<Option<i32>>{None}
    fn entity_pick_result(&self,_:i32)->Option<Stack>{None}
    fn picked_stack(&mut self,_:[i32;3],_:bool)->Option<Stack>{self.trace.push("pick");Some(Stack{item:"minecraft:stone".into(),count:1})}
    fn has_control_down(&self)->bool{false}
    fn pick(&self)->Option<BlockHit>{match self.hit {Hit::Block{pos,face:_}=>Some(BlockHit{pos,face:[0,1,0]}),_=>None}}
    fn entity_targeted(&self)->bool{matches!(self.hit,Hit::Entity)}
    fn remote_use_targets_entity(&self)->bool{self.entity_targeted()}
    fn native_interaction(&mut self,_:u8,_:Hand)->Option<bool>{None}
    fn swing_hand(&mut self,_:Hand,_:SwingAnimation){self.trace.push("swing");}
    fn swing_arm(&mut self){self.trace.push("swing");}
    fn mod_use_start(&mut self)->bool{false}
    fn remote_use_entity(&mut self,_:Hand)->bool{self.trace.push("use");true}
    fn remote_use_block(&mut self,_:Hand)->bool{self.trace.push("use");true}
    fn remote_use_hands(&mut self){self.trace.push("use");}
    fn remote_start_attack(&mut self)->bool{self.trace.push("attack");self.instant}
    fn remote_send_carried(&mut self){self.trace.push("carried");}
    fn remote_break_restricted(&self,_:[i32;3])->bool{false}
    fn remote_mining(&mut self,_:bool){self.trace.push("dig");}
    fn send(&mut self,_:S){}
    fn stop_remote_mining(&mut self)->bool{self.trace.push("stop");false}
    fn crack_particle(&mut self,_:[i32;3],_:[i32;3],_:u32){}
    fn tutorial_destroy(&mut self,_:u32,_:f32){}
    fn dialog_open(&self)->bool{false}
    fn pick_tick_targets(&mut self){self.trace.push("pick_hit");}
    fn release_tick_targets(&mut self){self.trace.push("release_hit");}
    fn mod_use_tick(&mut self,_:usize,_:u32,_:bool)->bool{self.keep_using}
    fn release_remote_use(&mut self){self.trace.push("release");self.using=None;}
    fn tick_using(&mut self){if self.using.is_some()&&(!self.right||self.complete_on_tick)&&!self.keep_using{self.trace.push("release");self.using=None;}}
    fn keybind_frame(&mut self)->Result<()> {__FRAME__}
    __NATIVE__
    __LOCAL_TICK__
    __LOCAL_USE__
    __PICK_ITEM__
    __PLAY_MOUSE__
    __REMOTE_MOUSE__
    __REMOTE_USE__
    __REMOTE_CONTINUE__
}
struct RemoteUseWorld<'g>(&'g mut Game);
__REMOTE_USE_WORLD__
struct RemoteInputWorld<'g>(&'g mut Game);
impl interaction_input::InteractionWorld for RemoteInputWorld<'_> {
    __REMOTE_INPUT_METHODS__
    fn pick(&mut self){self.0.client_pick_item();}
}
struct GameHands<'g>{game:&'g mut Game,error:Option<()>}
impl HandWorld for GameHands<'_> {
    __HAND_QUERY__
    fn hit(&mut self)->Hit{self.game.hit}
    fn is_air(&mut self,_:[i32;3])->bool{false}
    fn creative(&self)->bool{false}fn spectator(&self)->bool{false}fn instabuild(&self)->bool{false}
    fn hands_busy(&self)->bool{self.game.busy}fn using_item(&self)->bool{self.game.using.is_some()}
    fn item_enabled(&mut self,_:Hand)->bool{true}fn held_empty(&mut self,_:Hand)->bool{false}
    fn held_count(&mut self,_:Hand)->i32{1}fn infinite_materials(&self)->bool{false}
    fn cannot_attack(&mut self)->bool{false}fn piercing_weapon(&mut self)->bool{false}
    fn attack_animation(&mut self)->SwingAnimation{SwingAnimation::DEFAULT}
    fn interact_animation(&mut self,_:Hand)->SwingAnimation{SwingAnimation::DEFAULT}
    fn piercing_attack(&mut self,_:SwingAnimation){}
    fn attack_entity(&mut self)->bool{self.game.trace.push("attack");true}
    fn block_action_restricted(&mut self,_:[i32;3])->bool{false}fn within_border(&mut self,_:[i32;3])->bool{true}
    fn attack_block(&mut self,_:[i32;3]){self.game.trace.push("attack_block");}
    fn destroy_progress(&mut self,_:[i32;3])->f32{self.game.trace.push("dig");0.2}
    fn destroy_block(&mut self,_:[i32;3])->bool{true}fn main_hand_key(&mut self)->String{"item".into()}
    fn destroy_stage(&mut self,_:[i32;3],_:Option<i32>){}
    fn swing(&mut self,_:Hand,_:SwingAnimation){self.game.trace.push("swing");}fn reset_attack_strength(&mut self){}
    fn entity_within_border(&mut self)->bool{true}fn use_on_entity(&mut self,_:Hand)->Option<UseResult>{None}
    fn use_on_block(&mut self,_:Hand)->UseResult{self.game.trace.push("use");UseResult::Success{swing:false}}
    fn use_item(&mut self,_:Hand)->UseResult{self.game.trace.push("use");UseResult::Success{swing:false}}
    fn item_used(&mut self,_:Hand){}fn release_using(&mut self){self.game.release_remote_use();}
}

#[cfg(test)]mod tests {
    use super::*;
    use interaction_input::{Button,InteractionInput};
    fn setup(mask:u8,enabled:bool)->Game{*MODULE.lock().unwrap()=(enabled,multi_actions::MultiActions{actions:mask});Game::new()}
    fn actions(game:&Game)->Vec<&'static str>{game.trace.iter().copied().filter(|v|matches!(*v,"release"|"attack"|"use"|"pick"|"dig")).collect()}
    #[test]fn gson_enumset_and_java_tag_comparator_oracle(){
        let facts:Value=serde_json::from_str(include_str!("actions-oracle.json")).unwrap();
        for row in facts["rows"].as_array().unwrap(){let mut state=multi_actions::MultiActions::default();let result=state.deserialize_actions(&row["input"]);assert_eq!(state.actions as u64,row["mask"].as_u64().unwrap(),"{row}");assert_eq!(result.is_err(),!row["error"].is_null(),"{row}");}
    }
    #[test]fn native_enabled_policy_and_identity_guards(){let _lock=SCENE.lock().unwrap();
        for mask in 0..8 {let game=setup(mask,true);assert_eq!(game.client_multi_actions(),(mask&1!=0,mask&2!=0,mask&4!=0));}
        let game=setup(7,false);assert_eq!(game.client_multi_actions(),(false,false,false));
        for guard in 0..6 {let mut game=setup(7,true);match guard{0=>game.window=None,1=>game.lan_acting=Some(1),2=>game.lan_acting_uuid=Some(1),3=>game.player=None,4=>game.player.as_mut().unwrap().server_copy=true,_=>game.player.as_mut().unwrap().replaying=true}assert_eq!(game.client_multi_actions(),(false,false,false));}
    }
    #[test]fn direct_continue_attack_and_start_use_all_action_subsets(){let _lock=SCENE.lock().unwrap();
        for mask in 0..8 {let mut game=setup(mask,true);game.hit=Hit::Block{pos:[0;3],face:1};game.using=Some((0,0));let mut clicks=Clicks::default();clicks.continue_attack(&mut GameHands{game:&mut game,error:None},true);assert_eq!(game.trace.contains(&"dig"),mask&4!=0);
            game.trace.clear();clicks.destroying.active=true;clicks.start_use_item(&mut GameHands{game:&mut game,error:None});assert_eq!(game.trace.contains(&"use"),mask&1!=0);}
    }
    #[test]fn real_local_and_remote_use_guards_keep_cooldown_and_busy_order(){let _lock=SCENE.lock().unwrap();
        for remote in [false,true]{for allowed in [false,true]{let mut game=setup(u8::from(allowed),true);game.clicks.destroying.active=true;if remote{game.remote=Some(Remote{is_destroying:true,..Default::default()});game.remote_use();}else{game.use_with_hand().unwrap();}assert_eq!(game.trace.contains(&"use"),allowed);assert_eq!(game.use_delay,if allowed{4}else{0});
            game.trace.clear();game.use_delay=0;game.busy=true;if remote{game.remote_use();}else{game.use_with_hand().unwrap();}assert!(!game.trace.contains(&"use"));assert_eq!(game.use_delay,if allowed{4}else{0});}}
    }
    #[test]fn remote_continue_attack_uses_breaking_action_independently(){let _lock=SCENE.lock().unwrap();
        for mask in 0..8 {let mut game=setup(mask,true);game.remote=Some(Default::default());game.hit=Hit::Block{pos:[0;3],face:1};game.using=Some((0,0));game.remote_continue_attack(true);assert_eq!(game.trace.contains(&"dig"),mask&4!=0);}
    }
    #[test]fn actual_remote_input_drains_use_pick_and_preserves_release_tick_attack(){let _lock=SCENE.lock().unwrap();
        for enabled in [false,true]{for mask in 0..8{for held in [false,true]{let mut game=setup(mask,enabled);game.remote=Some(Default::default());game.using=Some((0,0));game.hit=Hit::Entity;let mut input=InteractionInput::default();for button in [Button::Attack,Button::Use,Button::Pick]{input.press(button);}input.tick(&mut RemoteInputWorld(&mut game),false,held);let mut expected=vec![];if !held{expected.push("release");}
        if enabled&&mask&2!=0{expected.push("attack");}assert_eq!(actions(&game),expected,"enabled {enabled}, mask {mask}, held {held}");game.trace.clear();input.tick(&mut RemoteInputWorld(&mut game),false,false);assert!(!game.trace.contains(&"attack"));}}}
    }
    #[test]fn local_queue_and_actual_tick_order_attack_use_pick_then_continuation(){let _lock=SCENE.lock().unwrap();let mut game=setup(7,true);game.play_mouse(MouseButton::Left,true).unwrap();game.play_mouse(MouseButton::Right,true).unwrap();game.play_mouse(MouseButton::Middle,true).unwrap();assert!(game.trace.is_empty());assert_eq!((game.attack_clicks,game.use_clicks,game.pick_clicks),(1,1,1));game.hit=Hit::Block{pos:[0;3],face:1};game.keybind_frame().unwrap();assert_eq!(actions(&game),["dig","use","pick","dig"]);assert_eq!((game.attack_clicks,game.use_clicks,game.pick_clicks),(0,0,0));}
    #[test]fn local_initial_using_snapshot_drains_on_release_or_completion_tick(){let _lock=SCENE.lock().unwrap();
        for completed in [false,true]{for mask in 0..8{let mut game=setup(mask,true);game.using=Some((0,0));game.right=completed;game.complete_on_tick=completed;game.attack_clicks=1;game.use_clicks=1;game.pick_clicks=1;game.keybind_frame().unwrap();let expected=if mask&2!=0{vec!["release","attack"]}else{vec!["release"]};assert_eq!(actions(&game),expected,"{mask} {completed}");}}
    }
    #[test]fn local_held_use_precedes_continue_attack_and_screens_drain_queue(){let _lock=SCENE.lock().unwrap();let mut game=setup(7,true);game.hit=Hit::Block{pos:[0;3],face:1};game.right=true;game.left=true;game.keybind_frame().unwrap();assert_eq!(actions(&game),["use","dig"]);game.trace.clear();game.screen=Screen::Other;game.attack_clicks=1;game.use_clicks=1;game.pick_clicks=1;game.keybind_frame().unwrap();assert!(game.trace.is_empty());assert_eq!((game.attack_clicks,game.use_clicks,game.pick_clicks),(0,0,0));}
    #[test]fn kept_remote_using_and_direct_clicks_tick_obey_attack_policy(){let _lock=SCENE.lock().unwrap();
        for mask in 0..8{let mut game=setup(mask,true);game.remote=Some(Default::default());game.using=Some((0,0));game.keep_using=true;let mut input=InteractionInput::default();input.press(Button::Attack);input.press(Button::Use);input.press(Button::Pick);input.tick(&mut RemoteInputWorld(&mut game),false,false);assert!(game.using.is_some());assert_eq!(actions(&game),if mask&2!=0{vec!["attack"]}else{vec![]});
        game.trace.clear();game.keep_using=false;let mut clicks=Clicks::default();clicks.tick(&mut GameHands{game:&mut game,error:None},1,1,false,false);assert_eq!(actions(&game),if mask&2!=0{vec!["release","attack"]}else{vec!["release"]});}
    }
    #[test]fn use_queue_is_only_filtered_at_initial_using_query(){let _lock=SCENE.lock().unwrap();let mut game=setup(7,true);game.use_clicks=2;game.begin_using=true;game.keybind_frame().unwrap();assert_eq!(actions(&game),["use","use"]);assert!(game.using.is_some());}
}
