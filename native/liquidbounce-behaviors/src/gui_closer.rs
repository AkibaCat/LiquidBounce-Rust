// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396 ModuleGUICloser.kt (CCBlueX, 2015-2026).
use java_regex::{PatternSyntaxError, Regex};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Matches,
    Contains,
}
impl Mode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Matches => "Matches",
            Self::Contains => "Contains",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        // Tagged.makeLookupTable uses Java CASE_INSENSITIVE_ORDER, including
        // dotless/dotted i and long s in Contains/Matches.
        fn fold(c: char) -> char {
            let mut upper = c.to_uppercase();
            let first = upper.next().unwrap_or(c);
            let upper = if upper.next().is_none() { first } else { c };
            upper.to_lowercase().next().unwrap_or(upper)
        }
        [Self::Matches, Self::Contains]
            .into_iter()
            .find(|mode| mode.tag().chars().map(fold).eq(value.chars().map(fold)))
    }
    fn test(self, regex: &Regex, title: &str) -> bool {
        match self {
            Self::Matches => regex.matches(title),
            Self::Contains => regex.contains_match(title),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Filter(Regex);
impl Filter {
    pub fn new(pattern: &str) -> Result<Self, PatternSyntaxError> {
        Regex::new(pattern).map(Self)
    }
    pub fn pattern(&self) -> &str {
        self.0.pattern()
    }
}
impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        self.pattern() == other.pattern()
    }
}
impl Eq for Filter {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiCloser {
    pub mode: Mode,
    // Gson accepts a null Regex element. A null before any match aborts the
    // handler through EventManager's exception catch, so retain its position.
    pub filters: Vec<Option<Filter>>,
    pub print_screen_title: bool,
}
impl Default for GuiCloser {
    fn default() -> Self {
        Self {
            mode: Mode::Matches,
            filters: vec![Some(Filter::new("^Vote$").expect("valid source default"))],
            print_screen_title: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decision {
    pub cancel: bool,
    pub print_title: bool,
    pub null_filter: bool,
}
impl GuiCloser {
    pub fn screen_opening(&self, running: bool, container: bool, title: &str) -> Decision {
        if !running || !container {
            return Decision::default();
        }
        for filter in &self.filters {
            let Some(filter) = filter else {
                return Decision { null_filter: true, ..Decision::default() };
            };
            if self.mode.test(&filter.0, title) {
                return Decision { cancel: true, ..Decision::default() };
            }
        }
        Decision { print_title: self.print_screen_title, ..Decision::default() }
    }
}
