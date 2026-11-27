// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{runtime::LiquidBounceHooks,block_placer_runtime::{self as placer,BlockPlacerRuntime,DeferredAction,LIQUID_FILLER_REQUESTER}};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{liquid_filler::Settings,block_placer::Crystal,random::random_i32_inclusive,rotations::Rotation};
use serde_json::{Value,json};
#[derive(Clone,Debug,Default)]
pub struct LiquidFillerRuntime{pub settings:Settings,pub placer:BlockPlacerRuntime}
pub(crate) fn settings(s:&Settings)->Value{json!({"placeIn":s.place_in,"order":s.order.tag(),"useSponge":s.use_sponge,"whitelist":s.whitelist,"blocks":s.blocks})}
fn query(host:&mut dyn Host,q:Value)->Value{serde_json::from_str(&host.client_liquid_filler_query(&q.to_string())).unwrap_or(Value::Null)}
pub(crate) fn slot(host:&mut dyn Host,s:&Settings,target:Option<&Value>)->Option<u8>{query(host,json!({"op":"slot","settings":settings(s),"target":target})).as_u64().and_then(|v|u8::try_from(v).ok())}
impl LiquidBounceHooks{
    pub(crate) fn liquid_filler_running(&self)->bool{let s=self.state();let running=s.behaviors.borrow().query_running(&s.registry,"LiquidFiller");running}
    pub(crate) fn liquid_filler_prepare(&self,host:&mut dyn Host){
        if !self.liquid_filler_running(){return;}
        let inventory=host.client_inventory_open();let look=host.client_player_rotation();
        let mut r=std::mem::take(&mut self.state().behaviors.borrow_mut().liquid_filler);
        let now=crate::behaviors::current_time_millis();
        let targets=query(host,json!({"op":"targets","settings":settings(&r.settings),"range":r.placer.settings.range.max(r.placer.settings.wall_range)}));
        if targets["available"]==false{if !r.placer.queue.blocks.is_empty(){r.placer.clear(now);}}
        else if let Ok(positions)=serde_json::from_value::<Vec<[i32;3]>>(targets["positions"].clone()){r.placer.update(&positions,now);}
        let s=&r.placer.settings;
        r.placer.queue.tick_start(||random_i32_inclusive(s.cooldown[0],s.cooldown[1]));
        let using=placer::native(host,json!({"op":"using"}))==true;
        let mut tasks=vec![];let mut request=None;
        if s.can_act(inventory,using)&&r.placer.queue.ticks_to_wait<=0&&!r.placer.queue.blocks.is_empty(){
            if let Some(default_slot)=slot(host,&r.settings,None){
                r.placer.queue.inaccessible.clear();let mut placements=0;
                let blocks=r.placer.queue.blocks.clone();
                let current=self.block_placer_rotations(host).0.unwrap_or(Rotation::new(look[0],look[1]));
                // suggestTarget compares both distances at the current eye.
                if let Some(old)=r.placer.destroyer.target{let facts=placer::native(host,json!({"op":"crystal","id":old.id,"range":r.placer.crystals.range,"walls":r.placer.crystals.wall_range,"look":[current.yaw,current.pitch]}));if let Some(distance)=facts["distanceSquared"].as_f64(){r.placer.destroyer.target=Some(Crystal{id:old.id,distance_squared:distance});}}
                for pos in blocks{
                    let plan=placer::native(host,json!({"op":"plan","position":pos,"slot":default_slot,"rotation":[current.yaw,current.pitch],"range":s.range,"walls":s.wall_range}));
                    if plan["inaccessible"]==true{r.placer.queue.inaccessible.push(pos);continue;}
                    if plan["blocked"]==true{
                        if r.placer.crystals.enabled{if let Some(id)=plan["crystal"]["id"].as_i64(){let facts=placer::native(host,json!({"op":"crystal","id":id,"range":r.placer.crystals.range,"walls":r.placer.crystals.wall_range,"look":[current.yaw,current.pitch]}));if let Some(distance)=facts["distanceSquared"].as_f64(){r.placer.destroyer.suggest(Crystal{id:id as i32,distance_squared:distance},facts["reachable"]==true);}}}
                        continue;
                    }
                    let target=plan["target"].clone();let Ok(rotation)=serde_json::from_value::<[f32;2]>(target["rotation"].clone())else{continue;};
                    if plan["interactable"]==true{r.placer.queue.sneak_times=random_i32_inclusive(s.sneak[0],s.sneak[1]);}
                    if s.rotation_mode=="Normal"{r.placer.pending_placement=Some(target);request=Some(rotation);break;}
                    tasks.push((DeferredAction::Place{requester:LIQUID_FILLER_REQUESTER,target,normal:false,send:true},s.none_post_move));placements+=1;if placements==s.placements{break;}
                }
            }
        }
        self.state().behaviors.borrow_mut().liquid_filler=r;
        {let state=self.state();let mut b=state.behaviors.borrow_mut();for(task,post)in tasks{b.block_placer_executor.add(task,post,false);}}
        if let Some(rotation)=request{self.block_placer_request(false,rotation);}
    }
}
