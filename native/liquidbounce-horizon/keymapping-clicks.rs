// SPDX-License-Identifier: GPL-3.0-or-later
//! KeyMapping.click / consumeClick queues. A click never changes held or toggle state.
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct ClickQueues {
    clicks: BTreeMap<String, i32>,
}

impl ClickQueues {
    /// Resolve the bound key now. Rebinding later cannot retarget these clicks.
    /// The unknown-key exclusion is LiquidBounce's MixinKeyMapping fix.
    pub fn click<'a>(&mut self, bound_key: &str, mappings: impl IntoIterator<Item = &'a str>) {
        if bound_key == "key.keyboard.unknown" {
            return;
        }
        for mapping in mappings {
            let count = self.clicks.entry(mapping.to_owned()).or_default();
            *count = count.wrapping_add(1);
        }
    }

    pub fn consume(&mut self, mapping: &str) -> bool {
        let Some(count) = self.clicks.get_mut(mapping) else { return false; };
        if *count == 0 {
            return false;
        }
        *count = count.wrapping_sub(1);
        true
    }

    pub fn pending(&self, mapping: &str) -> i32 {
        self.clicks.get(mapping).copied().unwrap_or_default()
    }

    pub fn release_all(&mut self) {
        self.clicks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_keeps_mapping_identity_and_unknown_is_excluded() {
        let mut clicks = ClickQueues::default();
        clicks.click("key.mouse.left", ["key.attack", "key.use", "key.forward", "key.screenshot"]);
        clicks.click("key.mouse.right", ["key.attack"]);
        assert_eq!(clicks.pending("key.attack"), 2);
        assert_eq!(clicks.pending("key.use"), 1);
        assert!(clicks.consume("key.attack"));
        assert!(clicks.consume("key.attack"));
        assert!(!clicks.consume("key.attack"));
        // An unconsumed movement/screenshot click remains queued; it is not a physical press.
        assert_eq!(clicks.pending("key.forward"), 1);
        assert_eq!(clicks.pending("key.screenshot"), 1);
        clicks.click("key.keyboard.unknown", ["key.attack", "key.use"]);
        assert_eq!(clicks.pending("key.attack"), 0);
        clicks.release_all();
        assert_eq!(clicks.pending("key.use"), 0);
    }

    #[test]
    fn java_click_count_wraps_at_signed_integer_bounds() {
        let mut clicks = ClickQueues::default();
        clicks.clicks.insert("key.attack".into(), i32::MAX);
        clicks.click("key.mouse.left", ["key.attack"]);
        assert_eq!(clicks.pending("key.attack"), i32::MIN);
        assert!(clicks.consume("key.attack"));
        assert_eq!(clicks.pending("key.attack"), i32::MAX);
    }
}
