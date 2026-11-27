// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual projectiles, held components, COLLIDER rays, hit faces and original indicators.
use super::{Game,indicator::{self,Arrow,Entry,Primitive},trajectories_math::{self as math,Hit,Info,Simulation,V3,World}};
use horizon_client::{items::ItemStack,mobs::Who,hud::HudBatch,text::Text,world_primitives::{WorldPrimitives,PrimitiveShape}};
use horizon_core::geometry::{Vec3,Aabb};
use serde_json::{Value,json};
use std::cell::RefCell;
fn array(p:Vec3)->V3{[p.x,p.y,p.z]}
fn point(p:V3)->Vec3{Vec3::new(p[0],p[1],p[2])}
fn corners(b:Aabb)->[V3;2]{[array(b.min_position()),array(b.max_position())]}
#[derive(Clone)]
struct Target{id:u32,position:V3,interpolated:V3,bounds:[V3;2],living:bool,potion:bool}
struct Display{result:Simulation,owner:u32,display_owner:Option<u32>,owner_position:V3,icon:Option<ItemStack>,real:bool}
thread_local!{static LABELS:RefCell<(Value,Vec<Display>)>=const{RefCell::new((Value::Null,Vec::new()))};}
struct Scene<'a,'r>{game:&'a Game<'r>,targets:&'a[Target]}
impl World for Scene<'_,'_>{
    fn min_y(&self)->f64{f64::from(self.game.world.shape.min_y)}
    fn nonempty_fluid(&self,p:V3)->bool{
        let s=self.game.world.block(p[0].floor()as i32,p[1].floor()as i32,p[2].floor()as i32);
        self.game.res.facts().get(s).fluid!=horizon_client::blocks::Fluid::None
    }
    fn block_hit(&self,a:V3,b:V3,owner:u32)->Option<Hit>{super::trajectories_ray::clip(self.game,point(a),point(b),owner as i32,false)}
    fn entity_hit(&self,a:V3,b:V3,owner:u32,radius:f64)->Option<Hit>{
        let source_root=self.game.client_reach_root_vehicle(owner);
        let margin=self.game.creatures.projectiles.iter().find(|p|p.id==owner).map_or(0.,|p|f64::from(((p.age as i32-2)as f32/20.).clamp(0.,0.3)));
        let mut nearest=None;let mut best=f64::MAX;
        for target in self.targets{
            if target.id==owner||self.game.client_reach_root_vehicle(target.id)==source_root{continue;}
            // ProjectileUtil selects using its own margin, then derives the visible hit from the trajectory radius.
            if let Some(at)=math::clip_aabb(a,b,target.bounds,margin){let d=math::distance(a,at);if d<best{best=d;let position=math::clip_aabb(a,b,target.bounds,radius).unwrap_or(at);nearest=Some(Hit::Entity{position,id:target.id,bounds:target.bounds});}}
        }
        nearest
    }
}
fn box_geometry(out:&mut WorldPrimitives,b:[V3;2],color:u32){
    let c:[V3;8]=std::array::from_fn(|i|std::array::from_fn(|j|b[usize::from(i&(1<<j)!=0)][j]));
    for face in [[0,1,3,2],[4,6,7,5],[0,4,5,1],[2,3,7,6],[0,2,6,4],[1,5,7,3]]{out.push(PrimitiveShape::LiquidBounceQuad{corners:face.map(|i|c[i]),colors:[color;4]}).through_walls();}
    for(a,b)in [(0,1),(0,2),(0,4),(1,3),(1,5),(2,3),(2,6),(3,7),(4,5),(4,6),(5,7),(6,7)]{out.line(c[a],c[b],color,1.).through_walls();}
}
impl Game<'_>{
    fn client_trajectories_options(&self)->Value{let mut out=String::new();self.native_client_trajectories("{\"phase\":\"options\"}",&mut out);serde_json::from_str(&out).unwrap_or(Value::Null)}
    pub(super) fn client_trajectories_reset(&self){LABELS.with_borrow_mut(|v|*v=(Value::Null,vec![]));let mut out=String::new();self.native_client_trajectories("{\"phase\":\"reset\"}",&mut out);}
    fn client_trajectories_owner(&self,owner:Option<Who>,guest:Option<u32>)->Option<u32>{match owner?{Who::Mob(id)=>Some(id),Who::Player=>guest.and_then(|id|self.lan.as_ref()?.hub.guests.iter().find(|g|g.conn==id).map(|g|g.entity_id as u32)).or_else(||Some(self.remote.as_ref().map_or(0,|r|r.entity_id)as u32))}}
    fn client_trajectories_stack(&self,s:&ItemStack)->Value{
        let view=horizon_client::item_render::stack_view(s);let c=self.pack.item_renderer.models.components_view(&view);
        json!({"id":s.item.as_ref(),"charged":c.get("minecraft:charged_projectiles").cloned().unwrap_or(json!([])),"multishot":s.enchantment_level("minecraft:multishot"),"color":c.get("minecraft:potion_contents").and_then(|c|c.get("custom_color")).cloned().or_else(||s.potion().map(|p|json!(horizon_client::brewing::potion_color(Some(p))))).unwrap_or(Value::Null)})
    }
    fn client_trajectories_targets(&self,alpha:f64)->Vec<Target>{
        use super::entity::{Entity as _,EntityHandle,with_entity};
        let mut out=vec![];
        for h in self.entity_handles(){
            // Entity.isPickable: live living entities, vehicles and hittable projectile kinds.
            let pickable=match h{EntityHandle::Mob(i)=>!self.mobs[i].extra.spectator,EntityHandle::Boat(_)|EntityHandle::Cart(_)=>true,EntityHandle::ArmorStand(i)=>!self.armor_stands[i].marker,EntityHandle::Projectile(i)=>super::deflect::attackable(&self.creatures.projectiles[i]),_=>false};
            if !pickable{continue;}
            let(id,at,b,alive,living)=with_entity!(self,h,e=>(e.id(),array(e.position()),corners(e.bounds()),e.is_alive()&&!e.is_removed(),e.is_living()));if !alive{continue;}
            let old=self.mobs.iter().find(|m|m.id==id).map_or(at,|m|array(m.previous));
            let interpolated=std::array::from_fn(|i|old[i]+(at[i]-old[i])*alpha);
            out.push(Target{id,position:at,interpolated,bounds:b,living,potion:living&&!self.mobs.iter().any(|m|m.id==id&&(m.kind.id.as_str()=="minecraft:armor_stand"||m.kind.id.as_str()=="minecraft:wither"||m.kind.id.as_str()=="minecraft:ender_dragon"))});
        }
        let mut offer=|id:u32,p:V3,b:[V3;2]|out.push(Target{id,position:p,interpolated:p,bounds:b,living:false,potion:false});
        for f in &self.item_frames{let(a,b)=f.bounds();offer(f.id,std::array::from_fn(|i|(a[i]+b[i])*0.5),[a,b]);}
        for p in &self.paintings{let(a,b)=p.bounds();offer(p.id,std::array::from_fn(|i|(a[i]+b[i])*0.5),[a,b]);}
        for p in &self.cushions{let(a,b)=p.bounds();offer(p.id,array(p.position),[a,b]);}
        for(&id,p)in &self.interactions{offer(id as u32,array(p.position),corners(p.bounds()));}
        if let Some(p)=self.player.as_ref(){if !self.spectator(){out.push(Target{id:self.remote.as_ref().map_or(0,|r|r.entity_id)as u32,position:array(p.position),interpolated:array(p.position),bounds:corners(p.bounds()),living:true,potion:true});}}
        for g in self.lan.iter().flat_map(|l|l.hub.guests.iter()).filter(|g|g.placed&&g.dimension==self.dim.id()){if g.spectator(){continue;}out.push(Target{id:g.entity_id as u32,position:g.position,interpolated:std::array::from_fn(|i|g.previous_position[i]+(g.position[i]-g.previous_position[i])*alpha),bounds:corners(g.bounds()),living:true,potion:true});}
        out
    }
    fn client_trajectories_hit(&self,out:&mut WorldPrimitives,hit:&Hit,color:u32,targets:&[Target],local:u32){
        match hit{
            Hit::Block{position,block,normal}=>{
                let boxes=self.outline_boxes(*block);let best=boxes.iter().filter(|b|(0..3).all(|i|position[i]>=b[i]-0.01&&position[i]<=b[i+3]+0.01)).min_by(|a,b|{
                    let d=|v:&&[f64;6]|(0..3).map(|i|((v[i]+v[i+3])*0.5-position[i]).powi(2)).sum::<f64>();d(a).total_cmp(&d(b))
                });
                if let Some(b)=best{let axis=normal.iter().position(|v|*v!=0).unwrap_or(1);let others=(0..3).filter(|i|*i!=axis).collect::<Vec<_>>();let c=std::array::from_fn(|i|{let mut p=[0.;3];p[axis]=b[axis+if normal[axis]>0{3}else{0}];p[others[0]]=b[others[0]+if i==1||i==2{3}else{0}];p[others[1]]=b[others[1]+if i>=2{3}else{0}];p});out.push(PrimitiveShape::LiquidBounceQuad{corners:c,colors:[color;4]}).through_walls();}
            }
            Hit::Entity{id,bounds,..}=>{if *id!=local{let b=targets.iter().find(|t|t.id==*id).map_or(*bounds,|t|{let d=math::sub(t.interpolated,t.position);t.bounds.map(|p|math::add(p,d))});box_geometry(out,b,color);}}
        }
    }
    fn client_trajectories_draw(&self,out:&mut WorldPrimitives,settings:&math::Trajectories,targets:&[Target],display:Display,kind:&str,offset:V3,color:u32,results:&mut Vec<Display>,snapshots:&mut Vec<Value>){
        let width=settings.number(if display.real{"ActiveLineWidth"}else{"LineWidth"})as f32;
        for segment in display.result.positions.windows(2){out.line(math::add(segment[0],offset),math::add(segment[1],offset),color,width).through_walls();}
        let local=self.remote.as_ref().map_or(0,|r|r.entity_id)as u32;
        if let Some(hit)=display.result.hit.as_ref(){let hit_color=if display.real{color}else if matches!(hit,Hit::Block{..}){0x964677ff}else{0x64ff0000};self.client_trajectories_hit(out,hit,hit_color,targets,local);
            if kind=="Potion"{let at=hit.position();for t in targets.iter().filter(|t|t.living&&t.potion&&t.id!=local&&math::distance(t.position,at)<=4.&&(t.position[1]-at[1]).abs()<=2.25){let d=math::sub(t.interpolated,t.position);box_geometry(out,t.bounds.map(|p|math::add(p,d)),hit_color);}}
        }
        snapshots.push(math::snapshot(&display.result,display.owner,kind,display.real));results.push(display);
    }
    pub(super) fn client_trajectories_geometry(&self,out:&mut WorldPrimitives,alpha:f64,camera:glam::DVec3,view:glam::Mat4){
        LABELS.with_borrow_mut(|v|*v=(Value::Null,vec![]));
        if !self.native_window()||self.player.is_none(){return;}
        let options=self.client_trajectories_options();if options["enabled"]!=true{return;}
        let mut settings=math::Trajectories::default();for(k,v)in options.as_object().into_iter().flatten(){if settings.values.contains_key(k){settings.values.insert(k.clone(),v.clone());}}
        let targets=self.client_trajectories_targets(alpha);let scene=Scene{game:self,targets:&targets};let local=self.remote.as_ref().map_or(0,|r|r.entity_id)as u32;
        let forward=(view.inverse()*glam::Vec4::new(0.,0.,-1.,0.)).truncate().as_dvec3().to_array();let camera=camera.to_array();let max=settings.number("MaxSimulatedTicks")as usize;
        let mut results=vec![];let mut snapshots=vec![];
        let mut real=|id:u32,owner:Option<u32>,pos:Vec3,vel:Vec3,kind:&str,icon:Option<ItemStack>,effect:Option<u32>|{
            if !settings.includes("TrajectoryTypes",kind)||settings.culled(array(pos),camera,forward){return;}
            let owner_position=owner.and_then(|id|self.native_entity(id)).and_then(|e|self.native_entity_position(e)).unwrap_or(array(pos));
            let info=Info::for_type(kind,true);let result=math::simulate(&scene,array(pos),array(vel),info,kind,owner.unwrap_or(id),max);
            let component=effect.or_else(||icon.as_ref().and_then(|s|self.client_trajectories_stack(s)["color"].as_u64().map(|n|n as u32)));
            self.client_trajectories_draw(out,&settings,&targets,Display{result,owner:owner.unwrap_or(id),display_owner:owner,owner_position,icon,real:true},kind,[0.;3],math::color(kind,component),&mut results,&mut snapshots);
        };
        use horizon_client::projectiles::ProjectileKind as K;
        for p in &self.creatures.projectiles{
            let arrow=matches!(p.kind,K::Arrow{..});if arrow&&!settings.includes("Show","ActiveTrajectoryArrow"){continue;}if !arrow&&!settings.includes("Show","ActiveTrajectoryOther"){continue;}
            let kind=match &p.kind{K::Arrow{..}if !p.arrow.as_ref().is_some_and(|a|a.in_ground)=>"Arrow",K::Potion(_,_)=>"Potion",K::Trident if !p.stuck=>"Trident",K::EnderPearl=>"EnderPearl",K::Snowball=>"Snowball",K::Egg=>"Egg",K::ExperienceBottle=>"ExpBottle",K::SmallFireball|K::LargeFireball{..}=>"Fireball",K::WindCharge=>"WindCharge",_=>continue};
            let icon=p.arrow.as_ref().map(|a|a.pickup_item.clone()).or_else(||match &p.kind{K::Potion(s,_)=>Some((**s).clone()),_=>p.item.clone()}).or_else(||match kind{"Snowball"=>Some(ItemStack::new("minecraft:snowball",1)),"Egg"=>Some(ItemStack::new("minecraft:egg",1)),"EnderPearl"=>Some(ItemStack::new("minecraft:ender_pearl",1)),"ExpBottle"=>Some(ItemStack::new("minecraft:experience_bottle",1)),_=>None});
            real(p.id,self.client_trajectories_owner(p.owner,p.guest),p.position,p.velocity,kind,icon,p.arrow.as_ref().filter(|a|a.effect_color!=-1).map(|a|a.effect_color as u32));
        }
        if settings.includes("Show","ActiveTrajectoryOther"){
            for(id,owner,pos,vel,stack)in self.client_trajectories_potions(){let kind=if stack.item.as_ref()=="minecraft:experience_bottle"{"ExpBottle"}else{"Potion"};real(id,owner,pos,vel,kind,Some(stack),None);}
            for t in &self.tridents{if !t.in_ground{real(t.id,Some(if t.owner==0{local}else{t.owner}),t.position,t.velocity,"Trident",Some(t.stack.clone()),None);}}
            for r in &self.rockets{real(r.id,self.client_trajectories_owner(r.owner,r.guest),r.position,r.velocity,"FireworkRocket",Some(r.item.clone()),None);}
            for(id,owner,pos,vel)in self.client_trajectories_hooks(){real(id,Some(owner),pos,vel,"FishingBobber",None,None);}
        }
        drop(real);
        let mut players=vec![(local,true)];if settings.includes("Show","OtherPlayers"){players.extend(self.mobs.iter().filter(|m|!m.removed&&horizon_client::command::args::normalize(&m.kind.id)=="minecraft:player").map(|m|(m.id,false)));players.extend(self.lan.iter().flat_map(|l|l.hub.guests.iter()).filter(|g|g.placed&&g.dimension==self.dim.id()).map(|g|(g.entity_id as u32,false)));}
        for(id,is_local)in players{
            let entity=self.native_entity(id);let pos=if is_local{array(self.player.as_ref().unwrap().position)}else{let Some(pos)=entity.and_then(|e|self.native_entity_position(e))else{continue;};pos};let eye=if is_local{f64::from(self.player.as_ref().unwrap().dimensions.eye_height)}else{entity.and_then(|e|self.native_entity_eye_height(e)).unwrap_or(1.62)};if !is_local&&settings.culled(math::add(pos,[0.,eye,0.]),camera,forward){continue;}
            let mut rotation=entity.and_then(|e|self.native_entity_rotation(e)).unwrap_or([self.yaw,self.pitch]);if is_local{if let Ok(r)=serde_json::from_value(options["rotation"].clone()){rotation=r;}}
            let velocity=if is_local{array(self.player.as_ref().unwrap().velocity)}else{entity.and_then(|e|self.native_entity_velocity(e)).unwrap_or([0.;3])};let ground=if is_local{self.player.as_ref().unwrap().ground.on_ground}else{entity.and_then(|e|self.native_entity_on_ground(e)).unwrap_or(false)};
            let old=if is_local{array(self.previous_position)}else{self.mobs.iter().find(|m|m.id==id).map_or(pos,|m|array(m.previous))};let interpolated=std::array::from_fn(|i|old[i]+(pos[i]-old[i])*alpha);
            let living=entity.and_then(|e|self.native_living(e));let hands=if is_local{[self.stack_in(self.client_hotbar_slot()).cloned(),self.offhand.clone()]}else{[living.and_then(|e|self.native_living_hand_item(e,horizon_client::first_person::Hand::Main)),living.and_then(|e|self.native_living_hand_item(e,horizon_client::first_person::Hand::Off))]};
            for(hand,stack)in hands.into_iter().enumerate(){let Some(stack)=stack.filter(|s|s.count>0)else{continue;};
                if stack.item.as_ref()=="minecraft:fishing_rod"&&settings.includes("Show","ActiveTrajectoryOther")&&settings.includes("TrajectoryTypes","FishingBobber")&&self.client_trajectories_has_hook(id){continue;}
                let use_ticks=if is_local{self.using.and_then(|(slot,ticks)|((hand==1)==(slot==super::shield::OFFHAND_SLOT)).then(||self.ticks_held(&stack,ticks)))}else{self.mobs.iter().find(|m|m.id==id&&m.extra.using_item>0&&m.extra.using_off_hand==(hand==1)).map(|m|m.extra.using_item as u32)};
                let mut stack_facts=self.client_trajectories_stack(&stack);if stack.item.as_ref()=="minecraft:bow"&&is_local{if let Some(ammo)=self.projectile_slot(&stack.item).and_then(|slot|self.stack_in(slot)){stack_facts["ammo"]=self.client_trajectories_stack(ammo);}}if let Some(first)=stack.data().and_then(|d|d.charged.first()){if let Some(charged)=stack_facts["charged"].as_array_mut(){if let Some(f)=charged.first_mut(){*f=self.client_trajectories_stack(first);}}}let shots=math::held_shots(&stack_facts,use_ticks,settings.includes("Show","AlwaysShowBow"),settings.boolean("ShowMultiShot"));if shots.is_empty(){continue;}
                for shot in shots{if !settings.includes("TrajectoryTypes",shot.kind){continue;}let mut angle=rotation;angle[0]+=shot.yaw_offset;let(pos,velocity,offset)=math::hypothetical(pos,eye,velocity,ground,angle,interpolated,shot.info,options["freeze"]==true);let result=math::simulate(&scene,pos,velocity,shot.info,shot.kind,id,max);let icon=Some(stack.clone());let component=shot.color_source["color"].as_u64().map(|n|n as u32);
                    self.client_trajectories_draw(out,&settings,&targets,Display{result,owner:id,display_owner:Some(id),owner_position:if is_local{array(self.player.as_ref().unwrap().position)}else{entity.and_then(|e|self.native_entity_position(e)).unwrap_or(pos)},icon,real:false},shot.kind,offset,math::color(shot.kind,component),&mut results,&mut snapshots);
                }
                break;
            }
        }
        let mut output=String::new();self.native_client_trajectories(&json!({"phase":"results","results":snapshots}).to_string(),&mut output);LABELS.with_borrow_mut(|v|*v=(options,results));
    }
    fn client_trajectories_name(&self,id:u32)->String{self.remote_player_name(id).map(str::to_owned).or_else(||self.lan.as_ref()?.hub.guests.iter().find(|g|g.entity_id as u32==id).map(|g|g.name.clone())).or_else(||self.mobs.iter().find(|m|m.id==id).map(|m|m.command_name().plain())).unwrap_or_default().chars().filter(|c|!c.is_control()&&*c!='§').take(256).collect()}
    pub(super) fn client_trajectories_overlay(&self,batch:&mut HudBatch){
        LABELS.with_borrow(|(options,results)|{
            if options["ShowDetailedInfo.Enabled"]!=true{return;}
            let Some(camera)=self.native_camera()else{return;};let Some(player)=self.player.as_ref()else{return;};let local=self.remote.as_ref().map_or(0,|r|r.entity_id)as u32;
            let curve=options["ShowDetailedInfo.Scale"].as_array().into_iter().flatten().map(|p|[p["x"].as_f64().unwrap_or(0.)as f32,p["y"].as_f64().unwrap_or(1.)as f32]).collect::<Vec<_>>();
            for(index,d)in results.iter().enumerate(){if d.result.positions.is_empty(){continue;}
                let mut settings=math::Trajectories::default();for(k,v)in options.as_object().into_iter().flatten(){if settings.values.contains_key(k){settings.values.insert(k.clone(),v.clone());}}
                let name=d.display_owner.filter(|id|*id!=local).map(|id|self.client_trajectories_name(id));
                let Some(label)=math::detail(&settings,&d.result,d.owner,d.owner_position,local,array(player.position),d.real,name.as_deref())else{continue;};
                let pos=label.position;let crosshair=label.crosshair;let lines=label.lines;
                let scale=super::trajectories_curve::sample(&curve,if crosshair{0.}else{math::distance(pos,camera.position)as f32});let color=options["ShowDetailedInfo.Color"].as_i64().unwrap_or(-1)as u32;let show_item=options["ShowDetailedInfo.Item"]==true;
                let mut tag=vec![];let mut width=if show_item{16.}else{0f32};let mut top=0.;let(_,font_height)=self.gui.atlas.font.liquidbounce_target_metrics();let font_scale=9./font_height;if show_item{if let Some(stack)=d.icon.as_ref(){tag.push(Primitive::Item{stack:stack.clone(),origin:[-8.,0.],scale});}top=16.;}
                for line in lines{let text=Text::literal(&line);let w=text.spans.iter().flat_map(|s|s.text.chars()).map(|c|self.gui.atlas.font.liquidbounce_target_glyph(c,false).advance).sum::<f32>()*font_scale; width=width.max(w);tag.push(Primitive::LiquidBounceText{text,origin:[-w*scale*0.5,top*scale],scale:scale*font_scale,color,shadow:true});top+=9.+font_scale;}
                if crosshair{self.client_trajectories_tag_at(batch,tag,[batch.width*0.5+50.,batch.height*0.5+index as f32*10.]);continue;}
                indicator::submit(Entry{position:pos,priority:0,color,visible:label.visible,edge_inset:label.edge_inset,style:Arrow::Triangle{width:8.,height:10.,notch:0.},in_view_style:Arrow::None,bounds:[-width*scale*0.5,0.,width*scale*0.5,top*scale],tag});
            }
        });
    }
    fn client_trajectories_tag_at(&self,batch:&mut HudBatch,tag:Vec<Primitive>,at:[f32;2]){for primitive in tag{self.client_indicator_primitive(batch,primitive,at);}}
}
