// Compile the transformed local attack entries verbatim. Physics is outside
// this entry-sequencing fixture; ids, mutation, cancellation and swings are
// observable so an event cannot leave a stale target index behind.
#[cfg(test)]
mod local_attack_entries {
    #[derive(Clone, Copy)]
    struct Kind;
    impl Kind {
        fn id(&self) -> &'static str {
            "minecraft:minecart"
        }
    }
    #[derive(Clone)]
    struct Vehicle {
        id: u32,
        kind: Kind,
    }
    impl Vehicle {
        fn bounds(&self) -> u32 {
            self.id
        }
    }
    struct Stack;
    impl Stack {
        fn enchantment_level(&self, _: &str) -> i32 {
            0
        }
    }
    struct Game {
        boats: Vec<Vehicle>,
        carts: Vec<Vehicle>,
        events: Vec<String>,
        hit: bool,
        cancel: bool,
        remove: bool,
        creative: bool,
        attack_ticks: i32,
    }
    impl Game {
        fn new() -> Self {
            let vehicles = vec![
                Vehicle { id: 17, kind: Kind },
                Vehicle { id: 42, kind: Kind },
            ];
            Self {
                boats: vehicles.clone(),
                carts: vehicles,
                events: Vec::new(),
                hit: true,
                cancel: false,
                remove: false,
                creative: false,
                attack_ticks: 9,
            }
        }
        fn boat_hit(&self) -> Option<(usize, f64)> {
            self.hit.then_some((1, 2.0))
        }
        fn cart_hit(&self) -> Option<(usize, f64)> {
            self.hit.then_some((1, 2.0))
        }
        fn before_block(&self, _: f64) -> bool {
            true
        }
        fn mob_hit(&self) -> Option<(usize, f64)> {
            None
        }
        fn client_weapon_before_attack(&mut self, id: u32, living: bool) -> bool {
            self.events.push(format!("event:{id}:{living}"));
            if self.remove {
                self.boats.retain(|v| v.id != id);
                self.carts.retain(|v| v.id != id);
            } else {
                // Reordering is another valid mutation during an event.
                self.boats.reverse();
                self.carts.reverse();
            }
            !self.cancel
        }
        fn strike_boat(&mut self, index: usize) {
            self.events.push(format!("hit:{}", self.boats[index].id));
        }
        fn swing_arm(&mut self) {
            self.events.push("swing".into());
        }
        fn held(&self) -> Option<&Stack> {
            None
        }
        fn melee_damage(&self, _: &str, _: bool) -> (f32, f32, bool, bool) {
            (1.0, 1.0, false, false)
        }
        fn hurt_cart_result(&mut self, index: usize, _: f32, _: bool, _: bool) -> bool {
            self.events.push(format!("hit:{}", self.carts[index].id));
            true
        }
        fn vehicle_attack_sounds(&self, _: u32, _: Option<i32>) {}
        fn extra_knockback(&self, _: &str, _: f32) -> (f32, bool) {
            (0.0, false)
        }
        fn special_attack_slowdown(&self, _: f32) {}
        fn special_attack_exhaustion(&self) {}
        /* BOAT_ENTRY */
        /* CART_ENTRY */
    }
    #[test]
    fn local_entries_dispatch_once_re_resolve_ids_and_preserve_start_attack_swing() {
        for cart in [false, true] {
            for cancel in [false, true] {
                for remove in [false, true] {
                    let mut game = Game::new();
                    game.cancel = cancel;
                    game.remove = remove;
                    assert!(if cart {
                        game.attack_cart()
                    } else {
                        game.attack_boat()
                    });
                    let expected = if cancel || remove {
                        vec!["event:42:false", "swing"]
                    } else {
                        vec!["event:42:false", "hit:42", "swing"]
                    };
                    assert_eq!(
                        game.events, expected,
                        "cart={cart} cancel={cancel} remove={remove}"
                    );
                }
            }
            let mut game = Game::new();
            game.hit = false;
            assert!(!if cart {
                game.attack_cart()
            } else {
                game.attack_boat()
            });
            assert!(game.events.is_empty());
        }
    }
}
