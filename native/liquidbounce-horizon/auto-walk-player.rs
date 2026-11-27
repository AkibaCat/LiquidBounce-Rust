// SPDX-License-Identifier: GPL-3.0-or-later
//! Epoch-scoped player/inventory handles. Only copied stacks enter workers.
use super::*;
use super::auto_walk_world::{self as world,NativeLevel,NativeState,NavigationRegistry};
use crate::navigation as nav;
use nav::{aabb::Aabb as NavBox,geometry::{BlockPos as NavPos,Vec3 as NavVec},terrain::TerrainLevel};
use std::{cell::RefCell,rc::Rc,sync::Arc};

pub(super) fn box_(value:Aabb)->NavBox{NavBox::new(world::vec(value.min_position()),world::vec(value.max_position()))}
pub(super) fn core_box(value:NavBox)->Aabb{Aabb::new(world::core_vec(value.min),world::core_vec(value.max))}
pub(super) fn nav_pos(value:horizon_core::position::BlockPos)->NavPos{NavPos::new(value.x,value.y,value.z)}
#[derive(Clone)]
pub(super) struct NativeInventory{pub level:NativeLevel}
impl NativeInventory{
 pub fn stack(&self,slot:i32)->NativeStack{world::read(|g|{assert_eq!(world::current_epoch(g),self.level.epoch);let stack=match slot{40=>g.offhand.clone(),38=>g.armor.get(2).cloned().flatten(),0..=35=>g.inventory.get(slot as usize).cloned().flatten(),_=>None};NativeStack::capture(g,self.level.registry.clone(),stack)})}
 pub fn suitable_slot(&self)->i32{world::read(|g|{(0..9).map(|i|(g.selected+i)%9).find(|i|g.inventory[*i].as_ref().is_none_or(|s|s.count==0)).unwrap_or(g.selected) as i32})}
}
#[derive(Clone)]
pub(super) struct NativeStack{pub stack:Option<ItemStack>,pub registry:Option<Arc<NavigationRegistry>>,pub components:serde_json::Value}
impl NativeStack{
 pub fn capture(g:&Game<'_>,registry:Arc<NavigationRegistry>,stack:Option<ItemStack>)->Self{let components=stack.as_ref().map(|s|{let view=horizon_client::item_render::stack_view(s);serde_json::Value::Object(g.pack.item_renderer.models.components_view(&view).to_map().into_iter().collect())}).unwrap_or(serde_json::Value::Null);Self{stack,registry:Some(registry),components}}
 pub fn empty()->Self{Self{stack:None,registry:None,components:serde_json::Value::Null}}
 pub fn item(&self)->&str{self.stack.as_ref().map_or("minecraft:air",|s|&*s.item)}
 pub fn count(&self)->i32{self.stack.as_ref().map_or(0,|s|s.count.min(i32::MAX as u32) as i32)}
 pub fn efficiency(&self)->i32{self.stack.as_ref().map_or(0,|s|s.enchantment_level("minecraft:efficiency") as i32)}
 pub fn speed(&self,state:&NativeState)->f32{self.registry.as_ref().map_or(1.0,|r|r.resources.items().mining_speed(self.stack.as_ref(),state.name()))}
 pub fn correct(&self,state:&NativeState)->bool{self.registry.as_ref().is_some_and(|r|r.resources.items().correct_for_drops(self.stack.as_ref(),state.name()))}
 pub fn max_damage(&self)->i32{self.components["minecraft:max_damage"].as_i64().unwrap_or(0).clamp(0,i32::MAX as i64) as i32}
 pub fn damage(&self)->i32{self.components["minecraft:damage"].as_i64().unwrap_or(0).clamp(0,i32::MAX as i64) as i32}
 pub fn boat(&self)->bool{self.registry.as_ref().is_some_and(|r|r.resources.items().item_in_tag("minecraft:boats",self.item()))}
 pub fn scaffold_placeable(&self)->bool{
  let Some(registry)=self.registry.as_ref()else{return false;};let Some(id)=registry.resources.items().get(self.item()).and_then(|i|i.block)else{return false;};let state=registry.state(id);
  if state.has_class("FallingBlock")||matches!(state.name(),"minecraft:tnt"|"minecraft:cobweb"|"minecraft:nether_portal"){return false;}
  world::read(|g|{let Some(player)=g.player.as_ref()else{return false;};let context=CollisionContext::for_entity(CollisionEntity{identity:horizon_world::entity_obstruction::EntityIdentity(0),kind:horizon_world::collision_context::EntityContextKind::Player,y:player.position.y,descending:player.shift_key_down,main_hand:Some(CollisionItem::EMPTY)},false);let mut scene=horizon_client::collisions::SceneCollisions{world:&g.world,binding:g.collision,subject:Some(player.powder_snow_subject())};g.collision.collision_shape_with_context(&mut scene,id,horizon_core::position::BlockPos::new(0,0,0),&context).is_ok_and(|shape|shape.is_block()||shape.face_shape(horizon_core::geometry::Direction::Up).is_ok_and(|face|horizon_core::geometry::VoxelShape::block().join_is_not_empty(&face,horizon_core::geometry::ShapeOp::XOR).is_ok_and(|differs|!differs)))})
 }
}
impl nav::walk_control::break_times::BreakStack<NativeState> for NativeStack{
 fn is_empty(&self)->bool{self.count()==0||self.item()=="minecraft:air"}
 fn copy_stack(&self)->Self{let mut copy=self.clone();copy.stack=copy.stack.map(|s|s.fresh_copy());copy}
 fn efficiency_level(&self)->i32{self.efficiency()}
 fn correct_for_drops(&self,state:&NativeState)->bool{self.correct(state)}
 fn destroy_speed(&self,state:&NativeState)->f32{self.speed(state)}
 fn empty_hand()->Self{Self::empty()}
}
impl nav::walk_control::ability_capture::CaptureStack<NativeState> for NativeStack{fn count(&self)->i32{self.count()}}
impl nav::snacking::PlainFoodStack for NativeStack{
 fn food_nutrition(&self)->Option<i32>{self.components.get("minecraft:food")?.get("nutrition")?.as_i64().map(|v|v as i32)}
 fn is_golden_apple(&self)->bool{self.item()=="minecraft:golden_apple"}
 fn is_enchanted_golden_apple(&self)->bool{self.item()=="minecraft:enchanted_golden_apple"}
 fn is_bad_food(&self)->bool{matches!(self.item(),"minecraft:rotten_flesh"|"minecraft:spider_eye"|"minecraft:poisonous_potato"|"minecraft:pufferfish"|"minecraft:chorus_fruit"|"minecraft:chicken"|"minecraft:suspicious_stew")}
}
impl nav::glide::abilities::GlideStack for NativeStack{
 fn can_glide_chest(&self)->bool{self.components.get("minecraft:glider").is_some()&&self.components["minecraft:equippable"]["slot"].as_str()==Some("chest")&&(!self.is_damageable()||self.damage()<self.max_damage()-1)}
 fn is_damageable(&self)->bool{self.max_damage()>0&&self.components.get("minecraft:damage").is_some()&&self.components.get("minecraft:unbreakable").is_none()}
 fn max_damage(&self)->i32{self.max_damage()}
 fn damage(&self)->i32{self.damage()}
 fn is_firework_rocket(&self)->bool{self.item()=="minecraft:firework_rocket"}
 fn count(&self)->i32{self.count()}
 fn firework_flight_duration(&self)->Option<i32>{self.components.get("minecraft:fireworks")?.get("flight_duration")?.as_i64().map(|v|v as i32)}
}
#[derive(Clone)]
pub(super) struct NativeBlock{state:NativeState}
impl nav::options::DefaultPlaceableStack for NativeStack{
 type Block=NativeBlock;
 fn item_block(&self)->Option<NativeBlock>{let registry=self.registry.as_ref()?;let id=registry.resources.items().get(self.item())?.block?;Some(NativeBlock{state:registry.state(id)})}
}
impl nav::options::DefaultPlaceableBlock for NativeBlock{
 type State=NativeState;
 fn default_state(&self)->NativeState{self.state.clone()}
 fn is_entity_block(&self)->bool{self.state.facts().block_entity}
 fn is_falling(&self)->bool{self.state.has_class("FallingBlock")}
 fn is_interactive(&self)->bool{self.state.has_class("DoorBlock")||self.state.has_class("TrapDoorBlock")||self.state.has_class("FenceGateBlock")||self.state.has_class("ButtonBlock")||self.state.has_class("LeverBlock")||self.state.has_class("BedBlock")||self.state.has_class("CraftingTableBlock")||self.state.has_class("AnvilBlock")||self.state.has_class("StonecutterBlock")||self.state.has_class("GrindstoneBlock")||self.state.has_class("CartographyTableBlock")||self.state.has_class("FletchingTableBlock")||self.state.has_class("SmithingTableBlock")}
 fn collision_full_in_empty_world(&self,state:&NativeState)->bool{state.facts().full_collision}
 fn friction(&self)->f32{nav::walk::WalkState::friction(&self.state)}
 fn speed_factor(&self)->f32{nav::walk::WalkState::speed_factor(&self.state)}
 fn jump_factor(&self)->f32{nav::walk::WalkState::jump_factor(&self.state)}
}
impl nav::walk_control::ability_capture::CaptureInventory for NativeInventory{type Stack=NativeStack;fn item(&self,slot:i32)->NativeStack{self.stack(slot)}}
impl nav::walk_control::block_targets::TargetInventory for NativeInventory{type Stack=NativeStack;fn item(&self,slot:i32)->NativeStack{self.stack(slot)}fn selected_slot(&self)->i32{world::read(|g|g.selected as i32)}}
impl nav::snacking::FoodInventory for NativeInventory{type Stack=NativeStack;fn item(&self,slot:i32)->NativeStack{self.stack(slot)}}
impl nav::glide::capture::GlideInventory for NativeInventory{type Stack=NativeStack;fn item(&self,slot:i32)->NativeStack{self.stack(slot)}}
impl nav::ride::boating::BoatInventory for NativeInventory{fn item_is_boat(&self,slot:i32)->bool{self.stack(slot).boat()}fn suitable_hotbar_slot(&self)->i32{self.suitable_slot()}}

#[derive(Clone)]
pub(super) struct NativePlayer{pub inventory:NativeInventory,pub simulated:Option<Rc<RefCell<PlayerMotion>>>}
impl NativePlayer{
 pub fn new(level:NativeLevel)->Self{Self{inventory:NativeInventory{level},simulated:None}}
 pub fn read<R>(&self,f:impl FnOnce(&PlayerMotion)->R)->R{if let Some(body)=&self.simulated{f(&body.borrow())}else{world::read(|g|{assert_eq!(world::current_epoch(g),self.inventory.level.epoch);f(g.player.as_ref().expect("navigation player"))})}}
 pub fn attribute(&self,id:&str)->f64{self.read(|p|p.attributes.computed_value_of(id).expect("vanilla player attribute"))}
 pub fn position(&self)->NavVec{self.read(|p|world::vec(p.position))}
 pub fn motion(&self)->NavVec{self.read(|p|world::vec(p.velocity))}
 pub fn on_ground(&self)->bool{self.read(|p|p.ground.on_ground)}
 pub fn eye(&self)->NavVec{self.read(|p|world::vec(p.position.add_vector(Vec3::new(0.,f64::from(p.dimensions.eye_height),0.))))}
 pub fn interpolated(&self,partial:f32)->NavVec{self.read(|p|world::vec(p.old_position.lerp(p.position,f64::from(partial))))}
}
impl nav::simulation::player_motion::MotionPlayer for NativePlayer{
 fn position(&self)->NavVec{self.position()}
 fn delta_movement(&self)->NavVec{self.motion()}
 fn on_ground(&self)->bool{self.on_ground()}
 fn is_sprinting(&self)->bool{self.read(|p|p.sprinting)}
 fn jump_delay(&self)->i32{self.read(|p|p.no_jump_delay)}
 fn is_crouching(&self)->bool{self.read(|p|p.pose==horizon_client::pose::Pose::Crouching)}
 fn supporting_block(&self)->Option<NavPos>{self.read(|p|p.ground.supporting_block.map(nav_pos))}
}
impl nav::walk_control::body::WalkPlayer for NativePlayer{
 type State=NativeState;type Level=NativeLevel;
 fn level(&self)->&NativeLevel{&self.inventory.level}
 fn y(&self)->f64{self.read(|p|p.position.y)}
 fn movement_speed(&self)->f64{self.attribute("minecraft:movement_speed")}
 fn jump_strength(&self)->f64{self.attribute("minecraft:jump_strength")}
 fn jump_boost_power(&self)->f32{self.read(|p|p.status.jump_boost.map_or(0.,|a|(a+1) as f32*0.1_f32))}
 fn mechanics(&self)->nav::simulation::Mechanics{nav::simulation::Mechanics::Modern}
 fn tick_count(&self)->i32{self.read(|p|p.health.tick_count)}
 fn horizontal_collision(&self)->bool{self.read(|p|p.collision.horizontal)}
 fn is_in_water(&self)->bool{self.read(|p|p.fluids.was_touching_water)}
 fn on_climbable(&self)->bool{self.read(|p|p.on_climbable)}
 fn is_using_item(&self)->bool{world::read(|g|g.using.is_some())}
 fn block_below_movement(&self)->NavPos{let (p,support)=self.read(|p|(p.position,p.ground.supporting_block));let y=(p.y-f64::from(horizon_world::movement_support::MOVEMENT_SUPPORT_OFFSET)).floor() as i32;if let Some(s)=support{let state=self.inventory.level.block_state(nav_pos(s));if state.has_class("WallBlock")||state.has_class("FenceGateBlock"){nav_pos(s)}else{NavPos::new(s.x,y,s.z)}}else{NavPos::new(p.x.floor() as i32,y,p.z.floor() as i32)}}
 fn food_level(&self)->i32{self.read(|p|p.health.food.food)}
 fn eye_position(&self)->NavVec{self.eye()}
 fn is_eye_in_water(&self)->bool{self.read(|p|p.is_eye_in_fluid(horizon_client::motion::WATER))}
 fn air_supply(&self)->i32{self.read(|p|p.air)}
 fn max_air_supply(&self)->i32{300}
}
impl nav::flight_handling::FlightPlayer for NativePlayer{
 fn position(&self)->NavVec{self.position()}fn delta_movement(&self)->NavVec{self.motion()}fn tick_count(&self)->i32{self.read(|p|p.health.tick_count)}
 fn horizontal_collision(&self)->bool{self.read(|p|p.collision.horizontal)}fn vertical_collision(&self)->bool{self.read(|p|p.collision.vertical)}
 fn y_rot(&self)->f32{self.read(|p|p.yaw)}fn is_sprinting(&self)->bool{self.read(|p|p.sprinting)}
}
impl nav::flight_body::PlayerFlightAbilities for NativePlayer{fn flying(&self)->bool{self.read(|p|p.flying)}fn may_fly(&self)->bool{self.read(|p|p.may_fly)}fn flying_speed(&self)->f32{self.read(|p|p.flying_speed)}}
impl nav::flight_body::FlightActor for NativePlayer{type Abilities=Self;fn abilities(&self)->&Self{self}fn y(&self)->f64{self.read(|p|p.position.y)}fn on_ground(&self)->bool{self.on_ground()}fn bounding_box(&self)->NavBox{self.read(|p|box_(p.bounds()))}fn interpolated_position(&self,partial:f32)->NavVec{self.interpolated(partial)}}
impl nav::glide::capture::EquippedGlidePlayer for NativePlayer{type Stack=NativeStack;type Inventory=NativeInventory;fn inventory(&self)->&NativeInventory{&self.inventory}fn offhand_item(&self)->NativeStack{self.inventory.stack(40)}fn chest_item(&self)->NativeStack{self.inventory.stack(38)}}
impl nav::glide::contacts::ContactPlayer for NativePlayer{fn is_fall_flying(&self)->bool{self.read(|p|p.fall_flying)}fn horizontal_collision(&self)->bool{self.read(|p|p.collision.horizontal)}fn vertical_collision(&self)->bool{self.read(|p|p.collision.vertical)}fn vertical_collision_below(&self)->bool{self.read(|p|p.collision.below)}fn bounding_box(&self)->NavBox{self.read(|p|box_(p.bounds()))}}
impl nav::glide::body::GlidePlayer for NativePlayer{fn position(&self)->NavVec{self.position()}fn on_ground(&self)->bool{self.on_ground()}fn delta_movement(&self)->NavVec{self.motion()}fn block_position(&self)->NavPos{NavPos::containing(self.position())}}
impl nav::glide::roof_bounce::RoofPlayer for NativePlayer{fn position(&self)->NavVec{self.position()}fn on_ground(&self)->bool{self.on_ground()}fn is_fall_flying(&self)->bool{self.read(|p|p.fall_flying)}}
