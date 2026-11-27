// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared BlockPlacer native target, obstruction, click, movement-packet and crystal services.
use super::{Game,Hand,block_placement::{self as placement,Target}};
use horizon_core::geometry::{Aabb,Vec3};
use horizon_world::level::classes::Class;
use serde_json::{Value,json};
fn target(t:&Target)->Value{json!({"interacted":t.interacted,"placed":t.placed,"direction":t.direction,"point":t.point,"minY":t.min_y,"rotation":t.rotation})}
fn read(v:&Value)->Option<Target>{Some(Target{interacted:serde_json::from_value(v["interacted"].clone()).ok()?,placed:serde_json::from_value(v["placed"].clone()).ok()?,direction:serde_json::from_value(v["direction"].clone()).ok()?,point:serde_json::from_value(v["point"].clone()).ok()?,min_y:v["minY"].as_f64()?,rotation:serde_json::from_value(v["rotation"].clone()).ok()?})}
fn area(p:[i32;3])->Aabb{Aabb::new(Vec3::new(p[0]as f64,p[1]as f64,p[2]as f64),Vec3::new(p[0]as f64+1.,p[1]as f64+1.,p[2]as f64+1.))}
fn can_reach(g:&Game<'_>,p:[i32;3],rotation:[f32;2],range:f64,walls:f64)->bool{
    let Some(eye)=g.eye_position()else{return false;};
    super::liquid_filler_core::distance(p,eye)<=walls*walls || placement::trace_world(g,rotation,range).is_some_and(|h|h.pos==p)
}
fn interactable(g:&Game<'_>,p:[i32;3])->bool{
    let state=g.world.block(p[0],p[1],p[2]);let b=g.res.behaviours();let data=b.data(state);let is=|c|data.is(c);
    let boolean=|n:&str|b.table.key_of(n).is_some_and(|k|b.table.bool(state,k));
    let integer=|n:&str|b.table.key_of(n).map_or(0,|k|b.table.int(state,k));
    [Class::AbstractBed,Class::AbstractChest,Class::AbstractFurnace,Class::Anvil,Class::Barrel,Class::Beacon,Class::Bell,Class::BrewingStand,Class::Button,Class::CandleCake,Class::CartographyTable,Class::Comparator,Class::Crafter,Class::CraftingTable,Class::DaylightDetector,Class::DecoratedPot,Class::Dispenser,Class::Door,Class::DragonEgg,Class::EnchantingTable,Class::FenceGate,Class::FlowerPot,Class::Grindstone,Class::Hopper,Class::Lectern,Class::Lever,Class::Note,Class::RedstoneWire,Class::Repeater,Class::RespawnAnchor,Class::ShulkerBox,Class::Stonecutter,Class::TrapDoor].into_iter().any(is)
        || is(Class::Cake)&&g.player.as_ref().is_some_and(|p|p.health.food.needs_food())
        || (is(Class::CaveVines)||is(Class::CaveVinesPlant))&&boolean("berries")
        || is(Class::Composter)&&integer("level")==8
        || is(Class::Jukebox)&&boolean("has_record")
        || is(Class::SweetBerryBush)&&integer("age")>1
        || [Class::Command,Class::Structure,Class::Jigsaw,Class::TestBlock,Class::TestInstanceBlock,Class::Light].into_iter().any(is)&&g.can_use_game_master_blocks()
}
fn crystal(g:&Game<'_>,id:i32)->Option<Value>{
    let c=g.crystals.iter().find(|c|c.id as i32==id)?;let p=c.position;let box_=c.bounds();let lo=box_.min_position();let hi=box_.max_position();let eye=g.eye_position()?;
    Some(json!({"id":id,"position":[p.x,p.y,p.z],"bounds":[lo.x,lo.y,lo.z,hi.x,hi.y,hi.z],"distanceSquared":(p.x-eye[0]).powi(2)+(p.y-eye[1]).powi(2)+(p.z-eye[2]).powi(2)}))
}
impl Game<'_>{
    pub(super) fn block_placer_query(&mut self,text:&str)->Option<String>{
        let q:Value=serde_json::from_str(text).ok()?;let range=q["range"].as_f64().unwrap_or(4.5);let walls=q["walls"].as_f64().unwrap_or(4.5);
        match q["op"].as_str()?{
            "using"=>Some(json!(self.using.is_some()).to_string()),
            "plan"=>{
                let pos=serde_json::from_value(q["position"].clone()).ok()?;
                if !super::client_liquid_filler::state(self,pos).replaceable{return Some(json!({"inaccessible":true}).to_string());}
                let a=area(pos);
                let blocked=self.with_level(|_,l,_|l.building_blocker_boxes().into_iter().any(|b|b.intersects(a)));
                if blocked {
                    // EntitySectionStorage walks signed X, then the packed
                    // unsigned Z/Y section key. The first crystal wins even
                    // when an earlier non-crystal blocks the placement.
                    let mut crystals=self.crystals.iter().enumerate().filter(|(_,c)|c.bounds().intersects(a)).map(|(i,c)|{let p=c.position;(((p.x.floor()as i32)>>4,((p.z.floor()as i32)>>4)as u32&0x3f_ffff,((p.y.floor()as i32)>>4)as u32&0xf_ffff),i,c.id as i32)}).collect::<Vec<_>>();
                    crystals.sort_by_key(|(s,i,_)|(*s,*i));
                    return Some(json!({"blocked":true,"crystal":crystals.first().and_then(|(_,_,id)|crystal(self,*id))}).to_string());
                }
                let slot=q["slot"].as_u64()? as usize;let stack=self.stack_in(slot)?.clone();let rotation=serde_json::from_value(q["rotation"].clone()).ok()?;
                let location=placement::capture_location(self,None,rotation)?;let mut options=placement::Options::center(&stack,location);options.consider_facing_away=walls>0.;
                let t=placement::find(self,pos,&mut options)?;
                if !can_reach(self,t.interacted,t.rotation,range,walls){return None;}
                Some(json!({"target":target(&t),"interactable":interactable(self,t.interacted)}).to_string())
            }
            "reached"=>{
                let t=read(&q["target"])?;let rotation=serde_json::from_value(q["rotation"].clone()).ok()?;
                Some(json!(placement::raytrace_block(self,t.interacted,rotation,range.max(walls)).is_some_and(|h|h.pos==t.interacted)).to_string())
            }
            "execute"=>{
                let t=read(&q["target"])?;let rotation=serde_json::from_value(q["rotation"].clone()).ok()?;
                if !can_reach(self,t.interacted,rotation,range,walls){return Some("false".into());}
                let failed=if q["constructFailResult"]==true{placement::FailedClick::PlannedHit}else{placement::FailedClick::Nothing};
                let hit=placement::verify(self,&t,rotation,range.max(walls),placement::ClickTrace::TargetShape,failed)?;
                let slot=q["slot"].as_u64()?as usize;
                let delay:[i32;2]=serde_json::from_value(q["reset"].clone()).ok()?;let delay=super::liquid_filler_random::random_i32_inclusive(delay[0],delay[1]);
                if slot!=40 && !self.native_client_select_slot_silently(q["requester"].as_u64()?,slot as i32,delay).ok()?{return Some("false".into());}
                let stack=self.stack_in(slot)?;let b=self.res.behaviours();
                let block_item=b.item_binding(&stack.item).is_some_and(|i|!matches!(i.kind,horizon_world::level::data::ItemKind::SolidBucket|horizon_world::level::data::ItemKind::Other));
                if block_item && !super::client_liquid_filler::state(self,t.placed).replaceable{return Some("false".into());}
                let requester=q["requester"].as_u64()?;let accepted=self.client_do_placement(hit,rotation,if slot==40{Hand::Off}else{Hand::Main},q["swing"].as_str()?,|g,_|{g.native_client_block_placer_success(requester,t.placed);true});
                Some(json!(accepted).to_string())
            }
            "rotationPacket"=>{
                let [yaw,pitch]:[f32;2]=serde_json::from_value(q["rotation"].clone()).ok()?;
                let normalize=|a:f32|{let a=a%360.;if a>=180.{a-360.}else if a< -180.{a+360.}else{a}};
                let p=self.player.as_ref()?;
                self.send(horizon_protocol::packets::play::Serverbound::MovePlayerRot(horizon_protocol::packets::play::MovePlayerRot{yaw:normalize(yaw),pitch:normalize(pitch),flags:u8::from(p.ground.on_ground)|u8::from(p.collision.horizontal)<<1}));Some("true".into())
            }
            "crystal"=>{
                let id=q["id"].as_i64()?as i32;let mut v=crystal(self,id)?;
                let c=&v["bounds"];let eye=self.eye_position()?;let look: [f32;2]=serde_json::from_value(q["look"].clone()).unwrap_or([self.yaw,self.pitch]);let direction=horizon_world::view_direction::view_vector(look[1],look[0]);let view=[direction.x,direction.y,direction.z];let point=std::array::from_fn::<_,3,_>(|i|eye[i]+range*view[i]);
                let ray:Value=serde_json::from_str(&self.aimbot_query(&json!({"op":"raytrace","bounds":c,"point":point,"range":range,"walls":walls}).to_string())).ok()?;
                if ray.is_array(){v["rotation"]=ray.clone();v["reachable"]=json!(true);}else{v["reachable"]=json!(false);}
                v["wouldKill"]=json!(self.block_placer_crystal_would_kill(id)?);Some(v.to_string())
            }
            "crystalReached"=>{
                let id=q["id"].as_i64()?;let hit:Value=serde_json::from_str(&self.kill_aura_query(&json!({"op":"hit","id":id,"rotation":q["rotation"],"range":range,"walls":walls}).to_string())).ok()?;Some(json!(hit["hit"]==true).to_string())
            }
            "crystalAttack"=>{
                let id=q["id"].as_i64()?as i32;if self.block_placer_crystal_would_kill(id)?{return Some("false".into());}
                let swing=match q["swing"].as_str()?{"HideForBoth"=>1,"HideForClient"=>2,"HideForServer"=>3,_=>0};
                Some(json!(self.kill_aura_attack_with_swing(id as u32,false,false,swing)).to_string())
            }
            _=>None
        }
    }
}
