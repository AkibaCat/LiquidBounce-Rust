#![cfg(test)]
extern crate self as horizon_client;
pub mod mod_api {
    pub use horizon_protocol::packets::play::{Clientbound as ClientboundPlayPacket,Serverbound as ServerboundPlayPacket};
    pub fn mods() -> &'static [()] { &[] }
    pub(super) fn dispatch_client_death_screen_opening(game:&mut super::fixture::Game,_:&[()]) {
        game.events+=1;
        let immediate=game.behavior.borrow_mut().screen_event(game.running,true);
        if immediate { game.native_client_respawn(); }
    }
}
#[path="PACKET_BRIDGE_PATH"]
pub mod packet_bridge;
#[path="PACKETS_PATH"]
pub mod packets;

#[allow(dead_code)]
mod fixture {
use super::*;
use horizon_protocol::{connection::{Connection,ConnWriter,Pipe},packets::play};
use liquidbounce_behaviors::auto_respawn::AutoRespawn;
use std::{cell::RefCell,rc::Rc,sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},time::Instant};
use play::Serverbound as S;
type Result<T> = std::result::Result<T,String>;
struct Session {writer:Arc<Mutex<Option<ConnWriter>>>,configuring:Arc<AtomicBool>,serial:u64,debug:bool}
impl Session { /* SESSION_METHODS */ }
struct Remote {session:Session,show_death_screen:bool}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
enum Screen {Playing,Dead,Inventory}
#[derive(Default)]
struct Player {server_copy:bool,replaying:bool}
impl Player { /* IDENTITY_METHOD */ }
#[derive(Default)]
struct Ui {death_time:u32,death_text:Option<String>,death_confirm:Option<Instant>,died_at:Option<Instant>}
#[derive(Default)]
struct Toggles {resets:usize}
impl Toggles {fn respawn(&mut self) {self.resets+=1;}}
struct Health {health:f32}
pub(super) struct Game {
    player:Option<Player>,lan_acting_uuid:Option<u128>,window:bool,screen:Screen,remote:Option<Remote>,ui:Ui,death_focus:Option<usize>,toggles:Toggles,
    pub(super) behavior:Rc<RefCell<AutoRespawn>>,pub(super) running:bool,pub(super) events:usize,captures:Vec<bool>,closed:usize,
}
impl Game {
    fn native_window(&self)->bool {self.window}
    fn capture(&mut self,value:bool) {self.captures.push(value);}
    fn close_dialogs(&mut self) {self.closed+=1;}
    fn remote_send_carried(&self) {panic!("respawn must not use carried-item sync");}
    /* HELPERS */
    /* RESPAWN */
    /* REMOTE_RESPAWN */
    /* GAME_SEND */
    fn open_health_screen(&mut self,health:f32) {
        let p=Health{health};
        let shown=self.remote.as_ref().is_none_or(|r|r.show_death_screen);
        /* HEALTH_OPEN */
    }
    fn open_kill_screen(&mut self,me:bool)->Result<()> {
        /* KILL_OPEN */
        Ok(())
    }
    fn open_saved_screen(&mut self,player:Player,saved_death:bool) {
        /* SAVED_OPEN */
    }
    // Actor-driver plumbing: production behavior supplies the continuations;
    // each reads the real current screen again after earlier effects complete.
    fn tick(&mut self) {
        let due=self.behavior.borrow_mut().game_tick(self.running);
        for _ in 0..due {
            if self.native_client_is_death_screen() {self.native_client_respawn();}
        }
    }
}

#[test]
fn raw_int_delay_boundaries_parallel_waits_and_scope_cancellation() {
    let mut module=AutoRespawn::default();
    assert_eq!(module.delay(),0);
    assert!(!module.screen_event(false,true));
    assert!(!module.screen_event(true,false));
    for delay in [i32::MIN,-1,0] {
        module.set_delay(delay);
        assert_eq!(module.delay(),delay);
        assert!(module.screen_event(true,true));
        assert_eq!(module.game_tick(true),0);
    }
    for delay in [1,2,20,21,100] {
        module.set_delay(delay);
        assert!(!module.screen_event(true,true));
        // Changes after waitTicks started cannot change that continuation.
        module.set_delay(-7);
        for _ in 1..delay {assert_eq!(module.game_tick(true),0);}
        assert_eq!(module.game_tick(true),1);
        assert_eq!(module.game_tick(true),0);
    }
    module.set_delay(i32::MAX);
    module.screen_event(true,true);
    assert_eq!(module.game_tick(true),0);
    module.on_disabled();
    assert_eq!(module.game_tick(true),0);
    module.set_delay(2);
    module.screen_event(true,true);
    module.screen_event(true,true);
    assert!(!module.screen_event(true,false));
    assert_eq!(module.game_tick(true),0);
    assert_eq!(module.game_tick(true),2);
    module.screen_event(true,true);
    assert_eq!(module.game_tick(false),0);
    assert_eq!(module.game_tick(true),1,"temporary not-running before deadline does not cancel a guard-free wait");
    assert_eq!(module.game_tick(true),0);
    module.screen_event(true,true);
    module.game_tick(true);
    assert_eq!(module.game_tick(false),0,"not-running exactly at resume cancels that continuation");
    assert_eq!(module.game_tick(true),0);
}

#[test]
fn exact_death_opening_and_respawn_bodies_emit_ordinary_framed_packets() {
    use packets::{PacketPipeline,PacketListener,PacketEvent};
    struct Observe {cancel:Arc<AtomicBool>}
    impl PacketListener for Observe {
        fn outgoing(&self,event:&mut PacketEvent<'_,S>) {
            if let S::ClientCommand(packet)=event.packet {
                assert_eq!(packet.action,0);
                assert!(event.context.original());
                if self.cancel.load(Ordering::Relaxed) {event.cancel();} else {packet.action=7;}
            }
        }
    }
    let pipeline=Arc::new(PacketPipeline::default());
    let previous=packet_bridge::set_interceptor(Some(pipeline.clone()));
    let cancel=Arc::new(AtomicBool::new(false));
    let listener=pipeline.add_listener(0,Arc::new(Observe{cancel:cancel.clone()}));
    let (client,server)=Pipe::pair();
    let (_reader,writer)=Connection::local(client).split();
    let mut server=Connection::local(server);
    let session=Session{writer:Arc::new(Mutex::new(Some(writer))),configuring:Arc::new(AtomicBool::new(false)),serial:91,debug:false};
    let behavior=Rc::new(RefCell::new(AutoRespawn::default()));
    let mut game=Game{player:Some(Player::default()),lan_acting_uuid:None,window:true,screen:Screen::Playing,remote:Some(Remote{session,show_death_screen:true}),ui:Ui::default(),death_focus:None,toggles:Toggles::default(),behavior:behavior.clone(),running:true,events:0,captures:vec![],closed:0};
    let next=|connection:&mut Connection| {let (id,data)=connection.recv().unwrap();S::decode(id,&data).unwrap()};
    // The undispatched zero-delay handler runs before setScreen(DeathScreen).
    // It respawns and clears the old screen; the outer uncancelled assignment
    // then proceeds. This preserves the pinned MixinGui HEAD reentrancy detail.
    game.open_health_screen(0.0);
    assert_eq!(next(&mut server),S::ClientCommand(play::ClientCommand{action:7}));
    assert_eq!(game.events,1);
    assert_eq!(game.toggles.resets,1);
    assert_eq!(game.screen,Screen::Dead);
    assert_eq!(game.captures,vec![true,false]);
    game.screen=Screen::Playing;
    behavior.borrow_mut().set_delay(2);
    game.open_kill_screen(true).unwrap();
    assert_eq!(game.events,2);
    game.tick();
    assert_eq!(game.screen,Screen::Dead);
    game.tick();
    assert_eq!(next(&mut server),S::ClientCommand(play::ClientCommand{action:7}));
    assert_eq!(game.screen,Screen::Playing);
    // Manual respawn / another screen during the delay leaves nothing to do.
    game.open_health_screen(0.0);
    game.screen=Screen::Inventory;
    game.tick();game.tick();
    assert_eq!(game.toggles.resets,2);
    // Source checks the type only when the wait ends, so closing and reopening
    // before its deadline still lets the older continuation act.
    game.screen=Screen::Playing;
    game.open_health_screen(0.0);
    game.tick();
    game.screen=Screen::Inventory;
    game.screen=Screen::Dead;
    game.tick();
    assert_eq!(next(&mut server),S::ClientCommand(play::ClientCommand{action:7}));
    // Two pending runs expire together: only the first sees DeathScreen.
    behavior.borrow_mut().screen_event(true,true);
    behavior.borrow_mut().screen_event(true,true);
    game.screen=Screen::Dead;
    game.tick();game.tick();
    assert_eq!(next(&mut server),S::ClientCommand(play::ClientCommand{action:7}));
    assert_eq!(game.toggles.resets,4);
    // An outgoing cancellation suppresses the actual packet, while the local
    // clear/reset effects still occur, as player.respawn then setScreen(null).
    cancel.store(true,Ordering::Relaxed);
    behavior.borrow_mut().set_delay(0);
    game.screen=Screen::Playing;
    game.open_kill_screen(true).unwrap();
    assert_eq!(game.toggles.resets,5);
    game.send(S::ClientTickEnd(horizon_protocol::packets::Empty));
    assert_eq!(next(&mut server),S::ClientTickEnd(horizon_protocol::packets::Empty));
    pipeline.remove_listener(listener);
    game.remote.as_mut().unwrap().show_death_screen=false;
    let events=game.events;
    game.screen=Screen::Playing;
    game.open_kill_screen(true).unwrap();
    assert_eq!(next(&mut server),S::ClientCommand(play::ClientCommand{action:0}));
    assert_eq!(game.events,events,"hidden death screen does not produce ScreenEvent");
    game.remote.as_mut().unwrap().show_death_screen=true;
    game.player=None;
    game.open_saved_screen(Player::default(),true);
    assert_eq!(next(&mut server),S::ClientCommand(play::ClientCommand{action:0}));
    assert_eq!(game.events,events+1,"saved death opens after installing actual player");
    // Guards reject server, swapped guests, copied and speculative bodies.
    for scenario in 0..4 {
        game.window=scenario!=0;
        game.lan_acting_uuid=(scenario==1).then_some(9);
        let player=game.player.as_mut().unwrap();
        player.server_copy=scenario==2;player.replaying=scenario==3;
        game.screen=Screen::Playing;
        let before=game.events;
        game.open_health_screen(0.0);
        assert_eq!(game.events,before);
        assert!(!game.native_client_is_death_screen());
        assert!(!game.native_client_respawn());
    }
    packet_bridge::set_interceptor(previous);
}
}
