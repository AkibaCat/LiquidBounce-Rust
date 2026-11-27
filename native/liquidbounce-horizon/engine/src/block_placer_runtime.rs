// SPDX-License-Identifier: GPL-3.0-or-later
//! Reusable non-support BlockPlacer and the global PostRotationExecutor.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{block_placer::{self as core,PostRotationExecutor},no_rotate_set::{RotationsSettings,SmoothMode},placement_renderer::{PlacementRenderer,FULL},rotations::{DefaultRotationTarget,Rotation}};
use serde_json::{Value,json};
pub const PLACEMENT_PROVIDER:u64=32;
pub const CRYSTAL_PROVIDER:u64=33;
pub const PLACEMENT_PROCESSOR:u64=480;
pub const CRYSTAL_PROCESSOR:u64=496;
pub const LIQUID_FILLER_REQUESTER:u64=0x4c49_5155_4944_4649;
#[derive(Clone,Debug)]
pub enum DeferredAction{Place{requester:u64,target:Value,normal:bool,send:bool},Crystal{requester:u64,rotation:[f32;2],send:bool}}
pub type Executor=PostRotationExecutor<DeferredAction>;
#[derive(Clone,Debug)]
pub struct BlockPlacerRuntime{
    pub settings:core::Settings,pub crystals:core::CrystalSettings,
    pub rotations:RotationsSettings,pub crystal_rotations:RotationsSettings,
    pub queue:core::Queue,pub destroyer:core::CrystalDestroyer,
    pub target_render:PlacementRenderer,pub placed_render:PlacementRenderer,
    pub pending_placement:Option<Value>,pub pending_crystal:Option<(i32,[f32;2])>,
}
impl Default for BlockPlacerRuntime{fn default()->Self{let mut target=PlacementRenderer::new(true,0x5a00ff00);target.enabled=false;Self{settings:Default::default(),crystals:Default::default(),rotations:Default::default(),crystal_rotations:Default::default(),queue:Default::default(),destroyer:core::CrystalDestroyer{last_attack:crate::behaviors::current_time_millis(),..Default::default()},target_render:target,placed_render:PlacementRenderer::new(false,0x5a00ff00),pending_placement:None,pending_crystal:None}}}
impl BlockPlacerRuntime{
    pub fn processor_mode(id:u64)->Option<(bool,SmoothMode)>{id.checked_sub(PLACEMENT_PROCESSOR).and_then(SmoothMode::from_processor_id).map(|m|(false,m)).or_else(||id.checked_sub(CRYSTAL_PROCESSOR).and_then(SmoothMode::from_processor_id).map(|m|(true,m)))}
    pub fn update(&mut self,positions:&[core::Pos],time:i64){let(removed,added)=self.queue.update(positions);for p in removed{self.target_render.remove(p,time);}for p in added{self.target_render.add(p,FULL,time,false);}self.target_render.update_all();}
    pub fn clear(&mut self,time:i64){for p in self.queue.clear(){self.target_render.remove(p,time);}}
    pub fn reset_world(&mut self){self.queue.reset();self.pending_placement=None;self.pending_crystal=None;self.target_render.clear();self.placed_render.clear();}
    pub fn disable(&mut self,time:i64){self.queue.reset();self.destroyer.target=None;self.pending_placement=None;self.pending_crystal=None;self.target_render.clear_silently(time);self.placed_render.clear_silently(time);}
    pub fn success(&mut self,p:core::Pos,bounds:[f64;6],time:i64){self.queue.success(p);self.target_render.remove(p,time);self.placed_render.add(p,bounds,time,true);}
}
pub(crate) fn native(host:&mut dyn Host,q:Value)->Value{serde_json::from_str(&host.client_block_placer_query(&q.to_string())).unwrap_or(Value::Null)}
fn rotation(v:&Value)->Option<[f32;2]>{serde_json::from_value(v.clone()).ok()}
impl LiquidBounceHooks{
    pub(crate) fn block_placer_first_tick(&self,host:&mut dyn Host){let tasks=self.state().behaviors.borrow_mut().block_placer_executor.first_tick();self.block_placer_run(host,tasks);}
    pub(crate) fn block_placer_after_move(&self,host:&mut dyn Host){let tasks={let s=self.state();let mut b=s.behaviors.borrow_mut();let current=b.rotations.lock().expect("RotationManager").current.is_some();b.block_placer_executor.after_move(current)};self.block_placer_run(host,tasks);}
    fn block_placer_run(&self,host:&mut dyn Host,tasks:Vec<DeferredAction>){for action in tasks{if matches!(&action,DeferredAction::Place{requester,..}|DeferredAction::Crystal{requester,..} if *requester==crate::hole_filler_runtime::HOLE_FILLER_REQUESTER){self.hole_filler_run(host,vec![action]);continue;}if matches!(&action,DeferredAction::Place{requester,..}|DeferredAction::Crystal{requester,..} if *requester==crate::bed_defender_runtime::BED_DEFENDER_REQUESTER){self.bed_defender_run(host,vec![action]);continue;}match action{
        DeferredAction::Place{requester,target,normal:_,send}=>{
            if requester!=LIQUID_FILLER_REQUESTER||!self.liquid_filler_running(){continue;}
            let(s,settings,wait)={let state=self.state();let b=state.behaviors.borrow();(b.liquid_filler.settings.clone(),b.liquid_filler.placer.settings.clone(),b.liquid_filler.placer.queue.ticks_to_wait)};
            if wait>0{continue;}
            let Some(planned)=rotation(&target["rotation"])else{continue;};
            if send&&settings.send_rotation_packet{native(host,json!({"op":"rotationPacket","rotation":planned}));}
            let verify=if settings.rotation_mode=="Normal"{self.block_placer_rotations(host).1}else{Rotation::new(planned[0],planned[1])};
            let slot=crate::liquid_filler_runtime::slot(host,&s,Some(&target["placed"]));
            if let Some(slot)=slot{native(host,json!({"op":"execute","requester":requester,"target":target,"rotation":[verify.yaw,verify.pitch],"slot":slot,"reset":settings.slot_reset_delay,"swing":settings.swing,"constructFailResult":settings.construct_fail_result,"range":settings.range,"walls":settings.wall_range}));}
            self.state().behaviors.borrow_mut().liquid_filler.placer.queue.ran_action=true;
        }
        DeferredAction::Crystal{requester,rotation,send}=>{
            if requester!=LIQUID_FILLER_REQUESTER||!self.liquid_filler_running(){continue;}
            if send&&self.state().behaviors.borrow().liquid_filler.placer.crystals.send_rotation_packet{native(host,json!({"op":"rotationPacket","rotation":rotation}));}
            self.block_placer_attack_crystal(host);
        }
    }}}
    pub(crate) fn block_placer_rotations(&self,host:&dyn Host)->(Option<Rotation>,Rotation){let s=self.state();let b=s.behaviors.borrow();let r=b.rotations.lock().expect("RotationManager");let _=host;(r.current,r.server_rotation(b.criticals.is_lagging(),b.query_running(&s.registry,"Freeze")))}
    pub(crate) fn block_placer_request(&self,crystal:bool,angles:[f32;2]){
        let s=self.state();let b=s.behaviors.borrow();let r=&b.liquid_filler.placer;let settings=if crystal{&r.crystal_rotations}else{&r.rotations};
        b.rotations.lock().expect("RotationManager").request_processed(if crystal{CRYSTAL_PROVIDER}else{PLACEMENT_PROVIDER},if crystal{35}else{0},DefaultRotationTarget{rotation:Rotation::new(angles[0],angles[1]),horizontal_turn_speed:180.,vertical_turn_speed:180.,ticks_until_reset:settings.ticks_until_reset,reset_threshold:settings.reset_threshold,consider_inventory:if crystal{!r.crystals.ignore_open_inventory}else{r.settings.ignore&1==0},movement_correction:settings.movement_correction},(if crystal{CRYSTAL_PROCESSOR}else{PLACEMENT_PROCESSOR})+settings.active.processor_id(),true);
    }
    pub(crate) fn block_placer_reached(&self,host:&mut dyn Host){
        if !self.liquid_filler_running(){return;}
        let (current,server)=self.block_placer_rotations(host);
        let (processor,pending,crystal_pending,s,cs)={let state=self.state();let b=state.behaviors.borrow();let r=&b.liquid_filler.placer;let processed=b.rotations.lock().expect("RotationManager").processed_request;(processed,r.pending_placement.clone(),r.pending_crystal,r.settings.clone(),r.crystals.clone())};
        if processor.is_some_and(|p|p>=PLACEMENT_PROCESSOR&&p<PLACEMENT_PROCESSOR+16){if let(Some(current),Some(target))=(current,pending){if native(host,json!({"op":"reached","target":target,"rotation":[current.yaw,current.pitch],"range":s.range,"walls":s.wall_range}))==true{let state=self.state();let mut b=state.behaviors.borrow_mut();b.liquid_filler.placer.pending_placement=None;b.block_placer_executor.add(DeferredAction::Place{requester:LIQUID_FILLER_REQUESTER,target,normal:true,send:false},s.normal_post_move,true);}}}
        if processor.is_some_and(|p|p>=CRYSTAL_PROCESSOR&&p<CRYSTAL_PROCESSOR+16){if let Some((captured_id,rotation))=crystal_pending{if native(host,json!({"op":"crystalReached","id":captured_id,"rotation":[server.yaw,server.pitch],"range":cs.range,"walls":cs.wall_range}))==true{let state=self.state();let mut b=state.behaviors.borrow_mut();b.liquid_filler.placer.pending_crystal=None;b.block_placer_executor.add(DeferredAction::Crystal{requester:LIQUID_FILLER_REQUESTER,rotation,send:false},cs.normal_post_move,true);}}}
    }
    pub(crate) fn block_placer_crystal_tick(&self,host:&mut dyn Host){
        if !self.liquid_filler_running(){return;}
        let now=crate::behaviors::current_time_millis();let(cs,id,elapsed)={let state=self.state();let b=state.behaviors.borrow();let r=&b.liquid_filler.placer;(r.crystals.clone(),r.destroyer.target.map(|c|c.id),r.destroyer.elapsed(now,r.crystals.delay))};
        let Some(id)=id.filter(|_|cs.enabled&&elapsed)else{return;};
        let look=self.block_placer_rotations(host).0.unwrap_or_else(||{let [y,p]=host.client_player_rotation();Rotation::new(y,p)});
        let facts=native(host,json!({"op":"crystal","id":id,"range":cs.range,"walls":cs.wall_range,"look":[look.yaw,look.pitch]}));
        if facts["wouldKill"]==true{self.state().behaviors.borrow_mut().liquid_filler.placer.destroyer.target=None;return;}
        let Some(rotation)=rotation(&facts["rotation"])else{return;};
        if cs.rotation_mode=="Normal"{
            let server=self.block_placer_rotations(host).1;
            if cs.normal_instant&&native(host,json!({"op":"crystalReached","id":id,"rotation":[server.yaw,server.pitch],"range":cs.range,"walls":cs.wall_range}))==true{self.block_placer_attack_crystal(host);return;}
            self.state().behaviors.borrow_mut().liquid_filler.placer.pending_crystal=Some((id,rotation));self.block_placer_request(true,rotation);
        }else{let task=DeferredAction::Crystal{requester:LIQUID_FILLER_REQUESTER,rotation,send:true};if cs.none_instant{self.block_placer_run(host,vec![task]);}else{self.state().behaviors.borrow_mut().block_placer_executor.add(task,cs.none_post_move,false);}}
    }
    fn block_placer_attack_crystal(&self,host:&mut dyn Host){
        let now=crate::behaviors::current_time_millis();let(cs,id,elapsed)={let state=self.state();let b=state.behaviors.borrow();let r=&b.liquid_filler.placer;(r.crystals.clone(),r.destroyer.target.map(|c|c.id),r.destroyer.elapsed(now,r.crystals.delay))};
        let Some(id)=id.filter(|_|elapsed)else{return;};
        let look=self.block_placer_rotations(host).0.unwrap_or_else(||{let [y,p]=host.client_player_rotation();Rotation::new(y,p)});
        let facts=native(host,json!({"op":"crystal","id":id,"range":cs.range,"walls":cs.wall_range,"look":[look.yaw,look.pitch]}));
        if facts["wouldKill"]==true{self.state().behaviors.borrow_mut().liquid_filler.placer.destroyer.target=None;return;}
        if facts.is_null(){return;}
        // attackEntity may be canceled by another module. The source still
        // resets its chronometer and clears its target after that call.
        native(host,json!({"op":"crystalAttack","id":id,"swing":cs.swing_mode}));self.state().behaviors.borrow_mut().liquid_filler.placer.destroyer.attacked(now);
    }
    pub(crate) fn block_placer_success(&self,requester:u64,position:[i32;3],bounds:[f64;6]){self.bed_defender_success(requester,position,bounds);self.hole_filler_success(requester,position,bounds);if requester==LIQUID_FILLER_REQUESTER{self.state().behaviors.borrow_mut().liquid_filler.placer.success(position,bounds,crate::behaviors::current_time_millis());}}
    pub(crate) fn block_placer_removed(&self,id:i32){self.bed_defender_removed(id);self.hole_filler_removed(id);if !self.liquid_filler_running(){return;}let state=self.state();let mut b=state.behaviors.borrow_mut();if b.liquid_filler.placer.crystals.enabled{b.liquid_filler.placer.destroyer.removed(id);}}
}
