// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared ItemStackListRenderer: immediate lists and one stable planned queue.
use super::{Game,ItemStack};
use horizon_client::{hud::{HudBatch,rgba},text::Text};
use serde_json::Value;
#[path="item_tags_layout.rs"]mod layout;

// A producer may capture owned render data (e.g. BedPlates distances/tool flags),
// while the live Game and atlas are supplied only when the common queue draws.
pub(super) type Renderer=Box<dyn Fn(&Game<'_>,&mut HudBatch,usize,&ItemStack,f32,f32,i32)>;
pub(super) struct List {
    pub stacks:Vec<Option<ItemStack>>,pub title:Option<Text>,pub title_color:u32,
    pub row_length:usize,pub center:[f32;2],pub scale:f32,pub background:Value,
    pub renderer:Option<Renderer>,
}
#[derive(Default)]
pub(super) struct Queue(pub Vec<List>);
impl Game<'_> {
    /// Original `.draw(graphics,rearrange)`: false draws now; true joins planned.
    pub(super) fn client_item_stack_list(&self,batch:&mut HudBatch,mut list:List,rearrange:bool){
        if list.stacks.is_empty()&&list.title.is_none(){return;}
        if !list.scale.is_finite()||list.scale<=0.||!list.center.iter().all(|v|v.is_finite()){return;}
        list.row_length=list.row_length.max(1);
        if rearrange{self.client_item_stack_lists.borrow_mut().0.push(list);}
        else{self.client_item_stack_list_draw(batch,list);}
    }
    /// READ_FINAL_STATE: all producers have submitted before this one flush.
    pub(super) fn client_item_stack_lists_flush(&self,batch:&mut HudBatch){
        let mut lists=std::mem::take(&mut self.client_item_stack_lists.borrow_mut().0);
        let mut boxes=lists.iter().map(|l|{
            let size=self.client_item_stack_list_size(l);
            let f=|key:&str,default:f32|l.background[key].as_f64().map_or(default,|v|v as f32);
            let texture=l.background["background"]=="Texture"||l.background["texture"]==true;
            let padding=if texture{0.}else{f("margin",2.)+if f("glow",0.3)>0.{3.}else{0.}};
            layout::bounds(l.center,size,padding,l.scale)
        }).collect::<Vec<_>>();
        layout::rearrange(&mut boxes);
        for(list,bounds)in lists.iter_mut().zip(boxes){list.center=[bounds[0]+(bounds[2]-bounds[0])*0.5,bounds[1]+(bounds[3]-bounds[1])*0.5];}
        for list in lists{self.client_item_stack_list_draw(batch,list);}
    }
    fn client_item_stack_list_size(&self,list:&List)->[i32;2]{
        let title_width=list.title.as_ref().map(|t|self.gui.atlas.font.width_text(t)as i32);
        layout::dimensions(list.stacks.len(),list.row_length,title_width)
    }
    fn client_item_stack_list_draw(&self,batch:&mut HudBatch,list:List){
        let size=self.client_item_stack_list_size(&list);let first=batch.vertices.len();
        let(width,height)=(size[0]as f32,size[1]as f32);let(left,top)=(list.center[0]-width*0.5,list.center[1]-height*0.5);
        let s=&list.background;let texture=s["background"]=="Texture"||s["texture"]==true;
        let margin=s["margin"].as_f64().unwrap_or(2.)as f32;
        if !texture{self.client_item_stack_list_panel(batch,[left-margin,top-margin,left+width+margin,top+height+margin],s);}
        let mut item_top=top;
        if let Some(title)=list.title.as_ref(){let tw=self.gui.atlas.font.width_text(title)as i32;batch.text_styled(&self.gui.atlas,title,left+(size[0]/2-tw/2)as f32,top,list.title_color,true);item_top+=11.;}
        let previous_origin=self.client_better_inventory_origin.replace((left,item_top));
        for(i,stack)in list.stacks.iter().enumerate(){
            let(x,y)=(left+(i%list.row_length)as f32*18.,item_top+(i/list.row_length)as f32*18.);
            if texture{batch.sprite_scaled(&self.gui.atlas,"gui/sprites/container/slot",x,y,18.,18.,u32::MAX);}
            let Some(stack)=stack else{continue;};
            let seed=(i%list.row_length*18+1)as i32+((i/list.row_length*18+1)as i32).wrapping_mul(176);
            if let Some(renderer)=list.renderer.as_ref(){renderer(self,batch,i,stack,x+1.,y+1.,seed);}
            else{self.client_item_stack_list_draw_stack(batch,stack,x+1.,y+1.,seed);}
        }
        self.client_better_inventory_origin.set(previous_origin);
        batch.scale_since(first,list.center[0],list.center[1],list.scale,list.scale);
        // The native 3D GUI item layers own pixel scissors, independent of pose.
        for(range,blend)in &mut batch.blends{if range.start>=first as u32{match blend{
            horizon_client::hud::Blend::Depth(clip)|horizon_client::hud::Blend::DepthGlint(clip)=>{
                let cx=list.center[0]*batch.scale;let cy=list.center[1]*batch.scale;
                let left=cx+(clip[0]as f32-cx)*list.scale;let top=cy+(clip[1]as f32-cy)*list.scale;
                let right=left+clip[2]as f32*list.scale;let bottom=top+clip[3]as f32*list.scale;
                let x=left.max(0.)as u32;let y=top.max(0.)as u32;*clip=[x,y,(right.max(0.)as u32).saturating_sub(x),(bottom.max(0.)as u32).saturating_sub(y)];
            },_=>{}
        }}}
    }
    fn client_item_stack_list_draw_stack(&self,batch:&mut HudBatch,stack:&ItemStack,x:f32,y:f32,seed:i32){
        if stack.count==0||stack.count as i32<=0||&*stack.item=="minecraft:air"{return;}
        let owner=self.player_owner(Some(stack));
        let state=self.resolve_item(stack,horizon_assets::item_model::DisplayContext::Gui,Some(&owner),seed);
        horizon_client::item_render::gui::draw(&state,&self.pack.item_renderer,&self.gui.atlas.sprites,&self.item_host(),batch,x,y,self.glint_frame.millis/1000.,self.glint_frame.speed,self.glint_frame.alpha);
        self.draw_item_decorations(batch,stack,x,y,None);
    }
    fn client_item_stack_list_panel(&self,batch:&mut HudBatch,rect:[f32;4],s:&Value){
        let argb=|k:&str,default:u32|s[k].as_i64().map(|v|v as u32).or_else(||s[k].as_u64().map(|v|v as u32)).unwrap_or(default);
        let color=argb("color",0x80000000);let outline=argb("outlineColor",argb("outline",0));
        if color>>24==0&&outline>>24==0{return;}
        let native=|v:u32|rgba((v>>16)as u8,(v>>8)as u8,v as u8,(v>>24)as u8);let f=|k:&str,default:f32|s[k].as_f64().map_or(default,|v|v as f32);let radius=f("radius",4.);let glow=f("glow",0.3);
        if glow>0.{let color=if outline>>24==0{color}else{outline};for layer in(1..=3).rev(){let spread=layer as f32;let fade=glow/layer as f32;let alpha=if fade>=1.{color>>24}else{((color>>24)as f32*fade)as u32};let faded=(color&0xffffff)|(alpha<<24);batch.liquidbounce_rounded_rect([rect[0]-spread,rect[1]-spread,rect[2]+spread,rect[3]+spread],radius+spread,native(faded),native(faded),0.);}}
        let gradient=f("gradient",0.3);let[a,r,g,b]=color.to_be_bytes();let shade=|v:u8|gradient.mul_add(-(v as f32),v as f32).clamp(0.,255.)as u8;let bottom=u32::from_be_bytes([a,shade(r),shade(g),shade(b)]);
        batch.liquidbounce_rounded_rect(rect,radius,native(color),native(bottom),0.);batch.liquidbounce_rounded_rect(rect,radius,native(outline),native(outline),1.);
    }
}
