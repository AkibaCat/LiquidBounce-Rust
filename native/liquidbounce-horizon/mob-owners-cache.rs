// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
include!("mob_owners_types.rs");
pub const LOADING: &str = "§oLoading...";
pub const FAILED: &str = "§c§oFailed to query Mojang API";
pub const CANCELED: &str = "§e§oQuery is canceled";
static CACHE: OnceLock<Mutex<HashMap<u128, String>>> = OnceLock::new();
static REMOTE: OnceLock<Mutex<HashMap<u32, u32>>> = OnceLock::new();
#[derive(Clone)]
pub struct Label {
    pub owner: u128,
    pub at: [f64; 3],
    pub height: f32,
    pub light: u32,
}
thread_local! {static FRAME:RefCell<(u8,Vec<Label>)>=const{RefCell::new((0,Vec::new()))};}
pub fn begin(mask: u8) {
    FRAME.with(|f| {
        let mut f = f.borrow_mut();
        f.0 = mask;
        f.1.clear();
    });
}
pub fn enabled(projectile: bool) -> bool {
    FRAME.with(|f| f.borrow().0 & if projectile { 2 } else { 1 } != 0)
}
pub fn record(label: Label) {
    FRAME.with(|f| f.borrow_mut().1.push(label));
}
pub fn labels() -> Vec<Label> {
    FRAME.with(|f| std::mem::take(&mut f.borrow_mut().1))
}
pub fn remember(entity: u32, owner: u32) {
    REMOTE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(entity, owner);
}
pub fn forget(entity: u32) {
    if let Some(m) = REMOTE.get() {
        m.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&entity);
    }
}
pub fn clear_remote() {
    if let Some(m) = REMOTE.get() {
        m.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}
pub fn remote_owner(entity: u32) -> Option<u32> {
    REMOTE
        .get()?
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&entity)
        .copied()
}
pub fn cached(owner: u128) -> String {
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(v) = cache.get(&owner) {
        return v.clone();
    }
    cache.insert(owner, LOADING.into());
    drop(cache);
    // The lookup persists across module disable, just like eventListenerScope.
    let result = std::thread::Builder::new()
        .name("LiquidBounce MobOwners".into())
        .spawn(move || {
            let response = ureq::get(&format!(
                "https://api.minecraftservices.com/minecraft/profile/lookup/{owner:032x}"
            ))
            .call()
            .ok()
            .and_then(|r| r.into_string().ok())
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["name"].as_str().map(str::to_owned))
            .unwrap_or_else(|| FAILED.into());
            CACHE
                .get()
                .unwrap()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(owner, response);
        });
    if result.is_err() {
        CACHE
            .get()
            .unwrap()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(owner, FAILED.into());
    }
    LOADING.into()
}
#[cfg(test)]
mod tests {
    #[test]
    fn cached_profile_survives_disable_without_a_request() {
        super::CACHE
            .get_or_init(Default::default)
            .lock()
            .unwrap()
            .insert(0x534d4f4b45, "CachedProfile".into());
        assert_eq!(super::cached(0x534d4f4b45), "CachedProfile");
        super::begin(0);
        assert_eq!(super::cached(0x534d4f4b45), "CachedProfile");
    }
}
