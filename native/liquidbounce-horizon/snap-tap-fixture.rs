#![cfg(test)]
extern crate self as horizon_client;

#[path = "__ROTATIONS_PATH__"]
pub mod rotations;
#[path = "__POLICY_PATH__"]
pub mod snap_tap;
#[path = "__CLOCK_PATH__"]
pub mod clock;
#[path = "config-fixture.rs"]
mod config;
#[path = "__TESTS_PATH__"]
mod policy_tests;

#[allow(dead_code)] // The real generated interface includes an unused prerequisite tick hook.
mod mod_api {
    pub use winit::keyboard::KeyCode;
    pub trait RenderHooks: Send + Sync {}
    mod host { include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs")); }
    pub use host::*;
    mod generated { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use generated::*;
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(Box::as_ref)
    }
}

pub mod game {
    use crate::mod_api::Host;
    use crate::snap_tap::{Action, BACK, FORWARD, LEFT, RIGHT, SnapTap};
    use horizon_client_input::keybinds::{self, KeyBinds};
    use winit::keyboard::KeyCode;

    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    enum ExecutionRole { #[default] Client, Server }
    #[derive(Default)]
    struct Options { execution_role: ExecutionRole }
    #[derive(Default)]
    struct Settings { keys: KeyBinds }
    #[derive(Default)]
    pub struct Game<'a> {
        options: Options,
        settings: Settings,
        marker: std::marker::PhantomData<&'a ()>,
    }
    impl Game<'_> { __NATIVE_WINDOW__ }
    mod native_hooks {
        use super::*;
        impl Game<'_> { __NATIVE_HELPER__ }
    }
    mod generated_host { include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs")); }

    #[test]
    fn generated_host_uses_real_default_and_rebound_physical_movement_bindings() {
        let mut game = Game::default();
        for (key, mask) in [(KeyCode::KeyA, LEFT), (KeyCode::KeyD, RIGHT),
                            (KeyCode::KeyS, BACK), (KeyCode::KeyW, FORWARD)] {
            assert_eq!(Host::client_snap_tap_key_mask(&game, key), mask);
        }
        game.settings.keys.set("key.left", "key.keyboard.left");
        game.settings.keys.set("key.forward", "key.keyboard.up");
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::ArrowLeft), LEFT);
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::ArrowUp), FORWARD);
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::KeyA), 0);
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::KeyW), 0);
    }

    #[test]
    fn native_mask_preserves_every_conflict_for_original_when_priority() {
        let mut game = Game::default();
        for mapping in ["key.left", "key.right", "key.back", "key.forward"] {
            game.settings.keys.set(mapping, "key.keyboard.x");
        }
        let mask = Host::client_snap_tap_key_mask(&game, KeyCode::KeyX);
        assert_eq!(mask, 15);
        let mut state = SnapTap::default();
        state.keyboard(true, RIGHT, Action::Press, || 1);
        state.keyboard(true, mask, Action::Press, || 2);
        let mut input = crate::rotations::DirectionalInput::default();
        state.movement_input(true, &mut input);
        assert!(input.left && !input.right && !input.forwards && !input.backwards);
    }

    #[test]
    fn native_helper_excludes_server_unknown_mouse_and_unmapped_keys() {
        let mut game = Game::default();
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::KeyQ), 0);
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::Fn), 0);
        game.settings.keys.set("key.left", "key.mouse.left");
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::KeyA), 0);
        game.settings.keys.set("key.left", "key.keyboard.unknown");
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::Fn), 0);
        game.options.execution_role = ExecutionRole::Server;
        assert_eq!(Host::client_snap_tap_key_mask(&game, KeyCode::KeyD), 0);
    }

    #[test]
    fn rebinding_release_uses_current_mapping_and_does_not_reset_old_axis_state() {
        let mut game = Game::default();
        let mut state = SnapTap::default();
        for (time, key) in [(1, KeyCode::KeyA), (2, KeyCode::KeyD)] {
            state.keyboard(true, Host::client_snap_tap_key_mask(&game, key), Action::Press, || time);
        }
        game.settings.keys.set("key.right", "key.keyboard.right");
        state.keyboard(true, Host::client_snap_tap_key_mask(&game, KeyCode::KeyD), Action::Release,
                       || panic!("release read clock"));
        let mut input = crate::rotations::DirectionalInput::default();
        state.movement_input(true, &mut input);
        assert!(input.right && !input.left);
        state.keyboard(true, Host::client_snap_tap_key_mask(&game, KeyCode::ArrowRight), Action::Release,
                       || panic!("release read clock"));
        input = crate::rotations::DirectionalInput::default();
        state.movement_input(true, &mut input);
        assert_eq!(input, crate::rotations::DirectionalInput::default());
    }
}
