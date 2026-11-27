// SPDX-License-Identifier: GPL-3.0-or-later
//! Original notification WebSocket payloads; retain the language at emission.
use liquidbounce_behaviors::anti_hunger::Warning;
use serde_json::{Value, json};
use std::sync::Mutex;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingNotification {
    pub warning: Warning,
    pub language: String,
}
#[derive(Default)]
pub struct NotificationQueue {
    pending: Mutex<Vec<PendingNotification>>,
}
impl NotificationQueue {
    pub fn publish(&self, warning: Warning, language: String) {
        self.pending
            .lock()
            .expect("AntiHunger notifications poisoned")
            .push(PendingNotification { warning, language });
    }
    pub fn drain(&self, resolve: &dyn Fn(&str, &str) -> String) -> Vec<(String, Value)> {
        let pending = std::mem::take(
            &mut *self
                .pending
                .lock()
                .expect("AntiHunger notifications poisoned"),
        );
        pending.into_iter().map(|n|("notification".into(),json!({
            "title":"AntiHunger","message":resolve(&n.language,n.warning.message_key()),"severity":"INFO",
        }))).collect()
    }
}
