// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleBetterTab, LiquidBounce ae0eb396 (CCBlueX, 2015-2026).
use java_regex::{PatternSyntaxError, Regex};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sorting {
    #[default]
    Vanilla,
    Ping,
    NameLength,
    DisplayNameLength,
    Alphabetical,
    ReverseAlphabetical,
    None,
}
pub fn tag_eq(a: &str, b: &str) -> bool {
    fn fold(c: char) -> char {
        let mut up = c.to_uppercase();
        let first = up.next().unwrap_or(c);
        if up.next().is_none() {
            first.to_lowercase().next().unwrap_or(first)
        } else {
            c
        }
    }
    a.chars().map(fold).eq(b.chars().map(fold))
}
impl Sorting {
    pub const ALL: [Self; 7] = [
        Self::Vanilla,
        Self::Ping,
        Self::NameLength,
        Self::DisplayNameLength,
        Self::Alphabetical,
        Self::ReverseAlphabetical,
        Self::None,
    ];
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Ping => "Ping",
            Self::NameLength => "NameLength",
            Self::DisplayNameLength => "DisplayNameLength",
            Self::Alphabetical => "Alphabetical",
            Self::ReverseAlphabetical => "ReverseAlphabetical",
            Self::None => "None",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| tag_eq(v.tag(), s))
    }
}
#[derive(Clone, Debug)]
pub struct Pattern(Regex);
impl Pattern {
    pub fn new(s: &str) -> Result<Self, PatternSyntaxError> {
        Regex::new(s).map(Self)
    }
    pub fn pattern(&self) -> &str {
        self.0.pattern()
    }
}
impl PartialEq for Pattern {
    fn eq(&self, b: &Self) -> bool {
        self.pattern() == b.pattern()
    }
}
impl Eq for Pattern {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub filter_by: u8,
    pub names: Vec<Option<Pattern>>,
}
impl Default for Filter {
    fn default() -> Self {
        Self {
            filter_by: 3,
            names: vec![],
        }
    }
}
impl Filter {
    pub fn matches(&self, name: &str, display: Option<&str>) -> Result<bool, &'static str> {
        if self.filter_by & 2 == 0 && (self.filter_by & 1 == 0 || display.is_none()) {
            return Ok(false);
        }
        for p in &self.names {
            let Some(p) = p else {
                return Err("null Regex in BetterTab player filter");
            };
            if (self.filter_by & 1 != 0 && display.is_some_and(|d| p.0.matches(d)))
                || (self.filter_by & 2 != 0 && p.0.matches(name))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Highlight {
    pub enabled: bool,
    pub color: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BetterTab {
    pub sorting: Sorting,
    pub visibility: u8,
    pub show_game_mode: bool,
    pub tab_size: i32,
    pub column_height: i32,
    pub highlight: bool,
    pub colors: [Highlight; 3],
    pub others_filter: Filter,
    pub accurate_latency: bool,
    pub append_ms_suffix: bool,
    pub player_hider: bool,
    pub hider_filter: Filter,
}
impl Default for BetterTab {
    fn default() -> Self {
        Self {
            sorting: Sorting::Vanilla,
            visibility: 3,
            show_game_mode: true,
            tab_size: 80,
            column_height: 20,
            highlight: true,
            colors: [
                Highlight {
                    enabled: true,
                    color: 0x5032c132,
                },
                Highlight {
                    enabled: true,
                    color: 0x501059cb,
                },
                Highlight {
                    enabled: true,
                    color: 0x50232323,
                },
            ],
            others_filter: Filter::default(),
            accurate_latency: true,
            append_ms_suffix: true,
            player_hider: false,
            hider_filter: Filter::default(),
        }
    }
}
impl BetterTab {
    pub fn hidden(&self, name: &str, display: Option<&str>) -> Result<bool, &'static str> {
        if self.player_hider {
            self.hider_filter.matches(name, display)
        } else {
            Ok(false)
        }
    }
    pub fn color(
        &self,
        name: &str,
        display: Option<&str>,
        own: &str,
        friend: bool,
    ) -> Result<Option<i32>, &'static str> {
        if !self.highlight {
            return Ok(None);
        }
        let selected = if self.colors[0].enabled && name == own {
            Some(0)
        } else if self.colors[1].enabled && friend {
            Some(1)
        } else if self.colors[2].enabled && self.others_filter.matches(name, display)? {
            Some(2)
        } else {
            None
        };
        Ok(selected.map(|i| self.colors[i].color))
    }
    pub fn dimensions(&self, count: usize) -> Result<(i32, i32), &'static str> {
        if self.tab_size < 0 {
            return Err("Stream.limit requires nonnegative TabSize");
        }
        let n = count.min(self.tab_size as usize) as i32;
        let height = self.column_height.max(1);
        let cols = ((n as f64 / height as f64).ceil() as i32).max(1);
        Ok((cols, (n as f64 / cols as f64).ceil() as i32))
    }
    pub fn latency(&self, latency: i32) -> (String, u32) {
        let v = latency.clamp(0, 9999);
        (
            format!("{v}{}", if self.append_ms_suffix { "ms" } else { "" }),
            if v < 150 {
                0xff00e970
            } else if v < 300 {
                0xffe7d020
            } else {
                0xffd74238
            },
        )
    }
}
