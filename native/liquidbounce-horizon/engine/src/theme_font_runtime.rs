// SPDX-License-Identifier: GPL-3.0-or-later
//! The real browser theme root is the authority for FontManager.nativeFont.
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};
type Stamp = Vec<(PathBuf, Option<(SystemTime, u64)>)>;
pub(crate) struct ThemeFontMonitor {
    root: PathBuf,
    stamp: Stamp,
    last: Instant,
}
fn stamp(root: &Path) -> Stamp {
    let metadata = root.join("metadata.json");
    let mut paths = vec![metadata.clone()];
    if let Ok(bytes) = std::fs::read(metadata) {
        if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            paths.extend(
                data["fonts"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(serde_json::Value::as_str)
                    .map(|p| root.join("fonts").join(p)),
            );
        }
    }
    paths
        .into_iter()
        .map(|path| {
            let state = std::fs::metadata(&path)
                .ok()
                .and_then(|m| Some((m.modified().ok()?, m.len())));
            (path, state)
        })
        .collect()
}
impl ThemeFontMonitor {
    pub(crate) fn new(root: PathBuf) -> Self {
        let mut monitor = Self {
            root,
            stamp: Vec::new(),
            last: Instant::now(),
        };
        monitor.reload();
        monitor
    }
    pub(crate) fn reload(&mut self) {
        self.stamp = stamp(&self.root);
        self.last = Instant::now();
        if let Err(e) = horizon_client::font::liquidbounce_load_theme_fonts(&self.root) {
            eprintln!("[LiquidBounce] Theme native font reload: {e}");
        }
    }
    pub(crate) fn refresh(&mut self) {
        if self.last.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.last = Instant::now();
        if stamp(&self.root) != self.stamp {
            self.reload();
        }
    }
}
