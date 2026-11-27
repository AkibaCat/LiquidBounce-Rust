#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
use std::sync::atomic::{AtomicU8, Ordering};
static FLAGS: AtomicU8 = AtomicU8::new(0);
#[path = "entity_control_behavior.rs"]
mod behavior;
mod entity_control;
mod items {
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ItemStack {
        pub item: Box<str>,
        pub damage: u32,
        pub extra: Option<()>,
        pub count: i32,
    }
    impl ItemStack {
        pub fn new(item: &str, count: i32) -> Self {
            Self {
                item: item.into(),
                count,
                damage: 0,
                extra: None,
            }
        }
    }
}
pub mod mobs {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Species {
        Horse,
        Donkey,
        Mule,
        SkeletonHorse,
        ZombieHorse,
        Camel,
        CamelHusk,
        Llama,
        TraderLlama,
        Pig,
        Strider,
        Nautilus,
        ZombieNautilus,
        HappyGhast,
    }
    #[derive(Default)]
    pub struct Extra {
        pub saddled: bool,
        pub saddle_stack: Option<crate::items::ItemStack>,
        pub anim_ticks: i32,
        pub stand_sliding: bool,
        pub dashing: bool,
    }
    pub struct Mob {
        pub id: u32,
        pub kind: Species,
        pub extra: Extra,
        pub rider: Option<equine::Rider>,
        pub living: bool,
        pub harness: bool,
        pub still: bool,
    }
    impl Mob {
        pub fn new(kind: Species) -> Self {
            Self {
                id: 1,
                kind,
                extra: Default::default(),
                rider: None,
                living: true,
                harness: false,
                still: false,
            }
        }
        pub fn species(&self) -> Species {
            self.kind
        }
        pub fn alive(&self) -> bool {
            self.living
        }
        /* MOB_GETTER */
        /* SADDLE_ITEM */
        /* SET_SADDLE */
    }
    pub mod happy_ghast {
        use super::Mob;
        pub fn controlled(m: &Mob) -> bool {
            m.rider.is_some() && harnessed_adult(m) && !on_still_timeout(m)
        }
        pub fn harnessed_adult(m: &Mob) -> bool {
            m.harness
        }
        pub fn on_still_timeout(m: &Mob) -> bool {
            m.still
        }
    }
    pub mod llama {
        use super::Species;
        pub fn is_llama(s: Species) -> bool {
            matches!(s, Species::Llama | Species::TraderLlama)
        }
    }
    pub mod camel {
        use super::{Mob, Species};
        pub fn is_camel(s: Species) -> bool {
            matches!(s, Species::Camel | Species::CamelHusk)
        }
        pub fn refuse_to_move(m: &Mob) -> bool {
            m.still
        }
        pub fn set_dashing(m: &mut Mob, value: bool) {
            m.extra.dashing = value;
        }
        pub fn voice(_: &Mob, _: i32) -> &'static str {
            "entity.camel.dash"
        }
    }
    pub mod nautilus {
        use super::{Mob, Species};
        pub fn is_nautilus(s: Species) -> bool {
            matches!(s, Species::Nautilus | Species::ZombieNautilus)
        }
        /* NAUTILUS_CONTROLLED */
        /* NAUTILUS_ON_JUMP */
    }
    pub mod equine {
        use super::{Mob, Species};
        #[derive(Clone, Copy, Default)]
        pub struct Rider {
            pub steer: bool,
            pub jump: f32,
        }
        /* IS_EQUINE */
        /* CONTROLLED */
        /* MOVED_BY_RIDER */
        /* JUMP_PENDING */
        /* EQUINE_ON_JUMP */
        fn stand_if_possible(_: &mut Mob) {}
        pub fn jump_sound(_: &Mob) -> &'static str {
            "entity.horse.jump"
        }
    }
}
pub mod hud {
    #[derive(Default)]
    pub struct HudBatch(pub Vec<(i32, f32)>);
}
pub mod hud_overlays {
    pub fn jump_bar(_: i32, _: i32, cooldown: i32, scale: f32) -> (i32, f32) {
        (cooldown, scale)
    }
}
pub mod hud_layout {
    pub fn paint(batch: &mut crate::hud::HudBatch, _: &(), draws: &(i32, f32)) {
        batch.0.push(*draws);
    }
}
mod play {
    #[derive(Debug, PartialEq, Eq)]
    pub struct PlayerCommand {
        pub id: i32,
        pub action: i32,
        pub data: i32,
    }
}
#[derive(Debug, PartialEq, Eq)]
enum S {
    PlayerCommand(play::PlayerCommand),
}
mod mod_api {
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(|m| &**m)
    }
    #[allow(unused_imports)]
    mod render {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/api-source/crates/horizon-client-render/src/gfx/native_render/generated.rs"
        ));
    }
    pub use render::*;
    #[allow(unused_imports)]
    mod host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs"));
    }
    pub use host::*;
    mod hooks {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs"));
    }
    pub use hooks::*;
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: std::sync::OnceLock<Vec<Box<dyn Hooks>>> = std::sync::OnceLock::new();
        MODS.get_or_init(|| vec![Box::new(TestHooks)])
    }
    struct TestHooks;
    impl Hooks for TestHooks {
        fn client_entity_control(&self, _: &dyn Host, event: &mut ClientEntityControlEvent) {
            let bits = crate::FLAGS.load(crate::Ordering::SeqCst);
            event.saddled = bits & 1 != 0;
            event.jump_strength = bits & 2 != 0;
        }
    }
}
mod game {
    use crate::{
        S,
        hud::HudBatch,
        mobs::{Mob, Species, equine, happy_ghast},
        play,
    };
    #[derive(Default, Clone, Copy)]
    struct Keys {
        jump: bool,
    }
    #[derive(Default)]
    struct PlayerMotion {
        server_copy: bool,
        replaying: bool,
    }
    impl PlayerMotion {
        /* IDENTITY */
    }
    #[derive(Default)]
    struct Gui {
        atlas: (),
    }
    /* MOUNT_STATE */
    impl MountState {
        /* RESET_JUMP */
    }
    struct Game<'a> {
        window: bool,
        lan_acting_uuid: Option<()>,
        player: Option<PlayerMotion>,
        mobs: Vec<Mob>,
        riding: Option<u32>,
        mount: MountState,
        gui: Gui,
        marker: std::marker::PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            Self {
                window: true,
                lan_acting_uuid: None,
                player: Some(Default::default()),
                mobs: vec![],
                riding: None,
                mount: Default::default(),
                gui: Default::default(),
                marker: std::marker::PhantomData,
            }
        }
    }
    impl Game<'_> {
        fn native_window(&self) -> bool {
            self.window
        }
        /* MOUNT_INDEX */
        /* CHARGE */
        /* JUMPS */
        /* JUMPABLE */
        /* MOUNT_JUMPING */
        /* DRAW_JUMP_BAR */
    }
    mod native_hooks {
        use super::Game;
        use crate::mod_api; /* NATIVE */
    }
    mod mod_api_host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs"));
    }
    /* CLIENT_MOVES */
    /* RIDING_JUMP */
    /* START_RIDING_JUMP */

    fn set_policy(bits: u8) {
        crate::FLAGS.store(bits, crate::Ordering::SeqCst);
        Game::default().native_client_tick_pre();
    }
    fn horse_game(kind: Species) -> Game<'static> {
        Game {
            mobs: vec![Mob::new(kind)],
            riding: Some(1),
            ..Default::default()
        }
    }
    #[test]
    fn generated_event_updates_global_mob_getter_but_only_actual_local_player_jump_scale() {
        set_policy(3);
        assert!(Mob::new(Species::Pig).is_saddled());
        let game = Game::default();
        assert_eq!(game.native_client_riding_jump_scale(0.3), 1.);
        for exclusion in 0..5 {
            let mut game = Game::default();
            match exclusion {
                0 => game.window = false,
                1 => game.lan_acting_uuid = Some(()),
                2 => game.player = None,
                3 => game.player.as_mut().unwrap().server_copy = true,
                _ => game.player.as_mut().unwrap().replaying = true,
            }
            assert_eq!(game.native_client_riding_jump_scale(0.3), 0.3);
        }
        crate::FLAGS.store(0, crate::Ordering::SeqCst);
        let mut server = Game {
            window: false,
            ..Default::default()
        };
        server.native_client_tick_pre();
        assert!(
            Mob::new(Species::Pig).is_saddled(),
            "server tick cannot overwrite window singleton snapshot"
        );
        Game::default().native_client_tick_pre();
        assert!(!Mob::new(Species::Pig).is_saddled());
    }
    #[test]
    fn full_charge_state_machine_uses_effective_scale_without_mutating_charge_storage() {
        for bits in 0..4 {
            set_policy(bits);
            let mut game = horse_game(Species::Horse);
            game.mobs[0].set_saddle(Some(crate::items::ItemStack::new("minecraft:saddle", 1)));
            assert_eq!(game.charge_mount_jump(0, Keys { jump: true }), 0.);
            for _ in 0..3 {
                assert_eq!(game.charge_mount_jump(0, Keys { jump: true }), 0.);
            }
            let stored = game.mount.jump_scale;
            assert_eq!(stored, 0.3);
            let jump = game.charge_mount_jump(0, Keys { jump: false });
            let expected = if bits & 2 != 0 {
                1.
            } else {
                equine::jump_pending_scale(30)
            };
            assert_eq!(jump, expected);
            assert_eq!(game.mount.jump_ticks, -10);
            assert_eq!(game.mount.jump_scale, stored);
            let S::PlayerCommand(packet) =
                riding_jump(7, game.native_client_riding_jump_scale(stored));
            assert_eq!(
                (packet.id, packet.action, packet.data),
                (7, 3, if bits & 2 != 0 { 100 } else { 30 })
            );
            let mut batch = HudBatch::default();
            game.draw_jump_bar(&mut batch, 800, 600);
            assert_eq!(batch.0[0].1, if bits & 2 != 0 { 1. } else { stored });
        }
        set_policy(0);
    }
    #[test]
    fn saddled_control_respects_stick_first_passenger_liveness_and_actual_equipment() {
        set_policy(1);
        for kind in [
            Species::Horse,
            Species::Llama,
            Species::Nautilus,
            Species::Pig,
            Species::Strider,
        ] {
            let mut game = horse_game(kind);
            assert_eq!(game.mobs[0].saddle_item(), None);
            let stick = matches!(kind, Species::Pig | Species::Strider);
            assert_eq!(client_moves(&game.mobs[0], true, false), !stick);
            assert!(client_moves(&game.mobs[0], true, true));
            assert!(!client_moves(&game.mobs[0], false, true));
            game.mobs[0].living = false;
            assert!(!client_moves(&game.mobs[0], true, true));
        }
        let mut game = horse_game(Species::Horse);
        assert!(game.jumpable_mount());
        assert!(start_riding_jump(&mut game.mobs[0], 100).is_some());
        assert!(game.mobs[0].extra.stand_sliding);
        set_policy(0);
        assert!(!game.jumpable_mount());
        assert!(start_riding_jump(&mut game.mobs[0], 100).is_none());
        assert!(!game.mobs[0].is_saddled());
        assert_eq!(game.mobs[0].saddle_item(), None);
    }
    #[test]
    fn actual_mc_control_and_jump_getter_oracle() {
        /* ORACLE */
        set_policy(0);
    }
}
