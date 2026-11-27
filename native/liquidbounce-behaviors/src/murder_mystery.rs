// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleMurderMystery and all three modes, pinned to LiquidBounce ae0eb396.
use std::collections::{HashMap, HashSet};
pub const MODES: [&str; 3] = ["Classic", "Infection", "Assassination"];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    #[default]
    Neutral,
    Detective,
    Murderer,
}
impl Role {
    pub fn color(self) -> Option<u32> {
        match self {
            Self::Neutral => None,
            Self::Detective => Some(0xff0090ff),
            Self::Murderer => Some(0xffcb0909),
        }
    }
    pub fn prefix(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Neutral => None,
            Self::Detective => Some(("[BOW] ", "aqua")),
            Self::Murderer => Some(("[MURD] ", "red")),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub sword: bool,
    pub mining: bool,
    pub boat: bool,
}
impl Item {
    pub fn is_bow(&self) -> bool {
        self.id == "minecraft:bow"
    }
    pub fn is_arrow(&self) -> bool {
        self.id == "minecraft:arrow"
    }
    pub fn is_sword(&self) -> bool {
        let id = self.id.strip_prefix("minecraft:").unwrap_or(&self.id);
        if self.id == "minecraft:wooden_shovel" || self.id == "minecraft:golden_shovel" {
            return false;
        }
        (self.id.starts_with("minecraft:") && KNOWN_SWORDS.contains(&id))
            || self.sword
            || self.mining
            || self.boat
    }
}
pub const KNOWN_SWORDS: &[&str] = &[
    "golden_carrot",
    "carrot",
    "carrot_on_a_stick",
    "bone",
    "tropical_fish",
    "pufferfish",
    "salmon",
    "blaze_rod",
    "pumpkin_pie",
    "name_tag",
    "apple",
    "feather",
    "cookie",
    "shears",
    "cooked_salmon",
    "stick",
    "quartz",
    "rose_bush",
    "ice",
    "cooked_beef",
    "nether_brick",
    "cooked_chicken",
    "music_disc_blocks",
    "music_disc_11",
    "music_disc_13",
    "music_disc_cat",
    "music_disc_chirp",
    "music_disc_far",
    "music_disc_mall",
    "music_disc_mellohi",
    "music_disc_stal",
    "music_disc_strad",
    "music_disc_ward",
    "music_disc_wait",
    "red_dye",
    "oak_boat",
    "book",
    "glistering_melon_slice",
    "jungle_sapling",
    "prismarine_shard",
    "charcoal",
    "sugar_cane",
    "flint",
    "bread",
    "lapis_lazuli",
    "leather",
    "sponge",
    "dead_bush",
    "redstone_torch",
    "chorus_plant",
];
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub uuid: String,
    pub name: String,
    pub position: [f64; 3],
    pub has_arrow: bool,
    pub using_bow: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Murderer(String),
    Bow(String),
    FirstInfected(String),
    Assassin(String),
    Target(String),
    MissingTarget(String),
}
#[derive(Clone, Debug, Default)]
pub struct UuidMode {
    pub bows: HashSet<String>,
    pub murderers: HashSet<String>,
    pub own_role: Role,
}
impl UuidMode {
    fn reset(&mut self) {
        self.bows.clear();
        self.murderers.clear();
    } // Source deliberately retains currentPlayerType.
    fn role(&self, uuid: &str) -> Role {
        if self.murderers.contains(uuid) {
            Role::Murderer
        } else if self.bows.contains(uuid) {
            Role::Detective
        } else {
            Role::Neutral
        }
    }
}
#[derive(Clone, Debug)]
pub struct State {
    pub mode: usize,
    pub set_team_prefix: bool,
    pub classic: UuidMode,
    pub infection: UuidMode,
    pub last_map: Option<i32>,
    pub target: Option<String>,
    pub assassin: Option<String>,
    pub play_hurt: bool,
    pub play_bow: bool,
    pub profiles: HashMap<String, String>,
    pub prefixes: Vec<(String, Role)>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            mode: 0,
            set_team_prefix: true,
            classic: Default::default(),
            infection: Default::default(),
            last_map: None,
            target: None,
            assassin: None,
            play_hurt: false,
            play_bow: false,
            profiles: Default::default(),
            prefixes: vec![],
        }
    }
}
impl State {
    pub fn set_mode(&mut self, mode: usize) {
        if mode != self.mode {
            self.mode = mode;
            self.reset_modes();
        }
    }
    pub fn reset_modes(&mut self) {
        self.classic.reset();
        self.infection.reset();
        self.target = None;
        self.assassin = None;
    }
    pub fn reset(&mut self) {
        self.play_hurt = false;
        self.play_bow = false;
        self.prefixes.clear();
        self.reset_modes();
    }
    pub fn disallows_arrow_dodge(&self, running: bool) -> bool {
        running && self.mode == 1 && self.infection.own_role == Role::Detective
    }
    pub fn role(&self, uuid: &str) -> Role {
        match self.mode {
            0 => self.classic.role(uuid),
            1 => self.infection.role(uuid),
            _ => {
                if self.target.as_deref() == Some(uuid) || self.assassin.as_deref() == Some(uuid) {
                    Role::Murderer
                } else {
                    Role::Neutral
                }
            }
        }
    }
    pub fn should_attack(&self, uuid: &str) -> bool {
        let role = self.role(uuid);
        if self.mode == 2 {
            return role == Role::Murderer;
        }
        let own = if self.mode == 0 {
            self.classic.own_role
        } else {
            self.infection.own_role
        };
        if own == Role::Murderer {
            role != Role::Murderer
        } else {
            role == Role::Murderer
        }
    }
    pub fn tag(&mut self, name: &str, uuid: Option<&str>) -> Option<(bool, Role)> {
        let uuid = uuid.or_else(|| self.profiles.get(name).map(String::as_str))?;
        let attack = self.should_attack(uuid);
        let role = self.role(uuid);
        if self.set_team_prefix && role != Role::Neutral {
            self.prefixes.push((name.to_owned(), role));
        }
        Some((attack, role))
    }
    pub fn item(&mut self, player: &Player, item: &Item) -> Option<Message> {
        self.profiles
            .insert(player.name.clone(), player.uuid.clone());
        if self.mode == 2 {
            return None;
        }
        let s = if self.mode == 0 {
            &mut self.classic
        } else {
            &mut self.infection
        };
        if item.is_sword() {
            if !s.murderers.insert(player.uuid.clone()) {
                return None;
            }
            if self.mode == 0 {
                self.play_hurt = true;
                Some(Message::Murderer(player.name.clone()))
            } else if s.murderers.len() == 1 {
                self.play_hurt = true;
                Some(Message::FirstInfected(player.name.clone()))
            } else {
                None
            }
        } else if item.is_bow() && s.bows.insert(player.uuid.clone()) {
            self.play_bow = true;
            Some(Message::Bow(player.name.clone()))
        } else {
            None
        }
    }
    pub fn tick(&mut self, hands: &[Item; 2], players: &[Player]) -> Vec<Message> {
        for p in players {
            self.profiles.insert(p.name.clone(), p.uuid.clone());
        }
        if self.mode == 2 {
            return vec![];
        }
        let role = hands
            .iter()
            .find_map(|i| {
                if i.is_bow() || i.is_arrow() {
                    Some(Role::Detective)
                } else if i.is_sword() {
                    Some(Role::Murderer)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        if self.mode == 0 {
            self.classic.own_role = role;
            return vec![];
        }
        self.infection.own_role = role;
        let bow = Item {
            id: "minecraft:bow".into(),
            ..Default::default()
        };
        players
            .iter()
            .filter(|p| p.using_bow || p.has_arrow)
            .filter_map(|p| self.item(p, &bow))
            .collect()
    }
    pub fn sound(&mut self, volume: f32, own: [f64; 3], players: &[Player]) -> Option<Message> {
        if self.mode != 2 {
            return None;
        }
        let expected = ((1.0f32 / volume) as f64 - 0.98272992) / 0.04342088;
        let score = |p: &Player| {
            (((0..3)
                .map(|i| (p.position[i] - own[i]).powi(2))
                .sum::<f64>()
                .sqrt() as f32) as f64
                - expected)
                .abs()
        };
        let mut best = None;
        let mut distance = f64::NAN;
        for p in players {
            let d = score(p);
            if best.is_none() || d.total_cmp(&distance).is_lt() {
                best = Some(p);
                distance = d;
            }
        }
        let p = best?;
        let changed = self.assassin.as_deref() != Some(&p.uuid);
        self.assassin = Some(p.uuid.clone());
        changed.then(|| Message::Assassin(p.name.clone()))
    }
    pub fn contract(
        &mut self,
        is_map: bool,
        id: Option<i32>,
        colors: Option<&[u8]>,
        online: &[(String, String)],
    ) -> Option<Message> {
        if self.mode != 2 {
            return None;
        }
        if !is_map {
            self.last_map = None;
            return None;
        }
        let id = id?;
        if self.last_map == Some(id) {
            return None;
        }
        let colors = colors?;
        if colors.len() < 128 * 112 {
            return None;
        }
        self.last_map = Some(id);
        let text = read_contract(colors);
        let word = text.split(' ').next().unwrap_or("");
        let name = word.strip_prefix("NAME:")?.to_lowercase().trim().to_owned();
        let mut best = None;
        let mut score = usize::MAX;
        for p in online {
            let d = levenshtein(&name, p.1.to_lowercase().trim());
            if d < score {
                score = d;
                best = Some(p);
            }
        }
        if let Some((uuid, name)) = best {
            self.target = Some(uuid.clone());
            Some(Message::Target(name.clone()))
        } else {
            Some(Message::MissingTarget(name))
        }
    }
    pub fn take_sounds(&mut self) -> (bool, bool) {
        (
            std::mem::take(&mut self.play_hurt),
            std::mem::take(&mut self.play_bow),
        )
    }
}
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<_> = a.encode_utf16().collect();
    let mut row: Vec<_> = (0..=a.len()).collect();
    for (j, c) in b.encode_utf16().enumerate() {
        let mut diagonal = row[0];
        row[0] = j + 1;
        for (i, &d) in a.iter().enumerate() {
            let old = row[i + 1];
            row[i + 1] = (row[i] + 1)
                .min(old + 1)
                .min(diagonal + usize::from(c != d));
            diagonal = old;
        }
    }
    row[a.len()]
}
include!("murder_mystery_font.rs");
pub fn read_contract(colors: &[u8]) -> String {
    if colors.len() < 128 * 112 {
        return String::new();
    }
    let background = |x: usize, y: usize| colors[(105 + y) * 128 + x] >> 2 == 13; // MapColor.WOOD, brightness discarded.
    let mut out = String::new();
    let mut last = None;
    let mut empty = 0;
    for x in 0..128 {
        let blank = (0..7).all(|y| background(x, y));
        if blank {
            let old = empty;
            empty += 1;
            if old > 3 {
                out.push(' ');
                empty = 0;
            }
        }
        if let Some(start) = last
            && blank
        {
            let mut fingerprint = vec![];
            for y in 0..7 {
                for x in start..x {
                    fingerprint.push(!background(x, y));
                }
            }
            out.push_str(
                LETTERS
                    .iter()
                    .find(|(_, v)| *v == fingerprint.as_slice())
                    .map_or("?", |(k, _)| *k),
            );
            last = None;
        }
        if !blank && last.is_none() {
            last = Some(x);
            empty = 0;
        }
    }
    out.trim_matches(|c| c <= ' ').to_owned()
}
