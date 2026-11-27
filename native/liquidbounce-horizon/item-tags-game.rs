// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game,ItemStack};
use horizon_client::{hud::HudBatch,text::Text};
use super::item_stack_list::List;
use serde_json::{Value,json};
use std::collections::HashMap;
#[derive(Default)]
pub(super) struct State {
    stacks:HashMap<u64,ItemStack>,
    positions:HashMap<u64,([f64;3],[f64;3])>,
}
impl Game<'_> {
    fn client_item_tags_query(&self,input:Value)->Value{
        let mut output=String::new();self.native_client_item_tags(&input.to_string(),&mut output);
        serde_json::from_str(&output).unwrap_or(Value::Null)
    }
    fn client_item_tags_description(&self,s:&ItemStack)->String{
        // BlockItem delegates to its block's descriptionId, independent of language/custom name.
        let block=self.items().get(&s.item).and_then(|i|i.block).map(|state|self.res.facts().get(state).name.to_string());
        let id=block.as_deref().unwrap_or(&s.item);let(ns,path)=id.split_once(':').unwrap_or(("minecraft",id));
        format!("{}.{ns}.{}",if block.is_some(){"block"}else{"item"},path.replace('/',"."))
    }
    fn client_item_tags_stack(&self,handle:u64,s:&ItemStack)->Value{
        // Canonical SNBT is a sorted NBT Compound. It preserves explicit component removals
        // and unknown components; neither count nor item defaults belong to this key.
        let patch=horizon_client::item_data::encode_components_with_items(s,self.items());
        let components=patch.as_ref().map(horizon_client::snbt::to_snbt).unwrap_or_default();
        json!({"handle":handle,"item":s.item.as_ref(),"count":s.count as i32,"description":self.client_item_tags_description(s),"components":components})
    }
    pub(super) fn client_item_tags_tick(&self){
        if !self.native_window()||self.player.is_none(){return;}
        if self.client_item_tags_query(json!({"action":"running"}))!=true{*self.client_item_tags.borrow_mut()=Default::default();return;}
        let Some(camera)=self.native_camera()else{return;};
        let mut native=State::default();let mut entities=Vec::new();
        // EntityLookup's item-filtered insertion walk. Join identity survives packet updates,
        // changes for a new object reusing an entity id, and is never an id-based cache.
        let mut dropped=self.dropped.iter().collect::<Vec<_>>();dropped.sort_by_key(|e|e.join);
        for e in dropped{
            let identity=e.join.get();let handle=e.stack.object_identity();
            entities.push(json!({"identity":identity,"position":[e.position.x,e.position.y,e.position.z],"stack":self.client_item_tags_stack(handle,&e.stack)}));
            native.stacks.insert(handle,e.stack.clone());native.positions.insert(identity,([e.previous.x,e.previous.y,e.previous.z],[e.position.x,e.position.y,e.position.z]));
        }
        *self.client_item_tags.borrow_mut()=native;
        self.client_item_tags_query(json!({"action":"tick","camera":camera.position,"entities":entities}));
    }
    fn client_item_tags_resolve(&self,sources:&HashMap<u64,ItemStack>,value:&Value,allow_live:bool)->Option<ItemStack>{
        if allow_live&&value["live"]==true&&let Some(stack)=self.dropped.iter().map(|e|&e.stack).find(|s|Some(s.object_identity())==value["handle"].as_u64()){return Some(stack.clone());}
        let mut stack=if value["bare"]==true{ItemStack::new(value["item"].as_str()?,1)}else{sources.get(&value["handle"].as_u64()?)?.clone()};
        stack.count=value["count"].as_i64()? as u32;Some(stack)
    }
    fn client_item_tags_hover(&self,s:&ItemStack)->Text{
        if let Some(custom)=s.data().and_then(|d|d.custom_name.as_ref()){return horizon_client::component::name_text(custom);}
        if let Some(key)=horizon_client::brewing::contents_name_key(s){return Text::translate(&key,&[]);}
        if let Some(name)=s.data().and_then(|d|d.other("minecraft:item_name")){return horizon_client::component::name_text(name);}
        if let Some(title)=s.data().and_then(|d|d.written.as_ref()).map(|w|w.title.clone()).filter(|t|!t.trim().is_empty()){return Text::literal(title);}
        if &*s.item=="minecraft:shield"&&let Some(color)=s.data().and_then(|d|d.base_color.as_deref()){return Text::translate(&format!("item.minecraft.shield.{color}"),&[]);}
        if let Some(head)=horizon_client::item_tooltip::player_head_name(s){return head.text();}
        Text::literal(horizon_client::item_tooltip::name_line(s,&|key|self.pack.names.display_get(key)))
    }
    fn client_item_tags_contents(&self,s:&ItemStack)->Vec<ItemStack>{
        let view=horizon_client::item_render::stack_view(s);let components=self.pack.item_renderer.models.components_view(&view);
        let Some(entries)=components.get("minecraft:container").and_then(Value::as_array)else{return Vec::new();};
        let mut entries=entries.iter().collect::<Vec<_>>();entries.sort_by_key(|entry|entry["slot"].as_u64().unwrap_or(0));
        entries.into_iter().filter_map(|entry|horizon_items::item_data::stack_from_hover(&entry["item"])).filter(|s|s.count>0&&&*s.item!="minecraft:air").collect()
    }
    fn client_item_tags_merge(&self,stacks:Vec<ItemStack>)->Vec<ItemStack>{
        let sources=stacks.into_iter().enumerate().map(|(i,s)|(i as u64,s)).collect::<HashMap<_,_>>();
        let mut handles=sources.keys().copied().collect::<Vec<_>>();handles.sort_unstable();
        let input=handles.into_iter().map(|id|self.client_item_tags_stack(id,&sources[&id])).collect::<Vec<_>>();
        let result=self.client_item_tags_query(json!({"action":"merge","stacks":input}));
        result.as_array().into_iter().flatten().filter_map(|s|self.client_item_tags_resolve(&sources,s,false)).collect()
    }
    pub(super) fn client_item_tags_overlay(&self,batch:&mut HudBatch){
        if !self.native_window()||self.player.is_none(){return;}
        let Some(camera)=self.native_camera()else{return;};
        let frame=self.client_item_tags_query(json!({"action":"frame"}));let settings=&frame["settings"];
        let offset=serde_json::from_value::<[f64;3]>(frame["offset"].clone()).unwrap_or([0.;3]);
        let matrix=glam::Mat4::from_cols_array(&camera.projection)*glam::Mat4::from_cols_array(&camera.view);
        let alpha=self.client_item_tags_alpha.get() as f32;
        let row=settings["rowLength"].as_i64().unwrap_or(100).max(1) as usize;
        let native=self.client_item_tags.borrow();let mut lists=Vec::new();
        for group in frame["groups"].as_array().into_iter().flatten(){
            let ids=group["entities"].as_array().into_iter().flatten().filter_map(Value::as_u64).collect::<Vec<_>>();if ids.is_empty(){continue;}
            let mut center=[0.;3];let mut found=0;
            for id in ids{
                let position=self.dropped.iter().find(|e|e.join.get()==id).map(|e|([e.previous.x,e.previous.y,e.previous.z],[e.position.x,e.position.y,e.position.z])).or_else(||native.positions.get(&id).copied());
                if let Some((previous,current))=position{for i in 0..3{center[i]+=previous[i]+(current[i]-previous[i])*f64::from(alpha);}found+=1;}
            }
            if found==0{continue;}for i in 0..3{center[i]=center[i]/f64::from(found)+offset[i];}
            let clip=matrix*glam::Vec4::new((center[0]-camera.position[0])as f32,(center[1]-camera.position[1])as f32,(center[2]-camera.position[2])as f32,1.);
            // WorldToScreen accepts any x/y, rejects points behind the camera only.
            if !clip.is_finite()||clip.w<=0.{continue;}let screen=[(clip.x/clip.w+1.)*0.5*batch.width,(1.-clip.y/clip.w)*0.5*batch.height];
            let scale=group["scale"].as_f64().unwrap_or(1.)as f32;if !scale.is_finite()||scale<=0.{continue;}
            let stacks=group["stacks"].as_array().into_iter().flatten().filter_map(|s|self.client_item_tags_resolve(&native.stacks,s,true)).collect::<Vec<_>>();
            if stacks.is_empty(){continue;}
            lists.push(List{stacks:stacks.iter().cloned().map(Some).collect(),title:None,title_color:u32::MAX,row_length:row,center:screen,scale,background:settings.clone(),renderer:None});
            if settings["shulker"]==true{for stack in &stacks{
                let contents=self.client_item_tags_contents(stack);if contents.is_empty(){continue;}
                let contents=if settings["mergeStacks"]==true{self.client_item_tags_merge(contents)}else{contents};
                let title=(settings["showTitle"]==true).then(||self.client_item_tags_hover(stack));
                lists.push(List{stacks:contents.into_iter().map(Some).collect(),title,title_color:u32::MAX,row_length:row,center:screen,scale,background:settings.clone(),renderer:None});
            }}
        }
        drop(native);
        for list in lists{self.client_item_stack_list(batch,list,settings["preventOverlap"]==true);}
    }
}
