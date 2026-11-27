// SPDX-License-Identifier: GPL-3.0-or-later
use crate::tests::{World, oracle, shape};
use horizon_core::{geometry::{Vec3, VoxelShape}, position::BlockPos};
use horizon_world::{registry::BlockStateId, world_ray::{RayBlockMode, RayFluidMode, RayWorld}};
use std::{collections::BTreeMap, sync::{LazyLock, Mutex}};

static SETTINGS: Mutex<(bool, Vec<String>)> = Mutex::new((false, Vec::new()));
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[allow(dead_code)]
pub mod mod_api {
    use super::*;
    pub trait RenderHooks: Send + Sync {}
    #[allow(unused_imports)] // The real empty Host trait has no helper type references.
    mod host { include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs")); }
    pub use host::*;
    mod generated { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use generated::*;
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(Box::as_ref)
    }
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: LazyLock<Vec<Box<dyn Hooks>>> = LazyLock::new(||vec![Box::new(Adapter)]);
        &MODS
    }
    struct Adapter;
    impl Hooks for Adapter {
        fn client_ghost_hand(&self, _: &dyn Host, event: &mut ClientGhostHandEvent) {
            let settings = SETTINGS.lock().unwrap();
            event.running = settings.0;
            event.targeted_blocks.clone_from(&settings.1);
        }
    }
}

#[derive(Clone,Copy,Debug)]
struct Hit { pos:[i32;3], face:[i32;3], point:[f64;3] }
struct Fact { name:Box<str> }
struct Facts(Vec<Fact>);
impl Facts { fn get(&self,state:BlockStateId)->&Fact {&self.0[state.0 as usize]} }
struct Res(Facts);
impl Res { fn facts(&self)->&Facts {&self.0} }
struct Cells(BTreeMap<[i32;3],usize>);
impl Cells { fn block(&self,x:i32,y:i32,z:i32)->BlockStateId {BlockStateId(*self.0.get(&[x,y,z]).unwrap_or(&0) as u32)} }
pub struct Game<'a> { world:Cells,res:Res,marker:std::marker::PhantomData<&'a ()> }
impl Game<'_> {
    fn new()->Self {
        Self {world:Cells(BTreeMap::from([([1,0,0],1),([3,0,0],2)])),
            res:Res(Facts(oracle()["states"].as_array().unwrap().iter().map(|s|Fact{name:s["id"].as_str().unwrap().into()}).collect())),
            marker:std::marker::PhantomData}
    }
    // Outline storage shell uses actual MC shapes and actual clipWithInteractionOverride.
    fn clip_declared_outline(&self,state:BlockStateId,pos:[i32;3],origin:[f64;3],dir:[f64;3],reach:f64)->Option<Hit> {
        let row=&oracle()["states"][state.0 as usize];
        let outline=shape(&row["shapes"]["OUTLINE"]);
        let interaction=shape(&row["interaction"]);
        let start=Vec3::new(origin[0],origin[1],origin[2]);
        let end=start.add_vector(Vec3::new(dir[0],dir[1],dir[2]).scale(reach));
        let hit=horizon_world::interaction_shapes::clip_with_interaction_override(outline.boxes().iter().copied(),
            interaction.boxes().iter().copied(),start,end,BlockPos::new(pos[0],pos[1],pos[2]))?;
        Some(Hit {pos,face:hit.direction.normal(),point:[hit.location.x,hit.location.y,hit.location.z]})
    }
}
mod native_hooks {
    use super::*;
    impl Game<'_> { __NATIVE__ }
}
mod generated_host {include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}
#[path="native-game-ray.rs"]mod client_ghost_hand;

struct BoundWorld(World);
impl RayWorld for BoundWorld {
    type Error=&'static str;type Block=BlockStateId;type Fluid=usize;
    fn block_state(&mut self,p:BlockPos)->Result<BlockStateId,Self::Error> {self.0.block_state(p).map(|s|BlockStateId(s as u32))}
    fn fluid_state(&mut self,p:BlockPos)->Result<usize,Self::Error> {self.0.fluid_state(p)}
    fn block_shape(&mut self,s:BlockStateId,p:BlockPos,m:RayBlockMode)->Result<VoxelShape,Self::Error> {self.0.block_shape(s.0 as usize,p,m)}
    fn interaction_shape(&mut self,s:BlockStateId,p:BlockPos)->Result<VoxelShape,Self::Error> {self.0.interaction_shape(s.0 as usize,p)}
    fn fluid_is_source(&mut self,s:usize)->Result<bool,Self::Error> {self.0.fluid_is_source(s)}
    fn fluid_is_empty(&mut self,s:usize)->Result<bool,Self::Error> {self.0.fluid_is_empty(s)}
    fn fluid_is_water(&mut self,s:usize)->Result<bool,Self::Error> {self.0.fluid_is_water(s)}
    fn fluid_shape(&mut self,s:usize,p:BlockPos)->Result<VoxelShape,Self::Error> {self.0.fluid_shape(s,p)}
}

#[test]
fn actual_generated_hook_and_game_adapter_deliver_current_target_set() {
    let _guard=TEST_LOCK.lock().unwrap();
    let game=Game::new();
    let mut provider=BoundWorld(World {cells:game.world.0.clone(),..World::default()});
    for (running,targets,expected) in [(true,vec!["minecraft:chest".into()],3),
        (false,vec!["minecraft:chest".into()],1),(true,Vec::new(),1),(true,vec!["minecraft:stone".into()],1)] {
        *SETTINGS.lock().unwrap()=(running,targets);
        let hit=client_ghost_hand::clip(&game,&mut provider,Vec3::new(0.5,0.5,0.5),Vec3::new(4.5,0.5,0.5),
            RayBlockMode::Outline,RayFluidMode::Any,20).unwrap();
        assert_eq!(hit.hit.block.x,expected);
    }
}

#[test]
fn actual_native_outline_entry_respects_targets_without_player_or_screen_guards() {
    let _guard=TEST_LOCK.lock().unwrap();
    let game=Game::new();
    *SETTINGS.lock().unwrap()=(true,vec!["minecraft:chest".into()]);
    let hit=game.client_ghost_hand_outline([0.5,0.5,0.5],[1.,0.,0.],4.).unwrap();
    assert_eq!(hit.pos,[3,0,0]);assert_eq!(hit.face,[-1,0,0]);assert_eq!(hit.point,[3.0625,0.5,0.5]);
    *SETTINGS.lock().unwrap()=(true,vec!["minecraft:water".into()]);
    assert!(game.client_ghost_hand_outline([0.5,0.5,0.5],[1.,0.,0.],4.).is_none());
    *SETTINGS.lock().unwrap()=(false,vec!["minecraft:chest".into()]);
    assert!(game.client_ghost_hand_outline([0.5,0.5,0.5],[1.,0.,0.],4.).is_none());
}
