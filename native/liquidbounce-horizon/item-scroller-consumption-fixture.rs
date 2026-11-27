#![forbid(unsafe_code)]
#![allow(dead_code)]
extern crate self as horizon_client;
pub mod item_scroller;
pub mod container_screen;
pub mod menu {
    #[derive(Clone,Copy,Debug,PartialEq,Eq)]
    pub enum Input {Pickup,QuickMove,Swap,Clone,Throw,QuickCraft,PickupAll}
}
pub mod horizon_protocol {
    pub mod packets {pub mod play {
        #[derive(Clone,Debug)]pub struct SetCreativeModeSlot {pub slot:i16,pub item:Option<crate::game::ItemStack>}
        #[derive(Clone,Debug)]pub enum Serverbound {SetCreativeModeSlot(SetCreativeModeSlot)}
    }}
}
pub mod game {
    use std::{cell::RefCell,collections::HashSet};
    use crate::{item_scroller::ItemScroller,menu::Input};
    #[derive(Clone,Copy,Hash,PartialEq,Eq)]pub enum KeyCode {ShiftLeft,ShiftRight,KeyX}
    #[derive(Clone,Copy,Hash,PartialEq,Eq)]pub enum MouseButton {Left,Right,Middle}
    #[derive(Clone,Debug,PartialEq,Eq)]pub struct ItemStack {pub item:Box<str>,pub count:u32}
    pub struct Config {pub width:u32,pub height:u32}
    pub struct Renderer {pub config:Config}
    pub struct HudBatch;
    impl HudBatch {pub fn new(_:[u32;2])->Self {Self}}
    pub struct Creative;
    impl Creative {pub const DISPLAY:usize=1000;}
    pub struct CreativeScreen {pub first:usize,pub items:Vec<ItemStack>}
    impl CreativeScreen {
        pub fn first_row(&self,_:&())->usize {self.first}
        pub fn items(&self,_:&())->&[ItemStack] {&self.items}
    }
    pub struct Pack {pub names:()}
    pub struct Remote {pub craft_result:Option<ItemStack>}
    pub struct Equipment;
    #[derive(Clone,Debug,PartialEq,Eq)]pub enum Event {Query,Click(Option<usize>,i32,Input),Equipment,Apply,Packet(i16),Cache,Changed(u32),Reset}
    pub struct Game {
        pub native:bool,pub lan_acting:Option<()>,pub lan_acting_uuid:Option<()>,
        pub physical_keys:HashSet<KeyCode>,pub physical_buttons:HashSet<MouseButton>,
        pub renderer:Option<Renderer>,pub cursor:(f32,f32),pub hovered:Option<usize>,
        pub creative_inventory:bool,pub creative_tab:bool,pub creative_screen:CreativeScreen,pub pack:Pack,
        pub carried:Option<ItemStack>,pub slot_stack:Option<ItemStack>,pub last_quick_moved:Option<ItemStack>,
        pub inventory:Vec<Option<ItemStack>>,pub armor:Vec<Option<ItemStack>>,pub offhand:Option<ItemStack>,pub craft_grid:Vec<Option<ItemStack>>,
        pub remote:Option<Remote>,pub container:Option<u32>,
        pub inv_clicks:crate::container_screen::Clicks,pub last_inv_click:Option<(std::time::Instant,Option<usize>,bool)>,
        pub behavior:RefCell<ItemScroller>,pub running:bool,pub events:RefCell<Vec<Event>>,
    }
    impl Default for Game {
        fn default()->Self {
            Self {native:true,lan_acting:None,lan_acting_uuid:None,physical_keys:HashSet::from([KeyCode::ShiftLeft]),physical_buttons:HashSet::from([MouseButton::Left]),renderer:Some(Renderer{config:Config{width:800,height:600}}),cursor:(0.0,0.0),hovered:Some(5),creative_inventory:false,creative_tab:false,creative_screen:CreativeScreen{first:0,items:vec![]},pack:Pack{names:()},carried:None,slot_stack:Some(stack("stone",8)),last_quick_moved:None,inventory:vec![None;36],armor:vec![None;4],offhand:None,craft_grid:vec![None;9],remote:None,container:Some(7),inv_clicks:Default::default(),last_inv_click:None,behavior:RefCell::new(ItemScroller{delay:[0,0],..Default::default()}),running:true,events:RefCell::new(vec![])}
        }
    }
    pub fn stack(name:&str,count:u32)->ItemStack {ItemStack{item:name.into(),count}}
    impl Game {
        fn native_window(&self)->bool {self.native}
        fn creative_inventory_open(&self)->bool {self.creative_inventory}
        fn creative_tab_open(&self)->bool {self.creative_tab}
        fn creative_trash_hovered(_:&HudBatch,cursor:(f32,f32))->bool {cursor==(999.0,999.0)}
        fn slot_at(&self,_:&HudBatch,_:f32,_:f32)->Option<usize> {self.hovered}
        fn slot_view(&mut self,slot:usize)->Option<ItemStack> {assert!(slot<Creative::DISPLAY);self.slot_stack.clone()}
        fn player_equipment(&self)->Equipment {Equipment}
        fn player_equipment_changed(&mut self,_:&Equipment) {self.events.borrow_mut().push(Event::Equipment);}
        fn apply_equipment(&mut self) {self.events.borrow_mut().push(Event::Apply);}
        fn menu_click(&mut self,slot:Option<usize>,button:i32,input:Input)->bool {
            assert_eq!(self.last_quick_moved,self.slot_stack);
            self.events.borrow_mut().push(Event::Click(slot,button,input));true
        }
        fn container_changed(&mut self,key:u32) {self.events.borrow_mut().push(Event::Changed(key));}
        fn to_network(&self,stack:&Option<ItemStack>)->Option<ItemStack> {stack.clone()}
        fn send(&mut self,packet:crate::horizon_protocol::packets::play::Serverbound) {
            let crate::horizon_protocol::packets::play::Serverbound::SetCreativeModeSlot(packet)=packet;
            assert!(packet.item.is_none());self.events.borrow_mut().push(Event::Packet(packet.slot));
        }
        fn remote_synced(&mut self) {self.events.borrow_mut().push(Event::Cache);}
    }
    pub mod native_hooks {
        use super::*;
        use crate::horizon_protocol;
        mod mod_api {
            use super::*;
            pub struct Mods;
            pub fn mods()->Mods {Mods}
            pub fn dispatch_client_item_scroller_query(game:&Game,_:Mods,shift:bool,left:bool,allowed:&mut bool) {
                game.events.borrow_mut().push(Event::Query);
                *allowed|=game.behavior.borrow().can_perform_scroll(game.running,shift,left,|first,_|first,Game::client_item_scroller_wall_time);
            }
            pub fn dispatch_client_item_scroller_reset(game:&Game,_:Mods,now:i64) {
                game.events.borrow_mut().push(Event::Reset);game.behavior.borrow_mut().reset_chronometer(now);
            }
        }
        impl Game {
/*HOST*/
        }
    }
    #[cfg(test)]mod tests {
        use super::*;
        use crate::container_screen::{Button,Release,Target};
        #[test]fn actual_helper_records_before_click_then_updates_release_click_and_timer() {
            let mut game=Game::default();let before=std::time::Instant::now();
            game.client_item_scroller_render(&HudBatch);
            assert_eq!(*game.events.borrow(),[Event::Query,Event::Click(Some(5),1,Input::QuickMove),Event::Changed(7),Event::Reset]);
            assert_eq!(game.last_quick_moved,game.slot_stack);
            let (at,slot,right)=game.last_inv_click.unwrap();assert!(at>=before);assert_eq!((slot,right),(Some(5),false));
            assert!(game.behavior.borrow().last_update>0);
            assert_eq!(game.inv_clicks.release(&Target::Slot(5),Button::Left,true,false,true,false),Release::Nothing);
        }
        #[test]fn render_carried_and_hover_guards_precede_query_but_click_cancellation_allows_carried() {
            let mut game=Game {carried:Some(stack("stone",1)),..Default::default()};
            game.client_item_scroller_render(&HudBatch);assert!(game.events.borrow().is_empty());
            assert!(game.client_item_scroller_cancel_click());assert_eq!(*game.events.borrow(),[Event::Query]);
            assert_eq!(game.behavior.borrow().last_update,0);
            game.events.borrow_mut().clear();game.carried=None;game.hovered=None;
            game.client_item_scroller_render(&HudBatch);assert!(!game.client_item_scroller_cancel_click());assert!(game.events.borrow().is_empty());
        }
        #[test]fn actual_helper_uses_physical_shift_and_left_and_original_enable_guard() {
            let mut game=Game::default();game.physical_keys.clear();assert!(!game.client_item_scroller_cancel_click());
            game.physical_keys.insert(KeyCode::ShiftRight);assert!(game.client_item_scroller_cancel_click());
            game.physical_buttons.clear();assert!(!game.client_item_scroller_cancel_click());
            game.physical_buttons.insert(MouseButton::Left);game.running=false;assert!(!game.client_item_scroller_cancel_click());
            game.running=true;game.native=false;let before=game.events.borrow().len();assert!(!game.client_item_scroller_cancel_click());assert_eq!(game.events.borrow().len(),before);
        }
        #[test]fn real_skip_next_release_assignment_preserves_quick_craft() {
            let mut game=Game::default();game.inv_clicks.press(&Target::Slot(5),Button::Left,false,false,false);
            assert_eq!(game.inv_clicks.quick_crafting(),Some(Button::Left));game.client_item_scroller_render(&HudBatch);
            assert_eq!(game.inv_clicks.quick_crafting(),Some(Button::Left));
            assert_eq!(game.inv_clicks.release(&Target::Slot(5),Button::Left,true,true,true,false),Release::Nothing);
        }
        #[test]fn creative_display_copies_actual_shown_stack_and_keeps_menu_callback() {
            let mut game=Game {creative_tab:true,hovered:Some(Creative::DISPLAY+2),..Default::default()};
            game.creative_screen.first=1;game.creative_screen.items=vec![stack("stone",3);12];
            game.creative_screen.items[11]=stack("diamond",64);game.slot_stack=Some(stack("diamond",64));
            game.client_item_scroller_render(&HudBatch);assert_eq!(game.last_quick_moved,Some(stack("diamond",64)));
            assert!(game.events.borrow().contains(&Event::Click(Some(Creative::DISPLAY+2),1,Input::QuickMove)));
        }
        #[test]fn creative_trash_clears_46_slots_sends_every_empty_then_updates_cache_and_timer() {
            let mut game=Game {creative_inventory:true,cursor:(999.0,999.0),remote:Some(Remote{craft_result:Some(stack("stone",1))}),carried:Some(stack("minecraft:air",1)),..Default::default()};
            game.inventory.fill(Some(stack("stone",64)));game.armor.fill(Some(stack("diamond",1)));game.offhand=Some(stack("torch",2));game.craft_grid[..4].fill(Some(stack("oak_log",1)));
            game.client_item_scroller_render(&HudBatch);
            assert!(game.inventory.iter().chain(&game.armor).all(Option::is_none));assert!(game.offhand.is_none());assert!(game.craft_grid[..4].iter().all(Option::is_none));assert!(game.remote.as_ref().unwrap().craft_result.is_none());
            assert_eq!(game.carried,Some(stack("minecraft:air",1)));assert!(game.last_quick_moved.is_none());
            let events=game.events.borrow();assert_eq!(events[0..3],[Event::Query,Event::Equipment,Event::Apply]);
            for slot in 0..46 {assert_eq!(events[slot+3],Event::Packet(slot as i16));}
            assert_eq!(events[49..],[Event::Cache,Event::Reset]);
        }
    }
}
