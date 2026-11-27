// Declaration providers only; the production Game bridge is included verbatim.
pub mod gfx {
    pub mod hats {
        pub use crate::render::{begin_frame, take_error};
    }
}
pub mod game {
    use std::cell::RefCell;
    #[derive(Clone, Copy, Default)]
    pub struct Vec3 {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }
    #[derive(Clone)]
    pub struct Stack {
        pub count: u32,
    }
    #[derive(Default)]
    pub struct Hurt {
        pub hurt_time: i32,
    }
    #[derive(Default)]
    pub struct Health {
        pub hurt: Hurt,
    }
    pub struct Dimensions {
        pub height: f32,
        pub eye_height: f32,
    }
    pub struct Player {
        pub position: Vec3,
        pub dimensions: Dimensions,
        pub health: Health,
    }
    impl Default for Player {
        fn default() -> Self {
            Self {
                position: Vec3::default(),
                dimensions: Dimensions {
                    height: 1.8,
                    eye_height: 1.62,
                },
                health: Health::default(),
            }
        }
    }
    pub struct Kind {
        pub id: String,
    }
    pub struct Mob {
        pub id: u32,
        pub age: i32,
        pub removed: bool,
        pub kind: Kind,
        pub position: Vec3,
        pub previous: Vec3,
        pub yaw: f32,
        pub previous_yaw: f32,
        pub head_pitch: f32,
        pub previous_head_pitch: f32,
        pub hurt_time: i32,
        pub head: bool,
    }
    impl Default for Mob {
        fn default() -> Self {
            Self {
                id: 3,
                age: 1,
                removed: false,
                kind: Kind {
                    id: "minecraft:player".into(),
                },
                position: Vec3::default(),
                previous: Vec3::default(),
                yaw: 80.,
                previous_yaw: 40.,
                head_pitch: 20.,
                previous_head_pitch: 0.,
                hurt_time: 1,
                head: true,
            }
        }
    }
    impl Mob {
        pub fn dims(&self) -> (f64, f64, f64) {
            (0.6, 1.8, 1.62)
        }
        pub fn eye_height(&self) -> f64 {
            1.62
        }
        pub fn equipment_stacks(&self) -> [Option<Stack>; 8] {
            std::array::from_fn(|i| {
                if i == 5 && self.head {
                    Some(Stack { count: 1 })
                } else {
                    None
                }
            })
        }
    }
    pub struct Guest {
        pub placed: bool,
        pub dimension: String,
        pub uuid: u128,
        pub name: String,
        pub motion: Option<Player>,
        pub position: [f64; 3],
        pub previous_position: [f64; 3],
        pub yaw: f32,
        pub pitch: f32,
        pub armor: [Option<Stack>; 4],
    }
    impl Default for Guest {
        fn default() -> Self {
            Self {
                placed: true,
                dimension: "overworld".into(),
                uuid: 7,
                name: "friend".into(),
                motion: Some(Player::default()),
                position: [2., 0., 0.],
                previous_position: [1., 0., 0.],
                yaw: 20.,
                pitch: 0.,
                armor: Default::default(),
            }
        }
    }
    #[derive(Default)]
    pub struct Hub {
        pub guests: Vec<Guest>,
    }
    #[derive(Default)]
    pub struct Lan {
        pub hub: Hub,
    }
    pub struct Dim;
    impl Dim {
        pub fn id(&self) -> &str {
            "overworld"
        }
    }
    pub struct Camera;
    impl Camera {
        pub fn first_person(&self) -> bool {
            true
        }
    }
    pub struct Game<'a> {
        pub player: Option<Player>,
        pub previous_position: Vec3,
        pub player_model: Option<Mob>,
        pub rope_rot_old: (f32, f32),
        pub yaw: f32,
        pub pitch: f32,
        pub armor: [Option<Stack>; 4],
        pub mobs: Vec<Mob>,
        pub lan: Option<Lan>,
        pub dim: Dim,
        pub camera_type: Camera,
        pub input: RefCell<String>,
        pub hooks: crate::runtime::LiquidBounceHooks,
        pub client_hats_rotations: RefCell<hats::Rotations>,
        pub marker: std::marker::PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            Self {
                player: Some(Player::default()),
                previous_position: Vec3::default(),
                player_model: Some(Mob::default()),
                rope_rot_old: (0., 0.),
                yaw: 20.,
                pitch: 10.,
                armor: Default::default(),
                mobs: vec![Mob::default()],
                lan: Some(Lan {
                    hub: Hub {
                        guests: vec![Guest::default()],
                    },
                }),
                dim: Dim,
                camera_type: Camera,
                input: Default::default(),
                hooks: Default::default(),
                client_hats_rotations: Default::default(),
                marker: Default::default(),
            }
        }
    }
    impl crate::mod_api::Host for Game<'_> {
        fn chat(&mut self, _: &str) {}
    }
    impl Game<'_> {
        pub fn remote_player_name(&self, _: u32) -> Option<&str> {
            Some("friend")
        }
        pub fn native_client_hats(&self, input: &str, geometry: &mut String) {
            *self.input.borrow_mut() = input.to_owned();
            self.hooks.hats(
                self,
                &mut crate::mod_api::ClientHatsEvent { input, geometry },
            );
        }
        pub fn frame(&self, a: f64) {
            self.client_hats_frame(a)
        }
        pub fn tick(&mut self) {
            self.client_hats_tick()
        }
        pub fn reset(&self) {
            self.client_hats_reset()
        }
    }
    #[path = "NATIVE_PATH"]
    mod hats;
}
