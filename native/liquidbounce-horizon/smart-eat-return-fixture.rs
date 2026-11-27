//! Actual transformed RETURN wrapper bodies. Native effects are explicit
//! controllable stubs so tests distinguish RETURN ordering from effect output.
use horizon_items::items::ItemStack;
use std::{collections::VecDeque,cell::RefCell};
type Result<T>=std::result::Result<T,&'static str>;
#[derive(Clone,Copy,Debug,PartialEq)]enum Hand {Main,Off}
#[derive(Clone,Copy,Debug,PartialEq)]enum GuestUse {OnBlock(Hand),Item(Hand)}
struct Hit;
struct Player {yaw:f32,pitch:f32}
struct NativeInput {claimed:bool}
impl NativeInput {fn claimant(&self,_:&str)->Option<()> {self.claimed.then_some(())}}
struct Game {
    owner:bool,spectator:bool,guest_use:Option<GuestUse>,hand:Hand,held:Option<ItemStack>,pass:Option<bool>,native_input:NativeInput,
    yaw:f32,pitch:f32,player:Option<Player>,trace:RefCell<Vec<String>>,local:VecDeque<Result<bool>>,remote_result:bool,
}
impl Game {
    fn new()->Self {Self {owner:true,spectator:false,guest_use:None,hand:Hand::Main,held:Some(ItemStack::new("minecraft:diamond_pickaxe",1)),pass:Some(true),native_input:NativeInput {claimed:false},yaw:10.,pitch:20.,player:Some(Player {yaw:11.,pitch:21.}),trace:RefCell::new(vec![]),local:VecDeque::new(),remote_result:false}}
    fn client_smart_eat_player(&self)->bool {self.owner}
    fn spectator(&self)->bool {self.spectator}
    fn held(&self)->Option<&ItemStack> {self.held.as_ref()}
    fn with_hand<T>(&mut self,hand:Hand,call:impl FnOnce(&mut Self)->T)->T {let previous=self.hand;self.hand=hand;let result=call(self);self.hand=previous;result}
    fn client_item_use_pass(&self,stack:&ItemStack)->Option<bool> {self.trace.borrow_mut().push(format!("classify:{}:{:?}:{}:{}",stack.item,self.hand,self.yaw,self.pitch));self.owner.then_some(self.pass).flatten()}
    fn native_client_item_interacted(&mut self,pass:bool) {self.trace.borrow_mut().push(format!("event:{pass}:{:?}:{}:{}",self.guest_use,self.yaw,self.pitch));}
    fn client_use_hand_committed(&mut self,hit:Option<&Hit>)->Result<bool> {self.trace.borrow_mut().push(format!("local:{}:{:?}",hit.is_some(),self.guest_use));self.local.pop_front().unwrap_or(Ok(false))}
    fn client_remote_use_item_committed(&mut self,hand:Hand)->bool {self.trace.borrow_mut().push(format!("remote:{hand:?}"));self.remote_result}
    fn client_remote_use_item_rotated_committed(&mut self,hand:Hand,yaw:f32,pitch:f32)->bool {self.native_with_item_rotation(yaw,pitch,|g|g.trace.borrow_mut().push(format!("predict:{hand:?}:{}:{}",g.yaw,g.pitch)));self.trace.borrow_mut().push("send".into());self.remote_result}
    __LOCAL_WRAPPER__
    __REMOTE_WRAPPER__
    __ROTATED_WRAPPER__
    __ROTATION__
}

#[test]fn actual_return_boundary_excludes_consuming_block_phase_and_restores_context(){
    let mut g=Game::new();g.local.push_back(Ok(true));
    assert!(g.use_hand(Some(&Hit)).unwrap());
    assert_eq!(g.trace.borrow().as_slice(),["local:true:Some(OnBlock(Main))"]);
    assert_eq!(g.guest_use,None);
    g.trace.borrow_mut().clear();g.local.extend([Ok(false),Ok(true)]);
    assert!(!g.use_hand(Some(&Hit)).unwrap()); // pre-use PASS wins over combined server action bool
    assert_eq!(g.trace.borrow().as_slice(),["local:true:Some(OnBlock(Main))","classify:minecraft:diamond_pickaxe:Main:10:20","local:false:Some(Item(Main))","event:true:None:10:20"]);
}

#[test]fn actual_local_errors_and_nonclient_turns_emit_no_return_event(){
    let mut g=Game::new();g.local.push_back(Err("block"));
    assert_eq!(g.use_hand(Some(&Hit)),Err("block"));assert_eq!(g.guest_use,None);
    g.trace.borrow_mut().clear();g.local.push_back(Err("item"));
    assert_eq!(g.use_hand(None),Err("item"));assert_eq!(g.guest_use,None);
    assert_eq!(g.trace.borrow().len(),2); // classification and failed effect, no RETURN
    g.trace.borrow_mut().clear();g.guest_use=Some(GuestUse::Item(Hand::Off));
    assert!(!g.use_hand(None).unwrap());
    assert_eq!(g.trace.borrow().as_slice(),["local:false:Some(Item(Off))"]);
    g.guest_use=None;g.owner=false;g.trace.borrow_mut().clear();
    assert!(!g.use_hand(None).unwrap());assert_eq!(g.trace.borrow().as_slice(),["local:false:None"]);
}

#[test]fn actual_empty_hand_is_skipped_by_normal_click_but_direct_item_use_emits_pass(){
    let mut g=Game::new();g.held=None;
    assert!(!g.use_hand(None).unwrap());assert!(g.trace.borrow().is_empty());
    assert!(!g.client_use_item_only(true).unwrap());
    assert_eq!(g.trace.borrow().as_slice(),["classify:minecraft:air:Main:10:20","local:false:Some(Item(Main))","event:true:None:10:20"]);
}

#[test]fn actual_remote_return_pass_is_independent_of_native_effect_boolean(){
    for pass in [true,false] {for consumed in [true,false] {
        let mut g=Game::new();g.pass=Some(pass);g.remote_result=consumed;
        assert_eq!(g.remote_use_item(Hand::Off),consumed);
        assert_eq!(g.trace.borrow().as_slice(),["classify:minecraft:diamond_pickaxe:Off:10:20","remote:Off",&format!("event:{pass}:None:10:20")]);
        assert_eq!(g.hand,Hand::Main);
    }}
    let mut g=Game::new();g.native_input.claimed=true;
    assert!(!g.remote_use_item(Hand::Main));assert_eq!(g.trace.borrow().as_slice(),["remote:Main"]);
}

#[test]fn actual_rotated_return_classifies_under_managed_angles_and_emits_after_send_and_restore(){
    let mut g=Game::new();g.remote_result=true;
    assert!(g.remote_use_item_rotated(Hand::Off,40.,50.));
    assert_eq!(g.trace.borrow().as_slice(),["classify:minecraft:diamond_pickaxe:Off:40:50","predict:Off:40:50","send","event:true:None:10:20"]);
    assert_eq!((g.yaw,g.pitch),(10.,20.));
    assert_eq!((g.player.as_ref().unwrap().yaw,g.player.as_ref().unwrap().pitch),(11.,21.));
}

#[test]fn actual_spectator_return_emits_pass_without_prediction_or_packet(){
    let mut g=Game::new();g.spectator=true;
    assert!(!g.remote_use_item(Hand::Main));
    assert_eq!(g.trace.borrow().as_slice(),["classify:minecraft:diamond_pickaxe:Main:10:20","event:true:None:10:20"]);
    g.trace.borrow_mut().clear();
    assert!(!g.client_use_item_only(false).unwrap());
    assert_eq!(g.trace.borrow().as_slice(),["event:true:None:10:20"]);
}
