// SPDX-License-Identifier: GPL-3.0-or-later
//! BedDefender uses real default collision, chest classes, bed parts and armor RGB.
use super::Game;
use serde_json::{Value,json};
impl Game<'_>{
    fn bed_defender_beds(&self,radius:f32)->Option<Vec<Value>>{
        let eye=self.eye_position()?;let radius=f64::from(radius);let lo=eye.map(|v|(v-radius).floor()as i32);let hi=eye.map(|v|(v+radius).floor()as i32);let mut beds=vec![];
        let min=self.world.shape.min_y;let max=self.world.shape.max_y()-1;
        for sx in lo[0]>>4..=hi[0]>>4{for sz in lo[2]>>4..=hi[2]>>4{for sy in(lo[1]>>4).max(min>>4)..=(hi[1]>>4).min(max>>4){for y in lo[1].max(sy<<4)..=hi[1].min((sy<<4)+15){for z in lo[2].max(sz<<4)..=hi[2].min((sz<<4)+15){for x in lo[0].max(sx<<4)..=hi[0].min((sx<<4)+15){
            let state=self.world.block(x,y,z);let facts=self.res.facts().get(state);if !facts.has_class("BedBlock"){continue;}
            let properties=&self.pack.assets.registry.state(state)?.properties;let mut facing=match properties.get("facing").map(String::as_str){Some("north")=>[0,0,-1],Some("east")=>[1,0,0],Some("west")=>[-1,0,0],_=>[0,0,1]};
            // searchBedLayer starts by walking away from the opposite part.
            if properties.get("part").is_some_and(|p|p=="foot"){facing=facing.map(|v|-v);}
            let name=facts.name.strip_prefix("minecraft:").unwrap_or(&facts.name).strip_suffix("_bed").unwrap_or("white");let dye=horizon_client::block_entity_render::DYES.iter().position(|(s,_)|*s==name).unwrap_or(0);
            beds.push(json!({"pos":[x,y,z],"facing":facing,"item":facts.item.as_ref(),"dye":dye,"outline":0,"surrounding":[]}));
        }}}}}}Some(beds)
    }
    pub(super) fn bed_defender_query(&self,text:&str)->Option<String>{
        let q:Value=serde_json::from_str(text).ok()?;
        match q["op"].as_str()?{
            "slots"=>{let slots=std::iter::once(40).chain(0..9).filter_map(|index|{let stack=self.stack_in(index).filter(|s|s.count>0)?;let facts=self.scaffold_block_facts(stack)?;let b=self.res.behaviours();let binding=b.item_binding(&stack.item)?;let chest=b.block_data(binding.block).is(horizon_world::level::classes::Class::AbstractChest);Some(json!({"index":index,"count":stack.count as i32,"hardness":facts.hardness,"full":facts.full_cube,"chest":chest}))}).collect::<Vec<_>>();Some(json!({"selected":self.client_hotbar_slot(),"slots":slots}).to_string())},
            "status"=>Some(json!({"sneaking":self.player.as_ref()?.shift_key_down}).to_string()),
            "targets"=>{let armor:[Option<u32>;4]=std::array::from_fn(|i|{let stack=self.armor[i].as_ref()?;let view=horizon_client::item_render::stack_view(stack);let components=self.pack.item_renderer.models.components_view(&view);let color=components.get("minecraft:dyed_color")?;color.as_i64().or_else(||color["rgb"].as_i64()).map(|v|v as u32|0xff000000)});Some(json!({"eye":self.eye_position()?,"sneaking":self.player.as_ref()?.shift_key_down,"armor":armor,"beds":self.bed_defender_beds(q["radius"].as_f64()?as f32)?}).to_string())},_=>None
        }
    }
    fn bed_defender_event(&self,input:Value)->Value{let mut output=String::new();self.native_client_bed_defender(&input.to_string(),&mut output);serde_json::from_str(&output).unwrap_or(Value::Null)}
    pub(super) fn bed_defender_position(&self,raw:[f64;3]){if let Some(p)=self.player.as_ref(){self.bed_defender_event(json!({"action":"position","old":[p.position.x,p.position.y,p.position.z],"raw":raw}));}}
    pub(super) fn bed_defender_key(&self,key:winit::keyboard::KeyCode,pressed:bool){
        if !pressed||!self.native_window(){return;}let Some(name)=horizon_client::keybinds::key_name(key)else{return;};let options=self.bed_defender_event(json!({"action":"options"}));if options["mode"]!="Manual"||options["track"]!=name&&options["untrack"]!=name{return;}
        let beds=if options["track"]==name{self.bed_defender_beds(16.).unwrap_or_default().into_iter().map(|b|b["pos"].clone()).collect::<Vec<_>>()}else{vec![]};
        self.bed_defender_event(json!({"action":"key","key":name,"eye":self.eye_position(),"beds":beds}));
    }
}
