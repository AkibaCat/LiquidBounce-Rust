// SPDX-License-Identifier: GPL-3.0-or-later
//! LiquidBounce ae0eb396 ModuleAntiStaff and TextExtensions address semantics.
use std::collections::{BTreeMap, BTreeSet};

/// Guava HostAndPort: bracketless IPv6 has no port; malformed ports leave input intact.
pub fn drop_port(input: &str) -> &str {
    if let Some(rest) = input.strip_prefix('[') {
        if let Some(end) = rest.rfind(']') {
            let host = &rest[..end];
            let suffix = &rest[end + 1..];
            if host.contains(':')
                && (suffix.is_empty() || suffix.strip_prefix(':').is_some_and(valid_port))
            {
                return host;
            }
        }
        return input;
    }
    if input.bytes().filter(|b| *b == b':').count() == 1 {
        let (host, port) = input.split_once(':').unwrap();
        if valid_port(port) {
            return host;
        }
    }
    input
}
fn valid_port(port: &str) -> bool {
    port.is_empty()
        || (!port.starts_with('+')
            && port.bytes().all(|b| b.is_ascii_digit())
            && port.parse::<u16>().is_ok())
}
pub fn root_domain(input: &str) -> String {
    let mut domain = input.trim().to_lowercase();
    let ip = domain
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(&domain)
        .split('%')
        .next()
        .unwrap_or("");
    let ipv6 = domain.contains(':') && ip.parse::<std::net::Ipv6Addr>().is_ok();
    let parts: Vec<_> = domain.split('.').collect();
    let ipv4 = parts.len() == 4
        && parts
            .iter()
            .all(|s| (1..=3).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit()));
    if ipv6 || ipv4 {
        return domain;
    }
    if domain.ends_with('.') {
        domain.pop();
    }
    let parts: Vec<_> = domain.split('.').collect();
    if parts.len() > 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        domain
    }
}
pub fn address(input: &str) -> String {
    root_domain(drop_port(input))
}
pub fn staff_lines(text: &str) -> BTreeSet<String> {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .split('\n')
        .map(str::to_owned)
        .collect()
}

pub struct State {
    pub show_in_tab_list: bool,
    enabled: bool,
    running: bool,
    enable_pending: bool,
    current: Option<String>,
    pending: Option<String>,
    pub generation: u64,
    cache: BTreeMap<String, BTreeSet<String>>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            show_in_tab_list: true,
            enabled: false,
            running: false,
            enable_pending: false,
            current: None,
            pending: None,
            generation: 0,
            cache: BTreeMap::new(),
        }
    }
}
impl State {
    pub fn sync(&mut self, enabled: bool, running: bool) {
        if enabled != self.enabled {
            self.generation = self.generation.wrapping_add(1);
            self.enable_pending = enabled;
            if !enabled {
                self.pending = None;
            }
        }
        self.enabled = enabled;
        self.running = running;
    }
    fn reserve(&mut self, domain: &str) -> bool {
        if self.cache.contains_key(domain) {
            return false;
        }
        self.cache.insert(domain.into(), BTreeSet::new());
        true
    }
    pub fn frame(
        &mut self,
        server: Option<&str>,
        in_game: bool,
        screen_open: bool,
    ) -> Option<String> {
        let next = server.map(address);
        let changed = self.current != next;
        self.current = next.clone();
        if !self.enabled {
            return None;
        }
        if std::mem::take(&mut self.enable_pending) {
            if let Some(domain) = next.as_deref() {
                if self.reserve(domain) {
                    return Some(domain.into());
                }
            }
        } else if changed {
            self.pending = None;
            if let Some(domain) = next.as_deref() {
                if self.reserve(domain) {
                    self.pending = Some(domain.into());
                }
            }
        }
        if in_game && screen_open {
            return self.pending.take();
        }
        None
    }
    pub fn accept(&mut self, generation: u64, domain: &str, staffs: BTreeSet<String>) -> bool {
        if !self.enabled || generation != self.generation {
            return false;
        }
        self.cache.insert(domain.into(), staffs);
        true
    }
    pub fn is_staff(&self, name: &str) -> bool {
        self.running
            && self.enabled
            && self
                .current
                .as_ref()
                .and_then(|s| self.cache.get(s))
                .is_some_and(|s| s.contains(name))
    }
    pub fn tab_staff(&self) -> Vec<String> {
        if !self.enabled || !self.running || !self.show_in_tab_list {
            return vec![];
        }
        self.current
            .as_ref()
            .and_then(|d| self.cache.get(d))
            .into_iter()
            .flatten()
            .cloned()
            .collect()
    }
}
