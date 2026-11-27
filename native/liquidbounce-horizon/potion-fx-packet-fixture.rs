// SPDX-License-Identifier: GPL-3.0-or-later
// Actual packet method below; actor/entity/ray providers remain fixture declarations.
#[cfg(test)]
mod packet_fixture {
    use serde_json::json;
    use std::{cell::RefCell, collections::BTreeMap};
    #[derive(Clone, Copy, Default)]
    struct Point {
        x: f64,
        y: f64,
        z: f64,
    }
    #[derive(Clone)]
    struct EffectInstance {
        id: &'static str,
        duration: i32,
        amplifier: i32,
        ambient: bool,
        visible: bool,
        show_icon: bool,
    }
    impl EffectInstance {
        fn new(id: &'static str, duration: i32, amplifier: i32) -> Self {
            Self {
                id,
                duration,
                amplifier,
                ambient: false,
                visible: true,
                show_icon: true,
            }
        }
    }
    #[derive(Default)]
    struct Effects {
        list: Vec<EffectInstance>,
    }
    #[derive(Default)]
    struct NativeState {
        effects: RefCell<BTreeMap<i32, Effects>>,
    }
    #[derive(Default)]
    struct Potions {
        thrown: Vec<(&'static str, Point, Point)>,
    }
    impl Potions {
        fn thrown_entities(&self) -> Vec<(&'static str, Point, Point)> {
            self.thrown.clone()
        }
    }
    struct Update {
        id: i32,
        effect: i32,
        duration: i32,
        amplifier: i32,
        flags: i8,
    }
    struct Remove {
        id: i32,
        effect: i32,
    }
    struct RemoveEntities {
        ids: Vec<i32>,
    }
    struct LevelEvent {
        kind: i32,
        pos: [i32; 3],
        data: i32,
    }
    enum Clientbound {
        UpdateMobEffect(Update),
        RemoveMobEffect(Remove),
        RemoveEntities(RemoveEntities),
        LevelEvent(LevelEvent),
        Other,
    }
    struct Game {
        liquidbounce_potion_fx: NativeState,
        potions: Potions,
        hooks: crate::runtime::LiquidBounceHooks,
        native: bool,
        ground: Option<[f64; 3]>,
    }
    impl Default for Game {
        fn default() -> Self {
            Self {
                liquidbounce_potion_fx: Default::default(),
                potions: Default::default(),
                hooks: Default::default(),
                native: true,
                ground: None,
            }
        }
    }
    impl Game {
        fn native_window(&self) -> bool {
            self.native
        }
        fn remote_effect_id(&self, id: i32) -> Option<&'static str> {
            (id == 1).then_some("speed")
        }
        fn client_potion_fx_ground(&self, _: [f64; 3]) -> Option<[f64; 3]> {
            self.ground
        }
        fn native_client_potion_fx(&self, input: &str, geometry: &mut String) {
            self.hooks.potion_fx(
                &(),
                &mut crate::mod_api::ClientPotionFxEvent { input, geometry },
            );
        }
        // ACTUAL_PACKET_FUNCTION
    }
    #[test]
    fn actual_packet_requires_2002_splash_entity_box_and_uses_ground_or_center() {
        let mut g = Game::default();
        let event = |kind| {
            Clientbound::LevelEvent(LevelEvent {
                kind,
                pos: [0, 1, 2],
                data: 0x80123456u32 as i32,
            })
        };
        g.client_potion_fx_packet(&event(2002));
        assert!(
            g.hooks
                .state()
                .behaviors
                .borrow()
                .potion_fx
                .splashes
                .is_empty()
        );
        g.potions.thrown.push((
            "minecraft:lingering_potion",
            Point {
                x: 0.,
                y: 1.,
                z: 2.,
            },
            Point::default(),
        ));
        g.client_potion_fx_packet(&event(2002));
        assert!(
            g.hooks
                .state()
                .behaviors
                .borrow()
                .potion_fx
                .splashes
                .is_empty()
        );
        g.potions.thrown[0].0 = "minecraft:splash_potion";
        g.client_potion_fx_packet(&event(2007));
        assert!(
            g.hooks
                .state()
                .behaviors
                .borrow()
                .potion_fx
                .splashes
                .is_empty()
        );
        g.client_potion_fx_packet(&event(2002));
        assert_eq!(
            g.hooks.state().behaviors.borrow().potion_fx.splashes[0].position,
            [0.5, 1.5, 2.5]
        );
        g.ground = Some([0.5, 1.25, 2.5]);
        g.client_potion_fx_packet(&event(2002));
        assert_eq!(
            g.hooks.state().behaviors.borrow().potion_fx.splashes[1].position,
            [0.5, 1.25, 2.5]
        );
        g.client_potion_fx_packet(&Clientbound::UpdateMobEffect(Update {
            id: 7,
            effect: 1,
            duration: 20,
            amplifier: 2,
            flags: 2,
        }));
        assert_eq!(
            g.liquidbounce_potion_fx.effects.borrow()[&7].list[0].amplifier,
            2
        );
        g.client_potion_fx_packet(&Clientbound::RemoveEntities(RemoveEntities {
            ids: vec![7],
        }));
        assert!(g.liquidbounce_potion_fx.effects.borrow().is_empty());
    }
}
