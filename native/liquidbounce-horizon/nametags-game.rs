// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game,ItemStack,nametags_snapshot::{Source,UseRef}};
use horizon_client::{hud::{HudBatch,Rect,rgba},text::Text,hud_layout,hud_overlays};
use serde_json::{json,Value};
use std::collections::HashMap;
#[derive(Default)]pub(super) struct State{pub sources:HashMap<u64,Source>,pub uses:HashMap<u64,UseRef>}
fn color(c:u32)->u32{rgba((c>>16)as u8,(c>>8)as u8,c as u8,(c>>24)as u8)}
fn argb(v:&Value,default:u32)->u32{v.as_u64().map(|v|v as u32).or_else(||v.as_i64().map(|v|v as u32)).unwrap_or(default)}
impl Game<'_>{
    pub(super) fn client_nametags_query(&self,input:Value)->Value{let mut output=String::new();self.native_client_nametags(&input.to_string(),&mut output);serde_json::from_str(&output).unwrap_or(Value::Null)}
    fn client_nametags_perspective(&self)->i32{match self.camera_type{super::perspective::CameraType::FirstPerson=>0,super::perspective::CameraType::ThirdPersonBack=>1,super::perspective::CameraType::ThirdPersonFront=>2}}
    pub(super) fn client_nametags_tick(&self){
        if !self.native_window()||self.player.is_none(){return;}
        let perspective=self.client_nametags_perspective();let policy=self.client_nametags_query(json!({"action":"policy","perspective":perspective}));
        if policy["running"]!=true{*self.client_nametags.borrow_mut()=Default::default();return;}
        let Some(camera)=self.native_camera()else{return;};let(entities,sources,uses)=self.client_nametags_snapshot();
        self.client_nametags_query(json!({"action":"tick","perspective":perspective,"camera":camera.position,"entities":entities}));
        *self.client_nametags.borrow_mut()=State{sources,uses};
    }
    /// Apply only to ordinary labels: hooks that supplied a custom label retain it.
    pub(super) fn client_nametags_vanilla(&self,id:u32,custom:bool)->bool{custom||self.client_nametags_query(json!({"action":"vanilla","id":id as i32,"perspective":self.client_nametags_perspective(),"target":serde_json::from_str::<Value>(&self.native_client_targeting_entity(id as i32)).unwrap_or(Value::Null)}))!=false}
    pub(super) fn client_nametags_overlay(&self,batch:&mut HudBatch){
        if !self.native_window()||self.player.is_none(){return;}
        let perspective=self.client_nametags_perspective();let policy=self.client_nametags_query(json!({"action":"policy","perspective":perspective}));if policy["running"]!=true{*self.client_nametags.borrow_mut()=Default::default();return;}if policy["refresh"]==true{self.client_nametags_tick();}
        let frame=self.client_nametags_query(json!({"action":"frame"}));let Some(camera)=self.native_camera()else{return;};let matrix=glam::Mat4::from_cols_array(&camera.projection)*glam::Mat4::from_cols_array(&camera.view);let alpha=self.client_item_tags_alpha.get();let live=self.client_tracers_entities(alpha);let native_sources=self.client_nametags.borrow().sources.clone();let settings=&frame["settings"];
        for tag in frame["tags"].as_array().into_iter().flatten(){
            let Some(id)=tag["identity"].as_u64()else{continue;};let Some(source)=native_sources.get(&id)else{continue;};let mut pos=source.position;let mut eye=source.eye;
            if let Some(e)=live.iter().find(|e|e["identity"].as_u64()==Some(id)){if let Ok(p)=serde_json::from_value(e["interpolated"].clone()){pos=p;}if id==1{if let Some(p)=self.player.as_ref(){eye=f64::from(p.dimensions.eye_height);}}else if id&7==0{if let Some(m)=self.mobs.iter().find(|m|m.join.get()<<3==id){eye=m.eye_height();}}}
            pos[1]+=eye+0.55;let clip=matrix*glam::Vec4::new((pos[0]-camera.position[0])as f32,(pos[1]-camera.position[1])as f32,(pos[2]-camera.position[2])as f32,1.);if !clip.is_finite()||clip.w<=0.{continue;}
            let center=[(clip.x/clip.w+1.)*batch.width*0.5,(1.-clip.y/clip.w)*batch.height*0.5];let scale=tag["scale"].as_f64().unwrap_or(1.)as f32;if !scale.is_finite()||scale<=0.{continue;}
            let selected=settings["slots"].as_array().into_iter().flatten().filter_map(Value::as_u64).filter_map(|v|usize::try_from(v).ok()).filter(|v|*v<8).filter(|v|settings["skipEmpty"]!=true||source.stacks[*v].is_some()).collect::<Vec<_>>();
            let stacks=selected.iter().map(|slot|source.stacks[*slot].clone()).collect::<Vec<_>>();let cells=selected.iter().map(|slot|tag["cells"][*slot].clone()).collect::<Vec<_>>();let used=tag["using"].as_u64();let local=tag["local"]==true;let draw_settings=settings.clone();
            self.client_item_stack_list(batch,super::item_stack_list::List{stacks,title:None,title_color:u32::MAX,row_length:100,center:[center[0],center[1]-15.*scale],scale,background:json!({"background":"Rect","color":0,"outlineColor":0,"glow":0,"gradient":0,"margin":0}),renderer:Some(Box::new(move|game,batch,index,stack,x,y,seed|{
                let owner=game.player_owner(Some(stack));let state=game.resolve_item(stack,horizon_assets::item_model::DisplayContext::Gui,Some(&owner),seed);
                horizon_client::item_render::gui::draw(&state,&game.pack.item_renderer,&game.gui.atlas.sprites,&game.item_host(),batch,x,y,game.glint_frame.millis/1000.,game.glint_frame.speed,game.glint_frame.alpha);
                if draw_settings["showInfo"]==true{game.client_nametags_decorations(batch,stack,x,y,local);}
                if draw_settings["highlight"]==true&&used==Some(stack.object_identity()){let fill=color(argb(&draw_settings["fill"],0x64ff0000));let outline=color(argb(&draw_settings["outline"],0));batch.liquidbounce_rounded_rect([x,y,x+16.,y+16.],0.,fill,fill,0.);batch.liquidbounce_rounded_rect([x,y,x+16.,y+16.],0.,outline,outline,1.);}
                game.client_nametags_enchantment_labels(batch,cells.get(index).unwrap_or(&Value::Null),&draw_settings,x,y);
            }))},false);
            let text=horizon_client::scoreboard::comp_text(&tag["text"]).resolve_legacy();let(_,height)=self.gui.atlas.font.liquidbounce_target_metrics();let font_scale=9./height;let total=font_scale*scale;let width=self.client_nametags_width(&text,true);let left=center[0]-width*total*0.5;let top=center[1]-height*total*0.5;
            let rect=[left-4.3*total,top-0.1*height*total,left+(width+8.6)*total,top+1.1*height*total];let radius=settings["backgroundRadius"].as_f64().unwrap_or(2.)as f32*scale;let stroke=settings["borderWidth"].as_f64().unwrap_or(1.)as f32*scale;
            batch.liquidbounce_rounded_rect(rect,radius,color(0x80000000),color(0x80000000),0.);if stroke>0.{batch.liquidbounce_rounded_rect(rect,radius,color(0xff000000),color(0xff000000),stroke);}
            self.client_nametags_text(batch,&text,[left,top],total,true);
        }
    }
    fn client_nametags_width(&self,text:&Text,shadow:bool)->f32{text.spans.iter().flat_map(|s|s.text.chars().map(move|c|(c,s.style.bold==Some(true)))).map(|(c,bold)|self.gui.atlas.font.liquidbounce_target_glyph(c,bold).advance).sum::<f32>()+if shadow{2.}else{0.}}
    fn client_nametags_text(&self,batch:&mut HudBatch,text:&Text,at:[f32;2],scale:f32,shadow:bool){
        let(ascent,_)=self.gui.atlas.font.liquidbounce_target_metrics();let glyphs=text.spans.iter().flat_map(|s|s.text.chars().map(move|c|(c,&s.style))).map(|(c,s)|(self.gui.atlas.font.liquidbounce_target_glyph(c,s.bold==Some(true)),s)).collect::<Vec<_>>();
        for is_shadow in[true,false]{if is_shadow&&!shadow{continue;}let offset=if is_shadow{2.*scale}else{0.};let mut x=at[0]+offset;let y=at[1]+ascent*scale+offset;
            for(g,s)in &glyphs{let fill=color(if is_shadow{0x96000000}else{0xff000000|s.color.unwrap_or(0xffffff)&0xffffff});if let Some(Rect{u0,v0,u1,v1,..})=g.rect{let shear=if s.italic==Some(true){0.2}else{0.};let(x0,x1,y0,y1)=(x+g.left*scale,x+g.right*scale,y+g.top*scale,y+g.bottom*scale);let(top,bottom)=(-g.top*scale*shear,-g.bottom*scale*shear);batch.quad([[x0+top,y0],[x1+top,y0],[x1+bottom,y1],[x0+bottom,y1]],[[u0,v0],[u1,v0],[u1,v1],[u0,v1]],fill);}
                if s.underlined==Some(true){batch.fill(&self.gui.atlas,x,y+2.*scale,g.advance*scale,scale,fill);}if s.strikethrough==Some(true){batch.fill(&self.gui.atlas,x,y-ascent*scale*0.4,g.advance*scale,scale,fill);}x+=g.advance*scale;
            }
        }
    }
    fn client_nametags_enchantment_labels(&self,batch:&mut HudBatch,cells:&Value,settings:&Value,x:f32,y:f32){
        let Some(cells)=cells.as_array().filter(|s|!s.is_empty())else{return;};let(_,height)=self.gui.atlas.font.liquidbounce_target_metrics();let setting_scale=settings["enchantmentScale"].as_f64().unwrap_or(0.8)as f32;let scale=setting_scale*9./height;let row_height=height*scale+1.;let total=cells.len()as f32*row_height+(cells.len()-1)as f32*0.5;let mut top=y-1.5-total;
        for c in cells{let text=Text::colored(c["text"].as_str().unwrap_or(""),argb(&c["color"],u32::MAX)&0xffffff);let width=self.client_nametags_width(&text,false)*scale;let bg=if c["curse"]==true{0x96b20000}else{0x80000000};batch.liquidbounce_rounded_rect([x+8.-width*0.5-1.25,top,x+8.+width*0.5+1.25,top+row_height],settings["enchantmentRadius"].as_f64().unwrap_or(1.)as f32*setting_scale,color(bg),color(bg),0.);self.client_nametags_text(batch,&text,[x+8.-width*0.5,top+0.5],scale,true);top+=row_height+0.5;}
    }
    fn client_nametags_decorations(&self,batch:&mut HudBatch,stack:&ItemStack,x:f32,y:f32,local:bool){
        if local{self.draw_item_decorations(batch,stack,x,y,None);return;}
        let info=self.items().get(&stack.item);let max_damage=if horizon_client::stations::is_damageable(self.items(),stack){info.map_or(0,|i|i.max_damage as i32)}else{0};let mut bar=horizon_client::bundle::Contents::of(self.items(),stack).map(|c|c.bar()).unwrap_or_else(||hud_overlays::damage_bar(stack.damage as i32,max_damage));
        if let Some((_,c))=bar.as_mut().filter(|_|stack.damage>0&&max_damage>0){*c=horizon_client::pack_palette::item_bar_color(&stack.item,stack.damage as f32/max_damage as f32).unwrap_or(*c);}self.native_item_bar(stack,&mut bar);
        let decorations=hud_overlays::ItemDecorations{count:stack.count,count_text:None,bar,cooldown:0.};let mut draws=vec![hud_layout::GuiDraw::Push(vec![hud_layout::PoseOp::Translate(x,y)])];draws.extend(hud_overlays::item_decorations(0,0,&decorations,&self.gui.atlas.font));draws.push(hud_layout::GuiDraw::Pop);hud_layout::paint(batch,&self.gui.atlas,&draws);self.native_item_decorations(stack,x,y,batch);
    }
}
