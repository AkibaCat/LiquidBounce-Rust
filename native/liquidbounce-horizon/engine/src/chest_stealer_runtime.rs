// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    inventory::InventoryActor, inventory_cleaner_runtime::ItemSnapshot, runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    chest_stealer::{self, ChestStealer},
    inventory_cleaner::{CleanupPlan, Item, Swap},
    inventory_manager::{Action, Context, Outcome, OwnerState, Request, Screen, Slot},
    no_rotate_set::{RotationsSettings, SmoothMode},
    rotations::{DefaultRotationTarget, Rotation},
};
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};
pub const OWNER: u64 = 0x4348_4553_5453_544c;
pub const ROTATION_PROVIDER: u64 = 15;
pub const PROCESSOR_BASE: u64 = 180;
#[derive(Clone, Debug)]
enum AuraWait {
    Container { pos: [i32; 3], remaining: i32 },
    Delay(i32),
}
#[derive(Default)]
pub struct ChestStealerRuntime {
    pub settings: ChestStealer,
    pub rotations: RotationsSettings,
    pub blocks_initialized: bool,
    pub pending: Option<u64>,
    screen: Option<Screen>,
    cleanup: Option<CleanupPlan>,
    swaps: VecDeque<Swap>,
    collect: VecDeque<Slot>,
    all_collect: Vec<Slot>,
    throw: VecDeque<Slot>,
    space: i32,
    swapped: bool,
    retry_pick: Option<Slot>,
    pub target: Option<Value>,
    interacted: BTreeSet<[i32; 3]>,
    attempts: i32,
    wait: Option<AuraWait>,
    pub should_hide: bool,
    pub last_block: Option<[i32; 3]>,
}
impl ChestStealerRuntime {
    pub(crate)fn storage_interacted(&self)->&BTreeSet<[i32;3]>{&self.interacted}
    pub fn initialize_storage(&mut self, blocks: &[String]) {
        if !self.blocks_initialized {
            self.settings.aura.blocks = blocks
                .iter()
                .filter(|id| {
                    ["chest", "shulker_box", "barrel", "furnace"]
                        .iter()
                        .any(|suffix| id.ends_with(suffix))
                        || matches!(
                            id.as_str(),
                            "minecraft:brewing_stand" | "minecraft:dispenser" | "minecraft:hopper"
                        )
                })
                .cloned()
                .collect();
            self.blocks_initialized = true;
        }
    }
    pub fn clear_aura(&mut self) {
        self.target = None;
        self.interacted.clear();
        self.attempts = 0;
        self.wait = None;
    }
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn clear(&mut self) {
        self.pending = None;
        self.screen = None;
        self.cleanup = None;
        self.swaps.clear();
        self.collect.clear();
        self.all_collect.clear();
        self.throw.clear();
        self.space = 0;
        self.swapped = false;
        self.retry_pick = None;
        self.target = None;
        self.interacted.clear();
        self.attempts = 0;
        self.wait = None;
        self.should_hide = false;
        self.last_block = None;
    }
    pub fn owner(&self, running: bool) -> (u64, OwnerState) {
        (
            OWNER,
            OwnerState {
                running,
                guards_pass: true,
                constraints: self.settings.inventory.constraints.clone(),
            },
        )
    }
    fn finish(&mut self, inventory: &mut InventoryActor) {
        if let Some(id) = self.pending.take() {
            inventory.manager.end_plan(id);
        }
        self.screen = None;
        self.cleanup = None;
    }
    fn submit(
        &self,
        inventory: &mut InventoryActor,
        c: &Context,
        actions: Vec<Action>,
        priority: i32,
    ) {
        if let Some(id) = self.pending {
            inventory.manager.submit(Request::new(
                id,
                OWNER,
                priority,
                actions,
                c,
                &inventory.manager,
            ));
        }
    }
    fn drive(
        &mut self,
        inventory: &mut InventoryActor,
        c: &Context,
        snapshot: &ItemSnapshot,
        armor: &liquidbounce_behaviors::inventory_cleaner::ArmorSettings,
    ) {
        let (Some(id), Some(screen), Some(cleanup)) =
            (self.pending, self.screen.clone(), self.cleanup.as_ref())
        else {
            return;
        };
        if c.screen.as_ref().is_none_or(|s| s.object != screen.object) {
            self.finish(inventory);
            return;
        }
        let items = &snapshot.items;
        while let Some(swap) = self.swaps.pop_front() {
            let from = inventory.manager.slot_of(id, swap.from);
            if let Some(actions) = self.settings.quick_swap(
                items,
                &cleanup.useful,
                &screen,
                from,
                swap.to,
                snapshot.offhand_swap,
            ) {
                self.swapped = true;
                self.submit(inventory, c, actions, swap.priority);
                return;
            }
        }
        if self.swapped {
            self.finish(inventory);
            return;
        }
        if let Some(slot) = self.retry_pick.take() {
            if let Some(actions) = self.settings.pick(items, &screen, slot) {
                let priority = chest_stealer::item(items, slot).map_or(i32::MIN, |i| {
                    chest_stealer::value(i, snapshot.legacy, snapshot.baseline, armor)
                });
                self.submit(inventory, c, actions, priority);
                return;
            }
        }
        while let Some(slot) = self.collect.pop_front() {
            if chest_stealer::item(items, slot).is_none() {
                continue;
            }
            if let Some(actions) = self.settings.pick(items, &screen, slot) {
                let priority = chest_stealer::value(
                    chest_stealer::item(items, slot).expect("checked"),
                    snapshot.legacy,
                    snapshot.baseline,
                    armor,
                );
                self.submit(inventory, c, actions, priority);
                return;
            }
            if self.space > 0 {
                if let Some(actions) = self.settings.throw_next(items, &screen, &mut self.throw) {
                    self.space -= 1;
                    self.retry_pick = Some(slot);
                    self.submit(inventory, c, actions, 0);
                    return;
                } else {
                    break;
                }
            }
        }
        if self.settings.inventory.auto_close
            && self
                .all_collect
                .iter()
                .all(|s| chest_stealer::item(items, *s).is_none())
        {
            self.all_collect.clear();
            self.screen = None;
            self.submit(
                inventory,
                c,
                vec![Action::CloseScreen {
                    container_id: screen.container_id,
                }],
                0,
            );
            return;
        }
        self.finish(inventory);
    }
}
impl LiquidBounceHooks {
    pub(crate) fn chest_stealer_manual(&self, pos: [i32; 3]) {
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        if b.query_running(&state.registry, "ChestStealer")
            && b.chest_stealer.settings.aura.enabled
            && b.chest_stealer.settings.aura.track_manual
        {
            b.chest_stealer.interacted.insert(pos);
        }
    }
    fn chest_stealer_snapshot(&self, host: &mut dyn Host) -> Option<ItemSnapshot> {
        let policy = self
            .state()
            .behaviors
            .borrow()
            .item_policy
            .block_selection
            .to_string();
        host.client_inventory_cleaner_context(&policy)
            .and_then(|s| ItemSnapshot::parse(&s))
    }
    pub(crate) fn chest_stealer_plan(&self, host: &mut dyn Host, c: &Context) {
        {
            let state = self.state();
            let b = state.behaviors.borrow();
            if !b.query_running(&state.registry, "ChestStealer")
                || b.chest_stealer.pending.is_some()
            {
                return;
            }
        }
        let Some(screen) = c.screen.as_ref() else {
            return;
        };
        let Some(meta) = host
            .client_chest_cleaner_context()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return;
        };
        let translated: Vec<Vec<String>> =
            serde_json::from_value(meta["translated"].clone()).unwrap_or_default();
        let Some(snapshot) = self.chest_stealer_snapshot(host) else {
            return;
        };
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let b = &mut *b;
        if !b.chest_stealer.settings.inventory.accepts_screen(
            c.inventory_screen,
            meta["menu"].as_str(),
            meta["title"].as_str().unwrap_or(""),
            &translated,
        ) {
            return;
        }
        let locked = snapshot
            .items
            .iter()
            .filter(|i| match i.slot {
                Slot::Player(slot) => c
                    .player_stacks
                    .get(usize::from(slot))
                    .is_some_and(|s| b.inventory.manager.is_locked(s)),
                Slot::Container(slot) => usize::try_from(slot)
                    .ok()
                    .and_then(|slot| c.menu.stacks.get(slot))
                    .is_some_and(|s| b.inventory.manager.is_locked(s)),
            })
            .map(|i| i.slot)
            .collect::<BTreeSet<_>>();
        let cleanup = if b.query_running(&state.registry, "InventoryCleaner") {
            b.inventory_cleaner.settings.plan(
                &snapshot.items,
                snapshot.legacy,
                snapshot.baseline,
                &b.item_policy.armor,
                &locked,
                b.item_policy.offhand_operating,
                snapshot.offhand_swap,
            )
        } else {
            CleanupPlan {
                useful: snapshot
                    .items
                    .iter()
                    .filter(|i| matches!(i.slot, Slot::Container(_)))
                    .map(|i| i.slot)
                    .collect(),
                swaps: vec![],
                mergeable: vec![],
            }
        };
        let mut collect = cleanup
            .useful
            .iter()
            .filter(|s| matches!(s, Slot::Container(_)))
            .copied()
            .collect::<Vec<_>>();
        let settings = &b.chest_stealer.settings;
        liquidbounce_behaviors::random::with_thread_random(|r| {
            settings.select(
                &mut collect,
                &snapshot.items,
                snapshot.legacy,
                snapshot.baseline,
                &b.item_policy.armor,
                r,
            )
        });
        let mut swaps = if settings.quick_swaps {
            cleanup.swaps.clone()
        } else {
            vec![]
        };
        if settings.selection == 0 {
            swaps.sort_by_key(|s| std::cmp::Reverse(s.priority));
        }
        let before = if settings.selection == 0 {
            collect
                .iter()
                .take_while(|slot| {
                    !swaps.is_empty()
                        && swaps.iter().all(|s| {
                            s.from != **slot
                                && chest_stealer::item(&snapshot.items, **slot).is_some_and(|i| {
                                    chest_stealer::value(
                                        i,
                                        snapshot.legacy,
                                        snapshot.baseline,
                                        &b.item_policy.armor,
                                    ) > s.priority
                                })
                        })
                })
                .copied()
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let skip_swaps = !before.is_empty();
        let s = &mut b.chest_stealer;
        s.space = chest_stealer::required_space(&snapshot.items, &cleanup, collect.len());
        s.throw = snapshot
            .items
            .iter()
            .filter(|i| matches!(i.slot, Slot::Player(_)) && !cleanup.useful.contains(&i.slot))
            .map(|i| i.slot)
            .collect();
        s.all_collect = collect.clone();
        s.collect = if skip_swaps {
            before.into()
        } else {
            collect.into()
        };
        s.swaps = if skip_swaps {
            VecDeque::new()
        } else {
            swaps.into()
        };
        s.screen = Some(screen.clone());
        s.cleanup = Some(cleanup);
        s.swapped = false;
        s.pending = Some(b.inventory.id());
        s.drive(&mut b.inventory, c, &snapshot, &b.item_policy.armor);
    }
    pub(crate) fn chest_stealer_outcome(
        &self,
        host: &mut dyn Host,
        outcome: &Outcome,
        c: &Context,
    ) {
        let pending = self.state().behaviors.borrow().chest_stealer.pending;
        match *outcome {
            Outcome::Completed { request } if pending == Some(request) => {
                let snapshot = self.chest_stealer_snapshot(host);
                let state = self.state();
                let mut b = state.behaviors.borrow_mut();
                let b = &mut *b;
                if let Some(snapshot) = snapshot {
                    if b.chest_stealer.screen.is_none() {
                        b.chest_stealer.finish(&mut b.inventory);
                    } else {
                        b.chest_stealer
                            .drive(&mut b.inventory, c, &snapshot, &b.item_policy.armor);
                    }
                } else {
                    b.chest_stealer.finish(&mut b.inventory);
                }
            }
            Outcome::Cancelled { request } if pending == Some(request) => {
                let state = self.state();
                let mut b = state.behaviors.borrow_mut();
                let b = &mut *b;
                b.chest_stealer.finish(&mut b.inventory);
            }
            _ => {}
        }
    }
    fn chest_aura_request(&self, host: &mut dyn Host) -> Option<Value> {
        let snapshot: Value = serde_json::from_str(&host.client_inventory_snapshot()).ok()?;
        let state = self.state();
        let b = state.behaviors.borrow();
        let s = &b.chest_stealer;
        if !s.settings.aura.running(
            b.query_running(&state.registry, "ChestStealer"),
            state.combat.in_combat(),
            snapshot["using"] == true,
        ) {
            return None;
        }
        let p = &s.settings.aura;
        let rotation = b
            .rotations
            .lock()
            .expect("RotationManager")
            .server_rotation(false, false);
        Some(
            json!({"aura":true,"range":p.range,"walls":p.walls,"blocks":p.blocks,"interacted":s.interacted,"target":s.target,"rotation":[rotation.yaw,rotation.pitch],"hideSwing":p.hide_swing}),
        )
    }
    pub(crate) fn chest_aura_prepare(&self, host: &mut dyn Host) {
        if !self
            .state()
            .behaviors
            .borrow()
            .chest_stealer
            .blocks_initialized
        {
            let blocks = host.client_auto_tool_block_registry();
            self.state()
                .behaviors
                .borrow_mut()
                .chest_stealer
                .initialize_storage(&blocks);
        }
        let Some(request) = self.chest_aura_request(host) else {
            self.state().behaviors.borrow_mut().chest_stealer.target = None;
            return;
        };
        let result = host
            .client_auto_deposit_target(&request.to_string())
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let previous = b.chest_stealer.target.as_ref().map(|v| v["pos"].clone());
        if previous != result.as_ref().map(|v| v["pos"].clone()) {
            b.chest_stealer.attempts = 0;
        }
        b.chest_stealer.target = result.clone();
        let Some(result) = result else {
            return;
        };
        let (Some(yaw), Some(pitch)) = (
            result["rotation"][0].as_f64(),
            result["rotation"][1].as_f64(),
        ) else {
            return;
        };
        let r = &b.chest_stealer.rotations;
        let target = DefaultRotationTarget {
            rotation: Rotation::new(yaw as f32, pitch as f32),
            horizontal_turn_speed: 180.,
            vertical_turn_speed: 180.,
            ticks_until_reset: r.ticks_until_reset,
            reset_threshold: r.reset_threshold,
            consider_inventory: true,
            movement_correction: r.movement_correction,
        };
        let processor = PROCESSOR_BASE + r.active.processor_id();
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(ROTATION_PROVIDER, 20, target, processor, true);
    }
    pub(crate) fn chest_aura_tick(&self, host: &mut dyn Host) {
        let meta = host
            .client_chest_stealer_context()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or(Value::Null);
        let open = meta["abstract"] == true;
        {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let running = b.query_running(&state.registry, "ChestStealer");
            let s = &mut b.chest_stealer;
            if !running || !s.settings.aura.enabled {
                s.wait = None;
                return;
            }
            match s.wait.take() {
                Some(AuraWait::Container { pos, mut remaining }) => {
                    remaining -= 1;
                    if open || remaining <= 0 {
                        if open || s.attempts >= s.settings.aura.max_retries {
                            s.interacted.insert(pos);
                            s.target = None;
                        } else {
                            s.attempts += 1;
                        }
                    } else {
                        s.wait = Some(AuraWait::Container { pos, remaining });
                    }
                    return;
                }
                Some(AuraWait::Delay(mut remaining)) => {
                    remaining -= 1;
                    if remaining > 0 {
                        s.wait = Some(AuraWait::Delay(remaining));
                        return;
                    }
                    s.target = None;
                    return;
                }
                None => {}
            }
        }
        if open {
            return;
        }
        let Some(request) = self.chest_aura_request(host) else {
            return;
        };
        let Some(pos) = request["target"]["pos"]
            .as_array()
            .filter(|v| v.len() == 3)
            .and_then(|v| {
                Some([
                    v[0].as_i64()? as i32,
                    v[1].as_i64()? as i32,
                    v[2].as_i64()? as i32,
                ])
            })
        else {
            return;
        };
        if !host.client_chest_stealer_use_on(&request.to_string()) {
            return;
        }
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let s = &mut b.chest_stealer;
        if s.settings.aura.await_container {
            s.wait = Some(AuraWait::Container {
                pos,
                remaining: s.settings.aura.timeout,
            });
        } else {
            s.interacted.insert(pos);
            s.target = None;
            s.wait = Some(AuraWait::Delay(s.settings.aura.delay));
        }
    }
    pub(crate) fn chest_stealer_render(
        &self,
        _host: &dyn Host,
        query: &str,
        draw: &mut String,
    ) -> bool {
        let Ok(v) = serde_json::from_str::<Value>(query) else {
            return false;
        };
        if !v["kind"]
            .as_str()
            .is_some_and(|s| s.starts_with("chestStealer"))
        {
            return false;
        }
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let running = b.query_running(&state.registry, "ChestStealer");
        let s = &mut b.chest_stealer;
        if v["kind"] == "chestStealerScreen" {
            let translated = serde_json::from_value::<Vec<Vec<String>>>(v["translated"].clone())
                .unwrap_or_default();
            s.should_hide = running
                && s.settings.silent.enabled
                && v["abstract"] == true
                && s.settings.inventory.accepts_screen(
                    v["inventory"] == true,
                    v["menu"].as_str(),
                    v["title"].as_str().unwrap_or(""),
                    &translated,
                );
        }
        if v["kind"] == "chestStealerUseOn" {
            let positions =
                serde_json::from_value::<Vec<[i32; 3]>>(v["positions"].clone()).unwrap_or_default();
            if running && s.settings.silent.enabled {
                s.last_block = positions.first().copied();
            }
        }
        let p = &s.settings.silent;
        *draw=json!({"hidden":running&&p.enabled&&s.should_hide,"unlock":p.unlock,"draw":p.draw,"texture":p.texture,"color":p.color,"outline":p.outline,"margin":p.margin,"radius":p.radius,"gradient":p.gradient,"glow":p.glow,"scale":p.scale,"offset":p.offset,"title":p.title,"pos":s.last_block}).to_string();
        true
    }
}
