// SPDX-License-Identifier: GPL-3.0-or-later
//! Live epoch-scoped vehicle handles; route workers only see copied Vehicle facts.
use super::*;
use horizon_protocol::packets::play::{self,Serverbound as S};
use super::{auto_walk_host::NativeHost,auto_walk_player::{NativeInventory,box_},auto_walk_world::{self as world,NativeLevel,NativeState}};
use crate::navigation as nav;
use nav::{aabb::Aabb as NavBox,geometry::{BlockPos as NavPos,Vec3 as NavVec},ride::{body::{RideBody,RideEntity,RideLook},boating::{BoatBody,BoatEntity,BoatFluid,BoatLevel},vehicle::{EntityKind,VehicleEntity}},terrain::TerrainLevel,walk_control::{turning::LookPurpose,view_rotation::ViewRotation}};

#[derive(Clone)]
pub(super) struct NativeEntity{pub level:NativeLevel,pub id:Option<u32>}
#[derive(Clone,Copy)]
struct Data{at:NavVec,old:NavVec,motion:NavVec,yaw:f32,pitch:f32,width:f32,height:f32,eye:f32,step:f32,jump:f64,boost:f32,safe_fall:f64,speed:f64,can_jump:bool,on_ground:bool,water:bool,alive:bool,passenger:Option<u32>,kind:EntityKind,bounds:NavBox}
impl NativeEntity{
 pub fn player(level:NativeLevel)->Self{Self{level,id:None}}
 fn data(&self)->Option<Data>{self.level.game(|g|{
  if let Some(id)=self.id{
   if let Some(b)=g.boats.iter().find(|b|b.id==id){let bounds=box_(b.bounds());return Some(Data{at:world::vec(b.position),old:world::vec(b.previous),motion:world::vec(b.velocity),yaw:b.yaw,pitch:b.pitch,width:(bounds.max.x-bounds.min.x) as f32,height:(bounds.max.y-bounds.min.y) as f32,eye:((bounds.max.y-bounds.min.y)*0.85) as f32,step:0.,jump:0.,boost:0.,safe_fall:3.,speed:0.,can_jump:false,on_ground:b.on_ground,water:b.under_water(),alive:true,passenger:None,kind:EntityKind::Boat,bounds});}
   if let Some(m)=g.mobs.iter().find(|m|m.id==id){let kind=match &*m.kind.id{"minecraft:horse"|"minecraft:donkey"|"minecraft:mule"|"minecraft:skeleton_horse"|"minecraft:zombie_horse"=>EntityKind::Horse,"minecraft:camel"|"minecraft:camel_husk"=>EntityKind::Camel,_=>EntityKind::Other};return Some(Data{at:world::vec(m.position),old:world::vec(m.previous),motion:world::vec(m.velocity),yaw:m.yaw,pitch:m.head_pitch,width:m.width(),height:m.height(),eye:m.eye_height() as f32,step:m.kind.step_height as f32,jump:m.attrs.jump_strength,boost:m.jump_boost_power(),safe_fall:m.kind.safe_fall_distance,speed:m.movement_speed(),can_jump:m.is_saddled()&&matches!(kind,EntityKind::Horse|EntityKind::Camel),on_ground:m.on_ground,water:m.fluids.in_water,alive:!m.removed&&m.health>0.,passenger:m.extra.mount,kind,bounds:box_(m.bounds())});}
   if let Some(item)=g.dropped.iter().find(|item|item.id==id){return Some(Data{at:world::vec(item.position),old:world::vec(item.previous),motion:world::vec(item.velocity),yaw:item.rotation[0],pitch:item.rotation[1],width:0.25,height:0.25,eye:0.2125,step:0.,jump:0.,boost:0.,safe_fall:3.,speed:0.,can_jump:false,on_ground:item.on_ground,water:item.was_touching_water,alive:item.health>0&&item.stack.count>0,passenger:None,kind:EntityKind::Other,bounds:box_(item.bounds())});}
   return None;
  }
  let p=g.player.as_ref()?;let attr=|id|p.attributes.computed_value_of(id).expect("bound player navigation attribute");Some(Data{at:world::vec(p.position),old:world::vec(p.old_position),motion:world::vec(p.velocity),yaw:g.yaw,pitch:g.pitch,width:p.dimensions.width,height:p.dimensions.height,eye:p.dimensions.eye_height,step:attr("minecraft:step_height") as f32,jump:attr("minecraft:jump_strength"),boost:p.status.jump_boost.map_or(0.,|v|(v+1) as f32*0.1),safe_fall:attr("minecraft:safe_fall_distance"),speed:attr("minecraft:movement_speed"),can_jump:false,on_ground:p.ground.on_ground,water:p.fluids.was_touching_water,alive:p.health.health()>0.,passenger:g.riding,kind:EntityKind::Other,bounds:box_(p.bounds())})
 })}
 fn at(&self)->NavVec{self.data().map_or(NavVec::ZERO,|d|d.at)}
}
impl VehicleEntity for NativeEntity{
 fn entity_kind(&self)->EntityKind{self.data().map_or(EntityKind::Other,|d|d.kind)}
 fn passenger_riding_position(&self,_rider:&Self)->NavVec{self.level.game(|g|{let Some(id)=self.id else{return world::vec(g.player.as_ref().expect("navigation player").position);};if let Some(b)=g.boats.iter().find(|b|b.id==id){let index=g.remote.as_ref().map_or(0,|r|r.passenger_index);let count=usize::from(g.riding==Some(id))+g.mobs.iter().filter(|m|m.extra.mount==Some(id)).count();return world::vec(b.seat(index,count,false,f64::from(0.6_f32),false));}if let Some(m)=g.mobs.iter().find(|m|m.id==id){let attachment=if horizon_client::mobs::camel::is_camel(m.species()){horizon_client::mobs::camel::passenger_attachment(m)}else{horizon_client::mobs::riders::passenger_attachment(m)};return world::vec(horizon_client::mobs::riders::seat(attachment,f64::from(0.6_f32),m.position,m.yaw));}NavVec::ZERO})}
 fn y(&self)->f64{self.at().y}fn bounding_height(&self)->f32{self.data().map_or(0.,|d|d.height)}fn bounding_width(&self)->f32{self.data().map_or(0.,|d|d.width)}fn eye_height(&self)->f32{self.data().map_or(0.,|d|d.eye)}fn max_up_step(&self)->f32{self.data().map_or(0.,|d|d.step)}fn can_jump(&self)->bool{self.data().is_some_and(|d|d.can_jump)}fn jump_strength(&self)->f64{self.data().map_or(0.,|d|d.jump)}fn jump_boost_power(&self)->f32{self.data().map_or(0.,|d|d.boost)}fn safe_fall_distance(&self)->f64{self.data().map_or(0.,|d|d.safe_fall)}fn movement_speed(&self)->f64{self.data().map_or(0.,|d|d.speed)}
}
impl RideEntity for NativeEntity{
 fn position(&self)->NavVec{self.at()}fn interpolated_position(&self,a:f32)->NavVec{self.data().map_or(NavVec::ZERO,|d|NavVec::new(d.old.x+(d.at.x-d.old.x)*f64::from(a),d.old.y+(d.at.y-d.old.y)*f64::from(a),d.old.z+(d.at.z-d.old.z)*f64::from(a)))}fn delta_movement(&self)->NavVec{self.data().map_or(NavVec::ZERO,|d|d.motion)}fn x(&self)->f64{self.at().x}fn z(&self)->f64{self.at().z}fn y_rot(&self)->f32{self.data().map_or(0.,|d|d.yaw)}fn x_rot(&self)->f32{self.data().map_or(0.,|d|d.pitch)}fn is_passenger(&self)->bool{self.data().is_some_and(|d|d.passenger.is_some())}
 fn controlled_vehicle(&self)->Option<Self>{let id=self.data()?.passenger?;let controlled=self.level.game(|g|{if g.boats.iter().any(|b|b.id==id){return g.remote.as_ref().is_none_or(|r|r.passenger_index==0);}g.mobs.iter().any(|m|m.id==id&&m.is_saddled())});controlled.then(||Self{level:self.level.clone(),id:Some(id)})}
 fn bounding_box(&self)->NavBox{self.data().map_or_else(||NavBox::new(NavVec::ZERO,NavVec::ZERO),|d|d.bounds)}
}
impl BoatEntity for NativeEntity{
 type Inventory=NativeInventory;
 fn on_ground(&self)->bool{self.data().is_some_and(|d|d.on_ground)}fn is_in_water(&self)->bool{self.data().is_some_and(|d|d.water)}fn eye_position(&self)->NavVec{self.data().map_or(NavVec::ZERO,|d|NavVec::new(d.at.x,d.at.y+f64::from(d.eye),d.at.z))}
 fn block_interaction_range(&self)->f64{self.level.game(|g|g.block_reach)}fn entity_interaction_range(&self)->f64{self.level.game(|g|g.entity_reach)}fn inventory(&self)->NativeInventory{NativeInventory{level:self.level.clone()}}fn main_hand_is_boat(&self)->bool{self.inventory().stack(self.level.game(|g|g.selected as i32)).boat()}
 fn passengers_empty(&self)->bool{self.level.game(|g|self.id.is_none_or(|id|g.riding!=Some(id)&&!g.mobs.iter().any(|m|m.extra.mount==Some(id))))}fn is_alive(&self)->bool{self.data().is_some_and(|d|d.alive)}fn distance_to(&self,other:&Self)->f32{self.at().distance_sqr(other.at()).sqrt() as f32}
 fn attack_strength_scale(&self,partial:f32)->f32{self.level.game(|g|if g.legacy_rules(){1.}else{g.hand_strength.scale(partial,g.main_hand_attack_speed())})}
 fn item_is_boat(&self)->bool{self.level.game(|g|self.id.and_then(|id|g.dropped.iter().find(|item|item.id==id)).is_some_and(|item|g.res.items().item_in_tag("minecraft:boats",&item.stack.item)))}
}
#[derive(Clone)]pub(super) struct NativeFluid{water:bool,own_height:f32}
impl BoatFluid for NativeFluid{fn is_water(&self)->bool{self.water}fn height<B:nav::ride::boating::BoatState,L:BoatLevel<B>>(&self,level:&L,pos:NavPos)->f32{if self.water&&level.block_state(pos.above()).is_water(){1.}else{self.own_height}}}
impl BoatLevel<NativeState> for NativeLevel{
 type Entity=NativeEntity;type Fluid=NativeFluid;
 fn fluid_state(&self,pos:NavPos)->NativeFluid{let state=self.block_state(pos);NativeFluid{water:state.facts().fluid==Fluid::Water,own_height:f32::from(state.facts().fluid_level)/9.}}
 fn boats(&self,bounds:NavBox)->Vec<NativeEntity>{let ids=self.game(|g|g.boats.iter().filter(|b|bounds.intersects(box_(b.bounds()))).map(|b|b.id).collect::<Vec<_>>());ids.into_iter().map(|id|NativeEntity{level:self.clone(),id:Some(id)}).collect()}
 fn items(&self,bounds:NavBox)->Vec<NativeEntity>{let ids=self.game(|g|g.dropped.iter().filter(|b|bounds.intersects(box_(b.bounds()))).map(|b|b.id).collect::<Vec<_>>());ids.into_iter().map(|id|NativeEntity{level:self.clone(),id:Some(id)}).collect()}
 fn boat_clip(&self,eye:NavVec,end:NavVec,_player:&NativeEntity)->Option<NavPos>{self.game(|g|{let delta=NavVec::new(end.x-eye.x,end.y-eye.y,end.z-eye.z);let length=(delta.x*delta.x+delta.y*delta.y+delta.z*delta.z).sqrt();if length==0.{return None;}g.clip_ray_fluids([eye.x,eye.y,eye.z],[delta.x/length,delta.y/length,delta.z/length],length,true).map(|h|NavPos::new(h.pos[0],h.pos[1],h.pos[2]))})}
}
impl RideBody for NativeHost{type Entity=NativeEntity;type State=NativeState;type Level=NativeLevel;fn player(&self)->NativeEntity{NativeEntity::player(self.level())}fn level(&self)->NativeLevel{self.level()}fn press(&mut self,input:nav::input::Input,exact:bool){self.context.press(input,exact)}fn look(&mut self,yaw:f32,pitch:f32,purpose:RideLook){self.turn(ViewRotation::new(yaw,pitch),match purpose{RideLook::View=>LookPurpose::View,RideLook::Aim=>LookPurpose::Aim})}}
impl BoatBody for NativeHost{
 fn can_boat(&self)->bool{self.options.can_boat}fn can_click_inventory(&self)->bool{self.can_click()}fn should_stand(&mut self){self.context.set_should_stand(true)}fn select_hotbar_slot(&mut self,slot:i32){self.selected(slot)}fn release_hotbar_slot(&mut self){self.release()}fn swap_with_hotbar(&mut self,a:i32,b:i32){self.swap(a,b)}fn sent_view_vector(&self)->NavVec{self.sent.view_vector()}fn use_main_hand(&mut self)->bool{self.use_hand(nav::walk_control::block_targets::Hand::Main)}
 fn sees(&self,entity:&NativeEntity)->bool{let eye=self.player.eye();let to=entity.eye_position();let hit=self.trace(eye,to,false);!hit.is_block}
 fn interact_main_hand(&mut self,entity:&NativeEntity){let Some(id)=entity.id else{return;};let eye=self.player.eye();let direction=self.sent.view_vector();let range=entity.entity_interaction_range();let end=NavVec::new(eye.x+direction.x*range,eye.y+direction.y*range,eye.z+direction.z*range);let bounds=entity.bounding_box();let point=bounds.clip(eye,end).unwrap_or_else(||bounds.center());let at=entity.position();world::write(|g|{g.remote_send_carried();let sneaking=g.player.as_ref().is_some_and(|p|p.shift_key_down);g.send(S::Interact(play::Interact{id:id as i32,hand:0,location:[point.x-at.x,point.y-at.y,point.z-at.z],secondary:sneaking}));if g.boats.iter().enumerate().any(|(i,b)|b.id==id&&b.out_of_control<60.&&g.boat_has_room(i))&&!sneaking{g.with_hand(Hand::Main,Game::swing_arm);}});}
 fn attack(&mut self,entity:&NativeEntity){let Some(id)=entity.id else{return;};world::write(|g|{if !g.client_weapon_before_attack(id,false){return;}g.remote_send_carried();g.send(S::Attack(play::Attack{id:id as i32}));g.reset_attack_strength();g.with_hand(Hand::Main,Game::swing_arm);});}
}
