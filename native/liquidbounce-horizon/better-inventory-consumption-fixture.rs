#![allow(dead_code)]
extern crate self as horizon_client;
extern crate self as horizon_items;
use serde_json::Value;
#[cfg(test)] use serde_json::json;
use std::{cell::{Cell,RefCell},collections::HashMap};
use liquidbounce_behaviors::better_inventory::BetterInventory;
#[cfg(test)] use liquidbounce_behaviors::better_inventory::{HighlightMode,CooldownMode};
#[derive(Clone)] struct ItemStack{count:u32,item:String,patch:Option<Value>}
impl ItemStack {fn new(item:&str,count:u32)->Self{Self{item:item.into(),count,patch:None}}}
struct Atlas;
struct Gui{atlas:Atlas}
#[derive(Clone,Debug)] struct Vertex{pos:[f32;2]}
#[derive(Clone,Debug)] struct Operation{kind:String,start:usize,color:u32}
#[derive(Default)] struct HudBatch{vertices:Vec<Vertex>,operations:Vec<Operation>}
impl HudBatch {
    fn rect(&mut self,kind:&str,x:f32,y:f32,w:f32,h:f32,color:u32){self.operations.push(Operation{kind:kind.into(),start:self.vertices.len(),color});for pos in [[x,y],[x+w,y],[x+w,y+h],[x,y+h]]{self.vertices.push(Vertex{pos});}}
    fn fill(&mut self,_:&Atlas,x:f32,y:f32,w:f32,h:f32,color:u32){self.rect("fill",x,y,w,h,color);}
    #[allow(clippy::too_many_arguments)]
    fn sprite_scaled(&mut self,_:&Atlas,name:&str,x:f32,y:f32,w:f32,h:f32,color:u32){self.rect(name,x,y,w,h,color);}
    fn text_width(_:&Atlas,text:&str)->f32{(text.chars().count()*6) as f32}
    fn text(&mut self,_:&Atlas,text:&str,x:f32,y:f32,color:u32,_:bool){self.rect(text,x,y,text.len() as f32*6.0,9.0,color);}
    // The production HudBatch scale_since body, with the same vertex coordinates.
    fn scale_since(&mut self,start:usize,cx:f32,cy:f32,sx:f32,sy:f32){for v in &mut self.vertices[start..]{v.pos[0]=cx+(v.pos[0]-cx)*sx;v.pos[1]=cy+(v.pos[1]-cy)*sy;}}
}
fn rgba(r:u8,g:u8,b:u8,a:u8)->u32{u32::from_be_bytes([r,g,b,a])}
pub(crate) mod item_render {pub(super) fn stack_view(stack:&super::ItemStack)->super::ItemStack{stack.clone()}}
pub(crate) mod item_data {pub(super) fn stack_from_hover(value:&super::Value)->Option<super::ItemStack>{let count=value.get("count").and_then(super::Value::as_u64).unwrap_or(1) as u32;(count>0).then(||super::ItemStack::new(value["id"].as_str().unwrap_or("minecraft:air"),count))}}
struct Models{defaults:HashMap<String,Value>}
impl Models{fn components_view(&self,stack:&ItemStack)->serde_json::Map<String,Value>{let mut values=serde_json::Map::new();if let Some(component)=stack.patch.as_ref().or_else(||self.defaults.get(&stack.item)).filter(|value|!value.is_null()){values.insert("minecraft:container".into(),component.clone());}values}}
struct Renderer{models:Models}
struct Pack{item_renderer:Renderer,names:()}
pub(crate) struct Creative;
impl Creative{const DISPLAY:usize=900;}
struct CreativeScreen{items:Vec<ItemStack>,first:usize}
impl CreativeScreen{fn first_row(&self,_:&())->usize{self.first} fn items(&self,_:&())->&[ItemStack]{&self.items}}
#[derive(Default)] struct Cooldowns{entries:HashMap<String,(i32,i32)>}
impl Cooldowns{fn render_entry(&self,group:&str)->Option<(i32,i32)>{self.entries.get(group).copied()}}
struct Game<'a>{
    module:BetterInventory,running:bool,last:i32,player:bool,
    gui:Gui,pack:Pack,cooldowns:Cooldowns,
    client_better_inventory_slots:RefCell<Vec<Option<usize>>>,client_better_inventory_origin:Cell<(f32,f32)>,
    mapping:Vec<Option<usize>>,carried:Option<ItemStack>,cursor:(f32,f32),hover:Option<usize>,slots:HashMap<usize,ItemStack>,creative:bool,inventory_tab:bool,creative_screen:CreativeScreen,
    marker:std::marker::PhantomData<&'a ()>,
}
impl Default for Game<'_>{fn default()->Self{Self{module:BetterInventory::default(),running:true,last:0,player:true,gui:Gui{atlas:Atlas},pack:Pack{item_renderer:Renderer{models:Models{defaults:HashMap::new()}},names:()},cooldowns:Cooldowns::default(),client_better_inventory_slots:RefCell::new(Vec::new()),client_better_inventory_origin:Cell::new((0.0,0.0)),mapping:vec![Some(300),Some(9)],carried:None,cursor:(150.0,90.0),hover:None,slots:HashMap::new(),creative:false,inventory_tab:false,creative_screen:CreativeScreen{items:Vec::new(),first:0},marker:std::marker::PhantomData}}}
impl Game<'_>{
    fn client_inventory_mapping(&mut self,_:bool)->Vec<Option<usize>>{self.mapping.clone()}
    fn creative_tab_open(&self)->bool{self.creative&&!self.inventory_tab}
    fn creative_inventory_open(&self)->bool{self.creative&&self.inventory_tab}
    fn client_auto_tool_player(&self)->bool{self.player}
    fn slot_at(&self,_:&HudBatch,_:f32,_:f32)->Option<usize>{self.hover}
    fn slot_view(&mut self,slot:usize)->Option<ItemStack>{self.slots.get(&slot).cloned()}
    fn draw_item(&self,batch:&mut HudBatch,stack:&ItemStack,x:f32,y:f32){batch.rect(&stack.item,x,y,16.0,16.0,u32::MAX);self.client_better_inventory_cooldown(batch,stack,x,y,0.5,&stack.item);}
    /*HOST*/
}
mod mod_api {
    pub(super) struct Mods;
    pub(super) fn mods()->Mods{Mods}
    pub(super) fn dispatch_client_better_inventory_render(game:&super::Game<'_>,_:Mods,query:&str,draw:&mut String){*draw=super::better_inventory_draw::render(&game.module,game.running,game.last,query).unwrap_or(super::Value::Null).to_string();}
}
#[test]
fn real_windows_system_format_locale_matches_jvm_default(){
    let oracle:Value=serde_json::from_str(include_str!("locale-oracle.json")).unwrap();
    assert_eq!(Game::client_better_inventory_locale().to_ascii_lowercase(),oracle["default"].as_str().unwrap().to_ascii_lowercase());
}
#[test]
fn native_border_and_texture_use_real_menu_index_even_for_empty_slots(){
    let mut game=Game::default();let mut batch=HudBatch::default();game.client_better_inventory_begin(120.0,50.0);
    game.client_better_inventory_slot(&mut batch,9,128.0,192.0);assert!(batch.operations.is_empty());
    game.client_better_inventory_slot(&mut batch,300,128.0,68.0);assert_eq!(batch.operations.len(),4);assert_eq!(batch.vertices[0].pos,[128.0,68.0]);assert_eq!(batch.vertices[2].pos,[144.0,69.0]);assert_eq!(batch.operations[0].color,0x4677ffff);
    game.module.highlight_mode=HighlightMode::Texture;game.client_better_inventory_slot(&mut batch,300,128.0,68.0);assert_eq!(batch.operations[4].kind,"gui/sprites/hud/hotbar_selection");assert_eq!(batch.vertices[16].pos,[125.0,65.0]);assert_eq!(batch.vertices[18].pos,[147.0,86.0]);
    game.running=false;game.client_better_inventory_slot(&mut batch,300,128.0,68.0);assert_eq!(batch.operations.len(),5);
}
#[test]
fn native_cooldown_uses_actual_entry_color_and_current_pose_origin(){
    let mut game=Game::default();game.module.cooldown_mode=CooldownMode::DurationTicks;game.module.cooldown_scale=2.0;game.module.cooldown_color=0x80123456_u32 as i32;game.cooldowns.entries.insert("pearl".into(),(30,11));
    game.client_better_inventory_origin.set((120.0,50.0));let mut batch=HudBatch::default();game.client_better_inventory_cooldown(&mut batch,&ItemStack::new("pearl",1),128.0,68.0,0.5,"pearl");
    assert_eq!(batch.operations[0].kind,"19");assert_eq!(batch.operations[0].color,0x12345680);assert_eq!(batch.vertices[0].pos,[140.0,86.0]);assert_eq!(batch.vertices[2].pos,[164.0,104.0]);
    game.player=false;game.client_better_inventory_cooldown(&mut batch,&ItemStack::new("pearl",1),128.0,68.0,0.5,"pearl");assert_eq!(batch.operations.len(),1);
}
#[test]
fn native_container_uses_resolved_defaults_holes_filtering_outer_gate_and_nested_scale(){
    let mut game=Game::default();game.module.container_scale=2.0;game.module.cooldown_scale=2.0;game.hover=Some(300);game.slots.insert(300,ItemStack::new("box",1));game.pack.item_renderer.models.defaults.insert("box".into(),json!([{"slot":2,"item":{"id":"stone","count":3}}]));
    game.client_better_inventory_begin(120.0,50.0);let mut batch=HudBatch::default();game.client_better_inventory_end(&mut batch);
    // center=(mouse + offset)=(300,90), width54 and height18, scaled2.
    assert_eq!(batch.operations.iter().filter(|op|op.kind=="gui/sprites/container/slot").count(),3);assert_eq!(batch.vertices[0].pos,[246.0,72.0]);
    let item=batch.operations.iter().find(|op|op.kind=="stone").unwrap();assert_eq!(batch.vertices[item.start].pos,[320.0,74.0]);
    let text=batch.operations.iter().find(|op|op.kind=="50%").unwrap();assert_eq!(batch.vertices[text.start].pos,[390.0,76.0]);assert_eq!(game.client_better_inventory_origin.get(),(0.0,0.0));
    game.module.skip_empty_stack=true;game.client_better_inventory_begin(120.0,50.0);let mut filtered=HudBatch::default();game.client_better_inventory_end(&mut filtered);assert_eq!(filtered.operations.iter().filter(|op|op.kind=="gui/sprites/container/slot").count(),1);
    game.carried=Some(ItemStack::new("box",1));game.client_better_inventory_begin(120.0,50.0);let mut carried=HudBatch::default();game.client_better_inventory_end(&mut carried);assert!(carried.operations.is_empty());
    game.carried=None;game.slots.get_mut(&300).unwrap().patch=Some(Value::Null);game.client_better_inventory_begin(120.0,50.0);let mut removed=HudBatch::default();game.client_better_inventory_end(&mut removed);assert!(removed.operations.is_empty());
}
#[test]
fn creative_display_and_trash_keep_their_own_menu_indices(){
    let mut game=Game{creative:true,mapping:(0..54).map(|i|if i<45{None}else{Some(i-45)}).collect(),..Default::default()};let mut batch=HudBatch::default();game.last=44;game.client_better_inventory_begin(10.0,20.0);game.client_better_inventory_slot(&mut batch,944,19.0,38.0);assert_eq!(batch.operations.len(),4);
    game.inventory_tab=true;game.mapping=(0..46).map(Some).collect();game.last=46;game.client_better_inventory_begin(10.0,20.0);game.client_better_inventory_slot(&mut batch,1023,183.0,132.0);assert_eq!(batch.operations.len(),8);
}
