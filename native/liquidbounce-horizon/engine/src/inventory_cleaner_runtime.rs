// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{inventory::InventoryActor, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_weapon,
    inventory_cleaner::{ArmorSettings, BlockFacts, Effect, InventoryCleaner, Item, Swap},
    inventory_manager::{Action, Context, Input, Outcome, OwnerState, Request, Slot},
};
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};
pub const OWNER: u64 = 0x494e_5643_4c45_414e;
/// Updated by Scaffold's BlockItemSelection tree and AutoArmor's active policy.
/// They are shared dependencies, not InventoryCleaner settings.
#[derive(Default)]
pub struct ItemPolicy {
    pub armor: ArmorSettings,
    pub block_selection: Value,
    pub offhand_operating: bool,
}
pub struct ItemSnapshot {
    pub items: Vec<Item>,
    pub legacy: bool,
    pub baseline: f64,
    pub offhand_swap: bool,
}
impl ItemSnapshot {
    pub fn parse(payload: &str) -> Option<Self> {
        let v: Value = serde_json::from_str(payload).ok()?;
        let records = v["items"].as_array()?;
        let weapons = json!({"hotbar":records.iter().map(|r|r["facts"]["weapon"].clone()).collect::<Vec<_>>(),"stick":v["stick"],"legacy":v["legacy"],"canSmash":false,"consumingMainHand":false,"currentSpeed":4.,"selected":0});
        let weapons = auto_weapon::Snapshot::parse(&weapons)?;
        let items = records
            .iter()
            .zip(weapons.hotbar)
            .map(|(record, weapon)| {
                let s = &record["facts"];
                let block = s.get("block").filter(|v| !v.is_null()).map(|v| BlockFacts {
                    valid: v["valid"] == true,
                    favourable: v["favourable"] == true,
                    full: v["full"] == true,
                    solid: v["solid"] == true,
                    friction: v["friction"].as_f64().unwrap_or(0.6) as f32,
                    jump: v["jump"].as_f64().unwrap_or(1.) as f32,
                    speed: v["speed"].as_f64().unwrap_or(1.) as f32,
                    hardness: v["hardness"].as_f64().unwrap_or(0.) as f32,
                });
                Some(Item {
                    slot: if let Some(slot) = record["player"].as_u64() {
                        Slot::Player(u8::try_from(slot).ok()?)
                    } else {
                        Slot::Container(i32::try_from(record["container"].as_i64()?).ok()?)
                    },
                    id: s["item"].as_str()?.into(),
                    components: s["components"].as_str()?.into(),
                    patch: s["patch"].as_str()?.into(),
                    count: s["count"].as_i64()? as i32,
                    maximum: s["maximum"].as_i64()? as i32,
                    default_maximum: s["defaultMaximum"].as_i64()? as i32,
                    stackable: s["stackable"] == true,
                    weapon,
                    classes: serde_json::from_value(s["classes"].clone()).ok()?,
                    enchants: serde_json::from_value(s["enchants"].clone()).ok()?,
                    enchantment_count: s["enchantmentCount"].as_i64()? as i32,
                    armor: s["armor"].as_u64().map(|v| v as u8),
                    defense: s["defense"].as_f64()? as f32,
                    toughness: s["toughness"].as_f64()? as f32,
                    curse_mask: s["curses"].as_u64()? as u8,
                    food: s["food"].as_array().and_then(|a| {
                        Some((a.first()?.as_i64()? as i32, a.get(1)?.as_f64()? as f32))
                    }),
                    block,
                    mining_speed: s["mining"].as_f64()? as f32,
                    custom_name: s["named"] == true,
                    effects: s["effects"]
                        .as_array()?
                        .iter()
                        .map(|e| {
                            Some(Effect {
                                id: e["id"].as_str()?.into(),
                                amplifier: e["amplifier"].as_i64()? as i32,
                                duration: e["duration"].as_i64()? as i32,
                            })
                        })
                        .collect::<Option<Vec<_>>>()?,
                    bucket_subtype: s["bucket"].as_i64()? as i32,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            items,
            legacy: weapons.legacy,
            baseline: weapons.stick.damage_score(weapons.legacy),
            offhand_swap: v["offhandSwap"] == true,
        })
    }
}
#[derive(Default)]
pub struct InventoryCleanerRuntime {
    pub settings: InventoryCleaner,
    pub pending: Option<u64>,
    swaps: VecDeque<Swap>,
    actions: VecDeque<(Vec<Action>, i32, bool)>,
    swapped: bool,
}
fn empty(c: &Context, slot: Slot) -> bool {
    match slot {
        Slot::Player(slot) => c.player_stacks.get(usize::from(slot)),
        Slot::Container(slot) => usize::try_from(slot)
            .ok()
            .and_then(|i| c.menu.stacks.get(i)),
    }
    .is_none_or(|s| s.empty())
}
impl InventoryCleanerRuntime {
    pub fn clear(&mut self) {
        self.pending = None;
        self.swaps.clear();
        self.actions.clear();
        self.swapped = false;
    }
    pub fn owner(&self, running: bool) -> (u64, OwnerState) {
        (
            OWNER,
            OwnerState {
                running,
                guards_pass: true,
                constraints: self.settings.constraints.clone(),
            },
        )
    }
    fn drive(&mut self, inventory: &mut InventoryActor, c: &Context) {
        let Some(id) = self.pending else {
            return;
        };
        while let Some(swap) = self.swaps.pop_front() {
            let from = inventory.manager.slot_of(id, swap.from);
            if from == swap.to {
                continue;
            }
            let Slot::Player(to) = swap.to else {
                continue;
            };
            self.swapped = true;
            inventory.manager.submit(Request::new(
                id,
                OWNER,
                0,
                vec![Action::Click {
                    screen: None,
                    slot: from,
                    button: i32::from(to),
                    input: Input::Swap,
                }],
                c,
                &inventory.manager,
            ));
            return;
        }
        if !self.swapped {
            while let Some((actions, priority, skip_empty)) = self.actions.pop_front() {
                if skip_empty
                    && actions
                        .first()
                        .is_some_and(|a| matches!(a,Action::Click{slot,..}if empty(c,*slot)))
                {
                    continue;
                }
                inventory.manager.submit(Request::new(
                    id,
                    OWNER,
                    priority,
                    actions,
                    c,
                    &inventory.manager,
                ));
                return;
            }
        }
        self.clear();
        inventory.manager.end_plan(id);
    }
    pub fn outcome(&mut self, inventory: &mut InventoryActor, outcome: &Outcome, c: &Context) {
        match *outcome {
            Outcome::Completed { request } if self.pending == Some(request) => {
                self.drive(inventory, c)
            }
            Outcome::Cancelled { request } if self.pending == Some(request) => {
                self.clear();
                inventory.manager.end_plan(request);
            }
            _ => {}
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn inventory_cleaner_plan(&self, host: &mut dyn Host, c: &Context) {
        let policy = {
            let state = self.state();
            let b = state.behaviors.borrow();
            if !b.query_running(&state.registry, "InventoryCleaner")
                || b.inventory_cleaner.pending.is_some()
            {
                return;
            }
            b.item_policy.block_selection.to_string()
        };
        let Some(snapshot) = host
            .client_inventory_cleaner_context(&policy)
            .and_then(|s| ItemSnapshot::parse(&s))
        else {
            return;
        };
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let b = &mut *b;
        let items = snapshot
            .items
            .into_iter()
            .filter(|s| matches!(s.slot, Slot::Player(_)))
            .collect::<Vec<_>>();
        let locked = items
            .iter()
            .filter(|s| {
                c.player_stacks
                    .get(match s.slot {
                        Slot::Player(i) => usize::from(i),
                        _ => usize::MAX,
                    })
                    .is_some_and(|s| b.inventory.manager.is_locked(s))
            })
            .map(|s| s.slot)
            .collect::<BTreeSet<_>>();
        let settings = &b.inventory_cleaner.settings;
        let plan = settings.plan(
            &items,
            snapshot.legacy,
            snapshot.baseline,
            &b.item_policy.armor,
            &locked,
            b.item_policy.offhand_operating,
            snapshot.offhand_swap,
        );
        let swaps = if settings.distance {
            b.inventory
                .manager
                .nearest_first(&plan.swaps, &c.inventory_menu, |s| s.from)
                .unwrap_or_else(|_| plan.swaps.clone())
        } else {
            let mut swaps = plan.swaps.clone();
            swaps.sort_by_key(|s| std::cmp::Reverse(s.priority));
            swaps
        };
        let merge = plan.merge_slots(&items);
        let mut throw = items
            .iter()
            .filter(|s| settings.blacklist.contains(&s.id))
            .chain(items.iter().filter(|s| !plan.useful.contains(&s.slot)))
            .map(|s| s.slot)
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        throw.retain(|s| seen.insert(*s) && !locked.contains(s));
        if settings.distance {
            throw = b
                .inventory
                .manager
                .nearest_first(&throw, &c.inventory_menu, |s| *s)
                .unwrap_or(throw);
        }
        let mut actions = VecDeque::new();
        let click = |slot, input, button| Action::Click {
            screen: None,
            slot,
            input,
            button,
        };
        for item in items
            .iter()
            .filter(|s| matches!(s.slot, Slot::Player(0..=8)) && s.classes.contains("ArrowItem"))
        {
            let Slot::Player(slot) = item.slot else {
                continue;
            };
            if settings.slots[usize::from(slot) + 1] == 22 || locked.contains(&item.slot) {
                continue;
            }
            if (9u8..36).any(|slot| {
                c.player_stacks
                    .get(usize::from(slot))
                    .is_none_or(|s| s.empty())
                    || items
                        .iter()
                        .find(|i| i.slot == Slot::Player(slot))
                        .is_some_and(|target| target.mergeable(item))
            }) {
                actions.push_back((vec![click(item.slot, Input::QuickMove, 0)], 0, false));
            }
        }
        for slot in merge {
            actions.push_back((
                vec![
                    click(slot, Input::Pickup, 0),
                    click(slot, Input::PickupAll, 0),
                    click(slot, Input::Pickup, 0),
                ],
                0,
                true,
            ));
        }
        for slot in throw {
            actions.push_back((vec![click(slot, Input::Throw, 1)], -20, true));
        }
        b.inventory_cleaner.pending = Some(b.inventory.id());
        b.inventory_cleaner.swapped = false;
        b.inventory_cleaner.swaps = swaps.into();
        b.inventory_cleaner.actions = actions;
        b.inventory_cleaner.drive(&mut b.inventory, c);
    }
}
