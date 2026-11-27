// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{inventory::InventoryActor, runtime::LiquidBounceHooks};
use horizon_client::mod_api::{Host, KeyCode};
use liquidbounce_behaviors::{
    inventory_manager::{Action, Context, Input, Outcome, OwnerState, Request, Slot},
    offhand::{self, Item, Plan},
};
use serde_json::{Value, json};
pub const OWNER: u64 = 0x4f46_4648_414e_4400;
struct Pending {
    id: u64,
    plan: Plan,
}
#[derive(Default)]
pub struct OffhandRuntime {
    pub state: offhand::State,
    pending: Option<Pending>,
    context: offhand::Context,
}
impl OffhandRuntime {
    pub fn enable(&mut self, crystal_aura: bool) {
        self.state.enable(&offhand::Context {
            crystal_aura,
            ..Default::default()
        });
    }
    pub fn cancel(&mut self, inventory: &mut InventoryActor) {
        inventory.manager.cancel_pending(OWNER);
        if let Some(p) = self.pending.take() {
            inventory.manager.end_plan(p.id);
        }
    }
    pub fn world_change(&mut self) {
        self.pending = None;
    }
    pub fn owner(&self, running: bool, c: &Context) -> (u64, OwnerState) {
        let guard = self.pending.as_ref().is_none_or(|p| {
            self.state.may_continue(
                &self.context,
                &p.plan,
                c.player_stacks
                    .get(usize::from(p.plan.slot))
                    .map(|s| (s.components.as_str(), s.count)),
            )
        });
        (
            OWNER,
            OwnerState {
                running,
                guards_pass: guard,
                constraints: self.state.settings.constraints.clone(),
            },
        )
    }
    pub fn outcome(&mut self, inventory: &mut InventoryActor, outcome: &Outcome) {
        match outcome {
            Outcome::Completed { request }
                if self.pending.as_ref().is_some_and(|p| p.id == *request) =>
            {
                let p = self.pending.take().unwrap();
                self.state.complete(
                    &p.plan,
                    crate::behaviors::current_time_millis(),
                    liquidbounce_behaviors::random::random_unit_f32(),
                );
                inventory.manager.end_plan(p.id);
            }
            Outcome::Cancelled { request }
                if self.pending.as_ref().is_some_and(|p| p.id == *request) =>
            {
                let p = self.pending.take().unwrap();
                inventory.manager.end_plan(p.id);
            }
            _ => {}
        }
    }
}
fn number(v: &Value, name: &str) -> f64 {
    v[name].as_f64().unwrap_or(0.)
}
fn items(v: &Value, c: &Context) -> Vec<Item> {
    let mut items = vec![Item::default(); 41];
    for (i, stack) in c.player_stacks.iter().take(41).enumerate() {
        items[i] = Item {
            item: serde_json::from_str::<Value>(&stack.components)
                .ok()
                .and_then(|v| v[0].as_str().map(str::to_owned))
                .unwrap_or_else(|| "minecraft:air".into()),
            components: stack.components.clone(),
            count: stack.count,
            ..Default::default()
        };
    }
    if let Some(records) = v["items"].as_array() {
        for r in records {
            let Some(slot) = r["player"].as_u64().map(|i| i as usize).filter(|&i| i < 41) else {
                continue;
            };
            let f = &r["facts"];
            let s = &mut items[slot];
            s.item = f["item"].as_str().unwrap_or("minecraft:air").into();
            s.death_protection =
                serde_json::from_str::<Value>(f["components"].as_str().unwrap_or("null"))
                    .ok()
                    .is_some_and(|v| v[1].get("minecraft:death_protection").is_some());
            s.strength = f["effects"].as_array().is_some_and(|effects| {
                effects
                    .iter()
                    .any(|e| matches!(e["id"].as_str(), Some("strength" | "minecraft:strength")))
            });
            s.valid_block = f["block"]["valid"] == true;
        }
    }
    items
}
impl LiquidBounceHooks {
    fn offhand_snapshot(&self, host: &mut dyn Host) -> Option<(offhand::Context, Value)> {
        let (settings, policy) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.offhand.state.settings.clone(),
                b.item_policy.block_selection.clone(),
            )
        };
        let query = json!({"entities":settings.entities,"blocks":settings.blocks&&settings.sphere_initialized,"fall":settings.fall,"policy":policy});
        let v: Value =
            serde_json::from_str(&host.client_offhand_context(&query.to_string())?).ok()?;
        let mut c = offhand::Context {
            now: crate::behaviors::current_time_millis(),
            creative: v["creative"] == true,
            spectator: v["spectator"] == true,
            dead: v["dead"] == true,
            health: number(&v, "health") as f32,
            max_health: number(&v, "maxHealth") as f32,
            missing_armor: v["missingArmor"] == true,
            burrowed: v["burrowed"] == true,
            hole: v["hole"] == true,
            fall_distance: number(&v, "fallDistance"),
            fall_flying_pose: v["fallFlying"] == true,
            fall_damage: number(&v, "fallDamage") as f32,
            sword: v["sword"] == true,
            strength_effect: v["strengthEffect"] == true,
            modern: v["modern"] == true,
            selected: v["selected"].as_u64().unwrap_or(0) as u8,
            ..Default::default()
        };
        c.entity_damage = v["entityDamage"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_f64().map(|d| d as f32))
                    .collect()
            })
            .unwrap_or_default();
        c.block_damage = v["blockDamage"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_f64().map(|d| d as f32))
                    .collect()
            })
            .unwrap_or_default();
        {
            let s = self.state();
            let b = s.behaviors.borrow();
            c.kill_aura = b.query_running(&s.registry, "KillAura");
            c.crystal_aura = b.query_running(&s.registry, "CrystalAura");
            c.no_fall = b.query_running(&s.registry, "NoFall");
            c.scaffold_enabled = b
                .query_ids
                .get("Scaffold")
                .is_some_and(|&id| s.registry.enabled_without_world(id));
            c.eagle_enabled = b
                .query_ids
                .get("Eagle")
                .is_some_and(|&id| s.registry.enabled_without_world(id));
        }
        if let Some(candidates) = v["candidates"].as_array() {
            for e in candidates {
                let Some(id) = e["id"].as_i64().and_then(|id| i32::try_from(id).ok()) else {
                    continue;
                };
                let target_json = if e["target"].is_object() {
                    e["target"].to_string()
                } else {
                    host.client_targeting_entity(id)
                };
                let Some(target) = crate::targeting::snapshot(&target_json) else {
                    continue;
                };
                let valid = {
                    let mut s = self.state();
                    let info = s.target_info(id, &target);
                    s.global_targets
                        .combat
                        .should_be_attacked(&target.entity, info)
                };
                if !valid {
                    continue;
                }
                let range = f64::from(settings.urgency_range);
                let in_cube = (0..3).all(|i| {
                    v["eye"][i].as_f64().is_some_and(|eye| {
                        e["min"][i].as_f64().is_some_and(|lo| lo < eye + range)
                            && e["max"][i].as_f64().is_some_and(|hi| hi > eye - range)
                    })
                });
                let distance = number(e, "distance");
                if in_cube
                    && distance >= 0.
                    && distance <= f64::from(settings.urgency_range * settings.urgency_range)
                {
                    c.enemy_distance_squared.push(distance);
                }
                c.mace_diving |= target.entity.kind.living
                    && e["mace"] == true
                    && number(e, "above") > 1.
                    && number(e, "drop") >= f64::from(settings.mace_speed)
                    && number(e, "horizontal") <= f64::from(settings.mace_range);
            }
        }
        Some((c, v))
    }
    pub(crate) fn offhand_refresh(&self, host: &mut dyn Host) {
        let running = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.query_running(&s.registry, "Offhand")
        };
        if running {
            if let Some((c, _)) = self.offhand_snapshot(host) {
                self.state().behaviors.borrow_mut().offhand.context = c;
            }
        }
    }
    pub(crate) fn offhand_key(&self, host: &mut dyn Host, key: KeyCode) {
        let running = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.query_running(&s.registry, "Offhand")
        };
        if !running {
            return;
        }
        let Some(key) = horizon_client::keybinds::key_name(key) else {
            return;
        };
        let Some((c, _)) = self.offhand_snapshot(host) else {
            return;
        };
        self.state()
            .behaviors
            .borrow_mut()
            .offhand
            .state
            .key(key, &c);
    }
    pub(crate) fn offhand_plan(&self, host: &mut dyn Host, c: &Context) {
        let running = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.query_running(&s.registry, "Offhand")
        };
        if !running {
            return;
        }
        if self.state().behaviors.borrow().offhand.pending.is_some() {
            return;
        }
        let Some((context, v)) = self.offhand_snapshot(host) else {
            return;
        };
        let items = items(&v, c);
        let (plan, tag_changed) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let before = b.offhand.state.active;
            let plan = b.offhand.state.plan(&context, &items);
            b.offhand.context = context;
            let operating = b.offhand.state.operating(true);
            b.item_policy.offhand_operating = operating;
            (plan, before != b.offhand.state.active)
        };
        #[cfg(feature = "webview")]
        if tag_changed {
            self.state()
                .events
                .borrow_mut()
                .push(("refreshArrayList".into(), json!({})));
        }
        let Some(plan) = plan else {
            return;
        };
        if plan.smart {
            host.client_offhand_smart(
                i32::from(plan.slot),
                v["selected"].as_i64().unwrap_or(0) as i32,
            );
        } else if plan.direct {
            for action in &plan.actions {
                let (slot, button, input) = match *action {
                    offhand::Action::Swap(slot) => (slot, 40, 2),
                    offhand::Action::Pickup(slot) => (slot, 0, 0),
                    _ => continue,
                };
                if let Some(slot) = Slot::Player(slot).server_id(None) {
                    host.client_inventory_click(0, slot, button, input);
                }
            }
        } else {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let b = &mut *b;
            let id = b.inventory.id();
            let actions = plan
                .actions
                .iter()
                .filter_map(|action| match *action {
                    offhand::Action::Swap(slot) => Some(Action::Click {
                        screen: None,
                        slot: Slot::Player(slot),
                        button: 40,
                        input: Input::Swap,
                    }),
                    offhand::Action::Pickup(slot) => Some(Action::Click {
                        screen: None,
                        slot: Slot::Player(slot),
                        button: 0,
                        input: Input::Pickup,
                    }),
                    _ => None,
                })
                .collect();
            b.inventory.manager.submit(Request::new(
                id,
                OWNER,
                0,
                actions,
                c,
                &b.inventory.manager,
            ));
            b.offhand.pending = Some(Pending { id, plan });
            return;
        }
        self.state().behaviors.borrow_mut().offhand.state.complete(
            &plan,
            crate::behaviors::current_time_millis(),
            liquidbounce_behaviors::random::random_unit_f32(),
        );
    }
}
