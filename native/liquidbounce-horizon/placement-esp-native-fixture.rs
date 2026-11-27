// Declaration-only native providers; production bridge is included unmodified.
pub mod position {
    #[derive(Clone, Copy)]
    pub struct BlockPos {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }
    impl BlockPos {
        pub fn new(x: i32, y: i32, z: i32) -> Self {
            Self { x, y, z }
        }
    }
}
pub mod game {
    use crate::{mod_api::*, position::BlockPos, runtime::LiquidBounceHooks};
    use std::collections::BTreeSet;
    #[derive(Default)]
    pub struct Position {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }
    #[derive(Default)]
    pub struct Player {
        pub position: Position,
        pub yaw: f32,
    }
    pub struct Falling {
        pub position: Position,
    }
    pub struct Facts {
        pub entries: [Fact; 2],
    }
    pub struct Fact {
        pub name: String,
        pub outline_offset: bool,
        pub outline: Vec<[f64; 6]>,
    }
    impl Default for Facts {
        fn default() -> Self {
            Self {
                entries: [
                    Fact {
                        name: "minecraft:air".into(),
                        outline_offset: false,
                        outline: vec![],
                    },
                    Fact {
                        name: "minecraft:stone".into(),
                        outline_offset: false,
                        outline: vec![[0., 0., 0., 1., 1., 1.]],
                    },
                ],
            }
        }
    }
    impl Facts {
        pub fn get(&self, state: u32) -> &Fact {
            &self.entries[state as usize]
        }
        pub fn is_air(&self, state: u32) -> bool {
            state == 0
        }
    }
    #[derive(Default)]
    pub struct Resources {
        pub facts: Facts,
    }
    impl Resources {
        pub fn facts(&self) -> &Facts {
            &self.facts
        }
    }
    pub struct Tables;
    impl Tables {
        pub fn offset(&self, _: u32, _: i32, _: i32) -> Option<[f64; 3]> {
            None
        }
    }
    pub struct Context {
        pub tables: Tables,
    }
    pub struct Pack {
        pub context: Context,
    }
    pub struct Layout;
    impl Layout {
        pub fn min_y(&self) -> i32 {
            -64
        }
    }
    #[derive(Default)]
    pub struct World {
        pub blocks: BTreeSet<[i32; 3]>,
    }
    impl World {
        pub fn block(&self, x: i32, y: i32, z: i32) -> u32 {
            u32::from(self.blocks.contains(&[x, y, z]))
        }
        pub fn layout(&self) -> Layout {
            Layout
        }
    }
    pub struct Level<'a> {
        pub sturdy: &'a BTreeSet<[i32; 3]>,
    }
    pub struct Ticks;
    pub struct Behaviours;
    impl Behaviours {
        pub fn client_placement_esp_sturdy(
            &self,
            l: &mut Level<'_>,
            _: &mut Ticks,
            p: BlockPos,
        ) -> bool {
            l.sturdy.contains(&[p.x, p.y, p.z])
        }
    }
    pub struct Game<'a> {
        pub world: World,
        pub res: Resources,
        pub pack: Pack,
        pub player: Option<Player>,
        pub falling: Vec<Falling>,
        pub hooks: LiquidBounceHooks,
        pub marker: std::marker::PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            Self {
                world: Default::default(),
                res: Default::default(),
                pack: Pack {
                    context: Context { tables: Tables },
                },
                player: Some(Player {
                    position: Position {
                        x: 0.,
                        y: 1.,
                        z: 0.,
                    },
                    yaw: 0.,
                }),
                falling: vec![],
                hooks: Default::default(),
                marker: Default::default(),
            }
        }
    }
    impl Game<'_> {
        pub fn native_window(&self) -> bool {
            true
        }
        pub fn context_outline(&self, _: [i32; 3], _: u32) -> Option<[f64; 6]> {
            Some([0., 0., 0., 1., 1., 1.])
        }
        pub fn with_level<R>(
            &mut self,
            f: impl FnOnce(&Behaviours, &mut Level<'_>, &mut Ticks) -> R,
        ) -> R {
            f(
                &Behaviours,
                &mut Level {
                    sturdy: &self.world.blocks,
                },
                &mut Ticks,
            )
        }
        pub fn native_client_placement_esp(&self, input: &str, output: &mut String) {
            self.hooks
                .placement_esp(&NullHost, &mut ClientPlacementEspEvent { input, output });
        }
        pub fn packet(&self, p: &crate::packets::play::Clientbound) {
            self.client_placement_esp_packet(p);
        }
        pub fn tick(&mut self) {
            self.client_placement_esp_tick();
        }
        pub fn player_tick(&mut self) {
            self.client_placement_esp_player_tick();
        }
        pub fn geometry(&self, p: &mut crate::world_primitives::WorldPrimitives) {
            self.client_placement_esp_geometry(p);
        }
    }
    #[path = "NATIVE_PATH"]
    mod placement_esp;
    #[path = "SEARCH_PATH"]
    mod placement_esp_search;
}
