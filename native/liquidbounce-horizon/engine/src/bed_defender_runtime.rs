// SPDX-License-Identifier: GPL-3.0-or-later
//! BedDefender shares BlockPlacer state, the native interaction pipeline and global executor.
use crate::{runtime::LiquidBounceHooks,block_placer_runtime::{BlockPlacerRuntime,DeferredAction,native}};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{bed_defender::{self as core,Settings},bed_plates::{Bed,BedPlates,Notice},block_placer::Crystal,no_rotate_set::SmoothMode,rotations::{DefaultRotationTarget,Rotation},random::random_i32_inclusive};
use serde_json::{Value,json};
pub const PLACEMENT_PROVIDER:u64=34;
pub const CRYSTAL_PROVIDER:u64=35;
pub const PLACEMENT_PROCESSOR:u64=512;
pub const CRYSTAL_PROCESSOR:u64=528;
pub const BED_DEFENDER_REQUESTER:u64=0x4245_445f_4445_4645;
#[derive(Default)]pub struct BedDefenderRuntime{pub settings:Settings,pub self_bed:BedPlates,pub placer:BlockPlacerRuntime}
impl BedDefenderRuntime{pub fn processor_mode(id:u64)->Option<(bool,SmoothMode)>{id.checked_sub(PLACEMENT_PROCESSOR).and_then(SmoothMode::from_processor_id).map(|m|(false,m)).or_else(||id.checked_sub(CRYSTAL_PROCESSOR).and_then(SmoothMode::from_processor_id).map(|m|(true,m)))}}
fn query(host:&mut dyn Host,q:Value)->Value{serde_json::from_str(&host.client_bed_defender_query(&q.to_string())).unwrap_or(Value::Null)}
fn rotation(v:&Value)->Option<[f32;2]>{serde_json::from_value(v.clone()).ok()}
pub(crate) fn slot(host:&mut dyn Host,s:&Settings,_target:Option<&Value>)->Option<u8>{let facts=query(host,json!({"op":"slots"}));let slots=facts["slots"].as_array()?.iter().filter_map(|v|Some(core::Slot{index:v["index"].as_u64()?as u8,count:v["count"].as_i64()?as i32,hardness:v["hardness"].as_f64()?as f32,full:v["full"].as_bool()?,chest:v["chest"].as_bool()?})).collect::<Vec<_>>();core::slot(s,&slots,facts["selected"].as_u64()?as u8)}
impl LiquidBounceHooks{
    pub(crate) fn bed_defender_running(&self)->bool{let state=self.state();let running=state.behaviors.borrow().query_running(&state.registry,"BedDefender");running}
    pub(crate) fn bed_defender_run(&self,host:&mut dyn Host,tasks:Vec<DeferredAction>){for action in tasks{match action{
        DeferredAction::Place{requester,target,normal:_,send}=>{
            if requester!=BED_DEFENDER_REQUESTER||!self.bed_defender_running(){continue;}
            let(s,settings,wait)={let state=self.state();let b=state.behaviors.borrow();(b.bed_defender.settings.clone(),b.bed_defender.placer.settings.clone(),b.bed_defender.placer.queue.ticks_to_wait)};
            if wait>0{continue;}
            let Some(planned)=rotation(&target["rotation"])else{continue;};
            if send&&settings.send_rotation_packet{native(host,json!({"op":"rotationPacket","rotation":planned}));}
            let verify=if settings.rotation_mode=="Normal"{self.block_placer_rotations(host).1}else{Rotation::new(planned[0],planned[1])};
            let slot=slot(host,&s,Some(&target["placed"]));
            if let Some(slot)=slot{native(host,json!({"op":"execute","requester":requester,"target":target,"rotation":[verify.yaw,verify.pitch],"slot":slot,"reset":settings.slot_reset_delay,"swing":settings.swing,"constructFailResult":settings.construct_fail_result,"range":settings.range,"walls":settings.wall_range}));}
            self.state().behaviors.borrow_mut().bed_defender.placer.queue.ran_action=true;
        }
        DeferredAction::Crystal{requester,rotation,send}=>{
            if requester!=BED_DEFENDER_REQUESTER||!self.bed_defender_running(){continue;}
            if send&&self.state().behaviors.borrow().bed_defender.placer.crystals.send_rotation_packet{native(host,json!({"op":"rotationPacket","rotation":rotation}));}
            self.bed_defender_attack_crystal(host);
        }
    }}}
    pub(crate) fn bed_defender_request(&self,crystal:bool,angles:[f32;2]){
        let s=self.state();let b=s.behaviors.borrow();let r=&b.bed_defender.placer;let settings=if crystal{&r.crystal_rotations}else{&r.rotations};
        b.rotations.lock().expect("RotationManager").request_processed(if crystal{CRYSTAL_PROVIDER}else{PLACEMENT_PROVIDER},if crystal{35}else{-20},DefaultRotationTarget{rotation:Rotation::new(angles[0],angles[1]),horizontal_turn_speed:180.,vertical_turn_speed:180.,ticks_until_reset:settings.ticks_until_reset,reset_threshold:settings.reset_threshold,consider_inventory:if crystal{!r.crystals.ignore_open_inventory}else{r.settings.ignore&1==0},movement_correction:settings.movement_correction},(if crystal{CRYSTAL_PROCESSOR}else{PLACEMENT_PROCESSOR})+settings.active.processor_id(),true);
    }
    pub(crate) fn bed_defender_reached(&self,host:&mut dyn Host){
        if !self.bed_defender_running(){return;}
        let (current,server)=self.block_placer_rotations(host);
        let (processor,pending,crystal_pending,s,cs)={let state=self.state();let b=state.behaviors.borrow();let r=&b.bed_defender.placer;let processed=b.rotations.lock().expect("RotationManager").processed_request;(processed,r.pending_placement.clone(),r.pending_crystal,r.settings.clone(),r.crystals.clone())};
        if processor.is_some_and(|p|p>=PLACEMENT_PROCESSOR&&p<PLACEMENT_PROCESSOR+16){if let(Some(current),Some(target))=(current,pending){if native(host,json!({"op":"reached","target":target,"rotation":[current.yaw,current.pitch],"range":s.range,"walls":s.wall_range}))==true{let state=self.state();let mut b=state.behaviors.borrow_mut();b.bed_defender.placer.pending_placement=None;b.block_placer_executor.add(DeferredAction::Place{requester:BED_DEFENDER_REQUESTER,target,normal:true,send:false},s.normal_post_move,true);}}}
        if processor.is_some_and(|p|p>=CRYSTAL_PROCESSOR&&p<CRYSTAL_PROCESSOR+16){if let Some((captured_id,rotation))=crystal_pending{if native(host,json!({"op":"crystalReached","id":captured_id,"rotation":[server.yaw,server.pitch],"range":cs.range,"walls":cs.wall_range}))==true{let state=self.state();let mut b=state.behaviors.borrow_mut();b.bed_defender.placer.pending_crystal=None;b.block_placer_executor.add(DeferredAction::Crystal{requester:BED_DEFENDER_REQUESTER,rotation,send:false},cs.normal_post_move,true);}}}
    }
    pub(crate) fn bed_defender_crystal_tick(&self,host:&mut dyn Host){
        if !self.bed_defender_running(){return;}
        let now=crate::behaviors::current_time_millis();let(cs,id,elapsed)={let state=self.state();let b=state.behaviors.borrow();let r=&b.bed_defender.placer;(r.crystals.clone(),r.destroyer.target.map(|c|c.id),r.destroyer.elapsed(now,r.crystals.delay))};
        let Some(id)=id.filter(|_|cs.enabled&&elapsed)else{return;};
        let look=self.block_placer_rotations(host).0.unwrap_or_else(||{let [y,p]=host.client_player_rotation();Rotation::new(y,p)});
        let facts=native(host,json!({"op":"crystal","id":id,"range":cs.range,"walls":cs.wall_range,"look":[look.yaw,look.pitch]}));
        if facts["wouldKill"]==true{self.state().behaviors.borrow_mut().bed_defender.placer.destroyer.target=None;return;}
        let Some(rotation)=rotation(&facts["rotation"])else{return;};
        if cs.rotation_mode=="Normal"{
            let server=self.block_placer_rotations(host).1;
            if cs.normal_instant&&native(host,json!({"op":"crystalReached","id":id,"rotation":[server.yaw,server.pitch],"range":cs.range,"walls":cs.wall_range}))==true{self.bed_defender_attack_crystal(host);return;}
            self.state().behaviors.borrow_mut().bed_defender.placer.pending_crystal=Some((id,rotation));self.bed_defender_request(true,rotation);
        }else{let task=DeferredAction::Crystal{requester:BED_DEFENDER_REQUESTER,rotation,send:true};if cs.none_instant{self.bed_defender_run(host,vec![task]);}else{self.state().behaviors.borrow_mut().block_placer_executor.add(task,cs.none_post_move,false);}}
    }
    fn bed_defender_attack_crystal(&self,host:&mut dyn Host){
        let now=crate::behaviors::current_time_millis();let(cs,id,elapsed)={let state=self.state();let b=state.behaviors.borrow();let r=&b.bed_defender.placer;(r.crystals.clone(),r.destroyer.target.map(|c|c.id),r.destroyer.elapsed(now,r.crystals.delay))};
        let Some(id)=id.filter(|_|elapsed)else{return;};
        let look=self.block_placer_rotations(host).0.unwrap_or_else(||{let [y,p]=host.client_player_rotation();Rotation::new(y,p)});
        let facts=native(host,json!({"op":"crystal","id":id,"range":cs.range,"walls":cs.wall_range,"look":[look.yaw,look.pitch]}));
        if facts["wouldKill"]==true{self.state().behaviors.borrow_mut().bed_defender.placer.destroyer.target=None;return;}
        if facts.is_null(){return;}
        // attackEntity may be canceled by another module. The source still
        // resets its chronometer and clears its target after that call.
        native(host,json!({"op":"crystalAttack","id":id,"swing":cs.swing_mode}));self.state().behaviors.borrow_mut().bed_defender.placer.destroyer.attacked(now);
    }
    pub(crate) fn bed_defender_success(&self,requester:u64,position:[i32;3],bounds:[f64;6]){if requester==BED_DEFENDER_REQUESTER{self.state().behaviors.borrow_mut().bed_defender.placer.success(position,bounds,crate::behaviors::current_time_millis());}}
    pub(crate) fn bed_defender_removed(&self,id:i32){if !self.bed_defender_running(){return;}let state=self.state();let mut b=state.behaviors.borrow_mut();if b.bed_defender.placer.crystals.enabled{b.bed_defender.placer.destroyer.removed(id);}}
    pub(crate) fn bed_defender_prepare(&self,host:&mut dyn Host){
        if !self.bed_defender_running(){return;}let inventory=host.client_inventory_open();let look=host.client_player_rotation();let using=native(host,json!({"op":"using"}))==true;
        let mut r=std::mem::take(&mut self.state().behaviors.borrow_mut().bed_defender);let now=crate::behaviors::current_time_millis();
        let range=r.placer.settings.range.max(r.placer.settings.wall_range);
        if r.placer.settings.can_act(inventory,using){
            if r.settings.requires_sneak&&query(host,json!({"op":"status"}))["sneaking"]!=true{r.placer.clear(now);}
            else if slot(host,&r.settings,None).is_some(){
                let snapshot=query(host,json!({"op":"targets","radius":range+r.settings.max_layers as f32+1.}));
                let beds=snapshot["beds"].as_array().into_iter().flatten().filter_map(Bed::from_json).collect::<Vec<_>>();
                let eye=serde_json::from_value(snapshot["eye"].clone()).unwrap_or([0.;3]);let armor=serde_json::from_value(snapshot["armor"].clone()).unwrap_or([None;4]);
                let positions=core::targets(&r.settings,&r.self_bed,&beds,&armor,eye,f64::from(range));
                if positions.is_empty(){r.placer.clear(now);}else{r.placer.update(&positions,now);}
            }
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
                    tasks.push((DeferredAction::Place{requester:BED_DEFENDER_REQUESTER,target,normal:false,send:true},s.none_post_move));placements+=1;if placements==s.placements{break;}
                }
            }
        }
        self.state().behaviors.borrow_mut().bed_defender=r;
        {let state=self.state();let mut b=state.behaviors.borrow_mut();for(task,post)in tasks{b.block_placer_executor.add(task,post,false);}}
        if let Some(rotation)=request{self.bed_defender_request(false,rotation);}
    }
    pub(crate) fn bed_defender_event(&self,event:&mut horizon_client::mod_api::ClientBedDefenderEvent<'_>){
        let Ok(input)=serde_json::from_str::<Value>(event.input)else{return;};if !self.bed_defender_running(){return;}let state=self.state();let notice={let mut b=state.behaviors.borrow_mut();let m=&mut b.bed_defender.self_bed;
            match input["action"].as_str().unwrap_or(""){
                "options"=>{*event.output=m.settings.options().to_string();None},
                "position"=>{if let(Ok(old),Ok(raw))=(serde_json::from_value(input["old"].clone()),serde_json::from_value(input["raw"].clone())){m.packet_position(old,raw);}None},
                "key"=>{let eye=serde_json::from_value(input["eye"].clone()).unwrap_or([0.;3]);let beds=serde_json::from_value::<Vec<[i32;3]>>(input["beds"].clone()).unwrap_or_default();m.key(input["key"].as_str().unwrap_or(""),eye,&beds)},_=>None
            }
        };if let Some(n)=notice{self.bed_defender_notice(n);}
    }
    pub(crate) fn bed_defender_world_change(&self){let notice={let state=self.state();let mut b=state.behaviors.borrow_mut();b.bed_defender.placer.reset_world();if b.query_running(&state.registry,"BedDefender"){b.bed_defender.self_bed.world_change()}else{None}};if let Some(n)=notice{self.bed_defender_notice(n);}}
    fn bed_defender_notice(&self,n:Notice){
        let locale=liquidbounce_interop::localization::Locale::from_game_language(&horizon_client::lang::code());
        let message=if locale==liquidbounce_interop::localization::Locale::English{n.english()}else{let mut text=liquidbounce_interop::localization::describe(locale.code(),&format!("liquidbounce.module.bedDefender.notifications.{}",n.kind));for(name,value)in["x","y","z"].into_iter().zip(n.pos){text=text.replace(&format!("{{{name}}}"),&value.to_string());}text};
        #[cfg(feature="webview")]
        self.state().events.borrow_mut().push(("notification".into(),json!({"title":if locale==liquidbounce_interop::localization::Locale::English{"SelfBed-Manual".into()}else{liquidbounce_interop::localization::describe(locale.code(),"liquidbounce.module.bedDefender.notifications.title")},"message":message,"severity":n.severity()})));
    }
}
