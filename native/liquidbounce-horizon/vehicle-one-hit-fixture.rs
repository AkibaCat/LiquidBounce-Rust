#![allow(dead_code)]
extern crate self as horizon_client;
extern crate self as horizon_core;
pub mod vehicle_one_hit;
mod no_swing;
mod version {
    /* VERSION */
}
pub mod first_person {
    /* HAND */
    /* ANIMATION */
    /* ANIMATION_IMPL */
}
mod item_render {
    pub fn stack_view(stack: &serde_json::Value) -> serde_json::Value { stack.clone() }
}
mod game {
    use crate::first_person::{Hand, SwingAnimation};
    use horizon_protocol::packets::play::Serverbound;
    struct Models;
    impl Models {
        fn components_view(&self, view: &serde_json::Value) -> serde_json::Value { view.clone() }
    }
    struct ItemRenderer { models: Models }
    struct Pack { item_renderer: ItemRenderer }
    struct Game {
        owner: bool,
        kind: Option<String>,
        inventory: Vec<Option<serde_json::Value>>,
        selected: usize,
        pack: Pack,
        no_swing: crate::no_swing::NoSwing,
        cancel_packets: bool,
        events: Vec<String>,
    }
    impl Default for Game {
        fn default() -> Self {
            Self { owner: true, kind: Some("minecraft:oak_boat".into()),
                inventory:vec![None,None],selected:1,pack:Pack{item_renderer:ItemRenderer{models:Models}},
                no_swing:Default::default(),cancel_packets:false,events:Vec::new() }
        }
    }
    impl Game {
        fn native_client_slot_owner(&self) -> bool { self.owner }
        fn native_entity(&self, target:u32) -> Option<u32> { self.kind.as_ref().map(|_|target) }
        fn native_entity_kind(&self, _:u32) -> Option<String> { self.kind.clone() }
        fn client_hotbar_slot(&self) -> usize { self.selected }
        fn send(&mut self, packet:Serverbound) {
            // Observe the ordinary send even when a packet listener cancels it.
            let Serverbound::Attack(packet)=packet else {panic!("unexpected packet: {packet:?}")};
            self.events.push(format!("attack:{}:{}",packet.id,self.cancel_packets));
        }
        fn swing_hand(&mut self, hand:Hand, animation:SwingAnimation) {
            self.events.push(format!("swing:{hand:?}:{}:{}:{}",animation.kind,animation.duration,self.no_swing.should_hide_for_client()));
        }
    }
    mod native_hooks {
        use super::Game;
        impl Game {
            /* HOST */
        }
    }
    #[test]
    fn real_native_helpers_resolve_effective_stack_and_keep_ordinary_packet_order() {
        use crate::vehicle_one_hit::{self,Action};
        let mut game=Game::default();
        let context=game.native_client_vehicle_one_hit_context(42).unwrap();
        assert_eq!(context,serde_json::json!({"kind":"minecraft:oak_boat","legacy":false}));
        // Visible slot zero differs from effective slot one.
        game.inventory[0]=Some(serde_json::json!({"minecraft:attack_animation":{"type":"none","duration":0}}));
        game.inventory[1]=Some(serde_json::json!({"minecraft:attack_animation":{"type":"stab","duration":11}}));
        for legacy in [false,true] {
            for no_swing in [false,true] {
                for cancel in [false,true] {
                    game.events.clear();game.cancel_packets=cancel;game.no_swing.set_running(no_swing);
                    let actions=vehicle_one_hit::actions(true,"minecraft:oak_boat",i32::MIN,legacy);
                    for action in actions {
                        match action {Action::Attack(id)=>game.native_client_vehicle_one_hit_attack(id),Action::Swing=>game.native_client_vehicle_one_hit_swing()}
                    }
                    let attack=format!("attack:{}:{cancel}",i32::MIN);
                    let swing=format!("swing:Main:2:11:{no_swing}");
                    let pair=if legacy {[swing,attack]} else {[attack,swing]};
                    let expected:Vec<_>=(0..if legacy {3}else{19}).flat_map(|_|pair.clone()).collect();
                    assert_eq!(game.events,expected);
                }
            }
        }
        game.owner=false;game.events.clear();
        assert!(game.native_client_vehicle_one_hit_context(42).is_none());
        game.native_client_vehicle_one_hit_attack(42);game.native_client_vehicle_one_hit_swing();
        assert!(game.events.is_empty());
        game.owner=true;game.kind=None;
        assert!(game.native_client_vehicle_one_hit_context(42).is_none());
    }
}
