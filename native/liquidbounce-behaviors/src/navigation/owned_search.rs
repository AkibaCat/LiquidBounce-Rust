// SPDX-License-Identifier: GPL-3.0-or-later
//! One terrain allocation shared by the background lifecycle and its owning search.
use super::{
    background::{BackgroundLevel, SliceSearch},
    geometry::{BlockPos, BlockRegion, Vec3},
    replanner::ReplannerLevel,
    search::Outcome,
};
use std::{cell::RefCell, marker::PhantomData};

pub trait OwnedTerrainSearch<T> {
    type Route;
    fn terrain_mut(&mut self) -> &mut T;
    fn advance(&mut self) -> Outcome<Self::Route>;
    fn preview(&mut self) -> Option<Vec<Vec3>>;
}

enum Stored<T, S> {
    Terrain(T),
    Search(S),
}
pub struct SearchSlot<T, S> {
    state: RefCell<Option<Stored<T, S>>>,
}
impl<T, S: OwnedTerrainSearch<T>> SearchSlot<T, S> {
    pub fn new(terrain: T) -> Self {
        Self {
            state: RefCell::new(Some(Stored::Terrain(terrain))),
        }
    }
    pub fn initialize(&mut self, create: impl FnOnce(T) -> S) -> StoredWorker {
        let Stored::Terrain(terrain) = self
            .state
            .get_mut()
            .take()
            .expect("search slot is occupied")
        else {
            panic!("search slot initializes once")
        };
        *self.state.get_mut() = Some(Stored::Search(create(terrain)));
        StoredWorker
    }
    pub fn terrain_mut(&mut self) -> &mut T {
        match self
            .state
            .get_mut()
            .as_mut()
            .expect("search slot is occupied")
        {
            Stored::Terrain(terrain) => terrain,
            Stored::Search(search) => search.terrain_mut(),
        }
    }
    fn with_search<R>(&self, call: impl FnOnce(&mut S) -> R) -> R {
        let mut state = self.state.borrow_mut();
        let Stored::Search(search) = state.as_mut().expect("search slot is occupied") else {
            panic!("search slot initializes before use")
        };
        call(search)
    }
}
#[derive(Clone, Copy, Default)]
pub struct StoredWorker;
impl<T, S: OwnedTerrainSearch<T>> SliceSearch<SearchSlot<T, S>, S::Route> for StoredWorker {
    fn advance(&mut self, terrain: &mut SearchSlot<T, S>) -> Outcome<S::Route> {
        terrain.with_search(OwnedTerrainSearch::advance)
    }
    fn preview(&self, terrain: &SearchSlot<T, S>) -> Option<Vec<Vec3>> {
        terrain.with_search(OwnedTerrainSearch::preview)
    }
}

pub struct StoredLevel<'a, L, T, S> {
    level: &'a mut L,
    marker: PhantomData<fn(T, S)>,
}
impl<'a, L, T, S> StoredLevel<'a, L, T, S> {
    pub fn new(level: &'a mut L) -> Self {
        Self {
            level,
            marker: PhantomData,
        }
    }
}
impl<L: BackgroundLevel<T>, T, S: OwnedTerrainSearch<T>> BackgroundLevel<SearchSlot<T, S>>
    for StoredLevel<'_, L, T, S>
{
    fn snapshot(&mut self, region: BlockRegion) -> Option<SearchSlot<T, S>> {
        self.level.snapshot(region).map(SearchSlot::new)
    }
    fn has_level(&mut self) -> bool {
        self.level.has_level()
    }
    fn invalidate(&mut self, slot: &mut SearchSlot<T, S>, pos: BlockPos) -> bool {
        self.level.invalidate(slot.terrain_mut(), pos)
    }
}
impl<L: ReplannerLevel<T>, T, S: OwnedTerrainSearch<T>> ReplannerLevel<SearchSlot<T, S>>
    for StoredLevel<'_, L, T, S>
{
    fn is_known(&mut self, pos: BlockPos) -> bool {
        self.level.is_known(pos)
    }
}
