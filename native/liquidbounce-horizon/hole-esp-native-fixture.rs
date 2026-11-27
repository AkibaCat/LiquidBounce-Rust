// SPDX-License-Identifier: GPL-3.0-or-later
// Declaration-only world/Host providers; the included production native bridge is unchanged.
pub mod chunk {
    #[derive(Clone)]pub struct Section(pub Vec<u32>);
    impl Section {pub fn get(&self,i:usize)->u32{self.0[i]}}
}
pub mod registry {#[derive(Clone,Copy)]pub struct BlockStateId(pub u32);}
pub mod gfx {pub mod hole_esp {
    use std::cell::RefCell;
    thread_local!{pub static FRAME:RefCell<String>=const{RefCell::new(String::new())};}
    pub fn begin_frame(s:&str){FRAME.with_borrow_mut(|v|*v=s.into());}
}}
mod native_game {
    use crate::{hole_tracker,chunk::Section,registry::BlockStateId,runtime::LiquidBounceHooks,mod_api::{TestHost,ClientHoleEspEvent}};
    use std::{collections::BTreeMap,cell::RefCell,sync::Arc};
    // @@NATIVE_MODULE@@
    struct Point{x:f64,y:f64,z:f64}struct Player{position:Point}
    struct Layout;impl Layout{fn min_y(&self)->i32{0}fn max_y(&self)->i32{16}}
    struct Chunk{sections:Vec<Arc<Section>>}impl Chunk{fn min_y(&self)->i32{0}}
    struct World{chunks:BTreeMap<(i32,i32),Chunk>}
    impl World{
        fn layout(&self)->Layout{Layout}
        fn block(&self,x:i32,y:i32,z:i32)->BlockStateId{BlockStateId(if !(0..16).contains(&y){0}else{self.chunks.get(&(x>>4,z>>4)).map_or(0,|c|c.sections[0].get((y*256+(z&15)*16+(x&15))as usize))})}
    }
    struct Fact{explosion_resistance:f32}struct Facts([Fact;4]);impl Facts{
        fn is_air(&self,id:BlockStateId)->bool{id.0==0}fn get(&self,id:BlockStateId)->&Fact{&self.0[id.0 as usize]}
    }
    struct Resources(Facts);impl Resources{fn facts(&self)->&Facts{&self.0}}
    struct Game<'a>{player:Option<Player>,world:World,res:Resources,client_hole_esp_cache:RefCell<hole_esp::Cache>,hooks:LiquidBounceHooks,marker:std::marker::PhantomData<&'a()>}
    impl Game<'_>{
        fn native_client_hole_esp(&self,input:&str,geometry:&mut String){self.hooks.hole_esp(&TestHost,&mut ClientHoleEspEvent{input,geometry});}
        fn set(&mut self,p:[i32;3],id:u32){Arc::make_mut(&mut self.world.chunks.get_mut(&(0,0)).unwrap().sections[0]).0[(p[1]*256+p[2]*16+p[0])as usize]=id;}
    }
    #[test]fn real_native_bridge_cow_fill_break_unload_disable_and_reset(){
        let blocks=crate::room([2,5,2],[1,1],hole_tracker::BLAST_RESISTANT);let mut values=vec![0;4096];for(p,class)in blocks{values[(p[1]*256+p[2]*16+p[0])as usize]=if class==hole_tracker::BLAST_RESISTANT{1}else{0};}
        let mut g=Game{player:Some(Player{position:Point{x:2.5,y:5.,z:2.5}}),world:World{chunks:BTreeMap::from([((0,0),Chunk{sections:vec![Arc::new(Section(values))]})])},res:Resources(Facts([Fact{explosion_resistance:0.},Fact{explosion_resistance:1200.},Fact{explosion_resistance:3_600_000.},Fact{explosion_resistance:6.}])),client_hole_esp_cache:Default::default(),hooks:Default::default(),marker:std::marker::PhantomData};
        g.client_hole_esp_tick();assert_eq!(g.client_hole_esp_cache.borrow().manager.tracker.holes.len(),1);g.client_hole_esp_frame();crate::gfx::hole_esp::FRAME.with_borrow(|s|assert_eq!(serde_json::from_str::<serde_json::Value>(s).unwrap()["quads"].as_array().unwrap().len(),5));
        g.set([2,7,2],3);g.client_hole_esp_tick();assert!(g.client_hole_esp_cache.borrow().manager.tracker.holes.is_empty());
        g.set([2,7,2],0);g.client_hole_esp_tick();assert_eq!(g.client_hole_esp_cache.borrow().manager.tracker.holes.len(),1);
        g.set([1,5,2],0);g.client_hole_esp_tick();assert!(g.client_hole_esp_cache.borrow().manager.tracker.holes.is_empty());g.set([1,5,2],1);g.client_hole_esp_tick();assert_eq!(g.client_hole_esp_cache.borrow().manager.tracker.holes.len(),1);
        g.hooks.state().behaviors.borrow_mut().running=false;g.client_hole_esp_frame();assert!(!g.client_hole_esp_cache.borrow().manager.active());g.hooks.state().behaviors.borrow_mut().running=true;g.client_hole_esp_tick();assert_eq!(g.client_hole_esp_cache.borrow().manager.tracker.holes.len(),1);
        g.world.chunks.clear();g.client_hole_esp_tick();assert!(g.client_hole_esp_cache.borrow().manager.tracker.holes.is_empty());g.client_hole_esp_reset();assert!(!g.client_hole_esp_cache.borrow().manager.active());
    }
}
