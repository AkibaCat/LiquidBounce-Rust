// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::target_lock::{Host as LockHost, Notification, Player, TargetLock, TrackedPlayer};
use liquidbounce_behaviors::targeting::TargetInfo;

pub(crate) fn player_snapshot(json: &str) -> Option<TrackedPlayer> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    Some(TrackedPlayer { player: Player { id: value["id"].as_i64()?.try_into().ok()?, name: value["name"].as_str()?.to_owned() },
        removed: value["removed"].as_bool()?, distance_squared: f64::from_bits(value["distance_squared_bits"].as_str()?.parse().ok()?) })
}
/// Called only from TargetInfo's compute-if-absent closure, in original module
/// registration order after FriendManager. Never invalidates the shared cache.
pub(crate) fn apply_tag(module: &TargetLock, running: bool, player: Option<&Player>, info: &mut TargetInfo) {
    use liquidbounce_behaviors::target_lock::TagDecision;
    match module.tag(running, player) { TagDecision::Unchanged => {}, TagDecision::DontTarget => info.dont_target(), TagDecision::Ignore => info.ignore() }
}
#[cfg(feature="webview")]
fn message(notification: &Notification, language: &str) -> String {
    let language=language.to_ascii_lowercase().replace('-', "_");
    let sc=matches!(language.as_str(),"zh_cn"|"zh_sg"|"zh_hans");
    let tc=matches!(language.as_str(),"zh_tw"|"zh_hk"|"zh_mo"|"zh_hant");
    match notification {
        Notification::LockedOn {name,seconds} if sc => format!("锁定{name}，持续{seconds}秒。"),
        Notification::LockedOn {name,seconds} if tc => format!("鎖定{name}，持續{seconds}秒。"),
        Notification::LockedOn {name,seconds} => format!("Locked on {name} for {seconds} seconds."),
        Notification::OutOfRange {name} if sc => format!("目标已超出范围，不再锁定{name}。"),
        Notification::OutOfRange {name} if tc => format!("目標已超出範圍，不再鎖定{name}。"),
        Notification::OutOfRange {name} => format!("No longer locked on {name} because they are out of range."),
        Notification::TimeUp {name} if sc => format!("时间已到，不再锁定{name}。"),
        Notification::TimeUp {name} if tc => format!("時間已到，不再鎖定{name}。"),
        Notification::TimeUp {name} => format!("No longer locked on {name} because the time is up."),
    }
}
struct Adapter<'a> { hooks: &'a LiquidBounceHooks, host: &'a mut dyn Host }
impl LockHost for Adapter<'_> {
    fn with_target_lock<R>(&mut self, operation: impl FnOnce(&mut TargetLock) -> R) -> R {
        let state=self.hooks.state(); let mut modules=state.behaviors.borrow_mut(); operation(&mut modules.target_lock)
    }
    fn now_millis(&mut self) -> i64 {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(time) => time.as_millis() as i64,
            Err(error) => (-(error.duration().as_nanos() as i128)).div_euclid(1_000_000) as i64,
        }
    }
    fn dead_or_dying(&mut self) -> bool { self.host.client_target_lock_dead().unwrap_or(false) }
    fn player_by_id(&mut self,id:i32) -> Option<TrackedPlayer> { player_snapshot(&self.host.client_target_lock_player(id)) }
    fn notify(&mut self, notification: Notification) {
        #[cfg(feature="webview")]
        self.hooks.state().events.borrow_mut().push(("notification".into(),serde_json::json!({"title":"TargetLock","message":message(&notification,&horizon_client::lang::code()),"severity":"INFO"})));
        #[cfg(not(feature="webview"))]
        let _=notification;
    }
}
impl LiquidBounceHooks {
    pub(crate) fn target_lock_attack(&self, host: &mut dyn Host, target: i32) {
        let running={let state=self.state();let modules=state.behaviors.borrow();modules.query_running(&state.registry,"TargetLock")};
        if !running {return;}
        let player=player_snapshot(&host.client_target_lock_player(target));
        TargetLock::attack(running,player.as_ref().map(|v|&v.player),&mut Adapter{hooks:self,host});
    }
    pub(crate) fn target_lock_tick(&self,host:&mut dyn Host) {
        let running={let state=self.state();let modules=state.behaviors.borrow();modules.query_running(&state.registry,"TargetLock")};
        TargetLock::tick(running,&mut Adapter{hooks:self,host});
    }
}
