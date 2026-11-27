// SPDX-License-Identifier: GPL-3.0-or-later
use crate::vomit::Constraints;
#[derive(Clone, Debug)]
pub struct BookBot {
    pub constraints: Constraints,
    pub mode: u8,
    pub pages: [i32; 2],
    pub ascii_only: bool,
    pub allow_space: bool,
    pub cyclic: bool,
    pub source: String,
    pub sign: bool,
    pub name: String,
    pub delay: f32,
}
impl Default for BookBot {
    fn default() -> Self {
        Self {
            constraints: Default::default(),
            mode: 0,
            pages: [50; 2],
            ascii_only: false,
            allow_space: true,
            cyclic: true,
            source: String::new(),
            sign: true,
            name: "Generated book #%count%".into(),
            delay: 0.5,
        }
    }
}
