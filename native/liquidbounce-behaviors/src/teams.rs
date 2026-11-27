// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTeams ae0eb396. Facts are captured before the actor borrow.
#[path = "navigation/java_case.rs"]
mod java_case;
pub const MATCHES: [&str; 3] = ["ScoreboardTeam", "NameColor", "Prefix"];
pub const ARMOR_SLOTS: [&str; 4] = ["Feet", "Legs", "Chest", "Head"];
pub const COLOR_SOURCES: [&str; 2] = ["Team", "Armor"];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSource {
    Team,
    Armor,
}
impl ColorSource {
    pub fn tag(self) -> &'static str {
        COLOR_SOURCES[self as usize]
    }
    pub fn parse(s: &str) -> Option<Self> {
        [Self::Team, Self::Armor]
            .into_iter()
            .find(|v| java_case::equals_ignore_case(v.tag(), s))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Teams {
    pub matches: u32,
    pub armor_slots: u32,
    pub color_sources: Vec<ColorSource>,
}
impl Default for Teams {
    fn default() -> Self {
        Self {
            matches: 3,
            armor_slots: 8,
            color_sources: vec![ColorSource::Team, ColorSource::Armor],
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityFacts {
    pub team: Option<String>,
    pub team_color: Option<u32>,
    pub name: String,
    pub name_color: Option<u32>,
    pub armor: [Option<u32>; 4],
    pub player: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    pub own: EntityFacts,
    pub other: EntityFacts,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decision {
    pub teammate: bool,
    pub color: Option<u32>,
}
fn prefix(name: &str) -> Option<String> {
    let mut chars = name.chars().peekable();
    let mut plain = String::new();
    while let Some(c) = chars.next() {
        if c == '§'
            && chars.peek().is_some_and(|v| {
                v.is_ascii_hexdigit()
                    || matches!(v.to_ascii_lowercase(), 'k' | 'l' | 'm' | 'n' | 'o' | 'r')
            })
        {
            chars.next();
            continue;
        }
        plain.push(c);
    }
    plain.split_once(' ').map(|(first, _)| first.to_owned())
}
impl Teams {
    pub fn teammate(&self, f: &Facts) -> bool {
        self.matches & 1 != 0 && f.own.team.is_some() && f.own.team == f.other.team
            || self.matches & 2 != 0
                && f.own.name_color.is_some()
                && f.own.name_color == f.other.name_color
            || self.matches & 4 != 0
                && prefix(&f.own.name).is_some_and(|p| Some(p) == prefix(&f.other.name))
            || f.other.player
                && (0..4).any(|i| {
                    self.armor_slots & (1 << i) != 0
                        && f.own.armor[i].is_some()
                        && f.own.armor[i] == f.other.armor[i]
                })
    }
    pub fn decide(&self, running: bool, living: bool, f: &Facts) -> Decision {
        if !running {
            return Decision::default();
        }
        let color = self
            .color_sources
            .iter()
            .find_map(|source| match source {
                ColorSource::Team => f.other.team_color,
                ColorSource::Armor if living => (0..4).find_map(|i| {
                    (self.armor_slots & (1 << i) != 0)
                        .then_some(f.other.armor[i])
                        .flatten()
                }),
                _ => None,
            })
            .map(|rgb| rgb | 0xff000000);
        Decision {
            teammate: living && self.teammate(f),
            color,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn team_name_prefix_armor_and_color_order_smoke() {
        let mut m = Teams::default();
        let mut f = Facts {
            own: EntityFacts {
                name: "§a[Red] Alice".into(),
                ..Default::default()
            },
            other: EntityFacts {
                name: "§c[Red] Bob".into(),
                player: true,
                ..Default::default()
            },
        };
        assert!(!m.decide(true, true, &f).teammate);
        m.matches = 4;
        assert!(m.teammate(&f));
        f.other.name = "§z[Red] Bob".into();
        assert!(!m.teammate(&f));
        m.matches = 0;
        f.own.armor[3] = Some(0xff123456);
        f.other.armor[3] = Some(0xff123456);
        assert!(m.teammate(&f));
        f.other.player = false;
        assert!(!m.teammate(&f));
        f.other.team_color = Some(0xabcdef);
        assert_eq!(m.decide(true, true, &f).color, Some(0xffabcdef));
        m.color_sources = vec![ColorSource::Armor, ColorSource::Team];
        assert_eq!(m.decide(true, true, &f).color, Some(0xff123456));
        assert_eq!(m.decide(true, false, &f).color, Some(0xffabcdef));
        m.color_sources.clear();
        assert_eq!(m.decide(true, true, &f).color, None);
        m.matches = 1;
        f.own.team = Some("a".into());
        f.other.team = Some("a".into());
        assert!(m.teammate(&f));
        assert_eq!(m.decide(false, true, &f), Decision::default());
    }
}
