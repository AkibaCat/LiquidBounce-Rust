// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTargetLock ae0eb396. State survives mode, enable and world changes.
#[path = "navigation/java_case.rs"]
mod java_case;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode { #[default] Temporary, Filter }
impl Mode {
    pub fn tag(self) -> &'static str { match self { Self::Temporary => "Temporary", Self::Filter => "Filter" } }
    pub fn parse(tag: &str) -> Option<Self> { match tag { "Temporary" => Some(Self::Temporary), "Filter" => Some(Self::Filter), _ => None } }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterType { #[default] Whitelist, Blacklist }
impl FilterType {
    pub fn tag(self) -> &'static str { match self { Self::Whitelist => "Whitelist", Self::Blacklist => "Blacklist" } }
    pub fn parse(tag: &str) -> Option<Self> { [Self::Whitelist, Self::Blacklist].into_iter().find(|v| java_case::equals_ignore_case(tag, v.tag())) }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WhenNoLock { #[default] AllowAll, AllowNone }
impl WhenNoLock {
    pub fn tag(self) -> &'static str { match self { Self::AllowAll => "AllowAll", Self::AllowNone => "AllowNone" } }
    pub fn parse(tag: &str) -> Option<Self> { [Self::AllowAll, Self::AllowNone].into_iter().find(|v| java_case::equals_ignore_case(tag, v.tag())) }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub combat_only: bool,
    pub usernames: Vec<Option<String>>,
    pub filter_type: FilterType,
    pub maximum_time: i32,
    pub maximum_range: f32,
    pub when_no_lock: WhenNoLock,
}
impl Default for Settings {
    fn default() -> Self { Self { mode: Mode::Temporary, combat_only: false,
        usernames: vec![Some("Notch".to_owned())], filter_type: FilterType::Whitelist,
        maximum_time: 30, maximum_range: 20.0, when_no_lock: WhenNoLock::AllowAll } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagDecision { Unchanged, DontTarget, Ignore }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notification { LockedOn { name: String, seconds: i32 }, OutOfRange { name: String }, TimeUp { name: String } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player { pub id: i32, pub name: String }
#[derive(Clone, Debug, PartialEq)]
pub struct TrackedPlayer { pub player: Player, pub removed: bool, pub distance_squared: f64 }
/// Adapters borrow state only inside this closure. Synchronous notifications,
/// clock and entity reads therefore run without an engine/module borrow.
pub trait Host {
    fn with_target_lock<R>(&mut self, operation: impl FnOnce(&mut TargetLock) -> R) -> R;
    fn now_millis(&mut self) -> i64;
    fn dead_or_dying(&mut self) -> bool;
    fn player_by_id(&mut self, id: i32) -> Option<TrackedPlayer>;
    fn closer_than(&mut self, entity: &TrackedPlayer, range: f64) -> bool { entity.distance_squared < range * range }
    fn notify(&mut self, notification: Notification);
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TargetLock {
    pub settings: Settings,
    /// Int2LongLinkedOpenHashMap: refreshes keep insertion order.
    pub locks: Vec<(i32, i64)>,
}
impl TargetLock {
    pub fn attack(running: bool, player: Option<&Player>, host: &mut impl Host) {
        if !running || !host.with_target_lock(|s| s.settings.mode == Mode::Temporary) { return; }
        let Some(player) = player else { return };
        let notice = host.with_target_lock(|s| (!s.locks.iter().any(|(id, _)| *id == player.id)).then(|| Notification::LockedOn {
            name: player.name.clone(), seconds: s.settings.maximum_time,
        }));
        if let Some(notice) = notice { host.notify(notice); }
        // Java evaluates notification before System.currentTimeMillis, then the
        // settings getter, then put. Notification callbacks may change settings.
        let now = host.now_millis();
        host.with_target_lock(|s| {
            let expiry = now.wrapping_add(i64::from(s.settings.maximum_time) * 1000);
            if let Some((_, time)) = s.locks.iter_mut().find(|(id, _)| *id == player.id) { *time = expiry; }
            else { s.locks.push((player.id, expiry)); }
        });
    }
    pub fn tick(running: bool, host: &mut impl Host) {
        if !running || !host.with_target_lock(|s| s.settings.mode == Mode::Temporary) { return; }
        if host.dead_or_dying() { host.with_target_lock(|s| s.locks.clear()); return; }
        let now = host.now_millis(); // Exactly once, even when the map is empty.
        let mut index = 0;
        while let Some((id, expiry)) = host.with_target_lock(|s| s.locks.get(index).copied()) {
            let remove = if let Some(entity) = host.player_by_id(id) {
                let outside = entity.removed || {
                    let range = host.with_target_lock(|s| f64::from(s.settings.maximum_range));
                    !host.closer_than(&entity, range)
                };
                if outside { host.notify(Notification::OutOfRange { name: entity.player.name }); true }
                else if expiry < now { host.notify(Notification::TimeUp { name: entity.player.name }); true }
                else { false }
            } else { true };
            // The notification observes the map entry before it is removed.
            if remove { host.with_target_lock(|s| { s.locks.remove(index); }); }
            else { index += 1; }
        }
    }
    pub fn tag(&self, running: bool, player: Option<&Player>) -> TagDecision {
        let Some(player) = player else { return TagDecision::Unchanged; };
        if !running || self.is_locked_on(player) { TagDecision::Unchanged }
        else if self.settings.combat_only { TagDecision::DontTarget } else { TagDecision::Ignore }
    }
    fn is_locked_on(&self, player: &Player) -> bool {
        match self.settings.mode {
            Mode::Filter => {
                let found = self.settings.usernames.iter().any(|name| name.as_ref().is_some_and(|name| java_case::equals_ignore_case(name, &player.name)));
                match self.settings.filter_type { FilterType::Whitelist => found, FilterType::Blacklist => !found }
            }
            Mode::Temporary if self.locks.is_empty() => self.settings.when_no_lock == WhenNoLock::AllowAll,
            Mode::Temporary => self.locks.iter().any(|(id, _)| *id == player.id),
        }
    }
}
