// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent Heading.kt over the owning follower and block-action controllers.
use super::{geometry::{BlockPos,Vec3}, input::{Input,MovementKeys}, look_ahead::{LookAhead,ViewHost}, simulation::player_motion::MotionPlayer,
    walk_control::{block_actions::{BlockActions,PlacementOrder},body::{ControlState,WalkBody,WalkPlayer},directional_input,follower::{Follower,FollowerHost},turning::LookPurpose,view_rotation::ViewRotation},walk_jump::{vector_yaw,wrap_degrees}};
#[derive(Default)]
pub struct Heading { pub target:Option<Vec3>,pub sprint:bool,last_look_y_rot:Option<f32> }
impl Heading {
    pub fn rotation<B:ControlState,H:FollowerHost+ViewHost,O:PlacementOrder>(&self,host:&H,follower:&Follower<B>,actions:&BlockActions<O>,input:Input)->ViewRotation where H::Player:WalkPlayer<State=B> {
        if let Some(aim)=actions.rotation{return aim;}
        if let Some(view)=follower.view(){return view;}
        let yaw=self.target.map(|target|self.look_y_rot(host,follower,target));
        if let (Some(target),Some(yaw))=(self.target,yaw) && !yaw.is_nan(){
            return ViewRotation::new(yaw,if follower.movement().dive {self.dive_x_rot(host,target)}else{host.view_x_rot()});
        }
        let yaw=input_y_rot(host.view_y_rot(),input);
        ViewRotation::new(yaw,host.view_x_rot())
    }
    pub fn look_along_path<B:ControlState,H:FollowerHost+ViewHost,O:PlacementOrder>(&mut self,host:&mut H,follower:&Follower<B>,actions:&BlockActions<O>,look:&LookAhead)where H::Player:WalkPlayer<State=B>{
        if let Some(aim)=actions.rotation{self.last_look_y_rot=Some(aim.y_rot);return;}
        if let Some(view)=follower.view(){self.last_look_y_rot=Some(view.y_rot);host.turn_look(view,LookPurpose::Movement);return;}
        let yaw=self.target.map_or(self.last_look_y_rot.unwrap_or(f32::NAN),|target|self.look_y_rot(host,follower,target));
        if yaw.is_nan(){return;}
        self.last_look_y_rot=Some(yaw);
        let pitch=match self.target{
            None=>host.view_x_rot(),
            Some(target) if follower.movement().dive=>self.dive_x_rot(host,target),
            Some(target)=>match follower.next_block(host){
                Some(pos)=>self.x_rot_at(host,pos),
                None=>look.x_rot(host,follower.point_ahead(host.player().position(),f64::from(host.look_settings().distance)).or(Some(target))),
            },
        };
        host.turn_look(ViewRotation::new(yaw,pitch),LookPurpose::Movement);
    }
    fn x_rot_at(&self,host:&impl WalkBody,pos:BlockPos)->f32{
        let eye=host.player().eye_position();
        ViewRotation::from_vector(Vec3::new(f64::from(pos.x)+0.5-eye.x,f64::from(pos.y)+0.5-eye.y,f64::from(pos.z)+0.5-eye.z)).x_rot
    }
    fn look_y_rot<B:ControlState,H:FollowerHost+ViewHost>(&self,host:&H,follower:&Follower<B>,target:Vec3)->f32 where H::Player:WalkPlayer<State=B>{
        let position=host.player().position();let dx=target.x-position.x;let dz=target.z-position.z;
        let look=if dx*dx+dz*dz>=0.75*0.75 {Some(target)}else{follower.look_ahead(target,position,0.75)};
        let Some(look)=look else{return self.last_look_y_rot.unwrap_or(f32::NAN)};
        let yaw=vector_yaw(look.x-position.x,look.z-position.z);
        let backwards=follower.building()&&self.last_look_y_rot.is_some_and(|last|last.is_finite()&&wrap_degrees(yaw-last).abs()>90.0);
        if backwards{wrap_degrees(yaw+180.0)}else{yaw}
    }
    fn dive_x_rot(&self,host:&impl ViewHost,target:Vec3)->f32{let position=host.view_position();ViewRotation::from_vector(Vec3::new(target.x-position.x,target.y-position.y,target.z-position.z)).x_rot}
    pub fn walk_towards<H:WalkBody>(&mut self,host:&H,keys:&mut MovementKeys,target:Vec3,sprint:bool){
        self.target=Some(target);self.sprint=sprint;let position=host.player().position();
        keys.directional_input=directional_input(Vec3::new(target.x-position.x,target.y-position.y,target.z-position.z),host.input_y_rot());
        keys.jump=host.player().is_in_water();keys.sneak=false;
    }
    pub fn stop<H:WalkBody>(&mut self,host:&H,keys:&mut MovementKeys){self.target=None;self.sprint=false;keys.directional_input=Input::EMPTY;keys.jump=host.player().is_in_water();keys.sneak=false;}
    pub fn forget_look(&mut self){self.last_look_y_rot=None;}
    pub fn clear(&mut self){self.target=None;self.sprint=false;self.forget_look();}
}
fn input_y_rot(yaw:f32,input:Input)->f32{
    let forward=input.forward&&!input.backward;let backward=input.backward&&!input.forward;
    let sideways=if backward{-45.0}else if forward{45.0}else{90.0};
    let mut yaw=if backward{yaw+180.0}else{yaw};
    if input.left&&!input.right{yaw-=sideways;}
    if input.right&&!input.left{yaw+=sideways;}
    yaw
}
