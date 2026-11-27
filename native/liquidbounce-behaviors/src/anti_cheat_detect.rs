// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned AntiCheat.kt evidence and AntiCheatDetect's transaction observer.
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guess {
    pub tag: &'static str,
    pub certainty: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Observation {
    pub transactions: Vec<i32>,
    pub keep_alives: Vec<i64>,
    pub channels: BTreeSet<String>,
    pub capturing: bool,
}
pub fn transaction_id(id: i32) -> i32 {
    if (id as u32) >> 16 == 0x4000 {
        id as i16 as i32
    } else {
        id
    }
}
fn counts_by(ids: &[i32], step: i32) -> bool {
    ids.windows(2).all(|p| p[1].wrapping_sub(p[0]) == step)
}
fn differences(ids: &[i32]) -> Vec<i32> {
    ids.windows(2).map(|p| p[1].wrapping_sub(p[0])).collect()
}
fn fold(c: char) -> char {
    let mut up = c.to_uppercase();
    let first = up.next().unwrap_or(c);
    let u = if up.next().is_none() { first } else { c };
    u.to_lowercase().next().unwrap_or(u)
}
fn suffix(address: &str, suffix: &str) -> bool {
    address.chars().count() >= suffix.chars().count()
        && address
            .chars()
            .rev()
            .map(fold)
            .zip(suffix.chars().rev().map(fold))
            .all(|(a, b)| a == b)
}
impl Observation {
    pub fn login(&mut self) {
        self.transactions.clear();
        self.keep_alives.clear();
        self.capturing = true;
    }
    pub fn disconnect(&mut self) {
        self.transactions.clear();
        self.keep_alives.clear();
        self.channels.clear();
    }
    pub fn ping(&mut self, id: i32) -> bool {
        if !self.capturing {
            return false;
        }
        self.transactions.push(transaction_id(id));
        if self.transactions.len() >= 5 {
            self.capturing = false;
            true
        } else {
            false
        }
    }
    pub fn keep_alive(&mut self, id: i64) {
        if self.keep_alives.len() < 5 {
            self.keep_alives.push(id);
        }
    }
    pub fn channels(&mut self, register: bool, data: &[u8]) {
        // Identifier.tryParse used by Fabric rejects invalid namespace/path.
        for raw in String::from_utf8_lossy(data)
            .split('\0')
            .filter(|s| !s.is_empty())
        {
            let (namespace, path) = raw.split_once(':').unwrap_or(("minecraft", raw));
            if path.is_empty()
                || !namespace
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c))
                || !path
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"/_.-".contains(&c))
            {
                continue;
            }
            let channel = format!(
                "{}:{path}",
                if namespace.is_empty() {
                    "minecraft"
                } else {
                    namespace
                }
            );
            if register {
                self.channels.insert(channel);
            } else {
                self.channels.remove(&channel);
            }
        }
    }
    pub fn guess(&self, address: Option<&str>) -> Option<Guess> {
        let has = |namespace: &str| {
            self.channels
                .iter()
                .any(|c| c.split_once(':').is_some_and(|(n, _)| n == namespace))
        };
        let claimed = usize::from(has("intave")) + usize::from(has("matrix"));
        let channel = |n: &str| claimed <= 1 && has(n);
        let ids = &self.transactions;
        let ready = ids.len() >= 5;
        let tx = |f: &dyn Fn(&[i32]) -> bool| ready && f(ids);
        let steady = |step: i32, min: i32, max: i32| {
            ready && (min..=max).contains(&ids[0]) && counts_by(ids, step)
        };
        let mut best: Option<Guess> = None;
        let mut evidence = |tag: &'static str, pieces: &[(bool, i32)]| {
            let doubt = pieces
                .iter()
                .filter(|(ok, _)| *ok)
                .fold(1.0, |doubt, (_, certainty)| {
                    doubt * (1.0 - f64::from(*certainty) / 100.0)
                });
            if doubt < 1.0 {
                let certainty = ((1.0 - doubt) * 100.0) as i32;
                if best.as_ref().is_none_or(|b| certainty > b.certainty) {
                    best = Some(Guess { tag, certainty });
                }
            }
        };
        evidence(
            "Watchdog",
            &[(address.is_some_and(|a| suffix(a, "hypixel.net")), 100)],
        );
        evidence("Grim", &[(steady(-1, 0, 0), 75), (steady(-1, -5, -1), 40)]);
        evidence(
            "Intave",
            &[
                (channel("intave"), 95),
                (
                    tx(&|i| {
                        i.iter().all(|v| *v < i16::MIN as i32)
                            && differences(i).into_iter().collect::<BTreeSet<_>>().len() > 1
                    }),
                    85,
                ),
                (steady(1, -23767, -23762), 85),
                (steady(-1, i32::MIN, -3001), 50),
                (tx(&|i| i[0] < -3000 && i.contains(&0)), 50),
            ],
        );
        evidence(
            "Vulcan",
            &[(
                tx(&|i| {
                    i[1].wrapping_sub(i[0]) != 1
                        && (i16::MIN as i32..=-32700).contains(&i[1])
                        && counts_by(&i[1..], 1)
                }),
                90,
            )],
        );
        evidence(
            "Matrix",
            &[
                (channel("matrix"), 95),
                (steady(1, 95, 105), 60),
                (steady(1, -20005, -19995), 60),
                (
                    tx(&|i| {
                        let d = differences(i);
                        i.iter().all(|v| (i16::MIN as i32..=-1).contains(v))
                            && d.iter().copied().collect::<BTreeSet<_>>().len() == d.len()
                    }),
                    50,
                ),
            ],
        );
        evidence(
            "Karhu",
            &[
                (
                    tx(&|i| {
                        i[0] == i16::MIN as i32 + 1
                            && (-3000..=-2995).contains(&i[1])
                            && counts_by(&i[1..], -1)
                    }),
                    90,
                ),
                (steady(-1, -3000, -2995), 60),
            ],
        );
        evidence(
            "Verus",
            &[
                (tx(&|i| i[0] == i[1] && counts_by(&i[2..], 1)), 60),
                (steady(1, i32::MIN, i32::MAX), 60),
            ],
        );
        evidence(
            "Polar",
            &[
                (
                    tx(&|i| {
                        let d = differences(i);
                        d[0] >= 100 && d[1..].iter().all(|v| *v == -1)
                    }),
                    60,
                ),
                (steady(-1, i32::MIN, i32::MAX), 30),
                (
                    self.keep_alives
                        .iter()
                        .filter(|v| (i16::MIN as i64..=-1).contains(v))
                        .count()
                        >= 2,
                    90,
                ),
            ],
        );
        best
    }
}
