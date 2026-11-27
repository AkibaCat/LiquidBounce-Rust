// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent RouteReport.kt: renderer events deduplicate by original reference identity.
use super::geometry::{BlockPos, Vec3};

/// Stable identity of an owning source object. Allocate monotonically and keep
/// the same value while that object lives; equal contents never imply identity.
/// Zero denotes Kotlin's shared emptyList singleton.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RouteIdentity(pub u64);
impl RouteIdentity { pub const EMPTY_LIST: Self = Self(0); }
pub struct IdentityRef<'a, T: ?Sized> { pub identity: RouteIdentity, pub value: &'a T }
impl<T: ?Sized> Copy for IdentityRef<'_, T> {}
impl<T: ?Sized> Clone for IdentityRef<'_, T> { fn clone(&self) -> Self { *self } }
impl<'a, T: ?Sized> IdentityRef<'a, T> {
 pub fn new(identity: RouteIdentity, value: &'a T) -> Self { Self { identity, value } }
}
/// A host chooses its live progress representation (for example, a stable
/// Follower/TargetProgress handle) and can retain that owner from the callback.
pub trait RouteListener {
 type Progress: ?Sized;
 fn route_searching(&mut self, _start: Vec3, _goal: Vec3, _radius: f64,
   _preview: Option<IdentityRef<'_, [Vec3]>>) {}
 fn route_following(&mut self, _points: IdentityRef<'_, [Vec3]>,
   _progress: IdentityRef<'_, Self::Progress>) {}
 fn blocks_ahead(&mut self, _breaks: IdentityRef<'_, [BlockPos]>,
   _places: IdentityRef<'_, [BlockPos]>) {}
 fn route_done(&mut self) {}
 fn route_invalid(&mut self) {}
 fn route_cleared(&mut self) {}
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shown { Nothing, Searching, Following, Done, Invalid }
pub struct RouteReport {
 shown: Shown,
 start: Option<Vec3>,
 goal: Option<Vec3>,
 radius: f64,
 preview: Option<RouteIdentity>,
 points: Option<RouteIdentity>,
 progress: Option<RouteIdentity>,
 breaks: RouteIdentity,
 places: RouteIdentity,
}
impl Default for RouteReport {
 fn default() -> Self {
  Self { shown: Shown::Nothing, start: None, goal: None, radius: 0.0,
   preview: None, points: None, progress: None,
   breaks: RouteIdentity::EMPTY_LIST, places: RouteIdentity::EMPTY_LIST }
 }
}
impl RouteReport {
 pub fn searching(&mut self, listener: &mut impl RouteListener, start: Vec3,
   goal: Vec3, radius: f64, preview: Option<IdentityRef<'_, [Vec3]>>) {
  let preview_identity = preview.map(|p| p.identity);
  if self.shown == Shown::Searching && self.start == Some(start)
    && self.goal == Some(goal) && self.radius == radius && self.preview == preview_identity { return; }
  self.shown = Shown::Searching;
  self.start = Some(start); self.goal = Some(goal); self.radius = radius;
  self.preview = preview_identity; self.points = None; self.progress = None;
  listener.route_searching(start, goal, radius, preview);
 }
 pub fn following<L: RouteListener>(&mut self, listener: &mut L,
   points: IdentityRef<'_, [Vec3]>, progress: IdentityRef<'_, L::Progress>) {
  if self.shown == Shown::Following && self.points == Some(points.identity)
    && self.progress == Some(progress.identity) { return; }
  self.shown = Shown::Following;
  self.points = Some(points.identity); self.progress = Some(progress.identity);
  listener.route_following(points, progress);
 }
 pub fn blocks(&mut self, listener: &mut impl RouteListener,
   breaks: IdentityRef<'_, [BlockPos]>, places: IdentityRef<'_, [BlockPos]>) {
  if self.breaks == breaks.identity && self.places == places.identity { return; }
  self.breaks = breaks.identity; self.places = places.identity;
  listener.blocks_ahead(breaks, places);
 }
 pub fn done(&mut self, listener: &mut impl RouteListener) {
  if self.shown != Shown::Done { self.shown = Shown::Done; listener.route_done(); }
 }
 pub fn invalid(&mut self, listener: &mut impl RouteListener) {
  if self.shown != Shown::Invalid { self.shown = Shown::Invalid; listener.route_invalid(); }
 }
 pub fn clear(&mut self, listener: &mut impl RouteListener) {
  self.start = None; self.goal = None; self.preview = None;
  self.points = None; self.progress = None;
  self.breaks = RouteIdentity::EMPTY_LIST; self.places = RouteIdentity::EMPTY_LIST;
  if self.shown != Shown::Nothing { self.shown = Shown::Nothing; listener.route_cleared(); }
 }
}
