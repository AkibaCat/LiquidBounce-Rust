#![cfg(test)]
#![allow(dead_code, unused_imports)]
extern crate self as horizon_client;
#[allow(clippy::needless_range_loop)] mod camera;
mod camera_control;
mod easing;
mod numeric;
pub use easing::Easing;
mod camera_clip;
mod dank_bobbing;
mod camera_behavior;
mod auto_f5;
#[path="vanilla_key_names.rs"] mod key_names;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum KeyCode { ControlLeft, ControlRight, KeyA, F5 }
mod keybinds {pub fn key_name(key:crate::KeyCode)->Option<&'static str>{Some(match key{
    crate::KeyCode::ControlLeft=>"key.keyboard.left.control",crate::KeyCode::ControlRight=>"key.keyboard.right.control",
    crate::KeyCode::KeyA=>"key.keyboard.a",crate::KeyCode::F5=>"key.keyboard.f5"})}}

#[derive(Default)] struct Modules {
    clip:camera_clip::CameraClip, clip_running:bool,
    auto:auto_f5::AutoF5, auto_running:bool,
    quick:camera_behavior::QuickPerspectiveSwap, quick_running:bool,
    free_look:bool, free_cam:bool,
    screen:Option<(Option<String>,String,Vec<String>)>,
    dank:dank_bobbing::DankBobbing,dank_running:bool,
    no_bob:bool,no_hurt:bool,perspective_ticks:usize,remembered:usize,
}
fn modules()->std::sync::MutexGuard<'static,Modules>{
    static STATE:std::sync::OnceLock<std::sync::Mutex<Modules>>=std::sync::OnceLock::new();
    STATE.get_or_init(||std::sync::Mutex::new(Modules::default())).lock().unwrap()
}
#[allow(clippy::too_many_arguments)] mod mod_api {
    pub fn entered<'a>(mods:&'a[Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(|m|&**m)}
    mod render{include!(concat!(env!("CARGO_MANIFEST_DIR"),"/api-source/crates/horizon-client-render/src/gfx/native_render/generated.rs"));}
    pub use render::*;
    mod host{include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}pub use host::*;
    mod hooks{include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}pub use hooks::*;
    pub fn mods()->&'static[Box<dyn Hooks>]{static MODS:std::sync::OnceLock<Vec<Box<dyn Hooks>>>=std::sync::OnceLock::new();MODS.get_or_init(||vec![Box::new(TestHooks)])}
    struct TestHooks;
    impl Hooks for TestHooks {
        fn client_camera_tick(&self,_:&dyn Host,event:&mut ClientCameraTickEvent){
            let mut m=crate::modules();m.perspective_ticks+=1;
            m.clip.perspective(m.clip_running,event.last_perspective,event.perspective,
                &mut event.last_distance,&mut event.distance,&mut event.no_clip);
            event.perspective=m.auto.perspective(m.auto_running,event.perspective,
                m.screen.as_ref().map(|(kind,title,tags)|(kind.as_deref(),title.as_str(),tags.as_slice())));
            event.perspective=m.quick.perspective(m.quick_running,m.free_look,event.managed_camera||m.free_cam,
                crate::camera_behavior::Perspective::from_code(event.perspective).unwrap()) as u8;
            let running=m.clip_running;
            if m.clip.final_perspective(running,event.perspective){m.remembered+=1;}
        }
        fn client_mouse_scroll_raw(&self,host:&dyn Host,event:&mut ClientMouseScrollRawEvent){
            let mut m=crate::modules();let running=m.clip_running;let free=m.free_look;
            let down=host.client_keyboard_scan_down(m.clip.modifier.value);
            m.clip.scroll(running,down,host.raw_camera_perspective(),free,event.vertical);
        }
        fn client_camera_perspective(&self,_:&dyn Host,_:&mut ClientCameraPerspectiveEvent){
            panic!("obsolete per-query dispatch must never mutate PerspectiveEvent");
        }
        fn client_camera_aspect(&self,_:&dyn Host,_:&mut ClientCameraAspectEvent){}
        fn client_camera_position(&self,_:&dyn Host,_:&mut ClientCameraPositionEvent<'_>){}
        fn client_dank_bobbing(&self,_:&dyn Host,event:&mut ClientDankBobbingEvent){
            let m=crate::modules();if let Some(value)=m.dank.motion(m.dank_running){event.enabled=true;event.motion=value;}
        }
        fn camera_bob_effects(&self,_:&dyn Host,event:&mut CameraBobEffectsEvent){
            let m=crate::modules();event.view &= !m.no_bob;event.hurt &= !m.no_hurt;
        }
    }
}

mod game {
    use crate::{camera::{self,CameraEntity,LivingCamera},KeyCode};
    use std::collections::{BTreeMap,HashSet};
    pub mod perspective {
        use super::Game;
        #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)] pub enum CameraType{#[default]FirstPerson,ThirdPersonBack,ThirdPersonFront}
        impl CameraType{pub fn first_person(self)->bool{self==Self::FirstPerson}}
        impl Game<'_>{__TICK__}
    }
    #[derive(Default)] struct Studio(bool);impl Studio{fn flying(&self)->bool{self.0}}
    #[derive(Default)] struct Kind{id:String,attributes:BTreeMap<String,f64>}
    #[derive(Default)] struct Extra{mount:Option<u32>,avatar:camera::AvatarBob}
    #[derive(Default)] struct Mob{id:u32,kind:Kind,extra:Extra,position:horizon_core::geometry::Vec3,
        previous:horizon_core::geometry::Vec3,head_pitch:f32,head_yaw:f32,previous_head_pitch:f32,previous_head_yaw:f32}
    impl Mob{fn eye_height(&self)->f64{1.62}}
    #[derive(Default)] struct ArmorStand{id:u32,marker:bool,vehicle:Option<u32>,position:horizon_core::geometry::Vec3,
        previous:horizon_core::geometry::Vec3,pitch:f32,yaw:f32,previous_yaw:f32}
    impl ArmorStand{fn probe_dimensions(&self)->(f64,f64,f64){(0.6,1.8,1.7775)}}
    #[derive(Default)] struct Hurt{hurt_duration:i32}
    #[derive(Default)] struct Health{hurt:Hurt,hurt_animation_time:i32,hurt_dir:f32}
    #[derive(Default)] struct Player{health:Health}
    struct File;impl File{fn double(&self,_:&str)->f64{0.73}}
    struct Settings{file:File,view_bobbing:bool}
    struct Game<'a>{camera_type:perspective::CameraType,studio:Studio,view:camera::Camera,
        physical_keys:HashSet<KeyCode>,native:bool,local:Option<CameraEntity>,
        remote_camera:Option<u32>,mobs:Vec<Mob>,armor_stands:Vec<ArmorStand>,
        avatar:camera::AvatarBob,player:Option<Player>,settings:Settings,marker:std::marker::PhantomData<&'a()>}
    impl Default for Game<'_>{fn default()->Self{Self{camera_type:Default::default(),studio:Default::default(),
        view:Default::default(),physical_keys:Default::default(),native:true,local:Some(entity()),remote_camera:None,
        mobs:vec![],armor_stands:vec![],avatar:Default::default(),player:Some(Player::default()),
        settings:Settings{file:File,view_bobbing:true},marker:std::marker::PhantomData}}}
    fn entity()->CameraEntity{CameraEntity{position:[0.;3],previous:[0.;3],x_rot:0.,y_rot:0.,x_rot_o:0.,y_rot_o:0.,eye_height:1.62,
        living:Some(LivingCamera{scale:1.,camera_distance:4.,sleeping:None,dead:None}),mount:None}}
    impl Game<'_>{
        fn native_window(&self)->bool{self.native}
        fn player_camera_entity(&mut self)->Option<CameraEntity>{self.local}
        fn remote_camera_id(&self)->Option<u32>{self.remote_camera}
        fn camera_mob(&self)->Option<&Mob>{let id=self.remote_camera_id()?;self.mobs.iter().find(|m|m.id==id)}
        fn death_time(&self)->Option<i32>{None}
        fn native_shake_matrix(&self,_:f32)->glam::Mat4{glam::Mat4::IDENTITY}
    }
    mod remote_camera{use super::*;__REMOTE_CAMERA__ impl Game<'_>{__CAMERA_ENTITY__}}
    #[allow(clippy::too_many_arguments)] mod native_hooks{use super::*;use crate::mod_api;impl Game<'_>{__NATIVE__}}
    mod fov{use super::Game;use crate::camera::{BobState,HurtBob,bob_matrix};impl Game<'_>{__PLAYER_BOB__ __BOB_MATRIX__}}
    mod mod_api_host{include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}

    #[test] fn singleton_updates_once_per_tick_then_all_visual_queries_observe_the_same_final_state(){
        *crate::modules()=crate::Modules{clip_running:true,quick_running:true,..Default::default()};
        let mut g=Game::default();g.tick_native_camera_perspective();
        assert_eq!(g.view.perspective_event.perspective,2);assert_eq!(g.view.perspective_event.distance,4.);
        for _ in 0..100{assert_eq!(g.effective_camera_type(),perspective::CameraType::ThirdPersonFront);}
        assert_eq!(crate::modules().perspective_ticks,1);assert_eq!(g.camera_type,perspective::CameraType::FirstPerson);
        g.camera_type=perspective::CameraType::ThirdPersonBack;
        assert_eq!(g.effective_camera_type(),perspective::CameraType::ThirdPersonFront);
        g.tick_native_camera_perspective();assert_eq!(crate::modules().perspective_ticks,2);
        // Previous FINAL front versus current RAW back resets lastDistance before Quick.
        assert_eq!(g.view.perspective_event.last_perspective,2);assert_eq!(g.view.perspective_event.last_distance,0.);
        assert_eq!(g.view.perspective_event.distance,2.4);assert_eq!(g.view.perspective_event.perspective,2);
        crate::modules().quick_running=false;g.tick_native_camera_perspective();
        assert_eq!(g.effective_camera_type(),perspective::CameraType::ThirdPersonBack);
        assert_eq!(g.native_raw_camera_perspective(),1);
    }
    #[test] fn auto_f5_then_quick_order_and_read_final_reset_survive_raw_first_person(){
        *crate::modules()=crate::Modules{clip_running:true,auto_running:true,quick_running:true,
            screen:Some((Some("minecraft:generic_9x3".into()),"箱子".into(),vec!["Chest".into()])),..Default::default()};
        let mut g=Game::default();{let mut m=crate::modules();m.clip.remember_scrolled=true;m.clip.scrolled_distance=9.;}
        g.tick_native_camera_perspective();assert_eq!(g.view.perspective_event.perspective,2);
        assert_eq!(crate::modules().clip.camera_distance,4.);assert_eq!(g.view.perspective_event.distance,4.);
        crate::modules().screen=None;crate::modules().quick_running=false;g.tick_native_camera_perspective();
        assert_eq!(g.view.perspective_event.perspective,0);assert_eq!(crate::modules().clip.camera_distance,9.);
        assert_eq!(crate::modules().remembered,1);
        // First-person raw options deny wheel even when AutoF5 makes the final view detached.
        crate::modules().auto_running=true;g.physical_keys.insert(KeyCode::ControlLeft);
        g.native_client_mouse_scroll_raw(20.);assert_eq!(crate::modules().clip.scrolled_distance,9.);
    }
    #[test] fn raw_scroll_uses_physical_modifier_and_fractional_vertical_before_hotbar_scaling(){
        *crate::modules()=crate::Modules{clip_running:true,..Default::default()};
        let mut g=Game{camera_type:perspective::CameraType::ThirdPersonBack,..Default::default()};
        g.native_client_mouse_scroll_raw(2.5);assert_eq!(crate::modules().clip.scrolled_distance,4.);
        g.physical_keys.insert(KeyCode::ControlRight);g.native_client_mouse_scroll_raw(2.5);
        assert_eq!(crate::modules().clip.scrolled_distance,4.);
        g.physical_keys.insert(KeyCode::ControlLeft);g.native_client_mouse_scroll_raw(2.5);
        assert_eq!(crate::modules().clip.scrolled_distance,4.75);
        assert!(crate::mod_api::Host::client_keyboard_scan_down(&g,224));
        assert!(!crate::mod_api::Host::client_keyboard_scan_down(&g,341));
        let mut m=crate::modules();assert!(m.clip.can_scroll(true,true,1,false));
        m.clip.keyboard_release(true,224);assert_eq!(m.clip.scrolled_distance,4.);
    }
    #[test] fn actual_spectator_helpers_keep_living_mounts_and_choose_only_player_avatar_history(){
        *crate::modules()=Default::default();let mut g=Game::default();
        g.mobs.push(Mob{id:1,kind:Kind{id:"minecraft:player".into(),attributes:BTreeMap::from([
            ("minecraft:scale".into(),3.),("minecraft:camera_distance".into(),2.)])},
            extra:Extra{mount:Some(2),avatar:camera::AvatarBob{walk_dist:5.,walk_dist_o:4.,bob:0.2,bob_o:0.1,..Default::default()}},..Default::default()});
        g.mobs.push(Mob{id:2,kind:Kind{id:"minecraft:pig".into(),attributes:BTreeMap::from([
            ("minecraft:scale".into(),1.),("minecraft:camera_distance".into(),8.)])},..Default::default()});
        g.remote_camera=Some(1);let e=g.camera_entity().unwrap();assert_eq!(e.mount,Some((1.,8.)));
        assert_eq!(camera::perspective_distance(Some(&e)),24.);assert_eq!(g.camera_player_bob(0.5),Some((-5.5,0.15)));
        g.mobs[0].kind.id="minecraft:mannequin".into();assert!(g.camera_player_bob(0.5).is_none());
        g.remote_camera=Some(3);g.armor_stands.push(ArmorStand{id:3,vehicle:Some(2),..Default::default()});
        assert_eq!(g.camera_entity().unwrap().mount,Some((1.,8.)));assert!(g.camera_player_bob(0.5).is_none());
    }
    #[test] fn dank_and_cancellation_use_the_real_shared_bob_consumer_and_remote_player_history(){
        *crate::modules()=crate::Modules{dank_running:true,no_hurt:true,..Default::default()};
        let mut g=Game{avatar:camera::AvatarBob{walk_dist:2.,walk_dist_o:1.,bob:0.1,bob_o:0.08,..Default::default()},..Default::default()};
        let state=camera::BobState{living:None,player:g.camera_player_bob(0.25)};
        assert_eq!(g.bob_matrix(0.25).to_cols_array(),camera::bob_matrix_dank(&state,0.73,true,Some(5.)).m);
        crate::modules().no_bob=true;assert_eq!(g.bob_matrix(0.25),glam::Mat4::IDENTITY);
        crate::modules().no_bob=false;g.settings.view_bobbing=false;assert_eq!(g.bob_matrix(0.25),glam::Mat4::IDENTITY);
        g.settings.view_bobbing=true;g.mobs.push(Mob{id:3,kind:Kind{id:"minecraft:creeper".into(),..Default::default()},..Default::default()});
        g.remote_camera=Some(3);assert_eq!(g.bob_matrix(0.25),glam::Mat4::IDENTITY);
        g.mobs[0].kind.id="minecraft:player".into();g.mobs[0].extra.avatar=camera::AvatarBob{walk_dist:12.,walk_dist_o:11.,bob:0.4,bob_o:0.3,..Default::default()};
        let remote=camera::BobState{living:None,player:g.camera_player_bob(0.25)};
        assert_eq!(g.bob_matrix(0.25).to_cols_array(),camera::bob_matrix_dank(&remote,0.73,true,Some(5.)).m);
    }
}

#[test] fn camera_clip_skips_all_collision_rays_and_interpolates_the_tick_cache_for_every_frame(){
    let entity=game_entity();let mut c=camera::Camera::default();let mut rays=0;
    c.perspective_event=camera::PerspectiveSnapshot{no_clip:true,last_distance:2.,distance:8.,..Default::default()};
    for partial in [0.,0.25,0.5,1.] {c.align_with_entity_position(&entity,partial,camera::CameraType::ThirdPersonBack,
        |_|0.5,|p|p,|_,_|{rays+=1;Some([0.,0.,-0.1])});
        assert!((c.position[2]+f64::from(2.+partial*6.)).abs()<1e-5);}
    assert_eq!(rays,0);
    c.perspective_event.no_clip=false;c.align_with_entity_position(&entity,1.,camera::CameraType::ThirdPersonBack,
        |d|d,|p|p,|_,_|{rays+=1;Some([0.,0.,-1.])});assert_eq!(rays,8);
    assert!((c.position[2]+1.).abs()<0.2);
    c.perspective_event=camera::PerspectiveSnapshot{no_clip:true,last_distance:-5.,distance:-5.,..Default::default()};
    c.align_with_entity_position(&entity,1.,camera::CameraType::ThirdPersonBack,|d|d,|p|p,|_,_|panic!());
    assert!((c.position[2]-5.).abs()<1e-5);
    assert_eq!(camera::PerspectiveSnapshot{distance:-0.,last_distance:0.,..Default::default()}.render_distance(0.5).to_bits(),(-0f32).to_bits());
}
fn game_entity()->camera::CameraEntity{camera::CameraEntity{position:[0.;3],previous:[0.;3],x_rot:0.,y_rot:0.,x_rot_o:0.,y_rot_o:0.,eye_height:1.62,living:None,mount:None}}

#[test] fn actual_26_3_mth_and_pose_stack_match_all_animation_and_dank_matrix_oracle_rows(){__ORACLE_ASSERTIONS__}
