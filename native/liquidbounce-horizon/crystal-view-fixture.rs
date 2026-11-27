#![cfg(test)]
#![allow(dead_code, unused_imports)]
extern crate self as horizon_client;
mod end_crystal;
#[path="behavior.rs"] mod behavior;
mod mesher { pub const BRIGHT:u8=2; }
mod mobs {
    #[derive(Default)] pub struct Part;
    pub mod anim {
        use std::collections::HashMap;
        #[derive(Default)] pub struct Poses {pub parts:HashMap<String,[f32;9]>,pub visibility:HashMap<String,bool>}
        impl Poses {
            pub fn new(_: &super::Part)->Self{Self::default()}
            pub fn visible(&mut self,name:&str,on:bool){self.visibility.insert(name.into(),on);}
            pub fn get(&mut self,name:&str)->&mut [f32;9]{self.parts.entry(name.into()).or_insert([0.;9])}
        }
    }
    pub mod render {
        use crate::game::Vertex;
        pub fn draw(_: &super::Part,base:glam::Mat4,poses:&super::anim::Poses,
            _: [f32;4],_:u32,_:u32,_:u32,out:&mut Vec<Vertex>){
            out.push(Vertex{at:base.transform_point3(glam::Vec3::ZERO),layer:0});
            out.push(Vertex{at:base.transform_point3(glam::Vec3::ONE),layer:0});
            assert!(poses.visibility.contains_key("base"));
        }
    }
}
mod mod_api {
    pub trait Host {}
    pub trait RenderHooks:Send+Sync {}
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(|m|&**m)}
    mod generated {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}
    pub use generated::*;
    pub fn mods()->&'static [Box<dyn Hooks>]{
        static MODS:std::sync::OnceLock<Vec<Box<dyn Hooks>>>=std::sync::OnceLock::new();
        MODS.get_or_init(||vec![Box::new(CrystalModule)])
    }
    static ON:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
    pub fn running(on:bool){ON.store(on,std::sync::atomic::Ordering::Release);}
    struct CrystalModule;
    impl Hooks for CrystalModule {
        fn client_crystal_view(&self,_:&dyn Host,event:&mut ClientCrystalViewEvent){
            let [size,y,spin,bounce]=crate::behavior::CrystalView::default().parameters(ON.load(std::sync::atomic::Ordering::Acquire));
            event.size=size;event.y_translate=y;event.spin_speed=spin;event.bounce=bounce;
        }
    }
}
mod game {
    use std::collections::HashMap;
    use crate::mobs::Part;
    #[derive(Debug)] pub struct Vertex {pub at:glam::Vec3,pub layer:u32}
    #[derive(Clone,Copy)] struct Rect {u0:f32,v0:f32,u1:f32,v1:f32}
    struct Atlas {sprites:HashMap<&'static str,Rect>}
    struct Gui {atlas:Atlas}
    struct Clock;impl Clock{fn fraction(&self)->f64{0.5}}
    struct World;impl World{fn light(&self,_:i32,_:i32,_:i32)->u8{0xff}}
    struct Position {x:f64,y:f64,z:f64}
    struct Crystal {position:Position,time:i32,show_bottom:bool,beam:Option<[i32;3]>}
    struct Game<'a>{crystal_model:Option<Part>,gui:Gui,clock:Clock,world:World,crystals:Vec<Crystal>,
        beams:std::cell::RefCell<Vec<(glam::DVec3,glam::DVec3)>>,marker:std::marker::PhantomData<&'a ()>}
    impl crate::mod_api::Host for Game<'_>{}
    impl Game<'_>{
        fn native_window(&self)->bool{true}
        fn crystal_beam(&self,from:glam::DVec3,delta:glam::DVec3,_:f32,_:glam::DVec3,_:&mut Vec<Vertex>){
            self.beams.borrow_mut().push((from,delta));
        }
        __END_CRYSTAL_GEOMETRY__
    }
    mod native_hooks {use super::Game;use crate::mod_api;impl Game<'_>{__NATIVE_CRYSTAL_HOOK__}}
    #[test] fn production_renderer_translates_before_scaling_without_changing_beams_or_bounds(){
        let game=Game{crystal_model:Some(Part),gui:Gui{atlas:Atlas{sprites:HashMap::from([
            ("entity/end_crystal/end_crystal",Rect{u0:0.,v0:0.,u1:1.,v1:1.})])}},clock:Clock,world:World,
            crystals:vec![Crystal{position:Position{x:10.,y:64.,z:20.},time:20,show_bottom:true,beam:Some([3,4,5])},
                Crystal{position:Position{x:1000.,y:64.,z:20.},time:20,show_bottom:false,beam:None}],
            beams:Default::default(),marker:Default::default()};
        crate::mod_api::running(false);let mut original=Vec::new();
        game.end_crystal_geometry(glam::DVec3::ZERO,&mut original);
        let old_beam=game.beams.borrow()[0];game.beams.borrow_mut().clear();
        crate::mod_api::running(true);let mut changed=Vec::new();
        game.end_crystal_geometry(glam::DVec3::ZERO,&mut changed);
        assert_eq!(changed.len(),2);assert_eq!(original.len(),2);
        assert_eq!(original[0].at,glam::Vec3::new(10.,63.,20.));
        assert!((changed[0].at.y-63.2).abs()<1e-5);
        assert!(((changed[1].at-changed[0].at)-glam::Vec3::splat(0.6)).length()<1e-5);
        assert_eq!(game.beams.borrow()[0],old_beam);
        assert_eq!(changed[0].layer,u32::from(crate::mesher::BRIGHT)<<16);
        crate::mod_api::running(false);
    }
}
#[test] fn complete_production_model_keeps_default_wrapper_and_only_scales_animation_locals(){
    let model=mobs::Part;
    for age in [0.,0.5,20.5,10000.25] {
        let old=end_crystal::poses(&model,age,true);
        let disabled=end_crystal::poses_with(&model,age,true,1.,1.);
        assert_eq!(old.parts,disabled.parts);
        let stopped=end_crystal::poses_with(&model,age,false,0.,0.25);
        let fixed=end_crystal::poses_with(&model,0.,false,0.,0.25);
        for part in ["outer_glass","inner_glass","cube"]{
            assert_eq!(stopped.parts[part][3..6],fixed.parts[part][3..6]);
        }
        assert_eq!(stopped.parts["outer_glass"][1].to_bits(),(end_crystal::y(age)*0.25*16./2.).to_bits());
        assert!(!stopped.visibility["base"]);
        let negative=end_crystal::poses_with(&model,age,true,1.,-1.);
        assert_eq!(negative.parts["outer_glass"][1],-old.parts["outer_glass"][1]);
    }
}
