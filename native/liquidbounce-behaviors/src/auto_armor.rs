// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoArmor and AutoArmorSaveArmor at LiquidBounce ae0eb396.
use crate::{
    inventory_cleaner::{ArmorMode, ArmorSettings, Item},
    inventory_manager::{Action, Context, Input, Slot},
    vomit::Constraints,
};
use std::collections::BTreeSet;

pub const CURSES: [&str; 2] = ["Binding", "Vanishing"];
#[derive(Clone, Debug)]
pub struct AutoArmor {
    pub constraints: Constraints,
    pub mode: ArmorMode,
    pub projectile: bool,
    pub hotbar: bool,
    pub can_swap: bool,
    pub save: bool,
    pub threshold: i32,
    pub auto_open: bool,
    pub skip_cursed: u8,
}
impl Default for AutoArmor {
    fn default() -> Self {
        Self {
            constraints: Default::default(),
            mode: ArmorMode::Smart,
            projectile: true,
            hotbar: true,
            can_swap: false,
            save: true,
            threshold: 24,
            auto_open: true,
            skip_cursed: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub source: Slot,
    pub armor: u8,
}
impl Piece {
    pub fn target(self) -> Slot {
        Slot::Player(36 + self.armor)
    }
}
impl AutoArmor {
    pub fn policy(&self) -> ArmorSettings {
        ArmorSettings {
            mode: self.mode,
            projectile: self.projectile,
            durability_threshold: if self.save { self.threshold } else { i32::MIN },
            skip_cursed: self.skip_cursed,
        }
    }
    pub fn candidates(&self, items: &[Item], locked: &BTreeSet<Slot>) -> Vec<Item> {
        let policy = self.policy();
        items
            .iter()
            .filter(|s| {
                matches!(s.slot, Slot::Player(_))
                    && (s.equipped() || !locked.contains(&s.slot))
                    && policy.allowed(s)
            })
            .cloned()
            .collect()
    }
    pub fn pieces(
        &self,
        items: &[Item],
        locked: &BTreeSet<Slot>,
        save_handler: bool,
    ) -> Vec<Piece> {
        let items = self.candidates(items, locked);
        let mut policy = self.policy();
        // The save handler deliberately uses ArmorEvaluation's default Smart mode.
        if save_handler {
            policy.mode = ArmorMode::Smart;
            policy.projectile = true;
        }
        policy
            .best(&items)
            .into_iter()
            .flatten()
            .filter_map(|i| {
                let s = &items[i];
                (!s.equipped()).then_some(Piece {
                    source: s.slot,
                    armor: s.armor?,
                })
            })
            .collect()
    }
    pub fn action(
        &self,
        piece: Piece,
        c: &Context,
        items: &[Item],
        locked: &BTreeSet<Slot>,
        inventory_open: bool,
    ) -> Option<Action> {
        let target = items.iter().find(|s| s.slot == piece.target());
        if locked.contains(&piece.source)
            || locked.contains(&piece.target())
            || target.is_some_and(|s| s.classes.contains("Glider"))
        {
            return None;
        }
        let worn = target.is_some_and(|s| !s.empty());
        if self.hotbar
            && !inventory_open
            && (!worn || self.can_swap)
            && let Slot::Player(slot @ (0..=8 | 40)) = piece.source
        {
            return Some(Action::UseItem { slot });
        }
        let has_space = (0..36).any(|slot| c.player_stacks.get(slot).is_none_or(|s| s.empty()));
        Some(Action::Click {
            screen: None,
            slot: if worn { piece.target() } else { piece.source },
            input: if worn && !has_space {
                Input::Throw
            } else {
                Input::QuickMove
            },
            button: if worn && !has_space { 1 } else { 0 },
        })
    }
    pub fn needs_save(&self, items: &[Item], replacements: &[Piece]) -> bool {
        items.iter().any(|s| {
            s.equipped()
                && !s.empty()
                && s.weapon.durability <= self.threshold
                && replacements.iter().any(|p| p.target() == s.slot)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenAction {
    Close,
    Open,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveWait {
    Open,
    Close(i64),
}
#[derive(Default, Debug)]
pub struct SaveState {
    pub opened: bool,
    pub previous_armor: i32,
    pub waiting: Option<SaveWait>,
}
impl SaveState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// Returns an immediate close, or registers a wait before the next effect.
    pub fn tick(
        &mut self,
        settings: &AutoArmor,
        c: &Context,
        spectator: bool,
        armor: i32,
        items: &[Item],
        replacements: &[Piece],
        close_delay: impl FnOnce() -> i32,
    ) -> Option<ScreenAction> {
        if self.waiting.is_some()
            || spectator
            || c.creative
            || !settings.save
            || !settings.auto_open
            || settings.constraints.requirements & 32 == 0
        {
            return None;
        }
        let lost = c.container_screen && armor < self.previous_armor;
        self.previous_armor = armor;
        if lost {
            return Some(ScreenAction::Close);
        }
        if settings.hotbar
            && settings.can_swap
            && replacements
                .iter()
                .any(|p| matches!(p.source, Slot::Player(0..=8 | 40)))
        {
            return None;
        }
        if self.opened && replacements.is_empty() {
            self.opened = false;
            self.waiting = Some(SaveWait::Close(
                c.now.saturating_add(i64::from(close_delay())),
            ));
        } else if settings.needs_save(items, replacements) && !c.inventory_screen {
            self.waiting = Some(SaveWait::Open);
            if !c.no_screen {
                return Some(ScreenAction::Close);
            }
        }
        None
    }
    /// Called after one game tick for Open, or after the sampled milliseconds.
    pub fn resume(&mut self, c: &Context) -> (Option<ScreenAction>, bool) {
        match self.waiting {
            None => (None, false),
            Some(SaveWait::Close(deadline)) if c.now < deadline => (None, true),
            Some(SaveWait::Close(_)) => {
                self.waiting = None;
                (c.inventory_screen.then_some(ScreenAction::Close), false)
            }
            Some(SaveWait::Open) if c.inventory_screen => {
                self.waiting = None;
                (None, false)
            }
            Some(SaveWait::Open) if c.no_screen => {
                self.waiting = None;
                self.opened = true;
                (Some(ScreenAction::Open), false)
            }
            Some(SaveWait::Open) => (Some(ScreenAction::Close), true),
        }
    }
}
