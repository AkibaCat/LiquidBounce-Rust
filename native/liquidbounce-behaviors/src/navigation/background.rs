// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac route/BackgroundSearch.kt. The level stays on the
//! caller thread; each worker owns its snapshot and search until a slice ends.
use super::{
    geometry::{BlockPos, BlockRegion, Vec3},
    search::{Failure, Outcome},
};
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        mpsc::{self, Receiver, TryRecvError},
        Arc,
    },
};

pub trait SliceSearch<T, R> {
    fn advance(&mut self, terrain: &mut T) -> Outcome<R>;
    fn preview(&self, _terrain: &T) -> Option<Vec<Vec3>> {
        None
    }
}

pub trait BackgroundLevel<T> {
    /// Takes the region copy on the calling game thread, or reports no level.
    fn snapshot(&mut self, region: BlockRegion) -> Option<T>;
    fn has_level(&mut self) -> bool;
    /// Reads the live block state and passes it to the snapshot terrain cache.
    fn invalidate(&mut self, terrain: &mut T, pos: BlockPos) -> bool;
}

pub trait Executor: Send + Sync {
    fn execute(&self, task: Box<dyn FnOnce() + Send>);
}

#[derive(Clone, Copy, Default)]
pub struct ThreadExecutor;
impl Executor for ThreadExecutor {
    fn execute(&self, task: Box<dyn FnOnce() + Send>) {
        std::thread::spawn(task);
    }
}

pub type SearchFactory<T, S> = Arc<dyn Fn(&mut T) -> S + Send + Sync>;
struct SearchState<T, S> {
    terrain: T,
    search: S,
}
struct Slice<T, S, R> {
    result: Receiver<(SearchState<T, S>, Outcome<R>)>,
}
struct Job<K, T, S, R> {
    key: K,
    region: BlockRegion,
    new_search: SearchFactory<T, S>,
    state: Option<SearchState<T, S>>,
    slice: Option<Slice<T, S, R>>,
    result: Outcome<R>,
    polls: i32,
}

pub struct BackgroundSearch<K, T, S, R> {
    executor: Arc<dyn Executor>,
    job: Option<Job<K, T, S, R>>,
    changes: Vec<BlockPos>,
    changes_overflowed: bool,
    preview: Option<Vec<Vec3>>,
}

impl<
        K: PartialEq + Clone,
        T: Send + 'static,
        S: SliceSearch<T, R> + Send + 'static,
        R: Clone + Send + 'static,
    > BackgroundSearch<K, T, S, R>
{
    pub fn new(executor: Arc<dyn Executor>) -> Self {
        Self {
            executor,
            job: None,
            changes: Vec::new(),
            changes_overflowed: false,
            preview: None,
        }
    }
    pub fn is_idle(&self) -> bool {
        self.job.is_none()
    }
    pub fn preview(&self) -> Option<&[Vec3]> {
        self.preview.as_deref()
    }

    pub fn request(
        &mut self,
        level: &mut impl BackgroundLevel<T>,
        key: K,
        region: BlockRegion,
        new_search: SearchFactory<T, S>,
    ) {
        if self.job.as_ref().is_some_and(|job| job.key == key) {
            return;
        }
        self.cancel();
        self.job = Self::start(level, key, region, new_search);
    }

    pub fn poll(&mut self, level: &mut impl BackgroundLevel<T>) -> Outcome<R> {
        let Some(mut job) = self.job.take() else {
            return Outcome::Searching;
        };
        job.polls = job.polls.wrapping_add(1);
        if let Some(slice) = job.slice.take() {
            match slice.result.try_recv() {
                Ok((state, result)) => {
                    job.state = Some(state);
                    job.result = result;
                    if matches!(job.result, Outcome::Searching) {
                        let state = job.state.as_ref().expect("completed slice retains search");
                        if let Some(preview) = state.search.preview(&state.terrain) {
                            self.preview = Some(preview);
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    job.slice = Some(slice);
                    self.job = Some(job);
                    return Outcome::Searching;
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("route executor discarded an uncanceled slice")
                }
            }
        }
        if job.polls >= 20
            && self.apply_changes(
                level,
                &mut job.state.as_mut().expect("idle search has terrain").terrain,
            )
        {
            let Some(restarted) = Self::start(level, job.key, job.region, job.new_search) else {
                self.cancel();
                return Outcome::Searching;
            };
            job = restarted;
        }
        if matches!(job.result, Outcome::Searching) {
            let mut state = job.state.take().expect("one slice per search");
            let (sender, result) = mpsc::channel();
            self.executor.execute(Box::new(move || {
                // The source cancels the dependent exceptionally future. Its upstream
                // supplyAsync still runs, including work that was only queued.
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    state.search.advance(&mut state.terrain)
                }))
                .unwrap_or(Outcome::Failed(Failure::NoPath));
                let _ = sender.send((state, outcome));
            }));
            job.slice = Some(Slice { result });
        }
        let result = job.result.clone();
        self.job = Some(job);
        result
    }

    pub fn cancel(&mut self) {
        self.job = None;
        self.preview = None;
        self.changes.clear();
        self.changes_overflowed = false;
    }
    pub fn level_changed(&mut self) {
        self.cancel();
    }
    pub fn block_changed(&mut self, pos: BlockPos) {
        let Some(job) = self.job.as_ref() else {
            return;
        };
        if self.changes_overflowed || !job.region.contains(pos) {
            return;
        }
        if self.changes.len() >= 1024 {
            self.changes_overflowed = true;
            self.changes.clear();
        } else {
            self.changes.push(pos);
        }
    }

    fn start(
        level: &mut impl BackgroundLevel<T>,
        key: K,
        region: BlockRegion,
        new_search: SearchFactory<T, S>,
    ) -> Option<Job<K, T, S, R>> {
        let mut terrain = level.snapshot(region)?;
        let search = new_search(&mut terrain);
        Some(Job {
            key,
            region,
            new_search,
            state: Some(SearchState { terrain, search }),
            slice: None,
            result: Outcome::Searching,
            polls: 0,
        })
    }
    fn apply_changes(&mut self, level: &mut impl BackgroundLevel<T>, terrain: &mut T) -> bool {
        if !level.has_level() {
            return false;
        }
        let mut affected = self.changes_overflowed;
        for &pos in &self.changes {
            affected = level.invalidate(terrain, pos) || affected;
        }
        self.changes.clear();
        self.changes_overflowed = false;
        affected
    }
}
