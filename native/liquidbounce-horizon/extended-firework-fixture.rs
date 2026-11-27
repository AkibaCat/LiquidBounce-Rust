#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
use horizon_core::geometry::{Vec3, trig};
use liquidbounce_behaviors::extended_firework::{ExtendedFirework, Mode, direction_from_rotation};
use serde_json::{Value, json};
#[allow(unused_imports)]
mod config;

fn bits(value: &Value) -> [u64; 3] {
    std::array::from_fn(|i| value[i].as_i64().unwrap() as u64)
}
fn xyz(value: &Value) -> [f64; 3] {
    ["x", "y", "z"].map(|key| value[key].as_f64().unwrap())
}
#[test]
fn actual_gson_vec3_adapter_imports_match_raw_bits_and_errors() {
    let oracle: Value = serde_json::from_str(include_str!("oracle.json")).unwrap();
    for (index, row) in oracle["imports"].as_array().unwrap().iter().enumerate() {
        let tree = json!({"name":"ExtendedFirework","value":[{"name":"Mode","active":"Normal","value":[],"choices":{"Normal":{"name":"Normal","value":[]},"Custom":{"name":"Custom","value":[{"name":"VelocityMultiplier","value":row["input"]}]} }}]});
        let patch = config::parse_settings(config::ModuleKind::ExtendedFirework, &tree).unwrap();
        let actual = patch.changes.iter().find_map(|change| match change {
            config::SettingChange::ExtendedFirework(config::ExtendedFireworkSetting::VelocityMultiplier(value))=>Some(*value),
            _=>None,
        });
        assert_eq!(actual.is_some(), row["ok"].as_bool().unwrap(), "import row {index}: {} issues {:?}", row["input"], patch.issues);
        if let Some(actual) = actual {
            assert_eq!(actual.map(f64::to_bits), bits(&row["bits"]), "import row {index}");
        }
    }
}
#[derive(Clone, Copy, Default)]
struct Override { multiplier: Option<[f64; 3]>, look: Option<[f64; 3]> }
thread_local! { static OVERRIDE: std::cell::Cell<Override> = const { std::cell::Cell::new(Override {multiplier: None, look:None}) }; }
pub mod mod_api {
    pub trait RenderHooks: Send + Sync {}
    #[allow(unused_imports)]
    mod generated_host { include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs")); }
    pub use generated_host::*;
    mod generated { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use generated::*;
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(Box::as_ref)}
    pub fn mods()->&'static [Box<dyn Hooks>] {
        static MODS:std::sync::LazyLock<Vec<Box<dyn Hooks>>>=std::sync::LazyLock::new(||vec![Box::new(Policy)]);
        &MODS
    }
    struct Policy;
    impl RenderHooks for Policy {}
    impl Hooks for Policy {
        fn client_firework_boost(&self,_:&dyn Host,event:&mut ClientFireworkBoostEvent<'_>) {
            crate::OVERRIDE.with(|cell| {
                let value=cell.get();
                if let Some(look)=value.look { *event.look=look; }
                if let Some(multiplier)=value.multiplier { *event.multiplier=multiplier;event.apply=true; }
            });
        }
    }
}
#[derive(Default)]
struct Player { position:Vec3,velocity:Vec3,fall_flying:bool,server_copy:bool,replaying:bool }
impl Player {
    fn native_client_queries_allowed(&self)->bool { !self.server_copy && !self.replaying }
}
#[derive(Default)]
struct Rocket {position:Vec3,velocity:Vec3}
mod game {
    use super::*;
    struct Game<'a> { window:bool,lan_acting_uuid:Option<()>,player:Option<Player>,pitch:f32,yaw:f32,marker:std::marker::PhantomData<&'a ()> }
    impl Game<'_> {
        fn native_window(&self)->bool {self.window}
        fn rocket_hand_angle(&self)->[f64;3] {[0.25,-0.1,-0.5]}
        /* FOLLOW */
    }
    mod native_hooks { use super::Game;use crate::mod_api; /* NATIVE */ }
    mod mod_api_host {include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs"));}
    fn game()->Game<'static> {
        Game {window:true,lan_acting_uuid:None,player:Some(Player {fall_flying:true,..Default::default()}),pitch:0.,yaw:0.,marker:std::marker::PhantomData}
    }
    #[test]
    fn actual_transformed_rocket_consumer_and_generated_dispatch_match_jvm_raw_bits() {
        let oracle: Value=serde_json::from_str(include_str!("oracle.json")).unwrap();
        for (index,row) in oracle["boosts"].as_array().unwrap().iter().enumerate() {
            let input=&row["input"];
            let module=ExtendedFirework {mode:if input["mode"]=="Normal" {Mode::Normal}else{Mode::Custom},custom:xyz(&input["custom"])};
            assert_eq!(module.multiplier().map(f64::to_bits),bits(&row["multiplierBits"]),"multiplier row {index}");
            let mut game=game();game.pitch=input["pitch"].as_f64().unwrap() as f32;game.yaw=input["yaw"].as_f64().unwrap() as f32;
            let managed=input["managed"].as_bool().unwrap();
            let look=if managed {direction_from_rotation(game.pitch,game.yaw,|v|[trig::sin(v),trig::cos(v)])}else{let v=horizon_world::view_direction::view_vector(game.pitch,game.yaw);[v.x,v.y,v.z]};
            assert_eq!(look.map(f64::to_bits),bits(&row["lookBits"]),"look row {index}");
            let enabled=input["enabled"].as_bool().unwrap();
            OVERRIDE.with(|cell|cell.set(Override {look:managed.then_some(look),multiplier:enabled.then_some(module.multiplier())}));
            let movement=xyz(&input["movement"]);
            game.player.as_mut().unwrap().velocity=Vec3::new(movement[0],movement[1],movement[2]);
            game.player.as_mut().unwrap().position=Vec3::new(1.,2.,3.);
            let mut rocket=Rocket::default();game.follow_own_player(&mut rocket);
            assert_eq!([rocket.velocity.x,rocket.velocity.y,rocket.velocity.z].map(f64::to_bits),bits(&row["resultBits"]),"boost row {index}");
            assert_eq!(rocket.position,Vec3::new(1.25,1.9,2.5));
            assert_eq!(rocket.velocity,game.player.as_ref().unwrap().velocity);
        }
    }
    #[test]
    fn native_identity_guards_and_nongliding_rockets_preserve_original_consumer() {
        OVERRIDE.with(|cell|cell.set(Override {look:Some([1.,0.,0.]),multiplier:Some([10.,0.,0.])}));
        for gate in 0..6 {
            let mut game=game();
            match gate {0=>game.window=false,1=>game.lan_acting_uuid=Some(()),2=>game.player.as_mut().unwrap().server_copy=true,3=>game.player.as_mut().unwrap().replaying=true,4=>game.player=None,_=>game.player.as_mut().unwrap().fall_flying=false}
            let mut look=[0.,0.,1.];let mut multiplier=[0.1,1.5,0.5];let mut apply=false;
            game.native_client_firework_boost(&mut look,[0.;3],&mut multiplier,&mut apply);
            if gate<5 {assert!(!apply);assert_eq!(look,[0.,0.,1.]);}
            if gate==5 {game.player.as_mut().unwrap().velocity=Vec3::new(1.,2.,3.);let mut r=Rocket::default();game.follow_own_player(&mut r);assert_eq!(r.velocity,Vec3::new(1.,2.,3.));}
        }
    }
}
