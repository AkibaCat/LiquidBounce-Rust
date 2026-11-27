use std::{collections::BTreeMap, sync::{LazyLock, Mutex}};
use crate::HandSway;

#[derive(Default)]
struct Ground { on_ground: bool }
#[derive(Default)]
struct Player { ground: Ground, yaw: f32 }
struct Stack { item: u32 }
struct Item { can_sprint: bool }
#[derive(Default)]
struct Settings { sensitivity: f32 }
#[derive(Default)]
enum Screen { #[default] None, Inventory, Creative, Other }
#[derive(Default)]
struct Game {
    window: bool, player: Option<Player>, yaw: f32, pitch: f32,
    using: Option<(usize, u32)>, offhand: Option<Stack>,
    inventory: Vec<Option<Stack>>, definitions: BTreeMap<u32, Item>,
    settings: Settings, screen: Screen, sway: HandSway,
}
impl Game {
    fn native_window(&self) -> bool { self.window }
    fn items(&self) -> &BTreeMap<u32, Item> { &self.definitions }
}
mod native_hooks {
    use super::*;
    impl Game { __SPRINT_HOST_HOOKS__ }
}
impl mod_api::Host for Game {}

type QueryContext = (f32, f32, bool, bool, bool, f32);
static CONTEXTS: Mutex<Vec<QueryContext>> = Mutex::new(Vec::new());
mod mod_api {
    use super::*;
    pub trait Host {}
    pub trait RenderHooks: Send + Sync {}
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(Box::as_ref)
    }
    #[allow(clippy::too_many_arguments)] // The engine generator emits positional hook dispatch.
    mod generated { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use generated::*;
    struct Recorder;
    impl Hooks for Recorder {
        fn client_sprint_queries(&self, _: &dyn Host, event: &mut ClientSprintQueriesEvent) {
            CONTEXTS.lock().unwrap().push((event.sideways, event.forward, event.on_ground, event.slow_item, event.shift, event.player_yaw));
            event.movement_override = 1;
            event.omnidirectional = true;
            event.ignore_blindness = true;
            event.ignore_hunger = true;
            event.ignore_collision = true;
            *event.jump_yaw = Some(11.0);
            *event.movement_yaw = Some(22.0);
            *event.managed_jump_yaw = Some(33.0);
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
    fn query(game: &Game) -> crate::sprint::ClientSprintQueries {
        let mut out = crate::sprint::ClientSprintQueries::default();
        game.client_sprint_queries_for_input(0.5, -0.75, true,
            &mut out.movement_override, &mut out.omnidirectional,
            &mut out.ignore_blindness, &mut out.ignore_hunger,
            &mut out.ignore_collision, &mut out.jump_yaw,
            &mut out.movement_yaw, &mut out.managed_jump_yaw);
        out
    }
    #[test]
    fn generated_six_input_event_carries_real_state_and_all_outputs() {
        CONTEXTS.lock().unwrap().clear();
        let mut game = Game {
            window: true, player: Some(Player { ground: Ground { on_ground: true }, yaw: 0.0 }),
            yaw: 135.0, inventory: vec![Some(Stack { item: 1 })], using: Some((0, 20)),
            offhand: Some(Stack { item: 2 }), ..Game::default()
        };
        game.definitions.insert(1, Item { can_sprint: false });
        game.definitions.insert(2, Item { can_sprint: true });
        let result = query(&game);
        assert_eq!(result.movement_override, 1);
        assert!(result.omnidirectional && result.ignore_blindness && result.ignore_hunger && result.ignore_collision);
        assert_eq!((result.jump_yaw, result.movement_yaw, result.managed_jump_yaw), (Some(11.0), Some(22.0), Some(33.0)));
        game.using = Some((crate::game::shield::OFFHAND_SLOT, 20));
        query(&game);
        game.using = Some((3, 20));
        query(&game);
        game.window = false;
        assert_eq!(query(&game), crate::sprint::ClientSprintQueries::default());
        game.window = true;
        game.player = None;
        assert_eq!(query(&game), crate::sprint::ClientSprintQueries::default());
        assert_eq!(*CONTEXTS.lock().unwrap(), vec![
            (0.5, -0.75, true, true, true, 135.0),
            (0.5, -0.75, true, false, true, 135.0),
            (0.5, -0.75, true, false, true, 135.0),
        ]);
    }
}
