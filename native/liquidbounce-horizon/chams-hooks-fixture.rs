// SPDX-License-Identifier: GPL-3.0-or-later
#[cfg(test)]
extern crate self as horizon_client;
#[cfg(test)]
struct Game<'a> {
    window: bool,
    raw: u8,
    marker: std::marker::PhantomData<&'a ()>,
}
#[cfg(test)]
impl Game<'_> {
    fn native_window(&self) -> bool { self.window }
    fn native_raw_camera_perspective(&self) -> u8 { self.raw }
}
#[cfg(test)]
mod mod_api {
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(|module| &**module)
    }
    #[allow(unused_imports)]
    mod render {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/api-source/crates/horizon-client-render/src/gfx/native_render/generated.rs"));
    }
    pub use render::*;
    #[allow(unused_imports)]
    mod host { include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs")); }
    pub use host::*;
    mod hooks { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use hooks::*;
    pub static ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    pub static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    struct Listener;
    impl Hooks for Listener {
        fn client_chams_settings(&self, _: &dyn Host, event: &mut ClientChamsSettingsEvent) {
            if ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
                *event.settings = r#"{"mode":"Normal","file":"","mapping":"Repeat","tileWidth":256,"filtering":"Nearest","offset":[0,0]}"#.into();
            }
        }
        fn client_chams_entity(&self, _: &dyn Host, event: &mut ClientChamsEntityEvent) {
            CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            event.target = ENABLED.load(std::sync::atomic::Ordering::Relaxed)
                && event.entity == 42 && event.raw_perspective == 2;
        }
    }
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: std::sync::OnceLock<Vec<Box<dyn Hooks>>> = std::sync::OnceLock::new();
        MODS.get_or_init(|| vec![Box::new(Listener)])
    }
}
#[cfg(test)]
mod game {
    pub type Game<'a> = crate::Game<'a>;
    mod native_hooks {
        use crate::{Game, mod_api};
        impl Game<'_> { __HOOKS__ }
    }
    mod mod_api_host { include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs")); }
}
#[cfg(test)]
mod native_dispatch_tests {
    use super::*;
    use std::sync::atomic::Ordering::Relaxed;
    static SCENE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    fn game() -> Game<'static> { Game { window: true, raw: 2, marker: std::marker::PhantomData } }
    #[test]
    fn generated_events_preserve_string_entity_raw_perspective_and_mutable_target() {
        let _lock=SCENE.lock().unwrap();mod_api::ENABLED.store(true,Relaxed);
        assert!(mod_api::mods()[0].render().is_none());
        let mut game=game();let mut settings="null".to_owned();
        game.native_client_chams_settings(&mut settings);gfx::chams::begin_frame(&settings);
        assert!(game.client_chams_entity(42));assert!(!game.client_chams_entity(43));
        game.raw=0;assert!(!game.client_chams_entity(42));
        game.raw=1;assert!(!game.client_chams_entity(42));
        game.raw=2;game.window=false;assert!(!game.client_chams_entity(42));
        settings="null".into();game.native_client_chams_settings(&mut settings);assert_eq!(settings,"null");
        gfx::chams::begin_frame("null");
    }
    #[test]
    fn disabled_frames_stop_dispatch_before_entity_context_can_be_established() {
        let _lock=SCENE.lock().unwrap();mod_api::ENABLED.store(false,Relaxed);
        let game=game();let mut settings="null".to_owned();game.native_client_chams_settings(&mut settings);
        gfx::chams::begin_frame(&settings);let before=mod_api::CALLS.load(Relaxed);
        assert!(!game.client_chams_entity(42));assert_eq!(mod_api::CALLS.load(Relaxed),before);
    }
}
