pub mod registry {
    pub struct BlockStateId(pub u32);
    pub struct BlockState {
        pub id: u32,
    }
    pub struct Registry {
        pub states: [BlockState; 3],
    }
    impl Default for Registry {
        fn default() -> Self {
            Self {
                states: [
                    BlockState { id: 0 },
                    BlockState { id: 1 },
                    BlockState { id: 2 },
                ],
            }
        }
    }
    impl Registry {
        pub fn state(&self, id: BlockStateId) -> Option<&BlockState> {
            self.states.get(id.0 as usize)
        }
    }
}
pub mod fluid {
    use crate::registry::BlockState;
    #[derive(PartialEq)]
    pub enum FluidKind {
        Empty,
        Water,
    }
    pub struct FluidState {
        id: u32,
    }
    impl FluidState {
        pub fn from_block(b: &BlockState) -> std::io::Result<Self> {
            Ok(Self { id: b.id })
        }
        pub fn kind(&self) -> FluidKind {
            if self.id == 0 {
                FluidKind::Empty
            } else {
                FluidKind::Water
            }
        }
        pub fn is_source(&self) -> bool {
            self.id == 1
        }
    }
}
pub mod game {
    use crate::{mod_api::*, registry::Registry, runtime::LiquidBounceHooks};
    #[derive(Default)]
    pub struct Position {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }
    #[derive(Default)]
    pub struct Player {
        pub position: Position,
    }
    #[derive(Default)]
    pub struct Assets {
        pub registry: Registry,
    }
    #[derive(Default)]
    pub struct Pack {
        pub assets: Assets,
    }
    pub struct Game<'a> {
        pub pack: Pack,
        pub player: Option<Player>,
        pub hooks: LiquidBounceHooks,
        pub marker: std::marker::PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            Self {
                pack: Default::default(),
                player: Some(Default::default()),
                hooks: Default::default(),
                marker: Default::default(),
            }
        }
    }
    impl Game<'_> {
        pub fn native_window(&self) -> bool {
            true
        }
        pub fn native_client_new_chunks(&self, input: &str, geometry: &mut String) {
            self.hooks
                .new_chunks(&NullHost, &mut ClientNewChunksEvent { input, geometry });
        }
        pub fn packet(&self, p: &crate::packets::play::Clientbound) {
            self.client_new_chunks_packet(p);
        }
        pub fn geometry(&self, p: &mut crate::world_primitives::WorldPrimitives) {
            self.client_new_chunks_geometry(p);
        }
    }
    #[path = "NATIVE_PATH"]
    pub mod new_chunks;
}
