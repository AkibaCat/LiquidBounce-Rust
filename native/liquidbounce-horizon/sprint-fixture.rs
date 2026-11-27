#![allow(dead_code)]
extern crate self as horizon_client;
use horizon_core::geometry::Vec3;
type Result<T, E = ()> = std::result::Result<T, E>;

mod input {
    __INPUT_LENGTH__
    __INPUT_DIRECTION__
}
mod sprint {
    __SPRINT__
}
#[path = "__LIVING_AIR_PATH__"]
mod living_air;
mod living_motion {
    __LIVING_MOTION__
}
use sprint::ClientSprintQueries;
use living_motion::LivingMotionWorld;

#[derive(Clone, Default)]
struct PlayerMotion {
    yaw: f32,
    client_side: bool,
    server_copy: bool,
    replaying: bool,
    client_sprint_queries: ClientSprintQueries,
}
impl PlayerMotion { __QUERIES__ }
struct MotionWorld {
    player: PlayerMotion,
    velocity: Vec3,
    sprinting: bool,
}
impl LivingMotionWorld for MotionWorld {
    type Error = ();
    fn velocity(&self) -> Vec3 { self.velocity }
    fn set_velocity(&mut self, v: Vec3) -> Result<(), ()> { self.velocity = v; Ok(()) }
    fn jump_strength(&mut self) -> Result<f64, ()> { Ok(0.42) }
    fn block_jump_factor(&mut self) -> Result<f32, ()> { Ok(1.0) }
    fn has_jump_boost(&mut self) -> Result<bool, ()> { Ok(false) }
    fn jump_boost_amplifier(&mut self) -> Result<i32, ()> { Ok(0) }
    fn sprinting(&mut self) -> Result<bool, ()> { Ok(self.sprinting) }
    fn yaw(&mut self) -> Result<f32, ()> { Ok(self.player.yaw) }
    __JUMP_BOOST__
    fn mark_needs_sync(&mut self) {}
    fn no_gravity(&mut self) -> Result<bool, ()> { Ok(false) }
    fn gravity_attribute(&mut self) -> Result<f64, ()> { Ok(0.08) }
    fn has_slow_falling(&mut self) -> Result<bool, ()> { Ok(false) }
    fn controlling_passenger_is_player(&mut self) -> Result<bool, ()> { Ok(false) }
    fn speed(&mut self) -> Result<f32, ()> { Ok(0.1) }
    fn omnidirectional_air_mover(&mut self) -> Result<bool, ()> { Ok(false) }
    fn air_drag_modifier(&mut self) -> Result<f64, ()> { Ok(0.0) }
}

mod camera {
    pub fn wrap_degrees(value: f32) -> f32 { crate::mobs::living::wrap(value) }
}
mod mobs {
    #[derive(Clone, Default, Debug, PartialEq)]
    pub struct Mob {
        pub body_yaw: f32, pub previous_body_yaw: f32,
        pub head_yaw: f32, pub previous_head_yaw: f32,
        pub head_pitch: f32, pub previous_head_pitch: f32,
        pub extra: Extra,
    }
    #[derive(Clone, Default, Debug, PartialEq)]
    pub struct Extra { pub vehicle_body: Option<(u32, f32, f32)>, pub mount: Option<u32> }
    pub mod living { __WRAP__ }
    pub mod riders {
        __SOLVE_BODY_ROT__
    }
    pub mod anim {
        use super::{Mob, living::wrap};
        __BODY_ROT__
    }
}
struct Game { target: Option<[f32; 3]> }
impl Game {
    fn native_client_rotation_model(&self, _: f32, body: &mut f32, head: &mut f32, pitch: &mut f32) {
        if let Some([b, h, p]) = self.target { *body = b; *head = h; *pitch = p; }
    }
    fn rendered(&self, m: &mobs::Mob, alpha: f64) -> mobs::Mob {
        __RENDER_EXTRACTION__
    }
}
#[derive(Default)]
struct HandSway { y_bob: f32, y_bob_o: f32, y_rot: f32, x_bob: f32, x_bob_o: f32 }
impl HandSway { __HAND_SWAY_RESET__ }

#[cfg(test)]
mod tests {
    use super::*;
    use sprint::{Conditions, Keys, SprintState};
    fn standing() -> Conditions { Conditions { enough_food: true, on_ground: true, ..Conditions::default() } }
    fn tick(state: &mut SprintState, keys: Keys, conditions: Conditions, queries: ClientSprintQueries) -> Vec<bool> {
        let mut out = Vec::new();
        state.tick_with_queries(keys, conditions, 7, queries, |value| { out.push(value); Ok::<(), ()>(()) }).unwrap();
        out
    }
    #[test]
    fn distinct_can_start_sprint_key_and_forced_stop_queries() {
        let mut state = SprintState::default();
        let keys = Keys { forward: true, ..Keys::default() };
        let forced = ClientSprintQueries { movement_override: 1, ..ClientSprintQueries::default() };
        // CRITICAL handler changes canStart's false result and Input.sprint's false result.
        assert_eq!(tick(&mut state, keys, Conditions { mobility_restricted: true,
            enough_food: false, slow_item: true, ..standing() }, forced), vec![true, false]);
        assert_eq!(tick(&mut state, keys, standing(), forced), vec![true]);
        // Prevention is also queried when already sprinting, independent of canStart.
        let prevented = ClientSprintQueries { movement_override: 0, ..forced };
        assert_eq!(tick(&mut state, keys, Conditions { sprinting: true, ..standing() }, prevented), vec![false]);
        assert_eq!(tick(&mut state, keys, standing(), prevented), Vec::<bool>::new());
    }
    #[test]
    fn ignore_queries_preserve_actual_conditions_and_collision_scope() {
        let conditions = Conditions { sprinting: true, mobility_restricted: true,
            enough_food: false, horizontal_collision: true, ..standing() };
        let queries = ClientSprintQueries { ignore_blindness: true, ignore_hunger: true,
            ignore_collision: true, ..ClientSprintQueries::default() };
        let keys = Keys { forward: true, ..Keys::default() };
        assert!(tick(&mut SprintState::default(), keys, conditions, queries).is_empty());
        assert!(conditions.mobility_restricted && !conditions.enough_food && conditions.horizontal_collision);
        assert_eq!(tick(&mut SprintState::default(), keys, conditions,
            ClientSprintQueries { ignore_collision: false, ..queries }), vec![false]);
        // Ignore Collision is only a shouldStopRunSprinting field interception;
        // shouldStopSwimSprinting keeps its own independent water condition.
        assert_eq!(tick(&mut SprintState::default(), keys, Conditions { swimming: true,
            in_water: false, ..conditions }, queries), vec![false]);
    }
    #[test]
    fn omni_high_threshold_low_threshold_and_previous_input_mode_change() {
        let mut state = SprintState::default();
        let omni = ClientSprintQueries { omnidirectional: true, ..ClientSprintQueries::default() };
        let keys = Keys { analog: Some([0.5, 0.0]), sprint: true, ..Keys::default() };
        assert!(tick(&mut state, keys, standing(), omni).is_empty());
        assert_eq!(tick(&mut state, keys, Conditions { under_water: true, ..standing() }, omni), vec![true]);
        assert_eq!(tick(&mut state, Keys { analog: Some([0.8, 0.0]), ..keys }, standing(), omni), vec![true]);
        let state_before = state;
        tick(&mut state, keys, standing(), ClientSprintQueries::default());
        assert!(!state.previous_forward);
        // Switching from Legit to Omni evaluates the retained previous axes,
        // rather than misreading the old forward-only history as a new press.
        state.trigger_time = 5;
        let out = tick(&mut state, Keys { sprint: false, analog: Some([1.0, 0.0]), ..keys }, standing(), omni);
        assert!(out.is_empty());
        assert_eq!(state_before.previous_axes, [0.8, 0.0]);
    }
    #[test]
    fn client_snapshot_clone_does_not_affect_server_or_replay() {
        let q = ClientSprintQueries { movement_override: 1, omnidirectional: true,
            ignore_blindness: true, ignore_hunger: true, ignore_collision: true,
            jump_yaw: Some(90.0), movement_yaw: Some(40.0), managed_jump_yaw: Some(30.0) };
        let p = PlayerMotion { client_side: true, client_sprint_queries: q, ..PlayerMotion::default() };
        assert_eq!(p.clone().client_sprint_queries(), q);
        for (client_side, server_copy, replaying) in [(false,false,false),(true,true,false),(true,false,true)] {
            let copy = PlayerMotion { client_side, server_copy, replaying, ..p.clone() };
            assert_eq!(copy.client_sprint_queries(), ClientSprintQueries::default());
        }
        assert_eq!(p.client_sprint_queries(), q);
    }
    #[test]
    fn managed_jump_uses_float_product_then_widens_and_overrides_event_yaw() {
        use horizon_core::geometry::trig;
        for (server_copy, replaying) in [(false,false),(true,false),(false,true)] {
            let active = !server_copy && !replaying;
            let mut world = MotionWorld { player: PlayerMotion { yaw: 37.0, client_side: true,
                server_copy, replaying, client_sprint_queries: ClientSprintQueries {
                    jump_yaw: Some(90.0), managed_jump_yaw: Some(-31.0), ..ClientSprintQueries::default() } },
                velocity: Vec3::ZERO, sprinting: true };
            living_motion::jump_from_ground(&mut world).unwrap();
            let yaw = if active { -31.0 } else { 37.0 };
            let angle = f64::from(yaw * trig::DEG_TO_RAD);
            let expected = if active { f64::from(-trig::sin(angle) * 0.2f32) } else { f64::from(-trig::sin(angle)) * 0.2 };
            assert_eq!(world.velocity.x.to_bits(), expected.to_bits());
            assert_eq!(world.velocity.y, f64::from(0.42f32));
        }
    }
    #[test]
    fn renderer_rotation_changes_only_render_clone_after_rider_solving() {
        let source = mobs::Mob { body_yaw: 12.0, previous_body_yaw: 10.0,
            head_yaw: 100.0, previous_head_yaw: 100.0, head_pitch: 9.0, previous_head_pitch: 7.0,
            extra: mobs::Extra { mount: Some(7), vehicle_body: Some((7,30.0,50.0)) } };
        assert_eq!(mobs::anim::body_rot(&source, 0.5), 52.0);
        let rendered = Game { target: Some([-90.0,-89.0,19.0]) }.rendered(&source,0.5);
        assert_eq!(mobs::anim::body_rot(&rendered,0.5), -90.0);
        assert_eq!((rendered.head_yaw,rendered.previous_head_yaw,rendered.head_pitch),(-89.0,-89.0,19.0));
        assert_eq!(source.extra.vehicle_body,Some((7,30.0,50.0)));
        assert_eq!(source.body_yaw,12.0);
        assert_eq!(Game { target: None }.rendered(&source,0.5),source);
    }
    #[test]
    fn resetting_hand_yaw_resets_wrapped_accumulator_and_both_bobs() {
        let mut sway = HandSway { y_bob: 71.0, y_bob_o: 70.0, y_rot: 72.0, x_bob: 9.0, x_bob_o: 8.0 };
        sway.reset_yaw(33.0);
        assert_eq!((sway.y_bob,sway.y_bob_o,sway.y_rot),(33.0,33.0,33.0));
        assert_eq!((sway.x_bob,sway.x_bob_o),(9.0,8.0));
    }
}
