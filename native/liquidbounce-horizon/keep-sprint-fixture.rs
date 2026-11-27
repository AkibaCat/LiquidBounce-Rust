#![allow(dead_code)]
extern crate self as horizon_client;
use horizon_core::geometry::{Aabb, Vec3};
use std::{collections::BTreeMap, sync::{LazyLock, Mutex}};
#[path = "__KEEP_SPRINT_PATH__"]
mod keep_sprint;
#[path = "__RANDOM_PATH__"]
mod random;
#[path = "__ATTACK_STRENGTH_PATH__"]
mod attack_strength;

#[derive(Default)]
struct Hurt { hurt_time: i32 }
#[derive(Default)]
struct Health { hurt: Hurt }
struct Player { sprinting: bool, server_copy: bool, replaying: bool, health: Health,
    velocity: Vec3, position: Vec3, auto_spin_attack_dmg: f32, set_calls: u32 }
impl Player {
    __CLIENT_GUARD__
    fn set_sprinting(&mut self, value: bool) -> Result<(), ()> { self.sprinting = value; self.set_calls += 1; Ok(()) }
}
struct Kind { id: String }
struct Mob { id: u32, kind: Kind }
mod particles {
    pub struct ParticleOptions;
    pub enum ParticleType { Crit }
    impl ParticleOptions { pub fn simple(_: ParticleType) -> Self { Self } }
}
enum TrackingTarget { Entity(i32) }
struct TrackingBody;
fn mob_tracking_body(_: &Mob) -> TrackingBody { TrackingBody }
#[derive(Default)]
struct Particles { emitted: usize }
impl Particles { fn tracking_emitter(&mut self, _: particles::ParticleOptions, _: TrackingTarget, _: TrackingBody) { self.emitted += 1; } }
#[derive(Default)]
struct Audio { sounds: Vec<String> }
impl Audio { fn play(&mut self, sound: &str, _: Option<[f64;3]>, _: f32, _: f32) { self.sounds.push(sound.into()); } }
#[derive(Default)]
struct World { random: () }
#[derive(Default)]
struct Enchantments { knockback: f32 }
impl Enchantments { fn modify_value(&self, _: Option<&()>, _: &str, _: &EffectContext<'_>, _: &mut (), _: f32) -> f32 { self.knockback } }
#[derive(Default)]
struct Stations { enchantments: Enchantments }
#[derive(Default)]
struct EffectContext<'a> { this: Option<&'a str>, attacker: Option<&'a str>, direct_attacker: Option<&'a str>, is_direct: bool, items: Option<&'a ()>, unused: bool }
macro_rules! world_stations { ($game:expr) => { $game.stations }; }
macro_rules! world_items { ($game:expr) => { &$game.items }; }
mod mace { pub struct Spared; }
struct Game { player: Option<Player>, mobs: Vec<Mob>, window: bool, lan_acting: Option<u32>, lan_acting_uuid: Option<u32>,
    hand_strength: attack_strength::AttackStrength, attributes: BTreeMap<String, f64>,
    particles: Particles, audio: Audio, stations: Stations, world: World, items: (), held: Option<()> }
impl Game {
    fn new() -> Self { let mut strength=attack_strength::AttackStrength::default(); strength.ticker=20; Self { player: Some(Player { sprinting:true, server_copy:false, replaying:false,
        health:Health::default(), velocity:Vec3::new(2.0,0.25,-3.0),position:Vec3::ZERO,
        auto_spin_attack_dmg:0.0,set_calls:0 }),
        mobs:vec![Mob {id:7,kind:Kind{id:"minecraft:player".into()}}], window:true,lan_acting:None,lan_acting_uuid:None,
        hand_strength:strength,attributes:BTreeMap::new(),
        particles:Particles::default(),audio:Audio::default(),stations:Stations::default(),world:World::default(),items:(),held:None } }
    fn native_window(&self) -> bool { self.window }
    fn own_attribute(&mut self, key: &str) -> Option<f64> { self.attributes.get(key).copied() }
    fn main_hand_attack_speed(&self) -> f64 { 4.0 }
    fn critical_attack(&self, _: f32) -> bool { false }
    fn attack_strength_scale(&self) -> f32 { self.hand_strength.scale(0.5, 4.0) }
    fn held(&self) -> Option<&()> { self.held.as_ref() }
    fn sweep_attack_position(&mut self, _: f32, _: bool) -> Option<[f64;3]> { None }
    fn sweep_attack(&mut self, _: [f64;3], _: Option<[f64;3]>, _: Option<(Aabb,mace::Spared)>, _: f32) {}
    fn area_critical_hit(&mut self, _: i32, _: Aabb, _: bool) {}
}
mod game_methods {
    use super::*;
    impl Game {
        __HOOKS__
        __REMOTE_ATTACK__
        __EXTRA_KNOCKBACK__
        __KNOCKBACK_TAIL__
        __SPECIAL_TAIL__
        __ATTACK_EFFECTS__
    }
}
struct ModuleState { enabled: bool, module: keep_sprint::KeepSprint, unit: f32, draws: usize }
static MODULE: Mutex<ModuleState> = Mutex::new(ModuleState { enabled:false, module:keep_sprint::KeepSprint {
    motion:[100.0;2],motion_when_hurt:[100.0;2],hurt_time:[1,10],chance:100.0,sprinting:false },unit:0.25,draws:0 });
static SCENE: Mutex<()> = Mutex::new(());
impl mod_api::Host for Game {}
mod mod_api {
    use super::*;
    pub trait Host {}
    pub trait RenderHooks: Send + Sync {}
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item=&'a dyn Hooks> { mods.iter().map(Box::as_ref) }
    mod generated { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use generated::*;
    struct Module;
    impl Hooks for Module {
        fn client_keep_sprint_attack(&self, _: &dyn Host, event: &mut ClientKeepSprintAttackEvent) {
            let state = MODULE.lock().unwrap(); event.sprinting=state.module.attack_sprinting(state.enabled,event.sprinting);
        }
        fn client_keep_sprint_motion(&self, _: &dyn Host, event: &mut ClientKeepSprintMotionEvent) {
            let mut state=MODULE.lock().unwrap(); if state.enabled {
                let module=state.module; event.multiplier=module.motion_multiplier(event.hurt_time,||{state.draws+=1;state.unit});
            }
        }
        fn client_keep_sprint_set(&self, _: &dyn Host, event: &mut ClientKeepSprintSetEvent) {
            let mut state=MODULE.lock().unwrap(); let enabled=state.enabled; event.apply=state.module.set_sprinting(enabled,event.requested);
        }
    }
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS:LazyLock<Vec<Box<dyn Hooks>>>=LazyLock::new(||vec![Box::new(Module)]); &MODS
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn reset(enabled:bool,sprinting:bool) {
        let mut state=MODULE.lock().unwrap(); *state=ModuleState {enabled,module:keep_sprint::KeepSprint {sprinting,..Default::default()},unit:0.25,draws:0};
    }
    #[test]
    fn actual_remote_attack_keeps_sprint_and_suppresses_second_same_tick_sprint_knockback() {
        let _scene=SCENE.lock().unwrap(); reset(true,true);
        let mut game=Game::new(); game.remote_player_attack(0);
        assert_eq!(game.player.as_ref().unwrap().velocity,Vec3::new(2.0,0.25,-3.0));
        assert!(game.player.as_ref().unwrap().sprinting);
        assert_eq!(game.player.as_ref().unwrap().set_calls,0);
        assert!(!MODULE.lock().unwrap().module.sprinting);
        game.remote_player_attack(0);
        assert_eq!(game.player.as_ref().unwrap().set_calls,0);
        let actual=game.native_client_combat_state()[0]!=0;
        MODULE.lock().unwrap().module.post_tick(true,actual);
        assert!(game.client_attack_sprinting(false));
        assert_eq!(game.extra_knockback("minecraft:player",1.0),(0.5,true));
    }
    #[test]
    fn chance_failure_still_keeps_sprint_and_motion_uses_inclusive_hurt_range() {
        let _scene=SCENE.lock().unwrap(); reset(true,true);
        let mut game=Game::new(); game.attributes.insert("minecraft:attack_knockback".into(),1.0);
        MODULE.lock().unwrap().module.chance=0.0;
        game.remote_player_attack(0);
        assert_eq!(game.player.as_ref().unwrap().velocity,Vec3::new(1.2,0.25,-1.7999999999999998));
        assert!(game.player.as_ref().unwrap().sprinting);
        assert_eq!(MODULE.lock().unwrap().draws,0);
        { let mut state=MODULE.lock().unwrap(); state.module.chance=100.0; state.module.motion_when_hurt=[20.0;2]; }
        game.player.as_mut().unwrap().health.hurt.hurt_time=10;
        game.special_attack_slowdown(1.0);
        assert_eq!(game.player.as_ref().unwrap().velocity.x,1.2*0.2);
        assert_eq!(game.player.as_ref().unwrap().velocity.y,0.25);
    }
    #[test]
    fn disabled_and_server_replay_paths_retain_vanilla_tail() {
        let _scene=SCENE.lock().unwrap(); reset(false,true);
        let mut game=Game::new(); game.remote_player_attack(0);
        assert!(!game.player.as_ref().unwrap().sprinting);
        assert_eq!(game.player.as_ref().unwrap().set_calls,1);
        assert_eq!(game.player.as_ref().unwrap().velocity.x,1.2);
        assert!(!MODULE.lock().unwrap().module.sprinting);
        for mode in 0..5 {
            reset(true,true); let mut game=Game::new();
            match mode {0=>game.window=false,1=>game.lan_acting=Some(1),2=>game.player.as_mut().unwrap().server_copy=true,3=>game.player.as_mut().unwrap().replaying=true,_=>game.lan_acting_uuid=Some(91)}
            game.remote_player_attack(0);
            assert!(!game.player.as_ref().unwrap().sprinting);
            assert_eq!(game.player.as_ref().unwrap().velocity.x,1.2);
            assert!(MODULE.lock().unwrap().module.sprinting);
        }
    }
    #[test]
    fn actual_remote_nondamaging_weak_and_nonplayer_paths_skip_the_tail() {
        let _scene=SCENE.lock().unwrap();
        for mode in 0..3 {
            reset(true,true); let mut game=Game::new();
            match mode {0=>game.attributes.insert("minecraft:attack_damage".into(),0.0),1=>{game.hand_strength.ticker=0;None},_=>{game.mobs[0].kind.id="minecraft:zombie".into();None}};
            game.remote_player_attack(0);
            assert_eq!(game.player.as_ref().unwrap().velocity.x,2.0);
            assert!(MODULE.lock().unwrap().module.sprinting);
        }
    }
    #[test]
    fn thread_local_random_matches_real_jdk_draws() {
        let mut random=random::ThreadRandom::seeded(__JAVA_SEED__,__JAVA_THREAD_ID__);
        for bits in __JAVA_BITS__ { assert_eq!(random.next_unit().to_bits(),bits); }
    }
    #[test]
    fn integer_ranges_match_real_kotlin_default_including_rejection_draw_count() {
        for (first,last,values,next) in __JAVA_INT_RANGES__ {
            let mut random=random::ThreadRandom::seeded(__JAVA_SEED__,__JAVA_THREAD_ID__);
            for expected in values { assert_eq!(random.next_i32_inclusive(first,last),expected); }
            assert_eq!(random.next_i32(),next);
        }
    }
}
