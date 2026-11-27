// Declaration-only Game/world providers. The actual paletted containers,
// storage, world primitives and unmodified production bridge are compiled.
pub mod registry {
    #[derive(Clone, Copy)]
    pub struct BlockStateId(pub u32);
}
pub mod chunk {
    #[derive(Clone)]
    pub struct Section {
        pub blocks: crate::paletted::PalettedContainer,
    }
    impl Section {
        pub fn get(&self, i: usize) -> u32 {
            self.blocks.get(i)
        }
    }
}
pub mod game {
    use crate::{
        chunk::Section, mod_api::*, registry::BlockStateId, runtime::LiquidBounceHooks,
        world_primitives::WorldPrimitives,
    };
    use std::{cell::RefCell, collections::BTreeMap, marker::PhantomData, sync::Arc};
    pub struct Position {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }
    pub struct Player {
        pub position: Position,
    }
    pub struct Fact {
        pub name: Arc<str>,
    }
    pub struct Facts {
        pub entries: Vec<Fact>,
    }
    impl Facts {
        pub fn is_air(&self, id: BlockStateId) -> bool {
            id.0 == 0
        }
        pub fn get(&self, id: BlockStateId) -> &Fact {
            &self.entries[id.0 as usize]
        }
    }
    pub struct Resources {
        pub facts: Facts,
    }
    impl Resources {
        pub fn facts(&self) -> &Facts {
            &self.facts
        }
    }
    pub struct Chunk {
        pub sections: Vec<Arc<Section>>,
        pub min_y: i32,
    }
    impl Chunk {
        pub fn min_y(&self) -> i32 {
            self.min_y
        }
    }
    pub struct World {
        pub chunks: BTreeMap<(i32, i32), Arc<Chunk>>,
    }
    pub struct Layout;
    impl Layout {
        pub fn min_y(&self) -> i32 {
            -64
        }
        pub fn max_y(&self) -> i32 {
            320
        }
    }
    impl World {
        pub fn layout(&self) -> Layout {
            Layout
        }
    }
    pub struct Stack {
        pub item: Arc<str>,
    }
    pub struct ItemInfo {
        pub block: Option<BlockStateId>,
    }
    pub struct Items {
        pub entries: BTreeMap<String, ItemInfo>,
    }
    impl Items {
        pub fn get(&self, id: &str) -> Option<&ItemInfo> {
            self.entries.get(id)
        }
    }
    pub struct Game<'a> {
        pub player: Option<Player>,
        pub world: World,
        pub res: Resources,
        pub selected: usize,
        pub offhand: Option<Stack>,
        pub mainhand: Option<Stack>,
        pub items: Items,
        pub hooks: LiquidBounceHooks,
        client_protection_zones_cache: RefCell<protection_zones::Cache>,
        phantom: PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            Self {
                player: Some(Player {
                    position: Position {
                        x: 22.,
                        y: 64.,
                        z: 0.,
                    },
                }),
                world: World {
                    chunks: BTreeMap::new(),
                },
                res: Resources {
                    facts: Facts {
                        entries: [
                            "minecraft:air",
                            "minecraft:emerald_block",
                            "minecraft:diamond_block",
                        ]
                        .map(|name| Fact { name: name.into() })
                        .into(),
                    },
                },
                selected: 0,
                offhand: None,
                mainhand: None,
                items: Items {
                    entries: [
                        (
                            "minecraft:emerald_block".into(),
                            ItemInfo {
                                block: Some(BlockStateId(1)),
                            },
                        ),
                        (
                            "minecraft:diamond_block".into(),
                            ItemInfo {
                                block: Some(BlockStateId(2)),
                            },
                        ),
                    ]
                    .into(),
                },
                hooks: Default::default(),
                client_protection_zones_cache: Default::default(),
                phantom: PhantomData,
            }
        }
    }
    impl Game<'_> {
        pub fn items(&self) -> &Items {
            &self.items
        }
        pub fn stack_in(&self, _: usize) -> Option<&Stack> {
            self.mainhand.as_ref()
        }
        fn native_client_protection_zones(&self, input: &str, geometry: &mut String) {
            self.hooks.protection_zones(
                &NullHost,
                &mut ClientProtectionZonesEvent { input, geometry },
            );
        }
        pub fn render(&self) -> WorldPrimitives {
            let mut out = WorldPrimitives::default();
            self.client_protection_zones_geometry(&mut out);
            out
        }
        pub fn insert(&mut self, cx: i32, cz: i32, index: usize, id: u32) {
            let mut blocks = crate::paletted::PalettedContainer::new(
                crate::paletted::Strategy {
                    kind: crate::paletted::Kind::Blocks,
                    global_bits: 4,
                },
                0,
            );
            blocks.set(index, id);
            self.world.chunks.insert(
                (cx, cz),
                Arc::new(Chunk {
                    sections: vec![Arc::new(Section { blocks })],
                    min_y: 64,
                }),
            );
        }
        pub fn set(&mut self, cx: i32, cz: i32, index: usize, id: u32) {
            let chunk = Arc::make_mut(self.world.chunks.get_mut(&(cx, cz)).unwrap());
            Arc::make_mut(&mut chunk.sections[0]).blocks.set(index, id);
        }
    }
    impl Clone for Chunk {
        fn clone(&self) -> Self {
            Self {
                sections: self.sections.clone(),
                min_y: self.min_y,
            }
        }
    }
    #[path = "NATIVE_PATH"]
    mod protection_zones;
}
