// SPDX-License-Identifier: GPL-3.0-or-later
//! HoleFiller shares BlockPlacer state, the native interaction pipeline and global executor.
use crate::{runtime::LiquidBounceHooks,block_placer_runtime::{BlockPlacerRuntime,DeferredAction,native}};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{hole_filler::{self as core,Settings},block_placer::Crystal,no_rotate_set::SmoothMode,rotations::{DefaultRotationTarget,Rotation},random::random_i32_inclusive};
use serde_json::{Value,json};
pub const PLACEMENT_PROVIDER:u64=37;
pub const CRYSTAL_PROVIDER:u64=38;
pub const PLACEMENT_PROCESSOR:u64=560;
pub const CRYSTAL_PROCESSOR:u64=576;
pub const HOLE_FILLER_REQUESTER:u64=0x484f_4c45_4649_4c4c;
#[derive(Default)]pub struct HoleFillerRuntime{pub settings:Settings,pub placer:BlockPlacerRuntime}
impl HoleFillerRuntime{pub fn processor_mode(id:u64)->Option<(bool,SmoothMode)>{id.checked_sub(PLACEMENT_PROCESSOR).and_then(SmoothMode::from_processor_id).map(|m|(false,m)).or_else(||id.checked_sub(CRYSTAL_PROCESSOR).and_then(SmoothMode::from_processor_id).map(|m|(true,m)))}}
fn query(host:&mut dyn Host,q:Value)->Value{serde_json::from_str(&host.client_hole_filler_query(&q.to_string())).unwrap_or(Value::Null)}
fn rotation(v:&Value)->Option<[f32;2]>{serde_json::from_value(v.clone()).ok()}
fn slots(host:&mut dyn Host)->(Vec<core::Slot>,u8){let facts=query(host,json!({"op":"slots"}));let slots=facts["slots"].as_array().into_iter().flatten().filter_map(|v|Some(core::Slot{index:v["index"].as_u64()?as u8,count:v["count"].as_i64()?as i32,block:v["block"].as_str().map(str::to_owned)})).collect();(slots,facts["selected"].as_u64().unwrap_or(0)as u8)}
pub(crate) fn slot(host:&mut dyn Host,s:&Settings,_target:Option<&Value>)->Option<u8>{let(v,selected)=slots(host);core::slot(s,&v,selected)}
impl LiquidBounceHooks{
    pub(crate) fn hole_filler_running(&self)->bool{let state=self.state();let running=state.behaviors.borrow().query_running(&state.registry,"HoleFiller");running}
    pub(crate) fn hole_filler_run(&self,host:&mut dyn Host,tasks:Vec<DeferredAction>){for action in tasks{match action{
        DeferredAction::Place{requester,target,normal:_,send}=>{
            if requester!=HOLE_FILLER_REQUESTER||!self.hole_filler_running(){continue;}
            let(s,settings,wait)={let state=self.state();let b=state.behaviors.borrow();(b.hole_filler.settings.clone(),b.hole_filler.placer.settings.clone(),b.hole_filler.placer.queue.ticks_to_wait)};
            if wait>0{continue;}
            let Some(planned)=rotation(&target["rotation"])else{continue;};
            if send&&settings.send_rotation_packet{native(host,json!({"op":"rotationPacket","rotation":planned}));}
            let verify=if settings.rotation_mode=="Normal"{self.block_placer_rotations(host).1}else{Rotation::new(planned[0],planned[1])};
            let slot=slot(host,&s,Some(&target["placed"]));
            if let Some(slot)=slot{native(host,json!({"op":"execute","requester":requester,"target":target,"rotation":[verify.yaw,verify.pitch],"slot":slot,"reset":settings.slot_reset_delay,"swing":settings.swing,"constructFailResult":settings.construct_fail_result,"range":settings.range,"walls":settings.wall_range}));}
            self.state().behaviors.borrow_mut().hole_filler.placer.queue.ran_action=true;
        }
        DeferredAction::Crystal{requester,rotation,send}=>{
            if requester!=HOLE_FILLER_REQUESTER||!self.hole_filler_running(){continue;}
            if send&&self.state().behaviors.borrow().hole_filler.placer.crystals.send_rotation_packet{native(host,json!({"op":"rotationPacket","rotation":rotation}));}
            self.hole_filler_attack_crystal(host);
        }
    }}}
    pub(crate) fn hole_filler_request(&self,crystal:bool,angles:[f32;2]){
        let s=self.state();let b=s.behaviors.borrow();let r=&b.hole_filler.placer;let settings=if crystal{&r.crystal_rotations}else{&r.rotations};
        b.rotations.lock().expect("RotationManager").request_processed(if crystal{CRYSTAL_PROVIDER}else{PLACEMENT_PROVIDER},if crystal{35}else{0},DefaultRotationTarget{rotation:Rotation::new(angles[0],angles[1]),horizontal_turn_speed:180.,vertical_turn_speed:180.,ticks_until_reset:settings.ticks_until_reset,reset_threshold:settings.reset_threshold,consider_inventory:if crystal{!r.crystals.ignore_open_inventory}else{r.settings.ignore&1==0},movement_correction:settings.movement_correction},(if crystal{CRYSTAL_PROCESSOR}else{PLACEMENT_PROCESSOR})+settings.active.processor_id(),true);
    }
    pub(crate) fn hole_filler_reached(&self,host:&mut dyn Host){
        if !self.hole_filler_running(){return;}
        let (current,server)=self.block_placer_rotations(host);
        let (processor,pending,crystal_pending,s,cs)={let state=self.state();let b=state.behaviors.borrow();let r=&b.hole_filler.placer;let processed=b.rotations.lock().expect("RotationManager").processed_request;(processed,r.pending_placement.clone(),r.pending_crystal,r.settings.clone(),r.crystals.clone())};
        if processor.is_some_and(|p|p>=PLACEMENT_PROCESSOR&&p<PLACEMENT_PROCESSOR+16){if let(Some(current),Some(target))=(current,pending){if native(host,json!({"op":"reached","target":target,"rotation":[current.yaw,current.pitch],"range":s.range,"walls":s.wall_range}))==true{let state=self.state();let mut b=state.behaviors.borrow_mut();b.hole_filler.placer.pending_placement=None;b.block_placer_executor.add(DeferredAction::Place{requester:HOLE_FILLER_REQUESTER,target,normal:true,send:false},s.normal_post_move,true);}}}
        if processor.is_some_and(|p|p>=CRYSTAL_PROCESSOR&&p<CRYSTAL_PROCESSOR+16){if let Some((captured_id,rotation))=crystal_pending{if native(host,json!({"op":"crystalReached","id":captured_id,"rotation":[server.yaw,server.pitch],"range":cs.range,"walls":cs.wall_range}))==true{let state=self.state();let mut b=state.behaviors.borrow_mut();b.hole_filler.placer.pending_crystal=None;b.block_placer_executor.add(DeferredAction::Crystal{requester:HOLE_FILLER_REQUESTER,rotation,send:false},cs.normal_post_move,true);}}}
    }
    pub(crate) fn hole_filler_crystal_tick(&self,host:&mut dyn Host){
        if !self.hole_filler_running(){return;}
        let now=crate::behaviors::current_time_millis();let(cs,id,elapsed)={let state=self.state();let b=state.behaviors.borrow();let r=&b.hole_filler.placer;(r.crystals.clone(),r.destroyer.target.map(|c|c.id),r.destroyer.elapsed(now,r.crystals.delay))};
        let Some(id)=id.filter(|_|cs.enabled&&elapsed)else{return;};
        let look=self.block_placer_rotations(host).0.unwrap_or_else(||{let [y,p]=host.client_player_rotation();Rotation::new(y,p)});
        let facts=native(host,json!({"op":"crystal","id":id,"range":cs.range,"walls":cs.wall_range,"look":[look.yaw,look.pitch]}));
        if facts["wouldKill"]==true{self.state().behaviors.borrow_mut().hole_filler.placer.destroyer.target=None;return;}
        let Some(rotation)=rotation(&facts["rotation"])else{return;};
        if cs.rotation_mode=="Normal"{
            let server=self.block_placer_rotations(host).1;
            if cs.normal_instant&&native(host,json!({"op":"crystalReached","id":id,"rotation":[server.yaw,server.pitch],"range":cs.range,"walls":cs.wall_range}))==true{self.hole_filler_attack_crystal(host);return;}
            self.state().behaviors.borrow_mut().hole_filler.placer.pending_crystal=Some((id,rotation));self.hole_filler_request(true,rotation);
        }else{let task=DeferredAction::Crystal{requester:HOLE_FILLER_REQUESTER,rotation,send:true};if cs.none_instant{self.hole_filler_run(host,vec![task]);}else{self.state().behaviors.borrow_mut().block_placer_executor.add(task,cs.none_post_move,false);}}
    }
    fn hole_filler_attack_crystal(&self,host:&mut dyn Host){
        let now=crate::behaviors::current_time_millis();let(cs,id,elapsed)={let state=self.state();let b=state.behaviors.borrow();let r=&b.hole_filler.placer;(r.crystals.clone(),r.destroyer.target.map(|c|c.id),r.destroyer.elapsed(now,r.crystals.delay))};
        let Some(id)=id.filter(|_|elapsed)else{return;};
        let look=self.block_placer_rotations(host).0.unwrap_or_else(||{let [y,p]=host.client_player_rotation();Rotation::new(y,p)});
        let facts=native(host,json!({"op":"crystal","id":id,"range":cs.range,"walls":cs.wall_range,"look":[look.yaw,look.pitch]}));
        if facts["wouldKill"]==true{self.state().behaviors.borrow_mut().hole_filler.placer.destroyer.target=None;return;}
        if facts.is_null(){return;}
        // attackEntity may be canceled by another module. The source still
        // resets its chronometer and clears its target after that call.
        native(host,json!({"op":"crystalAttack","id":id,"swing":cs.swing_mode}));self.state().behaviors.borrow_mut().hole_filler.placer.destroyer.attacked(now);
    }
    pub(crate) fn hole_filler_success(&self,requester:u64,position:[i32;3],bounds:[f64;6]){if requester==HOLE_FILLER_REQUESTER{self.state().behaviors.borrow_mut().hole_filler.placer.success(position,bounds,crate::behaviors::current_time_millis());}}
    pub(crate) fn hole_filler_removed(&self,id:i32){if !self.hole_filler_running(){return;}let state=self.state();let mut b=state.behaviors.borrow_mut();if b.hole_filler.placer.crystals.enabled{b.hole_filler.placer.destroyer.removed(id);}}
    pub(crate) fn hole_filler_prepare(&self,host:&mut dyn Host){
        if !self.hole_filler_running(){return;}let inventory=host.client_inventory_open();let look=host.client_player_rotation();let using=native(host,json!({"op":"using"}))==true;
        let snapshot=query(host,json!({"op":"snapshot"}));
        let mut r=std::mem::take(&mut self.state().behaviors.borrow_mut().hole_filler);let now=crate::behaviors::current_time_millis();
        let range=r.placer.settings.range.max(r.placer.settings.wall_range);
        if let Ok(player)=serde_json::from_value::<[f64;3]>(snapshot["player"].clone()){
            let holes=snapshot["holes"].as_array().into_iter().flatten().filter_map(|v|Some(liquidbounce_behaviors::hole_tracker::Hole{position:serde_json::from_value(v["position"].clone()).ok()?,size:serde_json::from_value(v["size"].clone()).ok()?,kind:v["kind"].as_u64()?as u8})).collect::<Vec<_>>();
            let mut entities=vec![];
            if r.settings.has(core::SMART){for row in snapshot["entities"].as_array().into_iter().flatten(){
                let Some(id)=row["id"].as_i64().map(|i|i as i32)else{continue;};
                let Some(target)=crate::targeting::snapshot(&host.client_targeting_entity(id))else{continue;};
                let valid={let mut state=self.state();let info=state.target_info(id,&target);state.global_targets.combat.should_be_attacked(&target.entity,info)};if !valid{continue;}
                if let(Ok(position),Ok(previous))=(serde_json::from_value(row["position"].clone()),serde_json::from_value(row["previous"].clone())){entities.push(core::Entity{position,previous});}
            }}
            let (items,_)=slots(host);let items=core::available(&r.settings,&items);
            if let Some(positions)=core::targets(&r.settings,&holes,player,snapshot["creative"]==true,range.ceil()as i32,items,&entities){r.placer.update(&positions,now);}
        }
        let s=&r.placer.settings;r.placer.queue.tick_start(||random_i32_inclusive(s.cooldown[0],s.cooldown[1]));let mut tasks=vec![];let mut request=None;
        if s.can_act(inventory,using)&&r.placer.queue.ticks_to_wait<=0&&!r.placer.queue.blocks.is_empty(){
            if let Some(default_slot)=slot(host,&r.settings,None){
                r.placer.queue.inaccessible.clear();let mut placements=0;let blocks=r.placer.queue.blocks.clone();let current=self.block_placer_rotations(host).0.unwrap_or(Rotation::new(look[0],look[1]));
                if let Some(old)=r.placer.destroyer.target{let facts=native(host,json!({"op":"crystal","id":old.id,"range":r.placer.crystals.range,"walls":r.placer.crystals.wall_range,"look":[current.yaw,current.pitch]}));if let Some(distance)=facts["distanceSquared"].as_f64(){r.placer.destroyer.target=Some(Crystal{id:old.id,distance_squared:distance});}}
                for pos in blocks{
                    let plan=native(host,json!({"op":"plan","position":pos,"slot":default_slot,"rotation":[current.yaw,current.pitch],"range":s.range,"walls":s.wall_range}));
                    if plan["inaccessible"]==true{r.placer.queue.inaccessible.push(pos);continue;}
                    if plan["blocked"]==true{if r.placer.crystals.enabled{if let Some(id)=plan["crystal"]["id"].as_i64(){let facts=native(host,json!({"op":"crystal","id":id,"range":r.placer.crystals.range,"walls":r.placer.crystals.wall_range,"look":[current.yaw,current.pitch]}));if let Some(distance)=facts["distanceSquared"].as_f64(){r.placer.destroyer.suggest(Crystal{id:id as i32,distance_squared:distance},facts["reachable"]==true);}}}continue;}
                    let target=plan["target"].clone();let Ok(rotation)=serde_json::from_value::<[f32;2]>(target["rotation"].clone())else{continue;};
                    if plan["interactable"]==true{r.placer.queue.sneak_times=random_i32_inclusive(s.sneak[0],s.sneak[1]);}
                    if s.rotation_mode=="Normal"{r.placer.pending_placement=Some(target);request=Some(rotation);break;}
                    tasks.push((DeferredAction::Place{requester:HOLE_FILLER_REQUESTER,target,normal:false,send:true},s.none_post_move));placements+=1;if placements==s.placements{break;}
                }
            }
        }
        self.state().behaviors.borrow_mut().hole_filler=r;
        {let state=self.state();let mut b=state.behaviors.borrow_mut();for(task,post)in tasks{b.block_placer_executor.add(task,post,false);}}
        if let Some(rotation)=request{self.hole_filler_request(false,rotation);}
    }
    pub(crate) fn hole_filler_event(&self,event:&mut horizon_client::mod_api::ClientHoleFillerEvent<'_>){
        let Ok(input)=serde_json::from_str::<Value>(event.input)else{return;};if input["action"]!="options"{return;}
        let state=self.state();let b=state.behaviors.borrow();let r=&b.hole_filler;*event.output=json!({"active":b.query_running(&state.registry,"HoleFiller"),"range":r.placer.settings.range.max(r.placer.settings.wall_range).ceil()as i32}).to_string();
    }
    pub(crate) fn hole_filler_world_change(&self){self.state().behaviors.borrow_mut().hole_filler.placer.reset_world();}
}
