// Test fixture for complete transformed production methods. Only unrelated
// world/species services are inert; movement/effect decisions remain original.
pub mod motion {
    pub use crate::ClientMovementQueries;
}

#[cfg(test)]
mod mount_production_tests {
    use crate::ClientMovementQueries;
    use horizon_core::geometry::Vec3;

    #[allow(dead_code)] // The complete production aiStep mentions these species.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Species { Horse, Parrot, Witch, Fox, Blaze, Ravager, Mannequin, Piglin, Hoglin }
    #[derive(Clone, Default)]
    struct Ai { no_jump_delay: i32, no_action_time: i32, xxa: f32, yya: f32, zza: f32, jumping: bool }
    #[derive(Clone, Default)]
    struct Jump { discard_friction: bool }
    #[derive(Clone, Default)]
    struct Extra {
        client_side: bool, client_movement_queries: ClientMovementQueries,
        effects: Vec<(&'static str, i32, i32)>, immovable: bool, frog: Jump, goat: Jump,
    }
    #[derive(Clone, Copy)]
    struct Rider { client: bool }
    #[derive(Clone, Default)]
    struct Flags { omnidirectional: bool, climber: bool }
    #[derive(Clone)]
    struct Kind { gravity: f64, flags: Flags }
    #[derive(Clone)]
    struct Mob {
        ai: Ai, extra: Extra, rider: Option<Rider>, velocity: Vec3, position: Vec3,
        no_gravity: bool, on_ground: bool, horizontal_collision: bool,
        fall_distance: f64, climbing: bool, kind: Kind,
    }
    struct MobWorld;
    struct MobEvent;
    struct LegacyRandom;
    struct Target { position: Vec3 }
    impl Mob {
        fn new(client: bool) -> Self {
            Self {
                ai: Ai { no_jump_delay: 7, ..Ai::default() },
                extra: Extra { client_side: client, ..Extra::default() },
                rider: Some(Rider { client: !client }), velocity: Vec3::new(0.0, -0.2, 0.0),
                position: Vec3::ZERO, no_gravity: false, on_ground: false,
                horizontal_collision: false, fall_distance: 5.0, climbing: false,
                kind: Kind { gravity: 0.08, flags: Flags::default() },
            }
        }
        fn has_effect(&self, id: &str) -> bool { self.extra.effects.iter().any(|e| e.0 == id) }
        fn species(&self) -> Species { Species::Horse }
        fn target_info(&self, _: &MobWorld) -> Option<Target> { None }
        fn alive(&self) -> bool { true }
        fn handle_jumping(&mut self, _: &MobWorld) {}
        fn apply_effects_from_blocks(&mut self, _: &MobWorld, _: &mut Vec<MobEvent>) {}
        fn block_friction(&self, _: &MobWorld) -> f32 { 0.6 }
        fn friction_speed(&self, _: f32) -> f32 { 0.1 }
        fn move_relative(&mut self, _: f32, _: Vec3) {}
        fn on_climbable(&self, _: &MobWorld) -> bool { false }
        fn do_move(&mut self, _: &MobWorld, _: &mut Vec<MobEvent>) {}
        fn travel(&mut self, w: &MobWorld, input: Vec3, events: &mut Vec<MobEvent>) {
            living_production::run(self, w, input, events);
        }
    }
    __MOB_QUERIES__

    mod sulfur_cube {
        pub struct Physics { pub friction_modifier: f32, pub air_drag_modifier: f32 }
        pub fn physics(_: &super::Mob) -> Option<Physics> { None }
    }
    mod breeze { pub fn discards_friction(_: &super::Mob) -> bool { false } }
    mod equine {
        use super::*;
        pub fn is_equine(s: Species) -> bool { s == Species::Horse }
        pub fn tail_tick(_: &mut Mob) {}
        pub fn immobile(_: &Mob) -> bool { false }
        pub fn moved_by_client(m: &Mob) -> bool { m.rider.is_some_and(|r| r.client) }
    }
    mod parrot { pub fn ai_step(_: &mut super::Mob, _: &super::MobWorld, _: &mut super::LegacyRandom, _: &mut Vec<super::MobEvent>) {} }
    mod witch { pub fn ai_step(_: &mut super::Mob, _: &mut Vec<super::MobEvent>, _: bool) {} }
    mod fox { pub fn pre_ai_step(_: &mut super::Mob) {} }
    mod nether { pub fn blaze_ai_step(_: &mut super::Mob) {} }
    mod raider { pub fn is_raider(_: super::Species) -> bool { false } }
    mod hostile { pub fn mob_brightness(_: &super::Mob, _: &super::MobWorld) -> f32 { 0.0 } }
    mod creaking {
        pub fn is_creaking(_: &super::Mob) -> bool { false }
        pub fn update(_: &mut super::Mob, _: &super::MobWorld, _: &mut Vec<super::MobEvent>) {}
    }
    mod ravager { pub fn immobile(_: &super::Mob) -> bool { false } }
    mod piglin { pub fn before_goals(_: &mut super::Mob, _: &super::MobWorld, _: &mut super::LegacyRandom, _: &mut Vec<super::MobEvent>) {} }

    mod living_production {
        use super::*;
        impl Mob {
            __GRAVITY__
            __EFFECTIVE_GRAVITY__
            __MODIFIED_FRICTION__
            __AIR_DRAG_MODIFIER__
            __CLIMBABLE_VELOCITY__
            __TRAVEL_IN_AIR__
        }
        pub(super) fn run(m: &mut Mob, w: &MobWorld, input: Vec3, events: &mut Vec<MobEvent>) {
            m.travel_in_air(w, input, events);
        }
    }
    mod ai_production {
        use super::*;
        __RESET_EFFECT_FALL_DISTANCE__
        __AI_STEP__
        fn is_monster(_: Species) -> bool { false }
        fn server_ai_step(_: &mut Mob, _: &MobWorld, _: &mut LegacyRandom, _: &mut Vec<MobEvent>) {}
        fn drop_tiny_motion(_: &mut Mob) {}
        fn update_fall_flying(_: &mut Mob, _: &MobWorld, _: &mut Vec<MobEvent>) {}
        fn hurt_teleports(_: &mut Mob, _: &MobWorld, _: &mut Vec<MobEvent>) {}
        fn push_entities(_: &mut Mob, _: &MobWorld, _: &mut Vec<MobEvent>) {}
        fn hurt_by_water(_: &mut Mob, _: &MobWorld, _: &mut Vec<MobEvent>) {}
        fn ageable_step(_: &mut Mob, _: &MobWorld, _: &mut LegacyRandom, _: &mut Vec<MobEvent>) {}
        fn animal_step(_: &mut Mob, _: &mut Vec<MobEvent>) {}
        fn burn_undead(_: &mut Mob, _: &MobWorld, _: &mut Vec<MobEvent>) {}
        fn species_ai_step(_: &mut Mob, _: &MobWorld, _: &mut LegacyRandom, _: &mut Vec<MobEvent>) {}
        pub(super) fn run(m: &mut Mob) { ai_step(m, &MobWorld, &mut LegacyRandom, &mut Vec::new()); }
        pub(super) fn reset_fall(m: &mut Mob) { reset_effect_fall_distance(m); }
    }
    struct MobRestitution<'a> { mob: &'a mut Mob }
    trait RestitutionWorld { fn suppressing_bounce(&mut self) -> Result<bool, ()>; }
    impl RestitutionWorld for MobRestitution<'_> { __RESTITUTION_SUPPRESS__ }
    struct MobLanding<'a> { mob: &'a mut Mob }
    trait BlockLandingWorld { fn suppressing_bounce(&mut self) -> Result<bool, String>; }
    impl BlockLandingWorld for MobLanding<'_> { __LANDING_SUPPRESS__ }

    fn active() -> ClientMovementQueries {
        ClientMovementQueries { reset_jump_delay: true, suppress_bounce: true, ignore_levitation: true, ignore_slow_falling: true }
    }
    #[test]
    fn controlled_mount_methods_clear_cooldown_and_ignore_only_motion_effects() {
        let mut mount = Mob::new(true);
        mount.extra.client_movement_queries = active();
        mount.extra.effects = vec![("minecraft:levitation", 80, 2), ("minecraft:slow_falling", 90, 0)];
        let stored = mount.extra.effects.clone();
        assert_eq!(mount.effective_gravity(), 0.08);
        ai_production::run(&mut mount);
        assert_eq!(mount.ai.no_jump_delay, 0);
        let expected = (-0.2 - 0.08) * f64::from(0.98f32);
        assert!((mount.velocity.y - expected).abs() < 1e-12);
        assert_eq!(mount.extra.effects, stored);
        // LivingEntity.aiStep's separate fall-distance effect queries stay active.
        mount.fall_distance = 7.0;
        ai_production::reset_fall(&mut mount);
        assert_eq!(mount.fall_distance, 0.0);
    }
    #[test]
    fn ordinary_and_server_copied_mounts_retain_vanilla_queries() {
        for (client_side, server_rider) in [(false, false), (false, true), (true, true)] {
            let mut m = Mob::new(client_side);
            m.rider = Some(Rider {client: server_rider});
            m.extra.client_movement_queries = active();
            m.extra.effects = vec![("minecraft:slow_falling", 80, 0)];
            assert_eq!(m.effective_gravity(), 0.01);
            ai_production::run(&mut m);
            assert_eq!(m.ai.no_jump_delay, 6);
            assert!(!MobRestitution {mob: &mut m}.suppressing_bounce().unwrap());
            assert!(!MobLanding {mob: &mut m}.suppressing_bounce().unwrap());
        }
        let mut m = Mob::new(true);
        m.extra.effects = vec![("minecraft:levitation", 80, 2)];
        living_production::run(&mut m, &MobWorld, Vec3::ZERO, &mut Vec::new());
        let expected = (-0.2 + (0.15 + 0.2) * 0.2) * f64::from(0.98f32);
        assert!((m.velocity.y - expected).abs() < 1e-12);
    }
    #[test]
    fn mount_suppresses_restitution_and_landing_without_changing_pose() {
        let mut m = Mob::new(true);
        m.extra.client_movement_queries = active();
        assert!(MobRestitution {mob: &mut m}.suppressing_bounce().unwrap());
        assert!(MobLanding {mob: &mut m}.suppressing_bounce().unwrap());
        m.extra.client_movement_queries = ClientMovementQueries::default();
        assert!(!MobRestitution {mob: &mut m}.suppressing_bounce().unwrap());
        assert!(!MobLanding {mob: &mut m}.suppressing_bounce().unwrap());
    }
}
