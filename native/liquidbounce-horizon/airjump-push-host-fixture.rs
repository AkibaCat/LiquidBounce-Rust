use super::*;
use std::sync::{LazyLock, Mutex};
use crate::actions::Action;
#[derive(Default)]
struct PhysicalKeys { jump: bool, sneak: bool }
impl PhysicalKeys {
    fn held(&self, action: Action) -> bool { match action { Action::Jump => self.jump, Action::Sneak => self.sneak } }
}
struct Game { player: Option<PlayerMotion>, keys: PhysicalKeys, window: bool }
impl Game { fn native_window(&self) -> bool { self.window } }
mod native_hooks {
    use super::*;
    impl Game { __AIRJUMP_PUSH_HOST_HOOKS__ }
}
impl mod_api::Host for Game {}
static JUMPS: Mutex<Vec<bool>> = Mutex::new(Vec::new());
mod mod_api {
    use super::*;
    pub trait Host {}
    pub trait RenderHooks: Send + Sync {}
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(Box::as_ref)
    }
    #[allow(clippy::too_many_arguments)]
    mod generated { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use generated::*;
    struct Recorder;
    impl Hooks for Recorder {
        fn client_airjump_push_queries(&self, _: &dyn Host, event: &mut ClientAirjumpPushQueriesEvent) {
            event.air_jump = true;
            event.double_jump_mode = true;
            event.ghost_block = event.physical_jump;
            event.allow_entities = false;
            event.allow_blocks = false;
            event.allow_fishing = false;
            event.allow_liquids = false;
        }
        fn client_player_jump(&self, _: &dyn Host, event: &mut ClientPlayerJumpEvent) {
            JUMPS.lock().unwrap().push(event.on_ground);
        }
    }
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: LazyLock<Vec<Box<dyn Hooks>>> = LazyLock::new(|| vec![Box::new(Recorder)]);
        &MODS
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_hooks_and_host_helpers_preserve_physical_state_and_client_guards() {
        let mut player = PlayerMotion::new();
        player.ground.on_ground = true;
        player.lava = 0.1;
        player.velocity = Vec3::new(1.0, -0.04, 2.0);
        let mut game = Game { player: Some(player), keys: PhysicalKeys { jump: true, sneak: false }, window: true };
        assert_eq!(game.native_client_motion_state(), [1.0, 1.0, 1.0, 0.0, -0.04]);
        assert_eq!(game.native_client_airjump_push_policy(), [true,true,true,false,false,false,false]);
        game.keys.jump = false;
        game.keys.sneak = true;
        assert_eq!(game.native_client_motion_state(), [1.0, 1.0, 0.0, 1.0, -0.04]);
        assert!(!game.native_client_airjump_push_policy()[2]);
        assert!(game.native_client_motion_velocity_y(0.0));
        assert_eq!(game.player.as_ref().unwrap().velocity, Vec3::new(1.0, 0.0, 2.0));
        assert!(!game.native_client_motion_velocity_y(f64::NAN));
        game.native_client_player_jump(false);
        game.native_client_player_jump(true);
        game.window = false;
        assert!(!game.native_client_motion_velocity_y(3.0));
        assert_eq!(game.native_client_airjump_push_policy(), [false,false,false,true,true,true,true]);
        game.native_client_player_jump(false);
        assert_eq!(*JUMPS.lock().unwrap(), vec![false, true]);
        game.player = None;
        assert_eq!(game.native_client_motion_state(), [0.0; 5]);
    }
}
