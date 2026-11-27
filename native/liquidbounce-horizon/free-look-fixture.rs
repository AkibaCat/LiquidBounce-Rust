#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
// Existing upstream AvatarBob uses indexed parallel history arrays.
#[allow(clippy::needless_range_loop)]
mod camera;
mod camera_control;
mod free_look;
mod mouse;
use std::sync::{Mutex,atomic::{AtomicBool,AtomicUsize,Ordering}};
static RUNNING:AtomicBool=AtomicBool::new(false);
static ROTATION_LISTENERS:AtomicUsize=AtomicUsize::new(0);
static FREE_LOOK:Mutex<free_look::FreeLook>=Mutex::new(free_look::FreeLook {
    perspective:free_look::PerspectiveChoice::Back,sense_boost:1.0,no_pitch_limit:true,
    camera_yaw:0.0,camera_pitch:0.0});
mod mod_api {
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks> {
        mods.iter().map(|module|&**module)
    }
    #[allow(unused_imports)]
    mod render {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/api-source/crates/horizon-client-render/src/gfx/native_render/generated.rs"));}
    pub use render::*;
    #[allow(unused_imports)]
    mod host {include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}
    pub use host::*;
    mod hooks {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}
    pub use hooks::*;
    pub fn mods()->&'static [Box<dyn Hooks>] {
        static MODS:std::sync::OnceLock<Vec<Box<dyn Hooks>>>=std::sync::OnceLock::new();
        MODS.get_or_init(||vec![Box::new(TestHooks),Box::new(RotationManagerWitness)])
    }
    struct TestHooks;
    impl Hooks for TestHooks {
        fn client_mouse_rotation(&self,_:&dyn Host,event:&mut ClientMouseRotationEvent) {
            event.canceled|=crate::FREE_LOOK.lock().unwrap().mouse_rotation(
                crate::RUNNING.load(crate::Ordering::SeqCst),event.cursor_x,event.cursor_y);
        }
        fn client_free_look_camera(&self,_:&dyn Host,event:&mut ClientFreeLookCameraEvent<'_>) {
            if let Some(rotation)=crate::FREE_LOOK.lock().unwrap().camera_rotation(crate::RUNNING.load(crate::Ordering::SeqCst)) {
                *event.rotation=rotation;event.apply=true;
            }
        }
    }
    struct RotationManagerWitness;
    impl Hooks for RotationManagerWitness {
        fn client_mouse_rotation(&self,_:&dyn Host,_:&mut ClientMouseRotationEvent) {
            // RotationManager subscribes independently and does not skip canceled events.
            crate::ROTATION_LISTENERS.fetch_add(1,crate::Ordering::SeqCst);
        }
    }
}
mod game {
    use crate::mouse::{MouseLook,LookOptions,Rotation};
    use std::time::Instant;
    #[derive(Clone,Copy,PartialEq)] enum Screen {Playing,Inventory}
    struct Player {allowed:bool}
    impl Player {fn native_client_queries_allowed(&self)->bool {self.allowed}}
    struct Studio;
    impl Studio {fn flying(&self)->bool {false}}
    #[derive(Default)] struct Tutorial {calls:usize,last:[f64;2]}
    impl Tutorial {fn mouse(&mut self,x:f64,y:f64) {self.calls+=1;self.last=[x,y];}}
    struct Game<'a> {
        window:bool,lan_acting_uuid:Option<()>,player:Option<Player>,studio:Studio,
        started:Instant,captured:bool,agent_pointer:bool,screen:Screen,focused:bool,
        mouse:MouseLook,tutorial:Tutorial,rotation:Rotation,turns:usize,
        zoom:f64,look:LookOptions,marker:std::marker::PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default()->Self {
            Self {window:true,lan_acting_uuid:None,player:Some(Player{allowed:true}),studio:Studio,
                started:Instant::now(),captured:true,agent_pointer:false,screen:Screen::Playing,focused:true,
                mouse:MouseLook::default(),tutorial:Tutorial::default(),
                rotation:Rotation{x_rot:20.,y_rot:30.,x_rot_o:19.,y_rot_o:29.},turns:0,
                zoom:1.,look:LookOptions{sensitivity:0.5,smooth_camera:false,scoping:false,invert_x:false,invert_y:false},
                marker:std::marker::PhantomData}
        }
    }
    impl Game<'_> {
        fn native_window(&self)->bool {self.window}
        fn look_options(&self)->LookOptions {self.look}
        fn zoom_turn_scale(&self)->f64 {self.zoom}
        fn turn_player(&mut self,x:f64,y:f64) {self.turns+=1;self.rotation.turn(x,y);}
        /* POINTER */
    }
    mod native_hooks {use super::Game;use crate::mod_api;/* NATIVE */}
    mod mod_api_host {include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}
    fn reset() {
        crate::RUNNING.store(true,crate::Ordering::SeqCst);
        crate::ROTATION_LISTENERS.store(0,crate::Ordering::SeqCst);
        *crate::FREE_LOOK.lock().unwrap()=crate::free_look::FreeLook::default();
        crate::FREE_LOOK.lock().unwrap().on_enabled([30.,20.]);
    }
    #[test]
    fn actual_mouse_handler_cancels_player_turn_after_sensitivity_inversion_and_zoom() {
        reset();let mut game=Game{zoom:0.5,..Default::default()};
        game.look.invert_y=true;
        game.mouse.add_motion(20.,40.);
        let before=game.rotation;
        game.apply_mouse_look();
        assert_eq!(game.turns,0);assert_eq!(game.rotation,before);
        assert_eq!(game.tutorial.calls,1);
        assert!((game.tutorial.last[0]-20.).abs()<2e-6);
        assert!((game.tutorial.last[1]+40.).abs()<4e-6);
        assert_eq!(crate::ROTATION_LISTENERS.load(crate::Ordering::SeqCst),1);
        let free=*crate::FREE_LOOK.lock().unwrap();
        assert_eq!([free.camera_yaw,free.camera_pitch],[31.5,17.0]);
        let(mut rotation,mut apply)=([0.;2],false);
        game.native_client_free_look_camera(&mut rotation,&mut apply);
        assert!(apply);assert_eq!(rotation,[31.5,17.]);
        crate::RUNNING.store(false,crate::Ordering::SeqCst);
        game.mouse.add_motion(20.,40.);game.apply_mouse_look();
        assert_eq!(game.turns,1);assert_eq!([game.rotation.y_rot,game.rotation.x_rot],[31.5,17.]);
        let free_now=*crate::FREE_LOOK.lock().unwrap();assert_eq!(free_now,free);
    }
    #[test]
    fn non_mouse_turns_and_non_client_identities_do_not_enter_the_event() {
        reset();let mut game=Game::default();game.look_turn(10.,20.);
        assert_eq!(game.turns,1);assert_eq!(crate::ROTATION_LISTENERS.load(crate::Ordering::SeqCst),0);
        for excluded in 0..4 {
            let mut game=Game::default();
            match excluded {0=>game.window=false,1=>game.lan_acting_uuid=Some(()),
                2=>game.player.as_mut().unwrap().allowed=false,_=>game.player=None}
            let mut canceled=false;game.native_client_mouse_rotation(12.,34.,&mut canceled);
            assert!(!canceled);
        }
        assert_eq!(crate::ROTATION_LISTENERS.load(crate::Ordering::SeqCst),0);
        let mut game=Game {screen:Screen::Inventory,..Default::default()};
        game.mouse.add_motion(50.,30.);game.apply_mouse_look();assert_eq!(game.turns,0);
        assert_eq!(crate::ROTATION_LISTENERS.load(crate::Ordering::SeqCst),0);
    }
}
fn entity()->camera::CameraEntity {
    camera::CameraEntity {position:[10.,60.,20.],previous:[8.,58.,18.],x_rot:10.,y_rot:40.,
        x_rot_o:0.,y_rot_o:20.,eye_height:0.,living:None,mount:None}
}
#[test]
fn freelook_skips_smoothing_distance_callbacks_and_sleeping_tail_for_any_camera_entity() {
    use camera::{Camera,CameraType,LivingCamera};
    let mut entity=entity();entity.living=Some(LivingCamera{scale:2.,camera_distance:3.,sleeping:Some(Some(90.)),dead:None});
    entity.mount=Some((3.,4.));let mut view=Camera::default();
    view.perspective_event.update(0,Some(&entity));
    assert_eq!(view.perspective_event.distance,12.);
    let mut rays=0;
    view.align_with_entity_free_look(&entity,0.5,CameraType::FirstPerson,|_|panic!("ordinary distance hook"),
        |_|panic!("SmoothCamera must not advance"),Some([0.,0.]),|_,_|{rays+=1;None});
    assert!(view.detached);assert_eq!(rays,8);
    assert_eq!(view.eye,[9.,59.,19.]);
    assert_eq!([view.y_rot,view.x_rot],[0.,0.]);
    assert!((view.position[2]+5.).abs()<1e-5); // source distance has a second living scale: 12*2.
    assert_eq!(view.position[1],59.); // sleeping branch's +0.15 is skipped.
    let mut expected=Camera::default();expected.set_rotation(180.,-10.);expected.position=view.eye;
    expected.move_by(-24.,0.,0.);
    view.align_with_entity_free_look(&entity,0.5,CameraType::ThirdPersonFront,|_|panic!(),|_|panic!(),
        Some([180.,-10.]),|_,_|None);
    assert_eq!(view.position,expected.position);
}
#[test]
fn freelook_preserves_eight_ray_clipping_and_noclip_uses_current_scaled_distance_without_lerp() {
    use camera::{Camera,CameraType,LivingCamera};
    let mut entity=entity();entity.living=Some(LivingCamera {scale:2.,camera_distance:4.,sleeping:None,dead:None});
    let mut view=Camera::default();view.perspective_event.distance=8.;
    let mut rays=0;
    view.align_with_entity_free_look(&entity,1.,CameraType::ThirdPersonBack,|_|panic!(),|_|panic!(),Some([0.,0.]),
        |_,_|{rays+=1;Some([10.,60.,18.])});
    assert_eq!(rays,8);assert!((view.position[2]-18.).abs()<1e-5);
    view.perspective_event.no_clip=true;view.perspective_event.last_distance=2.;
    view.align_with_entity_free_look(&entity,0.25,CameraType::FirstPerson,|_|panic!(),|_|panic!(),
        Some([0.,0.]),|_,_|panic!("noClip has zero rays"));
    assert_eq!(view.eye,[8.5,58.5,18.5]);
    assert!((view.position[2]-2.5).abs()<1e-5); // 8 * scale2, rather than lerp(2,8,.25).
}
#[test]
fn disabled_freelook_retains_existing_smoothing_and_ordinary_noclip_interpolation() {
    use camera::{Camera,CameraType};
    let mut view=Camera::default();view.perspective_event.no_clip=true;
    view.perspective_event.distance=8.;view.perspective_event.last_distance=2.;
    view.align_with_entity_free_look(&entity(),0.5,CameraType::ThirdPersonBack,|_|5.,
        |position|{assert_eq!(position,[9.,59.,19.]);[40.,80.,-20.]},None,|_,_|panic!());
    assert_eq!(view.eye,[40.,80.,-20.]);
    let mut expected=Camera::default();expected.set_rotation(30.,5.);expected.position=view.eye;
    expected.move_by(-5.,0.,0.);
    assert_eq!(view.position,expected.position);
}
#[test]
fn mouse_float_narrowing_and_separate_multiplications_match_java_source_oracle() {
    /* ORACLE */
}
