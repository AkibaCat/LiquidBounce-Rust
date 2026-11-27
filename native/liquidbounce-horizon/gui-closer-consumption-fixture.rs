extern crate self as horizon_client;
pub use horizon_client_core::text;
pub use horizon_client_core::lang;
pub use game::Screen;
pub mod mod_api {
    pub fn mods() -> &'static [()] { &[] }
    pub fn dispatch_client_gui_closer_screen(game: &crate::game::Game<'_>, _: &[()], title: &str, cancel: &mut bool, print_title: &mut bool) {
        game.events.borrow_mut().push("event");
        let decision=game.module.screen_opening(game.running,true,title);
        *cancel|=decision.cancel;*print_title|=decision.print_title;
    }
}
pub mod game {
    use crate::{mod_api,text::Text};
    use liquidbounce_behaviors::gui_closer::GuiCloser;
    use std::{cell::RefCell,marker::PhantomData};
    #[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Screen {Playing,Inventory,Creative,Options,WorldOptions,Paused}
    pub struct Panel {pub recipes_settings_return:Option<Screen>}
    pub struct CreativeScreen {pub reopened:usize}
    impl CreativeScreen {fn reopen(&mut self){self.reopened+=1;}}
    pub struct RecipeUi {pub reset:usize}
    impl RecipeUi {fn reset_screen(&mut self){self.reset+=1;}}
    pub struct Game<'a> {
        pub events:RefCell<Vec<&'static str>>,pub module:GuiCloser,pub running:bool,pub window:bool,
        pub lan_acting:Option<()>,pub lan_acting_uuid:Option<()>,pub screen:Screen,pub title:String,pub messages:Vec<Text>,pub right:bool,
        pub panel:Panel,pub options_to_world_options:bool,pub options_from_title:bool,pub dragging:bool,
        pub creative:bool,pub creative_screen:CreativeScreen,pub recipe_ui:RecipeUi,pub craft_size:i32,
        pub remote:Option<()>,
        marker:PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default()->Self {Self{
            events:RefCell::new(vec![]),module:GuiCloser::default(),running:true,window:true,
            lan_acting:None,lan_acting_uuid:None,screen:Screen::Playing,title:"Vote".into(),messages:vec![],right:true,
            panel:Panel{recipes_settings_return:None},options_to_world_options:false,options_from_title:false,dragging:true,
            creative:false,creative_screen:CreativeScreen{reopened:0},recipe_ui:RecipeUi{reset:0},craft_size:3,remote:None,marker:PhantomData,
        }}
    }
    impl Game<'_> {
        fn client_gui_closer_finish_screen(&mut self,accepted:bool)->bool{accepted}
        fn native_window(&self)->bool{self.window}
        fn client_container_menu_title(&self,next:Screen)->Option<(Option<String>,String)>{
            match next { Screen::Inventory=>Some((None,self.title.clone())),Screen::Creative=>Some((None,String::new())),_=>None }
        }
        fn system_message(&mut self,text:Text){self.events.borrow_mut().push("message");self.messages.push(text);}
        fn client_inventory_screen_replaced(&self){self.events.borrow_mut().push("lifecycle");}
        fn client_keymapping_screen(&self,next:Screen)->Screen{self.events.borrow_mut().push("keymapping");next}
        fn capture(&mut self,capture:bool){assert!(!capture);self.events.borrow_mut().push("capture");}
        fn show_calibration(&mut self,_: (f64,f64)){self.events.borrow_mut().push("calibration");}
        fn refresh_creative_tabs(&mut self){self.events.borrow_mut().push("creative_tabs");}
        fn refresh_saved_hotbars(&mut self){self.events.borrow_mut().push("saved_hotbars");}
        fn release_use_for_keybind_screen(&mut self){self.events.borrow_mut().push("release_use");self.right=false;}
        pub fn storage_open(&mut self){/*STORAGE_OPEN*/}
        pub fn dynamic_return(&mut self){/*DYNAMIC_RETURN*/}
        pub fn player_inventory(&mut self){/*PLAYER_INVENTORY*/}
        pub fn native_player_inventory(&mut self){/*NATIVE_PLAYER_INVENTORY*/}
        pub fn station_open(&mut self){/*STATION_OPEN*/}
        pub fn merchant_open(&mut self){/*MERCHANT_OPEN*/}
        pub fn server_open(&mut self){/*SERVER_BOUNDARY*/self.events.borrow_mut().push("native_init");}
    }
    mod native_hooks {
        use super::*;
        impl Game<'_> {
            /*HOST*/
        }
        #[cfg(test)] mod tests {
            use super::*;
            use crate::text::ClickEvent;
            use liquidbounce_behaviors::gui_closer::{Filter,Mode};
            #[test] fn cancellation_precedes_lifecycle_keymapping_capture_and_mouse_release() {
                let mut g=Game::default();g.storage_open();
                assert_eq!(g.screen,Screen::Playing);assert!(g.right);assert_eq!(*g.events.borrow(),["event"]);
                g.title="Shop".into();g.storage_open();
                assert_eq!(g.screen,Screen::Inventory);assert!(!g.right);
                assert_eq!(*g.events.borrow(),["event","event","lifecycle","keymapping","capture"]);
            }
            #[test] fn dynamic_rhs_is_evaluated_before_cancellation_and_cannot_leak_to_next_return() {
                let mut g=Game{screen:Screen::Options,..Game::default()};
                g.panel.recipes_settings_return=Some(Screen::Inventory);g.dynamic_return();
                assert!(g.panel.recipes_settings_return.is_none());assert_eq!(g.screen,Screen::Options);assert!(g.dragging);
                assert_eq!(*g.events.borrow(),["event"]);
                g.dynamic_return();assert_eq!(g.screen,Screen::Paused);assert!(!g.dragging);
                assert_eq!(*g.events.borrow(),["event","lifecycle","keymapping","calibration"]);
            }
            #[test] fn creative_and_inventory_init_and_keybind_use_release_obey_same_cancel() {
                for creative in [false,true] {
                    let mut g=Game{creative,..Game::default()};
                    g.module.filters=vec![Some(Filter::new(".*").unwrap())];
                    g.player_inventory();assert_eq!(g.screen,Screen::Playing);assert!(g.right);
                    assert_eq!(g.creative_screen.reopened,0);assert_eq!(g.recipe_ui.reset,0);assert_eq!(g.craft_size,3);
                    assert_eq!(*g.events.borrow(),["event"]);
                    g.running=false;g.player_inventory();
                    assert_eq!(g.screen,if creative{Screen::Creative}else{Screen::Inventory});assert!(!g.right);
                    assert_eq!(g.creative_screen.reopened,usize::from(creative));assert_eq!(g.recipe_ui.reset,1);assert_eq!(g.craft_size,2);
                    let events=g.events.borrow();assert!(events.iter().position(|v|*v=="event").unwrap()<events.iter().position(|v|*v=="lifecycle").unwrap());
                    assert_eq!(events.last(),Some(&"release_use"));
                }
            }
            #[test] fn real_text_has_original_gradient_gray_body_blue_bold_copy_and_translatable_hover() {
                let mut g=Game{title:"Title 😀 标题".into(),module:GuiCloser{print_screen_title:true,..GuiCloser::default()},..Game::default()};
                g.storage_open();assert_eq!(g.messages.len(),1);
                let t=&g.messages[0];assert_eq!(t.plain(),"LiquidBounce ▸ GUICloser: container screen title: Title 😀 标题");
                assert_eq!(t.spans.len(),14);assert_eq!(t.spans[0].style.color,Some(0x4677ff));assert_eq!(t.spans[11].style.color,Some(0x24aa7f));
                assert_eq!(t.spans[12].style.color,Some(0xaaaaaa));assert!(!t.spans[12].style.is_bold());
                let title=&t.spans[13];assert_eq!(title.style.color,Some(0x4677ff));assert!(title.style.is_bold());
                assert_eq!(title.style.click,Some(ClickEvent::CopyToClipboard(g.title.clone())));
                assert_eq!(title.style.hover.as_ref().unwrap().translation_key(),Some("chat.copy.click"));
                assert_eq!(*g.events.borrow(),["event","message","lifecycle","keymapping","capture"]);
            }
            #[test] fn native_mapped_inventory_cancels_before_recipe_observer_init_capture_and_release() {
                for creative in [false,true] {
                    let mut g=Game{creative,..Game::default()};
                    g.module.filters=vec![Some(Filter::new(".*").unwrap())];
                    g.native_player_inventory();assert_eq!(g.screen,Screen::Playing);assert!(g.right);
                    assert_eq!(g.creative_screen.reopened,0);assert_eq!(g.recipe_ui.reset,0);assert_eq!(g.craft_size,3);
                    assert_eq!(*g.events.borrow(),["event"]);
                    g.running=false;g.native_player_inventory();
                    assert_eq!(g.screen,if creative{Screen::Creative}else{Screen::Inventory});assert!(!g.right);
                    assert_eq!(g.creative_screen.reopened,usize::from(creative));assert_eq!(g.recipe_ui.reset,1);assert_eq!(g.craft_size,2);
                    assert_eq!(g.events.borrow().iter().filter(|v|**v=="lifecycle").count(),1);
                    assert_eq!(g.events.borrow().last(),Some(&"release_use"));
                }
            }
            #[test] fn remote_recipe_state_is_preserved_by_all_cancelled_open_branches() {
                for branch in 0..3 {
                    let mut g=Game{remote:Some(()),..Game::default()};
                    let open=|g:&mut Game<'_>|match branch{0=>g.station_open(),1=>g.merchant_open(),_=>g.server_open()};
                    open(&mut g);assert_eq!(g.recipe_ui.reset,0);assert_eq!(*g.events.borrow(),["event"]);
                    g.title="Shop".into();open(&mut g);assert_eq!(g.recipe_ui.reset,1);
                    assert!(g.events.borrow().contains(if branch==2{&"native_init"}else{&"capture"}));
                }
                let mut local=Game{title:"Shop".into(),..Game::default()};local.merchant_open();assert_eq!(local.recipe_ui.reset,0);
            }
            #[test] fn actual_language_selects_english_simplified_and_traditional_bodies_with_identical_title_style() {
                use crate::lang::{Language,scoped};
                for (code,body) in [("en_us","container screen title"),("zh_cn","容器界面标题"),("zh_tw","容器介面標題"),("fr_fr","container screen title")] {
                    scoped(Language::load(&[],code),||{
                        let mut g=Game{title:"Shop 😀 商店".into(),module:GuiCloser{print_screen_title:true,..GuiCloser::default()},..Game::default()};
                        g.storage_open();let text=&g.messages[0];
                        assert_eq!(text.plain(),format!("LiquidBounce ▸ GUICloser: {body}: Shop 😀 商店"));
                        let title=&text.spans[13];assert_eq!(title.text,g.title);assert_eq!(title.style.color,Some(0x4677ff));assert!(title.style.is_bold());
                        assert_eq!(title.style.click,Some(ClickEvent::CopyToClipboard(g.title.clone())));
                        assert_eq!(title.style.hover.as_ref().unwrap().translation_key(),Some("chat.copy.click"));
                    });
                }
            }
            #[test] fn gates_and_null_filter_short_circuit_use_real_behavior() {
                for gate in [0,1,2] {let mut g=Game::default();match gate{0=>g.window=false,1=>g.lan_acting=Some(()),_=>g.lan_acting_uuid=Some(())};g.storage_open();assert_eq!(*g.events.borrow(),["lifecycle","keymapping","capture"]);}
                let mut g=Game::default();g.module.mode=Mode::Contains;g.module.print_screen_title=true;g.module.filters=vec![None,Some(Filter::new("Vote").unwrap())];
                g.storage_open();assert!(g.messages.is_empty());assert_eq!(g.screen,Screen::Inventory);
                g.events.borrow_mut().clear();g.screen=Screen::Playing;g.module.filters.reverse();g.storage_open();assert_eq!(*g.events.borrow(),["event"]);assert_eq!(g.screen,Screen::Playing);
            }
        }
    }
}
