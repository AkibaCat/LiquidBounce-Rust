#![cfg(test)]
#![allow(dead_code, unused_imports)]
extern crate self as horizon_client;
type Result<T> = std::result::Result<T, ()>;

mod pose {
    __POSE_DECLARATIONS__
    impl PlayerMotion {
        __DIMENSIONS_FOR_POSE__
        __REFRESH_DIMENSIONS__
        __SET_POSE__
    }
}
mod motion {
    use crate::pose::{Dimensions, Pose};
    use horizon_core::geometry::Vec3;
    pub struct MotionWorld;
    pub struct PlayerMotion {
        pub pose: Pose,
        pub dimensions: Dimensions,
        pub scale: f32,
        pub sleeping: bool,
        pub server_copy: bool,
        pub replaying: bool,
        pub position: Vec3,
        pub velocity: Vec3,
    }
    impl Default for PlayerMotion {
        fn default() -> Self {
            Self {
                pose: Pose::Sleeping,
                dimensions: Pose::Sleeping.dimensions(1.0),
                scale: 1.0,
                sleeping: true,
                server_copy: false,
                replaying: false,
                position: Vec3::new(10.5, 64.5, -3.5),
                velocity: Vec3::new(0.2, -0.1, 0.3),
            }
        }
    }
    impl PlayerMotion {
        __IDENTITY__
        pub fn attribute(&mut self, _name: &str) -> crate::Result<f64> {
            Ok(f64::from(self.scale))
        }
    }
}

mod mod_api {
    use std::sync::{LazyLock, atomic::{AtomicBool, Ordering}};
    pub trait RenderHooks: Send + Sync {}
    mod host { include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs")); }
    pub use host::*;
    mod generated { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use generated::*;
    pub static RUNNING: AtomicBool = AtomicBool::new(false);
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _name: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(|module| &**module)
    }
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: LazyLock<Vec<Box<dyn Hooks>>> = LazyLock::new(|| vec![Box::new(SleepWalker)]);
        &MODS
    }
    struct SleepWalker;
    impl Hooks for SleepWalker {
        fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {
            if RUNNING.load(Ordering::Acquire) { host.client_sleep_walker_tick(); }
        }
    }
}

pub mod game {
    use crate::motion::PlayerMotion;
    use crate::pose::Pose;

    #[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
    enum ExecutionRole { #[default] Client, Server }
    #[derive(Default)]
    struct Options { execution_role: ExecutionRole }
    #[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
    enum Screen { Playing, #[default] Chat, Inventory, Paused, Mod }
    struct ChatScreen { text: String }
    impl ChatScreen { fn value(&self) -> &str { &self.text } }
    __CHAT_DRAFT__
    __CHAT_EXIT__
    __SHOULD_DISCARD_DRAFT__
    #[derive(Default)]
    struct Chat { resets: usize }
    impl Chat { fn reset_scroll(&mut self) { self.resets += 1; } }
    #[derive(Default)]
    struct Ui {
        chat: Chat,
        chat_screen: Option<ChatScreen>,
        chat_draft: Option<ChatDraft>,
        chat_draft_shown: bool,
        in_bed_chat: bool,
    }
    #[derive(Default)]
    struct Pvp { chat_search: bool, chat_search_aside: String }
    #[derive(Default)]
    struct Settings { file: SettingsFile }
    #[derive(Default)]
    struct SettingsFile { save: bool }
    impl SettingsFile { fn bool(&self, _name: &str) -> bool { self.save } }
    pub struct Game<'a> {
        options: Options,
        lan_acting_uuid: Option<u128>,
        player: Option<PlayerMotion>,
        sleeping: Option<[i32; 3]>,
        sleep_timer: u32,
        screen: Screen,
        native_mod_screen: Option<String>,
        ui: Ui,
        pvp: Pvp,
        settings: Settings,
        focused: bool,
        captures: Vec<bool>,
        marker: std::marker::PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            Self {
                options: Options::default(), lan_acting_uuid: None,
                player: Some(PlayerMotion::default()), sleeping: Some([10, 64, -4]),
                sleep_timer: 73, screen: Screen::Chat, native_mod_screen: None,
                ui: Ui { chat_screen: Some(ChatScreen { text: "/say 中文 草稿".into() }),
                    in_bed_chat: true, ..Ui::default() },
                pvp: Pvp::default(), settings: Settings::default(),
                focused: true, captures: Vec::new(), marker: std::marker::PhantomData,
            }
        }
    }
    impl Game<'_> {
        __NATIVE_WINDOW__
        fn pad_clear_focus(&mut self) { self.focused = false; }
        fn capture(&mut self, on: bool) { self.captures.push(on); }
        fn wake_up(&mut self) { panic!("SleepWalker must not invoke wake_up or its server packet"); }
        fn open_chat(&mut self, _text: &str, _command: bool) { panic!("SleepWalker must not reopen chat"); }
        __CLOSE_CHAT__
        __KEEP_CHAT_DRAFT__
        __TICK_IN_BED_CHAT__
    }
    mod native_hooks {
        use super::{Game, Screen};
        use crate::mod_api;
        impl Game<'_> { __NATIVE_HELPER__ __NATIVE_TICK__ }
    }
    mod generated_host { include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs")); }

    #[test]
    fn existing_tick_dispatch_invokes_the_generated_helper_only_while_running() {
        use crate::mod_api::RUNNING;
        use std::sync::atomic::Ordering;
        let mut game = Game::default();
        RUNNING.store(false, Ordering::Release);
        game.native_client_tick_pre();
        assert!(game.sleeping.is_some());
        RUNNING.store(true, Ordering::Release);
        game.native_client_tick_pre();
        assert!(game.sleeping.is_none());
        assert_eq!(game.player.as_ref().unwrap().pose, Pose::Standing);
        RUNNING.store(false, Ordering::Release);
    }

    #[test]
    fn real_host_restores_standing_clears_both_sleeping_fields_and_preserves_motion() {
        use crate::mod_api::Host;
        let mut game = Game::default();
        let player = game.player.as_ref().unwrap();
        let (position, velocity) = (player.position, player.velocity);
        assert!(Host::client_sleep_walker_tick(&mut game));
        let player = game.player.as_ref().unwrap();
        assert_eq!(player.pose, Pose::Standing);
        assert!(!player.sleeping);
        assert_eq!((player.dimensions.width, player.dimensions.height, player.dimensions.eye_height), (0.6, 1.8, 1.62));
        assert_eq!((player.position, player.velocity), (position, velocity));
        assert_eq!(game.sleeping, None);
        assert_eq!(game.sleep_timer, 73, "clearSleepingPos does not run wake-up timer logic");
        assert_eq!(game.screen, Screen::Playing);
        assert!(!game.focused);
        assert_eq!(game.captures, [true]);
        assert!(!game.ui.in_bed_chat);
        assert!(game.ui.chat_screen.is_none());
        let draft = game.ui.chat_draft.as_ref().unwrap();
        assert_eq!(draft.text, "/say 中文 草稿");
        assert!(draft.command);
        assert_eq!(game.ui.chat.resets, 1);
        assert!(!Host::client_sleep_walker_tick(&mut game));
        assert_eq!(game.captures, [true]);
    }

    #[test]
    fn sleeping_position_is_the_guard_independently_of_pose_or_cached_boolean() {
        use crate::mod_api::Host;
        let mut game = Game::default();
        {
            let player = game.player.as_mut().unwrap();
            player.pose = Pose::Standing;
            player.sleeping = false;
        }
        assert!(Host::client_sleep_walker_tick(&mut game));
        let player = game.player.as_mut().unwrap();
        player.pose = Pose::Sleeping;
        player.sleeping = true;
        assert!(!Host::client_sleep_walker_tick(&mut game));
        assert_eq!(game.player.as_ref().unwrap().pose, Pose::Sleeping);
    }

    #[test]
    fn every_new_sleep_is_handled_and_non_bed_screens_also_close() {
        use crate::mod_api::Host;
        for screen in [Screen::Playing, Screen::Chat, Screen::Inventory, Screen::Paused, Screen::Mod] {
            let mut game = Game { screen, native_mod_screen: Some("example".into()), ..Game::default() };
            assert!(Host::client_sleep_walker_tick(&mut game));
            assert_eq!(game.screen, Screen::Playing);
            assert_eq!(game.native_mod_screen, None);
            assert!(!game.ui.in_bed_chat);
            game.sleeping = Some([20, 70, 9]);
            game.player.as_mut().unwrap().sleeping = true;
            game.player.as_mut().unwrap().set_pose(Pose::Sleeping).unwrap();
            assert!(Host::client_sleep_walker_tick(&mut game));
            assert_eq!(game.captures, [true, true]);
            assert_eq!(game.player.as_ref().unwrap().pose, Pose::Standing);
        }
    }

    #[test]
    fn server_guest_replay_and_missing_player_do_not_mutate_sleep_or_screen() {
        use crate::mod_api::Host;
        for guard in 0..5 {
            let mut game = Game::default();
            match guard {
                0 => game.options.execution_role = ExecutionRole::Server,
                1 => game.lan_acting_uuid = Some(100),
                2 => game.player.as_mut().unwrap().server_copy = true,
                3 => game.player.as_mut().unwrap().replaying = true,
                _ => game.player = None,
            }
            assert!(!Host::client_sleep_walker_tick(&mut game));
            assert_eq!(game.sleeping, Some([10, 64, -4]));
            assert_eq!(game.screen, Screen::Chat);
            assert!(game.focused);
            assert!(game.captures.is_empty());
            assert_eq!(game.ui.chat_screen.as_ref().unwrap().value(), "/say 中文 草稿");
        }
    }
}
