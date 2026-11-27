// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoCrafter's recipe-book coroutine, LiquidBounce ae0eb396.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnFull {
    Disable,
    CloseScreen,
    #[default]
    Wait,
    Throw,
}
impl OnFull {
    pub const ALL: [Self; 4] = [Self::Disable, Self::CloseScreen, Self::Wait, Self::Throw];
    pub fn tag(self) -> &'static str {
        match self {
            Self::Disable => "Disable",
            Self::CloseScreen => "CloseScreen",
            Self::Wait => "Wait",
            Self::Throw => "Throw",
        }
    }
}
pub const CONTAINERS: [&str; 5] = [
    "Inventory",
    "CraftingTable",
    "Furnace",
    "BlastFurnace",
    "Smoker",
];
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub items: Vec<String>,
    pub delay: [i32; 2],
    pub stack: bool,
    pub sequential: bool,
    pub on_full: OnFull,
    pub allowed: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            items: ["polished_deepslate", "deepslate_bricks", "deepslate_tiles"]
                .map(|s| format!("minecraft:{s}"))
                .into(),
            delay: [2, 3],
            stack: true,
            sequential: true,
            on_full: OnFull::Wait,
            allowed: 2,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Recipe {
    pub display: i32,
    pub outputs: Vec<String>,
    pub uses: Vec<String>,
    pub craftable: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub menu: [u64; 2],
    pub kind: usize,
    pub book_open: bool,
    pub recipes: Vec<Recipe>,
    pub result_count: u32,
    pub capacity: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Place { display: i32, stack: bool },
    QuickMove,
    Throw,
    Disable,
    Close,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effect {
    pub action: Option<Action>,
    pub register: Option<u64>,
}
#[derive(Clone, Debug, PartialEq)]
struct Run {
    menu: [u64; 2],
    furnace: bool,
    items: Vec<String>,
    cursor: usize,
    wait: i32,
    exit: bool,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutoCrafter {
    pub settings: Settings,
    pub callback: u64,
    run: Option<Run>,
}
impl AutoCrafter {
    pub fn items(&self) -> &[String] {
        self.run.as_ref().map_or(&self.settings.items, |r| &r.items)
    }
    pub fn cancel(&mut self) {
        self.run = None;
        self.callback = self.callback.wrapping_add(1);
    }
    pub fn start(&mut self, c: &Context, sample: &mut impl FnMut([i32; 2]) -> i32) -> Effect {
        if self.run.is_some()
            || c.kind >= CONTAINERS.len()
            || !c.book_open
            || self.settings.allowed & (1 << c.kind) == 0
        {
            return Effect::default();
        }
        self.run = Some(Run {
            menu: c.menu,
            furnace: c.kind >= 2,
            items: self.settings.items.clone(),
            cursor: 0,
            wait: 0,
            exit: false,
        });
        self.advance(c, sample)
    }
    fn advance(&mut self, c: &Context, sample: &mut impl FnMut([i32; 2]) -> i32) -> Effect {
        let Some(run) = self.run.as_mut() else {
            return Effect::default();
        };
        while run.cursor < run.items.len() {
            let index = run.cursor;
            run.cursor += 1;
            let target = &run.items[index];
            let recipe = c.recipes.iter().find(|r| {
                r.craftable
                    && r.outputs.contains(target)
                    && !run.items[index + 1..]
                        .iter()
                        .any(|item| r.uses.contains(item))
            });
            let Some(recipe) = recipe else { continue };
            let action = if c.result_count == 0 {
                Some(Action::Place {
                    display: recipe.display,
                    stack: self.settings.stack,
                })
            } else if c.capacity >= c.result_count {
                Some(Action::QuickMove)
            } else {
                match self.settings.on_full {
                    OnFull::Disable => {
                        self.run = None;
                        return Effect {
                            action: Some(Action::Disable),
                            register: None,
                        };
                    }
                    OnFull::CloseScreen => {
                        self.run = None;
                        return Effect {
                            action: Some(Action::Close),
                            register: None,
                        };
                    }
                    OnFull::Wait => {
                        run.exit = true;
                        None
                    }
                    OnFull::Throw => Some(Action::Throw),
                }
            };
            run.wait = sample(self.settings.delay).max(0);
            self.callback = self.callback.wrapping_add(1);
            return Effect {
                action,
                register: Some(self.callback),
            };
        }
        self.run = None;
        Effect::default()
    }
    pub fn resume(
        &mut self,
        callback: u64,
        c: Option<&Context>,
        sample: &mut impl FnMut([i32; 2]) -> i32,
    ) -> (bool, Effect) {
        if callback != self.callback {
            return (false, Effect::default());
        }
        let Some(run) = self.run.as_mut() else {
            return (false, Effect::default());
        };
        let Some(c) = c.filter(|c| c.menu == run.menu) else {
            self.cancel();
            return (false, Effect::default());
        };
        if run.wait > 1 {
            run.wait -= 1;
            return (true, Effect::default());
        }
        if run.exit || run.furnace || self.settings.sequential {
            self.run = None;
            return (false, Effect::default());
        }
        (false, self.advance(c, sample))
    }
    pub fn waits_immediately(&self) -> bool {
        self.run.as_ref().is_some_and(|r| r.wait == 0)
    }
}

#[cfg(test)]
mod auto_crafter_tests {
    use super::*;
    fn context() -> Context {
        Context {
            menu: [5, 9],
            kind: 1,
            book_open: true,
            recipes: vec![
                Recipe {
                    display: 10,
                    outputs: vec!["minecraft:polished_deepslate".into()],
                    uses: vec!["minecraft:deepslate_bricks".into()],
                    craftable: true,
                },
                Recipe {
                    display: 11,
                    outputs: vec!["minecraft:polished_deepslate".into()],
                    uses: vec![],
                    craftable: true,
                },
                Recipe {
                    display: 12,
                    outputs: vec!["minecraft:deepslate_bricks".into()],
                    uses: vec![],
                    craftable: true,
                },
            ],
            ..Default::default()
        }
    }
    #[test]
    fn original_recipe_order_skips_future_ingredient_loops() {
        let mut m = AutoCrafter::default();
        let e = m.start(&context(), &mut |_| 2);
        assert_eq!(
            e.action,
            Some(Action::Place {
                display: 11,
                stack: true
            })
        );
        assert!(e.register.is_some());
    }
    #[test]
    fn suspended_nonsequential_run_keeps_list_but_uses_live_settings() {
        let mut m = AutoCrafter::default();
        m.settings.sequential = false;
        let mut c = context();
        let e = m.start(&c, &mut |_| 2);
        m.settings.items.clear();
        m.settings.stack = false;
        c.book_open = false;
        assert!(m.resume(e.register.unwrap(), Some(&c), &mut |_| 2).0);
        let (_, next) = m.resume(e.register.unwrap(), Some(&c), &mut |_| 2);
        assert_eq!(
            next.action,
            Some(Action::Place {
                display: 12,
                stack: false
            })
        );
        c.menu[1] += 1;
        assert_eq!(
            m.resume(next.register.unwrap(), Some(&c), &mut |_| 2),
            (false, Effect::default())
        );
    }
    #[test]
    fn all_full_actions_and_capacity_boundary_follow_original() {
        let mut c = context();
        c.result_count = 4;
        c.capacity = 3;
        for (mode, expected) in [
            (OnFull::Disable, Some(Action::Disable)),
            (OnFull::CloseScreen, Some(Action::Close)),
            (OnFull::Wait, None),
            (OnFull::Throw, Some(Action::Throw)),
        ] {
            let mut m = AutoCrafter::default();
            m.settings.on_full = mode;
            let e = m.start(&c, &mut |_| 2);
            assert_eq!(e.action, expected);
            assert_eq!(
                e.register.is_some(),
                matches!(mode, OnFull::Wait | OnFull::Throw)
            );
        }
        c.capacity = 4;
        assert_eq!(
            AutoCrafter::default().start(&c, &mut |_| 2).action,
            Some(Action::QuickMove)
        );
    }
    #[test]
    fn furnace_returns_after_wait_and_wait_full_exits_run() {
        let mut m = AutoCrafter::default();
        m.settings.sequential = false;
        m.settings.allowed = 31;
        let mut c = context();
        c.kind = 2;
        let e = m.start(&c, &mut |_| 1);
        assert_eq!(
            m.resume(e.register.unwrap(), Some(&c), &mut |_| 1),
            (false, Effect::default())
        );
        c.kind = 1;
        c.result_count = 4;
        let e = m.start(&c, &mut |_| 1);
        assert_eq!(
            m.resume(e.register.unwrap(), Some(&c), &mut |_| 1),
            (false, Effect::default())
        );
    }
}
