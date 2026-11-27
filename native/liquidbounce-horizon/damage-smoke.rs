#[path="../../native/liquidbounce-behaviors/src/damage.rs"] mod damage;
use damage::{Damage,Mode,Host};
#[derive(Default)] struct Game{position:[f64;3],velocity:[f64;3],sent:Vec<([f64;3],bool,bool)>,reads:usize}
impl Host for Game {
 fn position(&mut self)->[f64;3]{self.reads+=1;self.position}
 fn horizontal_collision(&mut self)->bool{self.sent.len()%2==0}
 fn send_position(&mut self,p:[f64;3],g:bool,c:bool){self.sent.push((p,g,c));self.position[0]+=1.0;}
 fn set_vertical_velocity(&mut self,v:f64){self.velocity[1]=v;}
}
#[test] fn all_modes_preserve_position_and_velocity_contracts(){
 let mut game=Game{position:[2.0,80.0,4.0],velocity:[7.0,-2.0,9.0],..Default::default()};
 Damage::default().run(&mut game);assert_eq!(game.sent.len(),131);assert_eq!(game.reads,1);
 assert_eq!(game.sent[0],([2.0,80.049,4.0],false,true));assert_eq!(game.sent[129],([2.0,80.0,4.0],false,false));assert_eq!(game.sent[130],([2.0,80.0,4.0],true,true));
 Damage{mode:Mode::Aac,damage:2}.run(&mut game);assert_eq!(game.velocity,[7.0,10.0,9.0]);
 game.sent.clear();game.reads=0;game.position=[2.0,80.0,4.0];Damage{mode:Mode::Verus,damage:20}.run(&mut game);
 assert_eq!(game.reads,4);assert_eq!(game.sent.iter().map(|v|v.0).collect::<Vec<_>>(),[[2.0,80.0,4.0],[3.0,83.25,4.0],[4.0,80.0,4.0],[5.0,80.0,4.0]]);
 game.sent.clear();Damage{mode:Mode::Ncp,damage:0}.run(&mut game);assert_eq!(game.sent.len(),1);assert!(game.sent[0].1);
 assert_eq!(Mode::parse("aac"),Some(Mode::Aac));
}
