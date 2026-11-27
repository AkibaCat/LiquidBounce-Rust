#![allow(dead_code, clippy::clone_on_copy)] // Recording stacks use u32; actual ItemStack is owned.
extern crate self as horizon_client;
pub use horizon_client_core::{text,scoreboard};
mod game {
    use std::{cell::Cell, collections::HashMap, marker::PhantomData};
    type ItemStack = u32;
    type BlockEntity = Vec<u32>;
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] enum Screen { #[default] Playing, Inventory, Creative }
    mod stations { #[derive(Debug)] pub(super) struct Station(pub Vec<u32>); }
    mod mount_menu { #[derive(Default, Debug)] pub(super) struct MountMenu(pub Vec<u32>); }
    mod native_hooks { #[derive(Debug)] pub(super) struct OpenNativeScreen(pub u64); }
    #[derive(Default)] struct Remote { menu:Option<(i32,String)>, menu_title:Option<crate::text::Text>, craft_result:Option<u32>, state_id:i32, packets:Vec<i32> }
    #[derive(Default)] struct World { block_entities:HashMap<(i32,i32,i32),BlockEntity> }
    #[derive(Default)] struct Game<'a> {
        screen:Screen, native:bool, lan_acting:Option<()>, lan_acting_uuid:Option<()>,
        client_inventory_screen_serial:Cell<u64>,
        client_gui_closer_view:Option<gui_closer_view::ContainerView>, client_gui_closer_other:Option<gui_closer_view::ContainerView>,
        client_gui_closer_in_view:bool, client_gui_closer_display_remote:Option<gui_closer_view::RemoteView>,
        client_gui_closer_display_title:Option<(Option<String>,crate::text::Text)>,
        carried:Option<u32>, craft_grid:Vec<Option<u32>>, craft_size:usize, craft_table:Option<[i32;3]>,
        container:Option<(i32,i32,i32)>, open_container_slots:Option<((i32,i32,i32),usize)>,
        station:Option<stations::Station>, mount_menu:mount_menu::MountMenu, mod_screen:Option<native_hooks::OpenNativeScreen>,
        world:World, remote:Option<Remote>, raw_title:crate::text::Text, other_half:Option<(i32,i32,i32)>,
        inventory:Vec<u32>, recipe_ui:Vec<u32>, marker:PhantomData<&'a ()>,
    }
    mod gui_closer_view { /*VIEW*/ }
    impl Game<'_> {
        fn native_window(&self)->bool { self.native }
        fn container_parts(&self)->Vec<((i32,i32,i32),usize)> { self.container.into_iter().chain(self.other_half).map(|key|(key,9)).collect() }
        fn client_container_menu_text(&self,_:Screen)->Option<(Option<String>,crate::text::Text)> { Some((None,self.raw_title.clone())) }
        /*REMOTE_MENU*/
    }
    fn old()->Game<'static> { Game { screen:Screen::Inventory,native:true,craft_size:2,craft_grid:vec![Some(1)],carried:Some(2),
        remote:Some(Remote{menu:Some((0,"inventory".into())),state_id:5,..Default::default()}),
        station:Some(stations::Station(vec![3])),mount_menu:mount_menu::MountMenu(vec![4]),mod_screen:Some(native_hooks::OpenNativeScreen(6)),
        raw_title:crate::text::Text::translate("container.crafting",&[]),inventory:vec![10],..Default::default() } }
    fn opened(g:&mut Game<'_>,id:i32) { g.client_gui_closer_prepare_menu();g.remote.as_mut().unwrap().menu=Some((id,"minecraft:generic_9x1".into()));g.craft_size=3;g.carried=Some(20);g.craft_grid=vec![Some(21)];g.station=Some(stations::Station(vec![22]));g.mount_menu=mount_menu::MountMenu(vec![23]);g.mod_screen=Some(native_hooks::OpenNativeScreen(24));assert!(!g.client_gui_closer_finish_screen(false)); }
    #[test] fn cancelled_old_screen_and_new_active_menu_have_independent_owned_state() {
        let mut g=old();opened(&mut g,7);
        assert_eq!(g.remote_menu().0,7);assert_eq!(g.carried,Some(20));
        g.with_gui_closer_view(|g| { assert_eq!(g.remote_menu().0,0);assert_eq!(g.carried,Some(2));assert_eq!(g.station.as_ref().unwrap().0,vec![3]);assert_eq!(g.mount_menu.0,vec![4]);assert_eq!(g.mod_screen.as_ref().unwrap().0,6);g.station.as_mut().unwrap().0.push(9);
            g.with_gui_closer_active(|g| { assert_eq!(g.remote_menu().0,7);g.carried=Some(30);g.station.as_mut().unwrap().0.push(31);g.remote.as_mut().unwrap().state_id=32;g.inventory[0]=33; });
            assert_eq!(g.carried,Some(2));assert_eq!(g.inventory[0],33);
        });
        assert_eq!(g.carried,Some(30));assert_eq!(g.station.as_ref().unwrap().0,vec![22,31]);assert_eq!(g.remote.as_ref().unwrap().state_id,32);
        g.with_gui_closer_view(|g| assert_eq!(g.station.as_ref().unwrap().0,vec![3,9]));
    }
    #[test] fn reused_remote_container_key_and_double_chest_halves_are_independent() {
        let mut g=old();let key=(i32::MIN,0,0);let half=(1,0,0);g.container=Some(key);g.other_half=Some(half);g.world.block_entities.insert(key,vec![1]);g.world.block_entities.insert(half,vec![2]);opened(&mut g,8);g.container=Some(key);g.world.block_entities.insert(key,vec![3]);
        g.with_gui_closer_view(|g| { assert_eq!(g.world.block_entities[&key],vec![1]);assert_eq!(g.world.block_entities[&half],vec![2]);g.with_gui_closer_active(|g|{assert_eq!(g.world.block_entities[&key],vec![3]);g.world.block_entities.insert(key,vec![4]);});assert_eq!(g.world.block_entities[&key],vec![1]); });
        assert_eq!(g.world.block_entities[&key],vec![4]);g.with_gui_closer_view(|g| assert_eq!(g.world.block_entities[&key],vec![1]));
    }
    #[test] fn repeated_cancellation_keeps_first_screen_and_later_active_session() {
        let mut g=old();opened(&mut g,7);opened(&mut g,8);assert_eq!(g.remote_menu().0,8);
        g.with_gui_closer_view(|g|{assert_eq!(g.remote_menu().0,0);g.with_gui_closer_view(|g|assert_eq!(g.carried,Some(2)));});assert_eq!(g.remote_menu().0,8);
    }
    #[test] fn stale_clicks_use_screen_menu_id_and_close_uses_new_active_menu() {
        let mut g=old();opened(&mut g,7);
        g.with_gui_closer_view(|g|{assert!(g.client_gui_closer_stale_input());g.with_gui_closer_active(|g|{assert!(!g.client_gui_closer_stale_input());let id=g.remote_menu().0;g.remote.as_mut().unwrap().packets.push(id);g.remote.as_mut().unwrap().menu=None;g.carried=None;g.station=None;g.screen=Screen::Playing;g.client_inventory_screen_serial.set(1);});});
        assert_eq!(g.remote.as_ref().unwrap().packets,vec![7]);assert_eq!(g.carried,None);assert_eq!(g.screen,Screen::Playing);assert!(g.client_gui_closer_view.is_none());
    }
    #[test] fn same_id_matches_vanilla_validation_and_accepting_same_enum_discards_view() {
        let mut g=old();opened(&mut g,0);g.with_gui_closer_view(|g|assert!(!g.client_gui_closer_stale_input()));
        assert!(g.client_gui_closer_finish_screen(true));g.client_inventory_screen_serial.set(1);g.with_gui_closer_view(|g|assert_eq!(g.carried,Some(20)));assert!(g.client_gui_closer_view.is_none());
    }
    #[test] fn abandoned_constructor_restores_owned_objects_and_screen_epoch_invalidates_retention() {
        let mut g=old();g.client_gui_closer_prepare_menu();assert!(g.station.is_none());g.client_gui_closer_abandon_menu();assert_eq!(g.station.as_ref().unwrap().0,vec![3]);
        opened(&mut g,7);g.client_inventory_screen_serial.set(1);g.with_gui_closer_view(|g|assert_eq!(g.carried,Some(20)));assert!(g.client_gui_closer_view.is_none());
    }
    #[test] fn native_host_reentrancy_keeps_view_scope_after_active_operation() {
        let mut g=old();opened(&mut g,7);g.with_gui_closer_view(|g|{g.with_gui_closer_active(|g|{assert_eq!(g.remote_menu().0,7);g.with_gui_closer_active(|g|g.carried=Some(99));});assert_eq!(g.remote_menu().0,0);assert_eq!(g.carried,Some(2));});assert_eq!(g.carried,Some(99));
    }
    #[test] fn failed_nested_active_constructor_preserves_current_menu_and_original_screen() {
        let mut g=old();opened(&mut g,7);g.with_gui_closer_view(|g| {
            g.with_gui_closer_active(|g| { g.client_gui_closer_prepare_menu();g.client_gui_closer_abandon_menu();assert_eq!(g.remote_menu().0,7);assert_eq!(g.station.as_ref().unwrap().0,vec![22]);assert_eq!(g.carried,Some(20)); });
            assert_eq!(g.remote_menu().0,0);assert_eq!(g.station.as_ref().unwrap().0,vec![3]);
        });assert_eq!(g.remote_menu().0,7);assert_eq!(g.carried,Some(20));
    }
    #[test] fn accepted_provider_failure_keeps_old_screen_until_real_replacement_observer() {
        let mut g=old();opened(&mut g,7);assert!(g.client_gui_closer_finish_screen(true));
        // Provider cannot construct its new Screen: no lifecycle observer.
        g.with_gui_closer_view(|g|assert_eq!(g.carried,Some(2)));
        g.client_inventory_screen_serial.set(1);g.with_gui_closer_view(|g|assert_eq!(g.carried,Some(20)));
    }
    #[test] fn recipe_screen_state_survives_cancellation_and_ghost_update_targets_old_ui() {
        let mut g=old();g.recipe_ui=vec![1,2];opened(&mut g,7);assert_eq!(g.recipe_ui,vec![1,2]);
        // Packet validation uses the new active menu, then RecipeUpdateListener
        // is the retained Screen, as in actual MC26.3 handlePlaceRecipe.
        assert_eq!(g.remote.as_ref().unwrap().menu.as_ref().unwrap().0,7);
        g.with_gui_closer_view(|g|{assert_eq!(g.craft_size,2);g.recipe_ui.push(3);});
        assert_eq!(g.recipe_ui,vec![1,2,3]);assert_eq!(g.craft_size,3);
    }
    #[test] fn retained_title_component_resolves_again_after_language_change() {
        struct Languages;
        impl horizon_client_core::lang::ResourceSource for Languages {
            fn read(&self,path:&str)->Option<Vec<u8>> { match path {
                "assets/minecraft/lang/en_us.json"=>Some(include_bytes!("en_us.json").to_vec()),
                "assets/minecraft/lang/zh_cn.json"=>Some(include_bytes!("zh_cn.json").to_vec()),
                "assets/minecraft/lang/zh_tw.json"=>Some(include_bytes!("zh_tw.json").to_vec()),_=>None,
            } }
        }
        let source=Languages;let mut g=old();opened(&mut g,7);
        let mut labels=Vec::new();for code in ["en_us","zh_cn","zh_tw"] {
            horizon_client_core::lang::scoped(horizon_client_core::lang::Language::load(&[&source],code),|| {
                let expected=crate::text::Text::translate("container.crafting",&[]).plain();
                assert_eq!(g.client_gui_closer_display_title().unwrap().1,expected);labels.push(expected);
            });
        }assert_ne!(labels[0],labels[1]);assert_ne!(labels[0],labels[2]);
    }
}
