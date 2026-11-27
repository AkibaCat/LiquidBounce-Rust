//! Exact native helper bodies, with explicit recording Game/player providers.
#[derive(Default)]
struct Ground {
    on_ground: bool,
}
struct Player {
    ground: Ground,
    velocity: glam::DVec3,
    allowed: bool,
}
impl Player {
    fn native_client_queries_allowed(&self) -> bool {
        self.allowed
    }
}
struct Game {
    player: Option<Player>,
    window: bool,
    lan_acting: Option<()>,
    lan_acting_uuid: Option<()>,
}
impl Game {
    fn native_window(&self) -> bool {
        self.window
    }
    /*HOST*/
}
#[test]
fn exact_helpers_set_all_velocity_axes_and_preserve_zero_sign_and_double_precision() {
    let mut game = Game {
        player: Some(Player {
            ground: Ground { on_ground: true },
            velocity: glam::DVec3::new(7.0, 8.0, 9.0),
            allowed: true,
        }),
        window: true,
        lan_acting: None,
        lan_acting_uuid: None,
    };
    assert_eq!(game.native_client_reset_vl_ground(), Some(true));
    for y in [0.1_f64, -0.1, -0.0] {
        game.native_client_reset_vl_velocity(y);
        assert_eq!(
            game.player
                .as_ref()
                .unwrap()
                .velocity
                .to_array()
                .map(f64::to_bits),
            [0.0_f64.to_bits(), y.to_bits(), 0.0_f64.to_bits()]
        );
    }
}
#[test]
fn absent_player_native_window_lan_actors_and_forbidden_queries_have_no_effect() {
    for blocked in 0..5 {
        let original = glam::DVec3::new(7.0, -0.0, 9.0);
        let mut game = Game {
            player: Some(Player {
                ground: Ground { on_ground: true },
                velocity: original,
                allowed: blocked != 4,
            }),
            window: blocked != 1,
            lan_acting: (blocked == 2).then_some(()),
            lan_acting_uuid: (blocked == 3).then_some(()),
        };
        if blocked == 0 {
            game.player = None;
        }
        assert_eq!(game.native_client_reset_vl_ground(), None);
        game.native_client_reset_vl_velocity(-0.1);
        if let Some(player) = game.player {
            assert_eq!(
                player.velocity.to_array().map(f64::to_bits),
                original.to_array().map(f64::to_bits)
            );
        }
    }
}
