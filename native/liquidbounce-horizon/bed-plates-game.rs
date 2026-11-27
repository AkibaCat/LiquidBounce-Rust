// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game,ItemStack};
use super::item_stack_list::List;
use horizon_client::{chunk::Section,hud::{HudBatch,rgba},keybinds};
use horizon_world::registry::BlockStateId;
use serde_json::{Value,json};
use std::{collections::{BTreeMap,BTreeSet},sync::{Arc,Weak}};
#[path="bed_plates_scan.rs"]mod scan;
#[derive(Default)]pub(super) struct State{
    sections:BTreeMap<(i32,i32,i32),(Weak<Section>,Vec<[i32;3]>)>,
    surrounding:BTreeMap<[i32;3],Value>,max_layers:i32,
}
impl Game<'_>{
    fn client_bed_plates_query(&self,input:Value)->Value{let mut output=String::new();self.native_client_bed_plates(&input.to_string(),&mut output);serde_json::from_str(&output).unwrap_or(Value::Null)}
    fn client_bed_plates_is_head(&self,state:BlockStateId)->bool{self.res.facts().get(state).has_class("BedBlock")&&self.pack.assets.registry.state(state).is_some_and(|s|s.properties.get("part").is_some_and(|p|p=="head"))}
    fn client_bed_plates_facing(&self,state:BlockStateId)->[i32;3]{match self.pack.assets.registry.state(state).and_then(|s|s.properties.get("facing")).map(String::as_str){Some("north")=>[0,0,-1],Some("east")=>[1,0,0],Some("west")=>[-1,0,0],_=>[0,0,1]}}
    fn client_bed_plates_surrounding(&self,pos:[i32;3],facing:[i32;3],max:i32)->Value{
        let mut counts:BTreeMap<(i32,String),(i32,BlockStateId)>=BTreeMap::new();
        for(p,layer)in scan::layers(pos,facing,max){let state=self.world.block(p[0],p[1],p[2]);if self.res.facts().is_air(state){continue;}let facts=self.res.facts().get(state);let default=self.pack.assets.registry.default_state(&facts.name).unwrap_or(state);let entry=counts.entry((layer,facts.name.to_string())).or_insert((0,default));entry.0=entry.0.wrapping_add(1);}
        let mut blocks=counts.into_iter().map(|((layer,block),(count,default))|{
            let facts=self.res.facts().get(default);let item=match block.as_str(){"minecraft:water"=>"minecraft:water_bucket","minecraft:lava"=>"minecraft:lava_bucket",_=>&facts.item};
            let conductor=self.res.behaviours().facts(default).flags&horizon_world::level::data::state_flags::CONDUCTOR!=0;
            let whitelist=matches!(block.as_str(),"minecraft:ladder"|"minecraft:water"|"minecraft:glass")||block.starts_with("minecraft:")&&block.ends_with("_stained_glass");
            let(ns,path)=block.split_once(':').unwrap_or(("minecraft",&block));let description=format!("block.{ns}.{}",path.replace('/',"."));
            json!({"block":block,"item":item,"description":description,"count":count,"layer":layer,"hardness":facts.hardness,"predefined":!self.res.facts().is_air(default)&&(conductor||whitelist),"requiresTool":facts.requires_tool})
        }).collect::<Vec<_>>();
        blocks.sort_by(|a,b|a["layer"].as_i64().cmp(&b["layer"].as_i64()).then_with(||b["count"].as_i64().cmp(&a["count"].as_i64())).then_with(||{let h=|v:&Value|{let n=v["hardness"].as_f64().unwrap_or(0.);if n == -1.{f64::INFINITY}else{n}};h(b).total_cmp(&h(a))}).then_with(||a["description"].as_str().unwrap_or("").encode_utf16().cmp(b["description"].as_str().unwrap_or("").encode_utf16())));json!(blocks)
    }
    pub(super) fn client_bed_plates_tick(&self){
        if !self.native_window(){return;}let options=self.client_bed_plates_query(json!({"action":"options"}));
        if options.is_null()||self.player.is_none(){*self.client_bed_plates.borrow_mut()=Default::default();return;}let Some(camera)=self.native_camera()else{return;};
        let max=options["maxLayers"].as_i64().unwrap_or(5)as i32;let mut cache=self.client_bed_plates.borrow_mut();let rescan=cache.max_layers!=max;cache.max_layers=max;
        let mut present=BTreeSet::new();let mut changed=Vec::new();
        for(&(cx,cz),chunk)in self.world.chunks.iter(){for(sy,section)in chunk.sections.iter().enumerate(){let key=(cx,chunk.min_y()+sy as i32*16,cz);present.insert(key);let weak=Arc::downgrade(section);if cache.sections.get(&key).is_some_and(|(previous,_)|Weak::ptr_eq(previous,&weak)){continue;}let mut beds=Vec::new();if section.blocks.maybe_has(|id|self.client_bed_plates_is_head(BlockStateId(id))){for i in 0..4096{if self.client_bed_plates_is_head(BlockStateId(section.get(i))){beds.push([cx.wrapping_mul(16).wrapping_add((i&15)as i32),key.1+(i>>8)as i32,cz.wrapping_mul(16).wrapping_add((i>>4&15)as i32)]);}}}cache.sections.insert(key,(weak,beds));changed.push(key);}}
        for key in cache.sections.keys().filter(|key|!present.contains(key)).copied().collect::<Vec<_>>(){cache.sections.remove(&key);changed.push(key);}
        let heads=cache.sections.values().flat_map(|(_,p)|p.iter().copied()).collect::<Vec<_>>();let heads_set=heads.iter().copied().collect::<BTreeSet<_>>();cache.surrounding.retain(|p,_|heads_set.contains(p));let mut beds=Vec::new();
        for pos in heads{let state=self.world.block(pos[0],pos[1],pos[2]);let facts=self.res.facts().get(state);let facing=self.client_bed_plates_facing(state);
            let dirty=rescan||!cache.surrounding.contains_key(&pos)||changed.iter().any(|&(cx,base,cz)|{let r=i64::from(max.max(0))+1;let x=i64::from(cx)*16;let y=i64::from(base);let z=i64::from(cz)*16;let p=pos.map(i64::from);x<=p[0]+r&&x+15>=p[0]-r&&y<=p[1]+r&&y+15>=p[1]&&z<=p[2]+r&&z+15>=p[2]-r});
            if dirty{cache.surrounding.insert(pos,self.client_bed_plates_surrounding(pos,facing,max));}
            let name=facts.name.strip_prefix("minecraft:").unwrap_or(&facts.name).strip_suffix("_bed").unwrap_or("white");let dye=horizon_client::block_entity_render::DYES.iter().position(|(s,_)|*s==name).unwrap_or(0);
            // DyeColor.mapColor is the wool color, independent of the bed state's wood map color.
            let map_id=[8usize,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29][dye];let outline=horizon_client::blocks::MAP_COLORS[map_id]|0xff000000;
            beds.push(json!({"pos":pos,"facing":facing,"item":facts.item.as_ref(),"dye":dye,"outline":outline,"surrounding":cache.surrounding[&pos]}));
        }
        drop(cache);self.client_bed_plates_query(json!({"action":"tick","camera":camera.position,"beds":beds}));
    }
    pub(super) fn client_bed_plates_position(&self,raw:[f64;3]){if let Some(player)=self.player.as_ref(){self.client_bed_plates_query(json!({"action":"position","old":[player.position.x,player.position.y,player.position.z],"raw":raw}));}}
    pub(super) fn client_bed_plates_key(&self,key:winit::keyboard::KeyCode,pressed:bool){
        if !pressed||!self.native_window(){return;}let Some(name)=keybinds::key_name(key)else{return;};let options=self.client_bed_plates_query(json!({"action":"options"}));if options["mode"]!="Manual"||options["track"]!=name&&options["untrack"]!=name{return;}let Some(eye)=self.eye_position()else{return;};let mut beds=Vec::new();
        if options["track"]==name{let from=eye.map(|n|(n-16.).floor()as i32);let to=eye.map(|n|(n+16.).floor()as i32);for z in from[2]..=to[2]{for y in from[1]..=to[1]{for x in from[0]..=to[0]{if self.res.facts().get(self.world.block(x,y,z)).has_class("BedBlock"){beds.push([x,y,z]);}}}}}
        self.client_bed_plates_query(json!({"action":"key","key":name,"eye":eye,"beds":beds}));
    }
    pub(super) fn client_bed_plates_world_change(&self){self.client_bed_plates_query(json!({"action":"worldChange"}));*self.client_bed_plates.borrow_mut()=Default::default();}
    pub(super) fn client_bed_plates_overlay(&self,batch:&mut HudBatch){
        if !self.native_window()||self.player.is_none(){return;}let Some(camera)=self.native_camera()else{return;};
        let armor:[Option<u32>;4]=std::array::from_fn(|i|{let stack=self.armor[i].as_ref()?;let view=horizon_client::item_render::stack_view(stack);let components=self.pack.item_renderer.models.components_view(&view);let color=components.get("minecraft:dyed_color")?;color.as_i64().or_else(||color["rgb"].as_i64()).map(|v|v as u32|0xff000000)});
        let frame=self.client_bed_plates_query(json!({"action":"frame","armor":armor}));let settings=&frame["settings"];let offset=serde_json::from_value::<[f64;3]>(frame["offset"].clone()).unwrap_or([0.;3]);let matrix=glam::Mat4::from_cols_array(&camera.projection)*glam::Mat4::from_cols_array(&camera.view);let mut rendered=0i32;
        for group in frame["groups"].as_array().into_iter().flatten(){if rendered>=settings["maxCount"].as_i64().unwrap_or(8)as i32{break;}let Ok(position)=serde_json::from_value::<[f64;3]>(group["position"].clone())else{continue;};let clip=matrix*glam::Vec4::new((position[0]+offset[0]-camera.position[0])as f32,(position[1]+offset[1]-camera.position[1])as f32,(position[2]+offset[2]-camera.position[2])as f32,1.);if !clip.is_finite()||clip.w<=0.{continue;}
            let scale=group["scale"].as_f64().unwrap_or(1.)as f32;if !scale.is_finite()||scale<=0.{continue;}let show_bed=settings["showBed"]==true;let mut stacks=Vec::new();let mut labels=Vec::new();if show_bed{stacks.push(Some(ItemStack::new(group["bed"].as_str().unwrap_or("minecraft:air"),1)));labels.push((0i32,1i32,false));}
            for block in group["surrounding"].as_array().into_iter().flatten(){let count=block["count"].as_i64().unwrap_or(0)as i32;stacks.push(Some(ItemStack::new(block["item"].as_str().unwrap_or("minecraft:air"),count as u32)));let name=block["block"].as_str().unwrap_or("minecraft:air");let red=settings["highlightUnbreakable"]==true&&block["requiresTool"]==true&&!self.inventory.iter().take(9).any(|s|self.items().has_correct_tool_for_drops(s.as_ref(),true,name));labels.push((block["layer"].as_i64().unwrap_or(0)as i32,count,red));}
            let distance=format!("{}m",group["distance"].as_f64().unwrap_or(0.)as i32);let compact=settings["compact"]==true;let shadow=settings["textShadow"]==true;
            let renderer=Box::new(move|game:&Game<'_>,batch:&mut HudBatch,index:usize,stack:&ItemStack,x:f32,y:f32,seed:i32|{
                if stack.count as i32>0&&&*stack.item!="minecraft:air"{let owner=game.player_owner(Some(stack));let state=game.resolve_item(stack,horizon_assets::item_model::DisplayContext::Gui,Some(&owner),seed);horizon_client::item_render::gui::draw(&state,&game.pack.item_renderer,&game.gui.atlas.sprites,&game.item_host(),batch,x,y,game.glint_frame.millis/1000.,game.glint_frame.speed,game.glint_frame.alpha);}
                if index==0&&show_bed{let width=game.gui.atlas.font.width(&distance)as f32;batch.text(&game.gui.atlas,&distance,x+17.-width,y+9.,u32::MAX,true);}else if let Some(&(layer,count,red))=labels.get(index){let color=if red{rgba(255,0,0,255)}else{u32::MAX};if !compact{let roman=["","I","II","III","IV","V","VI","VII","VIII"].get(layer as usize).copied().unwrap_or("");batch.text(&game.gui.atlas,roman,x,y,color,shadow);}let count=count.to_string();let width=game.gui.atlas.font.width(&count)as f32;batch.text(&game.gui.atlas,&count,x+17.-width,y+9.,color,shadow);}
            });
            let mut background=settings.clone();if let Some(outline)=group["outline"].as_u64(){background["outlineColor"]=json!(outline);}self.client_item_stack_list(batch,List{stacks,title:None,title_color:u32::MAX,row_length:i32::MAX as usize,center:[(clip.x/clip.w+1.)*0.5*batch.width,(1.-clip.y/clip.w)*0.5*batch.height],scale,background,renderer:Some(renderer)},settings["preventOverlap"]==true);rendered+=1;
        }
    }
}
