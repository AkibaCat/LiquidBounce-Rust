//! Executes the production Game ticker/provider with explicit tick-body storage
//! shells. The original scheduler and move/friction consumers have separate tests.
use super::{terrain_speed_bridge,terrain_speed_order,terrain_speed,Vec3};
use std::collections::HashMap;
type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;
#[derive(Clone)]struct Body{id:u32,velocity:Vec3}
struct Player{position:Vec3,velocity:Vec3}
struct Remote{entity_id:i32,entities:HashMap<i32,()>,level_players:HashMap<i32,()>,passenger_vehicles:HashMap<u32,u32>,terrain_order:terrain_speed_order::Order}
struct Game<'a>{remote:Option<Remote>,player:Option<Player>,mobs:Vec<Body>,dropped:Vec<Body>,orbs:Vec<Body>,boats:Vec<Body>,riding:Option<u32>,previous_position:Vec3,trace:Vec<String>,lifetime:std::marker::PhantomData<&'a ()>}
impl Game<'_>{
    __RESET__
    fn native_terrain_tick_entity(&mut self,id:u32){
        assert!(terrain_speed_bridge::selected(id));
        self.trace.push(format!("entity:{id}"));
        terrain_speed_bridge::friction_for(id,"minecraft:ice",0.98);
        terrain_speed_bridge::flush(&mut self.player.as_mut().unwrap().velocity);
    }
    fn native_terrain_tick_player(&mut self,_push:Vec3,_cancelled:bool)->Result<Option<u8>>{
        assert_eq!(terrain_speed_bridge::selection(),terrain_speed_order::Selection::Entity(0));
        let p=self.player.as_mut().unwrap();
        self.trace.push(format!("local:{}",p.velocity.x.to_bits()));
        let s=terrain_speed_bridge::snapshot().unwrap();let mut v=[p.velocity.x,p.velocity.y,p.velocity.z];
        s.settings.water_speed.player_tick(true,terrain_speed::WaterContext{in_water:true,physical_sprint:false,physical_jump:false,physical_sneak:false,moving:true,input:1,yaw:0.0},&mut v,|a|[horizon_core::jmath::sin(a),horizon_core::jmath::cos(a)]);
        p.velocity=Vec3::new(v[0],v[1],v[2]);self.trace.push("water:final:cancelled".into());
        Ok(Some(37))
    }
    fn native_terrain_position_player_ride(&mut self)->Result<bool>{
        self.trace.push("seat:local".into());self.player.as_mut().unwrap().position=Vec3::new(4.0,5.0,6.0);Ok(true)
    }
    fn position_riders(&mut self){self.trace.push("seat:other".into());}
}
__TICKER__
#[test]fn production_game_ticker_keeps_passenger_zero_water_final_seat_and_next_entity_order(){
    let mut settings=terrain_speed::TerrainSpeed::default();settings.ice_speed.motion_enabled=true;settings.ice_speed.horizontal_motion=0.3;
    let _scope=terrain_speed_bridge::enter(Some(terrain_speed_bridge::Snapshot{settings,forward:true,sprint:false,jump:false,sneak:false,input:1}));
    let mut order=terrain_speed_order::Order::default();order.player_added();order.added(1);order.added(2);order.set_passengers(1,&[0]);
    let mut game=Game{remote:Some(Remote{entity_id:0,entities:HashMap::from([(1,()),(2,())]),level_players:HashMap::new(),passenger_vehicles:HashMap::from([(0,1)]),terrain_order:order}),player:Some(Player{position:Vec3::new(1.,2.,3.),velocity:Vec3::new(0.3,-0.0,-0.4)}),mobs:vec![],dropped:vec![],orbs:vec![],boats:vec![],riding:Some(1),previous_position:Vec3::ZERO,trace:vec![],lifetime:std::marker::PhantomData};
    let mut ticker=TerrainEntityTicker{game:&mut game,input:None};terrain_speed_order::tick_entities(&mut ticker,true).unwrap();assert_eq!(ticker.input,Some(37));
    assert_eq!(game.trace,["entity:1","local:0","water:final:cancelled","seat:local","entity:2"]);
    assert_eq!(game.previous_position,Vec3::new(1.,2.,3.));assert_eq!(game.player.as_ref().unwrap().position,Vec3::new(4.,5.,6.));
    let mut expected=[0.;3];settings.water_speed.player_tick(true,terrain_speed::WaterContext{in_water:true,physical_sprint:false,physical_jump:false,physical_sneak:false,moving:true,input:1,yaw:0.0},&mut expected,|a|[horizon_core::jmath::sin(a),horizon_core::jmath::cos(a)]);
    settings.ice_speed.block_friction(true,"minecraft:ice",0.98,&mut expected);
    let p=game.player.as_ref().unwrap();assert_eq!([p.velocity.x,p.velocity.y,p.velocity.z].map(f64::to_bits),expected.map(f64::to_bits));
}
