#![cfg(test)]
#![allow(dead_code,unused_imports)]
extern crate self as horizon_client;
#[path="behavior.rs"] mod behavior;
mod text {
    use std::sync::atomic::{AtomicU8,Ordering};
    static LANGUAGE:AtomicU8=AtomicU8::new(0);
    pub fn language(code:u8){LANGUAGE.store(code,Ordering::Release);}
    #[derive(Clone)] pub struct Text(pub String);
    impl Text {
        pub fn literal(text:String)->Self{Self(text)}
        pub fn resolve_legacy(self)->Self{self}
        pub fn plain(self)->String{self.0}
        pub fn translate(key:&str,_:&[Text])->Self {
            let dictionary=match LANGUAGE.load(Ordering::Acquire){1=>crate::CN,2=>crate::TW,_=>crate::EN};
            Self(dictionary.iter().find(|(k,_)|*k==key).map_or(key,|(_,v)|*v).to_owned())
        }
    }
}
__LANGUAGE_DICTIONARIES__
mod containers {
    pub enum BlockEntity{Chest(()),Brewing(()),Crafter(()),Furnace(Furnace),Mod(()),Other}
    pub struct Furnace{pub kind:FurnaceKind}
    pub struct FurnaceKind(pub &'static str);
    impl FurnaceKind{pub fn recipe_type(&self)->&str{self.0}}
}
mod mod_api {
    pub trait RenderHooks:Send+Sync{}
    mod generated_host{include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}
    pub use generated_host::*;
    mod generated{include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}
    pub use generated::*;
    pub fn entered<'a>(mods:&'a[Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(|module|&**module)}
    pub fn mods()->&'static[Box<dyn Hooks>]{&[]}
}
mod game {
    use crate::containers::BlockEntity;
    use crate::text::Text;
    #[derive(Clone,Copy,Default,PartialEq,Eq)] enum Screen{#[default] Playing,Inventory,Creative,Chat}
    #[derive(Default)] struct MountMenu{mob:Option<u32>}
    impl MountMenu{fn opened_mob(&self)->Option<u32>{self.mob}}
    struct Mob{id:u32,custom_name:Option<String>,kind:MobKind}
    struct MobKind{id:String}
    mod stations {
        #[derive(Clone,Copy)] pub enum Kind{Enchanting,Anvil,Grindstone,Smithing,Stonecutter,Merchant,Beacon,Cartography,Loom}
        impl Kind{pub fn title_key(self)->&'static str {match self{
            Self::Enchanting=>"container.enchant",Self::Anvil=>"container.repair",Self::Grindstone=>"container.grindstone_title",
            Self::Smithing=>"container.upgrade",Self::Stonecutter=>"container.stonecutter",Self::Merchant=>"merchant.trades",
            Self::Beacon=>"container.beacon",Self::Cartography=>"container.cartography_table",Self::Loom=>"container.loom"}}}
    }
    struct Merchant{name:Option<String>,session:MerchantSession,progress:bool}
    impl Merchant{fn progress(&self)->bool{self.progress}}
    struct MerchantSession{profession:String}
    struct Station{kind:stations::Kind,merchant:Option<Merchant>}
    struct Storage{entity:BlockEntity,block:Option<std::sync::Arc<str>>,size:usize,rows:Option<usize>,custom:Option<String>,cart:bool}
    #[derive(Default)] struct Game<'a>{screen:Screen,title:Option<()>,mount_menu:MountMenu,mobs:Vec<Mob>,station:Option<Station>,
        craft_size:i32,container:Option<(i32,i32,i32)>,storage:Option<Storage>,remote_menu:Option<String>,remote_title:Option<Text>,
        marker:std::marker::PhantomData<&'a()>}
    impl Game<'_>{
        fn native_window(&self)->bool{true}
        fn translate(&self,key:&str,_fallback:&str)->String{Text::translate(key,&[]).plain()}
        fn remote_menu_type(&self)->Option<&str>{self.remote_menu.as_deref()}
        fn remote_menu_title(&self)->Option<Text>{self.remote_title.clone()}
        fn container_kind(&self)->Option<&BlockEntity>{self.storage.as_ref().map(|s|&s.entity)}
        fn container_size(&self)->usize{self.storage.as_ref().map_or(0,|s|s.size)}
        fn container_block(&self)->Option<std::sync::Arc<str>>{self.storage.as_ref().and_then(|s|s.block.clone())}
        fn container_block_is_shulker(&self)->bool{self.container_block().is_some_and(|name|name.ends_with("shulker_box"))}
        fn container_title(&self,default:String)->String{self.storage.as_ref().and_then(|s|s.custom.clone()).unwrap_or(default)}
        fn chest_rows(&self)->Option<usize>{self.storage.as_ref().and_then(|s|s.rows)}
        fn open_cart_kind(&self)->Option<()>{self.storage.as_ref().and_then(|s|s.cart.then_some(()))}
        fn mod_menu_at(&self,_:(i32,i32,i32))->Option<&'static str>{Some("test:menu")}
        __CHEST_TITLE__
        fn remote_chest_menu(&self)->Option<(usize,Text)>{None}
    }
    mod auto_f5;
    mod native_hooks{use super::{Game,Screen};use crate::mod_api;impl Game<'_>{__NATIVE_HELPERS__}}
    mod storage{use super::Game;impl Game<'_>{__STORAGE_HELPER__}}
    mod mod_api_host{include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}
    fn context(game:&Game<'_>)->Option<(Option<String>,String,Vec<String>)>{crate::mod_api::Host::auto_f5_screen(game)}
    #[test] fn actual_helper_restores_perspective_for_non_container_and_null_type_for_creative_inventory_mount(){
        let mut game=Game::default();assert!(context(&game).is_none());
        game.screen=Screen::Chat;assert!(context(&game).is_none());
        game.screen=Screen::Creative;assert_eq!(context(&game),Some((None,String::new(),vec![])));
        game.screen=Screen::Inventory;let c=context(&game).unwrap();assert!(c.0.is_none());
        assert_eq!(c.1,Text::translate("container.crafting",&[]).plain());
        game.craft_size=3;assert_eq!(context(&game).unwrap().0,Some("minecraft:crafting".into()));
        game.mount_menu.mob=Some(42);game.mobs.push(Mob{id:42,custom_name:Some("My Horse".into()),kind:MobKind{id:"minecraft:horse".into()}});
        game.remote_menu=Some("horizon:mount3".into());assert_eq!(context(&game),Some((None,"My Horse".into(),vec![])));
        game.title=Some(());assert!(context(&game).is_none());
    }
    #[test] fn imported_unknown_registry_types_match_actual_null_menu_on_inventory_creative_and_mount(){
        let mut module=crate::behavior::AutoF5{check_screen_title:false,types:[None].into(),..Default::default()};
        let mut game=Game{screen:Screen::Inventory,..Default::default()};
        for screen in [Screen::Inventory,Screen::Creative]{game.screen=screen;let c=context(&game).unwrap();
            assert_eq!(module.perspective(true,2,Some((c.0.as_deref(),&c.1,&c.2))),1);
        }
        game.screen=Screen::Inventory;game.mount_menu.mob=Some(1);game.mobs.push(Mob{id:1,custom_name:None,kind:MobKind{id:"minecraft:horse".into()}});
        let c=context(&game).unwrap();assert_eq!(module.perspective(true,2,Some((c.0.as_deref(),&c.1,&c.2))),1);
        module.type_filter=crate::behavior::Filter::Blacklist;
        assert_eq!(module.perspective(true,2,Some((c.0.as_deref(),&c.1,&c.2))),2);
    }
    #[test] fn actual_remote_title_matches_fresh_game_language_and_custom_titles_are_not_guessed(){
        let mut game=Game{screen:Screen::Inventory,remote_menu:Some("minecraft:generic_9x3".into()),..Default::default()};
        for language in [0,1,2]{
            crate::text::language(language);game.remote_title=Some(Text::translate("container.chest",&[]));
            let c=context(&game).unwrap();assert_eq!(c.2,vec!["Chest"]);
            assert_eq!(crate::behavior::AutoF5::default().perspective(true,2,Some((c.0.as_deref(),&c.1,&c.2))),1);
        }
        game.remote_title=Some(Text::literal("VIP Shop".into()));assert!(context(&game).unwrap().2.is_empty());
        crate::text::language(0);
    }
    #[test] fn complete_original_title_key_enum_matches_current_language_including_all_chest_boat_species(){
        assert_eq!(auto_f5::TITLE_KEYS.len(),16);
        for language in [0,1,2]{crate::text::language(language);
            for (expected,keys) in auto_f5::TITLE_KEYS {for key in *keys {
                let title=Text::translate(key,&[]).plain();
                assert!(auto_f5::matching_titles(&title,|key|Text::translate(key,&[]).plain()).iter().any(|tag|tag==expected));
            }}
        }
        crate::text::language(0);
    }
    #[test] fn actual_storage_and_station_helpers_use_registered_type_and_screen_title(){
        let mut game=Game{screen:Screen::Inventory,container:Some((0,0,0)),..Default::default()};
        for (entity,block,size,rows,expected_type,expected_title) in [
            (BlockEntity::Chest(()),"minecraft:chest",27,Some(3),"minecraft:generic_9x3","Chest"),
            (BlockEntity::Chest(()),"minecraft:chest",54,Some(6),"minecraft:generic_9x6","LargeChest"),
            (BlockEntity::Chest(()),"minecraft:barrel",27,Some(3),"minecraft:generic_9x3","Barrel"),
            (BlockEntity::Chest(()),"minecraft:ender_chest",27,Some(3),"minecraft:generic_9x3","EnderChest"),
            (BlockEntity::Chest(()),"minecraft:red_shulker_box",27,None,"minecraft:shulker_box","ShulkerBox"),
            (BlockEntity::Chest(()),"minecraft:hopper",5,None,"minecraft:hopper","Hopper"),
            (BlockEntity::Chest(()),"minecraft:dropper",9,None,"minecraft:generic_3x3","Dropper"),
            (BlockEntity::Brewing(()),"minecraft:brewing_stand",5,None,"minecraft:brewing_stand","BrewingStand"),
            (BlockEntity::Furnace(crate::containers::Furnace{kind:crate::containers::FurnaceKind("minecraft:blasting")}),"minecraft:blast_furnace",3,None,"minecraft:blast_furnace","BlastFurnace"),
        ] {
            game.storage=Some(Storage{entity,block:Some(block.into()),size,rows,custom:None,cart:false});
            let c=context(&game).unwrap();assert_eq!(c.0.as_deref(),Some(expected_type));assert!(c.2.iter().any(|tag|tag==expected_title));
            game.storage.as_mut().unwrap().custom=Some("Custom Chest".into());assert!(context(&game).unwrap().2.is_empty());
        }
        game.storage=None;game.container=None;
        game.station=Some(Station{kind:stations::Kind::Beacon,merchant:None});
        assert_eq!(context(&game).unwrap().0,Some("minecraft:beacon".into()));assert_eq!(context(&game).unwrap().2,vec!["Beacon"]);
        game.station=Some(Station{kind:stations::Kind::Merchant,merchant:Some(Merchant{name:Some("Trader Joe".into()),session:MerchantSession{profession:"minecraft:farmer".into()},progress:true})});
        assert_eq!(context(&game).unwrap().1,"Trader Joe");
    }
}
