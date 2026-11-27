use crate::auto_break::{AutoBreak, KeyQuery};
use crate::interaction_input::{Button, InteractionInput, InteractionWorld};
use crate::keybinds::KeyBinds;
use crate::keymapping_clicks::ClickQueues;

#[derive(Default)]
struct RemoteWorld {
    clicks: ClickQueues,
    binds: KeyBinds,
    enabled: bool,
    screen: bool,
    grabbed: bool,
    solid: bool,
    destroying: bool,
    miss: i32,
    instant: bool,
    use_opens_screen: bool,
    using: bool,
    events: Vec<String>,
}
impl RemoteWorld {
    fn new() -> Self {
        Self { enabled: true, grabbed: true, solid: true, ..Default::default() }
    }
    fn drain(&mut self, input: &mut InteractionInput) {
        for (mapping, button) in [("key.attack", Button::Attack), ("key.use", Button::Use), ("key.pickItem", Button::Pick)] {
            while self.clicks.consume(mapping) { input.press(button); }
        }
    }
    fn query(&mut self, original: bool, miss: i32, destroying: bool) -> bool {
        self.events.push(format!("query:{miss}:{destroying}"));
        let effect = AutoBreak.query(self.enabled, KeyQuery {
            attack:true,miss_time:miss,block_hit:true,block_state_available:true,air:!self.solid,destroying,
        }, original);
        if effect.click {
            let key = self.binds.get("key.attack");
            self.clicks.click(key,self.binds.mappings_for(key));
        }
        effect.pressed
    }
}
impl InteractionWorld for RemoteWorld {
    fn client_attack_key(&mut self, original: bool) -> bool { self.query(original,self.miss,self.destroying) }
    fn client_attack_query_allowed(&self) -> bool { !self.screen }
    fn client_attack_mouse_grabbed(&self) -> bool { self.grabbed }
    fn pick_hit_result(&mut self) { self.events.push("pick_hit".into()); }
    fn release_hit_result(&mut self) { self.events.push("release_hit".into()); }
    fn using_item(&self) -> bool { self.using }
    fn release_using(&mut self) { self.events.push("release_use".into());self.using=false; }
    fn start_attack(&mut self) -> bool { self.events.push("attack".into()); self.instant }
    fn start_use(&mut self) {
        self.events.push("use".into());
        if self.use_opens_screen { self.screen=true;self.clicks.release_all(); }
    }
    fn pick(&mut self) { self.events.push("pick".into()); }
    fn use_delay(&self) -> u32 { 1 }
    fn continue_attack(&mut self, down: bool) { self.events.push(format!("continue:{down}")); }
}

#[test]
fn queued_first_click_acts_next_tick_and_survives_disable_and_rebind() {
    let mut world=RemoteWorld::new();let mut input=InteractionInput::default();
    world.binds.set("key.use","key.mouse.left");
    world.binds.set("key.forward","key.mouse.left");
    world.binds.set("key.screenshot","key.mouse.left");
    input.tick(&mut world,false,false);
    assert_eq!(world.events,["pick_hit","query:0:false","continue:true","release_hit"]);
    assert_eq!(world.clicks.pending("key.attack"),1);
    assert_eq!(world.clicks.pending("key.use"),1);
    assert_eq!(world.clicks.pending("key.forward"),1);
    assert_eq!(world.clicks.pending("key.screenshot"),1);
    world.enabled=false;
    world.binds.set("key.attack","key.keyboard.z");
    world.binds.set("key.use","key.mouse.right");
    world.events.clear();world.drain(&mut input);
    input.tick(&mut world,false,false);
    assert_eq!(world.events,["pick_hit","attack","use","query:0:false","continue:false","release_hit"]);
    assert_eq!(world.clicks.pending("key.forward"),1);
    assert_eq!(world.clicks.pending("key.screenshot"),1);
    assert_eq!(world.clicks.pending("key.attack"),0);
}

#[test]
fn instant_attack_and_screen_open_short_circuit_but_ungrabbed_still_queries() {
    for instant in [false,true] {
        let mut world=RemoteWorld::new();let mut input=InteractionInput::default();
        world.instant=instant;input.press(Button::Attack);
        input.tick(&mut world,false,false);
        assert_eq!(world.events.iter().any(|event|event.starts_with("query:")),!instant);
    }
    let mut world=RemoteWorld::new();world.use_opens_screen=true;
    let mut input=InteractionInput::default();input.press(Button::Use);
    input.tick(&mut world,false,false);
    assert!(!world.events.iter().any(|event|event.starts_with("query:")));
    assert_eq!(world.clicks.pending("key.attack"),0);
    let mut world=RemoteWorld::new();world.grabbed=false;
    input.tick(&mut world,false,false);
    assert_eq!(world.clicks.pending("key.attack"),1);
    assert!(world.events.contains(&"continue:false".into()));
}

#[test]
fn using_item_drains_pending_queues_then_still_runs_late_key_query() {
    let mut world=RemoteWorld::new();world.using=true;
    let mut input=InteractionInput::default();
    for button in [Button::Attack,Button::Use,Button::Pick] { input.press(button); }
    input.tick(&mut world,false,false);
    assert_eq!(world.events,["pick_hit","release_use","query:0:false","continue:true","release_hit"]);
    assert_eq!(world.clicks.pending("key.attack"),1);
    world.clicks.release_all();
    assert_eq!(world.clicks.pending("key.attack"),0);
}

#[test]
fn destroying_and_miss_time_read_real_query_phase() {
    for (miss,destroying,click) in [(0,false,true),(0,true,false),(1,false,false),(-1,false,true)] {
        let mut world=RemoteWorld::new();world.miss=miss;world.destroying=destroying;
        InteractionInput::default().tick(&mut world,true,false);
        assert_eq!(world.clicks.pending("key.attack")!=0,click);
        // Original pressed=true never suppresses a needed first click.
        assert!(world.events.contains(&"continue:true".into()));
    }
    let mut world=RemoteWorld::new();world.binds.set("key.attack","key.keyboard.unknown");
    InteractionInput::default().tick(&mut world,false,false);
    assert_eq!(world.clicks.pending("key.attack"),0);
    assert!(world.events.contains(&"continue:true".into()));
}

mod local {
    use super::*;
    use crate::first_person::{Clicks,Hand,HandWorld,Hit,SwingAnimation,UseResult};

    struct LocalWorld(RemoteWorld);
    impl HandWorld for LocalWorld {
        fn client_attack_key(&mut self, original: bool, miss:i32,destroying:bool)->bool {self.0.query(original,miss,destroying)}
        fn client_attack_query_allowed(&self)->bool {!self.0.screen}
        fn client_attack_mouse_grabbed(&self)->bool {self.0.grabbed}
        fn hit(&mut self)->Hit {Hit::Block{pos:[0;3],face:1}}
        fn is_air(&mut self,_:[i32;3])->bool {!self.0.solid}
        fn creative(&self)->bool {false}
        fn spectator(&self)->bool {false}
        fn instabuild(&self)->bool {false}
        fn hands_busy(&self)->bool {false}
        fn using_item(&self)->bool {self.0.using}
        fn item_enabled(&mut self,_:Hand)->bool {true}
        fn held_empty(&mut self,_:Hand)->bool {true}
        fn held_count(&mut self,_:Hand)->i32 {0}
        fn infinite_materials(&self)->bool {false}
        fn cannot_attack(&mut self)->bool {false}
        fn piercing_weapon(&mut self)->bool {false}
        fn attack_animation(&mut self)->SwingAnimation {SwingAnimation::DEFAULT}
        fn interact_animation(&mut self,_:Hand)->SwingAnimation {SwingAnimation::DEFAULT}
        fn piercing_attack(&mut self,_:SwingAnimation) {}
        fn attack_entity(&mut self)->bool {false}
        fn block_action_restricted(&mut self,_:[i32;3])->bool {false}
        fn within_border(&mut self,_:[i32;3])->bool {true}
        fn attack_block(&mut self,_:[i32;3]) {self.0.events.push("attack_block".into());}
        fn destroy_progress(&mut self,_:[i32;3])->f32 {0.1}
        fn destroy_block(&mut self,_:[i32;3])->bool {false}
        fn main_hand_key(&mut self)->String {"hand".into()}
        fn destroy_stage(&mut self,_:[i32;3],_:Option<i32>) {}
        fn swing(&mut self,_:Hand,_:SwingAnimation) {self.0.events.push("swing".into());}
        fn reset_attack_strength(&mut self) {}
        fn entity_within_border(&mut self)->bool {true}
        fn use_on_entity(&mut self,_:Hand)->Option<UseResult> {None}
        fn use_on_block(&mut self,_:Hand)->UseResult {UseResult::Pass}
        fn use_item(&mut self,_:Hand)->UseResult {UseResult::Pass}
        fn item_used(&mut self,_:Hand) {}
        fn release_using(&mut self) {self.0.using=false;}
    }

    #[test]
    fn local_late_query_uses_mem_taken_click_state() {
        let mut clicks=Clicks { miss_time:7, ..Clicks::default() };
        let mut world=LocalWorld(RemoteWorld::new());
        clicks.tick(&mut world,0,0,false,false);
        assert_eq!(world.0.events[0],"query:7:false");
        assert_eq!(world.0.clicks.pending("key.attack"),0);
        clicks.miss_time=0;clicks.destroying.active=true;
        world.0.events.clear();
        clicks.tick(&mut world,0,0,false,false);
        assert_eq!(world.0.events[0],"query:0:true");
        assert_eq!(world.0.clicks.pending("key.attack"),0);
    }
}
