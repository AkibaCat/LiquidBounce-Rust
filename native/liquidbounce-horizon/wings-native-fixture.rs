// Declaration providers only; production game bridge included verbatim.
pub mod gfx {
    pub mod wings {
        pub use crate::render::begin_frame;
    }
}
pub mod mobs {
    pub mod riders {
        pub fn solve_body_rot(head: f32, body: f32) -> f32 {
            let d = (head - body + 180.).rem_euclid(360.) - 180.;
            body + d.clamp(-85., 85.)
        }
    }
}
pub mod game {
    use std::{cell::RefCell, collections::BTreeMap};
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
    }
    pub struct Player {
        pub position: Vec3,
        pub dimensions: Dimensions,
        pub health: Health,
        pub shift_key_down: bool,
        pub attributes: Attributes,
    }
    impl Default for Player {
        fn default() -> Self {
            Self {
                position: Vec3::default(),
                dimensions: Dimensions { height: 1.8 },
                health: Health::default(),
                shift_key_down: false,
                attributes: Attributes(1.5),
            }
        }
    }
    pub struct Attributes(f64);
    impl Attributes {
        pub fn computed_value_of(&self, _: &str) -> Result<f64, ()> { Ok(self.0) }
    }
    pub struct Kind {
        pub id: String,
    }
    pub struct Mob {
        pub id: u32,
        pub removed: bool,
        pub kind: Kind,
        pub position: Vec3,
        pub previous: Vec3,
        pub body_yaw: f32,
        pub previous_body_yaw: f32,
        pub hurt_time: i32,
        pub chest: bool,
        pub scale: f32,
    }
    impl Default for Mob {
        fn default() -> Self {
            Self {
                id: 3,
                removed: false,
                kind: Kind {
                    id: "minecraft:player".into(),
                },
                position: Vec3::default(),
                previous: Vec3::default(),
                body_yaw: 40.,
                previous_body_yaw: 0.,
                hurt_time: 1,
                chest: true,
                scale: 2.5,
            }
        }
    }
    impl Mob {
        pub fn model_scale(&self) -> f32 {
            self.scale
        }
        pub fn dims(&self) -> (f64, f64, f64) {
            (0.6, 1.8, 1.62)
        }
        pub fn equipment_stacks(&self) -> [Option<Stack>; 8] {
            std::array::from_fn(|i| {
                if i == 4 && self.chest {
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
        pub conn: u32,
        pub name: String,
        pub motion: Option<Player>,
        pub position: [f64; 3],
        pub previous_position: [f64; 3],
        pub yaw: f32,
        pub vehicle: Option<u32>,
        pub armor: [Option<Stack>; 4],
    }
    impl Default for Guest {
        fn default() -> Self {
            let mut p = Player::default();
            p.scale = 0.75;
            Self {
                placed: true,
                dimension: "overworld".into(),
                uuid: 7,
                conn: 2,
                name: "friend".into(),
                motion: Some(p),
                position: [2., 0., 0.],
                previous_position: [2., 0., 0.],
                yaw: 20.,
                vehicle: None,
                armor: std::array::from_fn(|i| {
                    if i == 1 {
                        Some(Stack { count: 1 })
                    } else {
                        None
                    }
                }),
            }
        }
    }
    #[derive(Default)]
    pub struct Play {
        pub swing: bool,
    }
    impl Play {
        pub fn client_wings_swinging(&self) -> bool {
            self.swing
        }
    }
    #[derive(Default)]
    pub struct Hub {
        pub guests: Vec<Guest>,
    }
    pub struct Lan {
        pub hub: Hub,
        pub play: BTreeMap<u32, Play>,
    }
    pub struct Boat {
        pub id: u32,
        pub yaw: f32,
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
            false
        }
    }
    pub struct Game<'a> {
        pub player: Option<Player>,
        pub previous_position: Vec3,
        pub player_model: Option<Mob>,
        pub yaw: f32,
        pub pitch: f32,
        pub armor: [Option<Stack>; 4],
        pub mobs: Vec<Mob>,
        pub boats: Vec<Boat>,
        pub lan: Option<Lan>,
        pub dim: Dim,
        pub camera_type: Camera,
        pub input: RefCell<String>,
        pub output: RefCell<String>,
        pub hooks: crate::runtime::LiquidBounceHooks,
        pub client_wings_bodies: RefCell<wings::Bodies>,
        pub marker: std::marker::PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            let mut model = Mob::default();
            model.previous_body_yaw = 170.;
            model.body_yaw = -170.;
            Self {
                player: Some(Player::default()),
                previous_position: Vec3::default(),
                player_model: Some(model),
                yaw: 20.,
                pitch: 10.,
                armor: std::array::from_fn(|i| {
                    if i == 1 {
                        Some(Stack { count: 1 })
                    } else {
                        None
                    }
                }),
                mobs: vec![Mob::default()],
                boats: vec![],
                lan: Some(Lan {
                    hub: Hub {
                        guests: vec![Guest::default()],
                    },
                    play: BTreeMap::from_iter([(2, Play::default())]),
                }),
                dim: Dim,
                camera_type: Camera,
                input: Default::default(),
                output: Default::default(),
                hooks: Default::default(),
                client_wings_bodies: Default::default(),
                marker: Default::default(),
            }
        }
    }
    impl crate::mod_api::Host for Game<'_> {}
    impl Game<'_> {
        pub fn remote_player_name(&self, _: u32) -> Option<&str> {
            Some("friend")
        }
        pub fn native_client_rotation_model(
            &self,
            _: f32,
            body: &mut f32,
            _: &mut f32,
            _: &mut f32,
        ) {
            *body += 20.;
        }
        pub fn native_client_wings(&self, input: &str, geometry: &mut String) {
            *self.input.borrow_mut() = input.into();
            self.hooks.wings(
                self,
                &mut crate::mod_api::ClientWingsEvent { input, geometry },
            );
            *self.output.borrow_mut() = geometry.clone();
        }
        pub fn frame(&self, alpha: f64) {
            self.client_wings_frame(alpha)
        }
        pub fn tick(&mut self) {
            self.client_wings_tick()
        }
        pub fn reset(&self) {
            self.client_wings_reset()
        }
    }
    #[path = "NATIVE_PATH"]
    mod wings;
    #[test]
    fn actual_bridge_body_interpolation_scale_chest_and_local_shift() {
        let mut g = Game::default();
        g.tick();
        g.frame(0.5);
        let input: serde_json::Value = serde_json::from_str(&g.input.borrow()).unwrap();
        assert_eq!(input["entities"][0]["bodyYaw"], 200.);
        assert_eq!(input["entities"][1]["bodyYaw"], 20.);
        assert_eq!(input["entities"][2]["bodyYaw"], 20.);
        assert_eq!(input["entities"][0]["scale"], 1.5);
        assert_eq!(input["entities"][1]["scale"], 2.5);
        assert_eq!(input["entities"][2]["scale"], 0.75);
        assert!(
            input["entities"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| e["chest"] == true)
        );
        let output: serde_json::Value = serde_json::from_str(&g.output.borrow()).unwrap();
        assert_eq!(output["quads"].as_array().unwrap().len(), 96);
        g.player.as_mut().unwrap().shift_key_down = true;
        g.lan.as_mut().unwrap().play.get_mut(&2).unwrap().swing = true;
        g.lan.as_mut().unwrap().hub.guests[0].yaw = 100.;
        g.tick();
        g.frame(0.5);
        let input: serde_json::Value = serde_json::from_str(&g.input.borrow()).unwrap();
        assert_eq!(input["shift"], true);
        assert_eq!(input["entities"][2]["bodyYaw"], 35.);
        g.reset();
    }
}
