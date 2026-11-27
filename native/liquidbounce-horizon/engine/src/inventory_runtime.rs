// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared InventoryManager native executor. Every Host call is outside the
//! registry borrow, including synchronous packet listeners and plan tails.
use crate::{
    auto_tool,
    auto_tool_packets::Observation,
    inventory::{self, Snapshot},
    runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    inventory_manager::{Action, Context, CreativeStack, Effect, Input, OwnerState},
    random::with_thread_random,
};

pub(crate) const REQUESTER: u64 = inventory::AUTO_TOOL_OWNER;

impl LiquidBounceHooks {
    /// The inventoryPlan continuation sets enabled=false before perform resumes.
    /// Registry callbacks may borrow behaviors again, so no actor borrow survives.
    fn elytra_swap_disable(&self, host: &dyn Host) {
        let options = crate::runtime::raw_options(host);
        let mut state = self.state();
        let id = state.behaviors.borrow().query_ids["ElytraSwap"];
        match state.with_registry(options, |registry, callbacks| {
            registry.set_enabled_sync(id, false, callbacks)
        }) {
            Ok(report) if report.failures.is_empty() => {}
            result => eprintln!("[LiquidBounce] ElytraSwap disable: {result:?}"),
        }
    }
    pub(crate) fn reset_inventory(&self) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        modules.inventory.world_change();
        modules.auto_tool.swaps.cancel_plan();
        modules.book_bot.pending=None;
        modules.chest_cleaner.clear();
        modules.auto_deposit.clear();
        modules.replenish.world_change();
        modules.offhand.world_change();
        modules.item_policy.offhand_operating=false;
        modules.inventory_cleaner.clear();
        modules.auto_wind_charge.clear();
        modules.auto_buff.clear();
        modules.auto_bow.clear();
        modules.auto_pearl.clear();
        modules.easy_pearl.behavior.disabled();
        modules.auto_bow.clear();
        modules.auto_rod.clear();
        modules.aimbot.clear();
        modules.kill_aura.clear();
        modules.kill_aura.release_on_disable=false;
        modules.auto_wind_charge.clear();
        modules.auto_armor.clear();
        modules.chest_stealer.clear();
        modules.inventory_packets.world_change();
    }
    fn inventory_publish(&self, in_game: bool) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        modules.inventory_packets.publish(
            in_game,
            modules.inventory.manager.inventory_open_server_side,
        );
    }
    fn inventory_register_waits(&self) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let waits = modules.inventory.manager.take_wait_registrations();
        for callback in waits {
            modules.coroutine_ticker.register(
                crate::coroutine_ticker::WaitOwner::InventoryManager,
                callback,
            );
        }
    }
    fn inventory_snapshot(&self, host: &mut dyn Host) -> Option<Snapshot> {
        let (combat, move_cursor) = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            (
                state.combat.in_combat(),
                modules
                    .better_inventory
                    .should_move_cursor(modules.query_running(&state.registry, "BetterInventory")),
            )
        };
        match Snapshot::parse(&host.client_inventory_snapshot(), combat, move_cursor) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                eprintln!("[LiquidBounce] inventory snapshot: {error}");
                None
            }
        }
    }
    fn tool_snapshot(
        &self,
        host: &dyn Host,
        pos: [i32; 3],
        scan_beds: bool,
    ) -> Option<auto_tool::Snapshot> {
        match auto_tool::Snapshot::parse(&host.client_auto_tool_snapshot(pos, scan_beds)) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                eprintln!("[LiquidBounce] AutoTool snapshot: {error}");
                None
            }
        }
    }
    fn drain_inventory_observations(&self, host: &dyn Host) {
        let packets = self.state().behaviors.borrow().inventory_packets.clone();
        let Some(session) = host.client_session() else {
            packets.world_change();
            return;
        };
        for observation in packets.drain(session) {
            let payload = match observation {
                Observation::UseOn(pos) => {
                    self.chest_stealer_manual(pos);
                    if !host.client_auto_tool_menu_provider(pos) {
                        continue;
                    }
                    serde_json::json!({"kind":"mayOpen"})
                }
                Observation::MayOpen => serde_json::json!({"kind":"mayOpen"}),
                Observation::Click(container) => {
                    serde_json::json!({"kind":"click","container":container})
                }
                Observation::Close => serde_json::json!({"kind":"close"}),
            };
            self.inventory_observe_payload(host, &payload.to_string());
        }
    }
    fn inventory_observe_payload(&self, host: &dyn Host, payload: &str) {
        let now = host.client_auto_tool_time();
        if let Err(error) = self
            .state()
            .behaviors
            .borrow_mut()
            .inventory
            .observe(payload, now)
        {
            eprintln!("[LiquidBounce] inventory observation: {error}");
        }
        self.inventory_publish(true);
    }
    pub(crate) fn auto_tool_observe(&self, host: &dyn Host, payload: &str) {
        if serde_json::from_str::<serde_json::Value>(payload).ok().is_some_and(|v|v["kind"]=="replenishScreen") {self.replenish_screen(payload);return;}
        self.drain_inventory_observations(host);
        self.inventory_observe_payload(host, payload);
    }
    /// InventoryPlan handlers run once per GameTick while their previous plan
    /// is suspended. They submit to the same queue before the manager drains.
    pub(crate) fn auto_tool_tick(&self, host: &mut dyn Host) {
        self.drain_inventory_observations(host);
        let Some(snapshot) = self.inventory_snapshot(host) else {
            self.inventory_publish(false);
            return;
        };
        let tools = self.tool_snapshot(host, [0; 3], false);
        let disable_elytra_swap = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let auto_running = modules.query_running(&state.registry, "AutoTool");
            let vomit_running = modules.query_running(&state.registry, "Vomit");
            let elytra_swap_running = modules.query_running(&state.registry, "ElytraSwap");
            let modules = &mut *modules;
            modules.inventory.begin_tick(&snapshot.context);
            if let Some(tools) = tools.as_ref() {
                modules.inventory.submit_auto_tool(
                    &mut modules.auto_tool,
                    auto_running,
                    tools,
                    &snapshot.context,
                );
            }
            if let Err(error) = with_thread_random(|random| {
                modules
                    .inventory
                    .submit_vomit(&mut modules.vomit, vomit_running, &snapshot, random)
            }) {
                eprintln!("[LiquidBounce] Vomit plan: {error}");
            }
            match modules.inventory.submit_elytra_swap(
                &mut modules.elytra_swap,
                elytra_swap_running,
                &snapshot,
            ) {
                Ok(disable) => disable,
                Err(error) => {
                    eprintln!("[LiquidBounce] ElytraSwap plan: {error}");
                    false
                }
            }
        };
        self.book_bot_plan(host,&snapshot.context);
        self.chest_cleaner_plan(host,&snapshot.context);
        self.auto_deposit_plan(host,&snapshot.context);
        self.replenish_plan(&snapshot.context);
        self.auto_armor_plan(host,&snapshot.context);
        self.offhand_plan(host,&snapshot.context);
        self.inventory_cleaner_plan(host,&snapshot.context);
        self.auto_buff_plan(host,&snapshot.context);
        self.chest_stealer_plan(host,&snapshot.context);
        if disable_elytra_swap {
            self.elytra_swap_disable(host);
        }
        self.inventory_publish(true);
        self.inventory_frame(host);
    }
    pub(crate) fn auto_tool_progress(&self, host: &dyn Host, position: [i32; 3]) {
        self.drain_inventory_observations(host);
        let scan_beds = self.state().behaviors.borrow().auto_tool.require_near_bed;
        let Some(snapshot) = self.tool_snapshot(host, position, scan_beds) else {
            return;
        };
        let switch = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "AutoTool");
            modules
                .auto_tool
                .progress(
                    running,
                    snapshot.mining_context(position, state.combat.in_combat()),
                    &snapshot.stacks,
                )
                .map(|slot| (slot, modules.auto_tool.swap_back_delay))
        };
        if let Some((slot, ticks)) = switch {
            if let Err(error) = host.client_select_slot_silently(REQUESTER, i32::from(slot), ticks)
            {
                eprintln!("[LiquidBounce] AutoTool slot selection: {error}");
            }
        }
        #[cfg(feature = "webview")]
        self.state().probe(
            "clientBlockBreakingProgress",
            serde_json::json!({"position":position,"block":snapshot.block}),
            serde_json::json!({"slot":switch.map(|s|s.0)}),
        );
    }
    pub(crate) fn auto_tool_frame(&self, host: &mut dyn Host) {
        self.inventory_frame(host);
    }
    pub(crate) fn inventory_callback(
        &self,
        host: &mut dyn Host,
        _in_game: bool,
        callback: u64,
    ) -> bool {
        self.state()
            .behaviors
            .borrow_mut()
            .inventory
            .manager
            .advance_callback(callback);
        self.inventory_frame(host);
        false
    }
    pub(crate) fn inventory_frame(&self, host: &mut dyn Host) {
        loop {
            self.drain_inventory_observations(host);
            let snapshot = self.inventory_snapshot(host);
            let context = snapshot
                .as_ref()
                .map(|s| s.context.clone())
                .unwrap_or_else(|| Context {
                    now: host.client_auto_tool_time(),
                    ..Context::default()
                });
            self.offhand_refresh(host);
            let tools = self.tool_snapshot(host, [0; 3], false);
            let book_bot_selected=host.client_book_bot_selected();
            let emission = {
                let state = self.state();
                let mut modules = state.behaviors.borrow_mut();
                let auto_running = modules.query_running(&state.registry, "AutoTool");
                let vomit_running = modules.query_running(&state.registry, "Vomit");
                let elytra_swap_running = modules.query_running(&state.registry, "ElytraSwap");
                let mut owners = tools
                    .as_ref()
                    .map(|tools| {
                        modules
                            .inventory
                            .owners(
                                &modules.auto_tool,
                                auto_running,
                                tools,
                                &modules.vomit,
                                vomit_running,
                                context.creative,
                            )
                            .to_vec()
                    })
                    .unwrap_or_default();
                owners.push(
                    modules
                        .inventory
                        .elytra_swap_owner(&modules.elytra_swap, elytra_swap_running),
                );
                owners.push(modules.book_bot.owner(modules.query_running(&state.registry,"BookBot"),book_bot_selected));
                owners.push(modules.chest_cleaner.owner(modules.query_running(&state.registry,"ChestCleaner")));
                owners.push(modules.auto_deposit.owner(modules.query_running(&state.registry,"AutoDeposit")));
                owners.push(modules.replenish.owner(modules.query_running(&state.registry,"Replenish"),&context));
                owners.push(modules.inventory_cleaner.owner(modules.query_running(&state.registry,"InventoryCleaner")));
                owners.push(modules.chest_stealer.owner(modules.query_running(&state.registry,"ChestStealer")));
                owners.push(modules.auto_armor.owner(modules.query_running(&state.registry,"AutoArmor")));
                owners.push(modules.offhand.owner(modules.query_running(&state.registry,"Offhand"),&context));
                owners.extend(modules.auto_buff.owners(modules.query_running(&state.registry,"AutoBuff")));
                modules.inventory.observe_context(&context);
                with_thread_random(|random| {
                    modules.inventory.manager.step(
                        &context,
                        &mut |id| {
                            owners
                                .iter()
                                .find(|(owner, _)| *owner == id)
                                .map(|(_, state)| state.clone())
                                .unwrap_or(OwnerState {
                                    running: false,
                                    guards_pass: false,
                                    constraints: Default::default(),
                                })
                        },
                        random,
                    )
                })
            };
            self.inventory_register_waits();
            self.inventory_publish(context.in_game);
            let emission = match emission {
                Err(error) => {
                    eprintln!("[LiquidBounce] inventory coroutine: {error}");
                    continue;
                }
                Ok(None) => {
                    let next = self
                        .state()
                        .behaviors
                        .borrow()
                        .inventory
                        .manager
                        .next_wakeup(context.now);
                    host.client_inventory_schedule(next);
                    return;
                }
                Ok(Some(emission)) => emission,
            };
            let succeeded = match &emission.effect {
                Effect::OpenInventory => true, // native 26.3's send1_11_1OpenInventory is a no-op
                Effect::CloseInventory => {
                    host.client_inventory_close(0, false);
                    true
                }
                Effect::MoveCursor(cursor) => {
                    host.client_inventory_move_cursor([cursor.x, cursor.y]);
                    #[cfg(feature = "webview")]
                    {
                        let actual = serde_json::from_str::<serde_json::Value>(
                            &host.client_inventory_snapshot(),
                        )
                        .unwrap_or(serde_json::Value::Null);
                        self.state().probe(
                            "clientInventoryMoveCursor",
                            serde_json::json!([cursor.x, cursor.y]),
                            actual["visibleCursor"].clone(),
                        );
                    }
                    true
                }
                Effect::Action(Action::Click {
                    screen,
                    slot,
                    button,
                    input,
                }) => slot.server_id(screen.as_ref()).is_some_and(|slot| {
                    let input = match input {
                        Input::Pickup => 0,
                        Input::QuickMove => 1,
                        Input::Swap => 2,
                        Input::Clone => 3,
                        Input::Throw => 4,
                        Input::QuickCraft => 5,
                        Input::PickupAll => 6,
                    };
                    host.client_inventory_click(
                        screen.as_ref().map_or(0, |screen| screen.container_id),
                        slot,
                        *button,
                        input,
                    )
                }),
                Effect::Action(Action::Creative { stack, slot }) => {
                    let slot = slot.map(|slot| slot.server_id(None));
                    if slot == Some(None) {
                        false
                    } else {
                        let slot = slot.flatten();
                        match stack {
                            CreativeStack::Block {
                                registry_index,
                                count,
                            } => host.client_inventory_creative(
                                Some(*registry_index),
                                "",
                                *count,
                                slot,
                            ),
                            CreativeStack::Existing(stack) => {
                                host.client_inventory_creative(None, &stack.wire, stack.count, slot)
                            }
                        }
                    }
                }
                Effect::Action(Action::CloseScreen { container_id }) => {
                    host.client_inventory_close(*container_id, true);
                    true
                }
                Effect::Action(Action::UseItem { slot }) => {
                    if *slot < 9 {
                        // InventoryAction.UseItem ignores the cancellable
                        // silent-selection result and still invokes useItem.
                        let _ = host.client_select_slot_silently(0, i32::from(*slot), 1);
                    }
                    (*slot < 9 || *slot == 40)
                        && host.client_inventory_use(if *slot == 40 { 1 } else { 0 })
                }
                Effect::Outcome(outcome) => {
                    let disable = {
                        let state = self.state();
                        let mut modules = state.behaviors.borrow_mut();
                        let auto_running = modules.query_running(&state.registry, "AutoTool");
                        let modules = &mut *modules;
                        modules.inventory.outcome(
                            outcome,
                            &mut modules.auto_tool,
                            auto_running,
                            &context,
                        );
                        modules.inventory.elytra_swap_outcome(outcome)
                    };
                    let book_completed={let state=self.state();let mut b=state.behaviors.borrow_mut();let b=&mut *b;b.book_bot.outcome(&mut b.inventory,outcome)};
                    if book_completed{self.book_bot_write_due(host);}
                    {let state=self.state();let mut b=state.behaviors.borrow_mut();let b=&mut *b;b.chest_cleaner.outcome(&mut b.inventory,outcome,&context);}
                    self.auto_deposit_outcome(host,outcome,&context);
                    self.replenish_outcome(outcome,&context);
                    self.auto_buff_outcome(host,outcome,&context);
                    self.auto_armor_outcome(host,outcome,&context);
                    {let s=self.state();let mut b=s.behaviors.borrow_mut();let b=&mut*b;b.offhand.outcome(&mut b.inventory,outcome);}
                    self.chest_stealer_outcome(host,outcome,&context);
                    {let state=self.state();let mut modules=state.behaviors.borrow_mut();let modules=&mut*modules;modules.inventory_cleaner.outcome(&mut modules.inventory,outcome,&context);}
                    if disable {
                        self.elytra_swap_disable(host);
                    }
                    true
                }
            };
            // Reentrant callbacks above may have changed the world or menu.
            // onPerformed expects exactly the state after the actual action.
            self.drain_inventory_observations(host);
            let after = self
                .inventory_snapshot(host)
                .map(|s| s.context)
                .unwrap_or_else(|| Context {
                    now: host.client_auto_tool_time(),
                    ..Context::default()
                });
            self.state()
                .behaviors
                .borrow_mut()
                .inventory
                .manager
                .acknowledge(emission.ticket, succeeded, &after);
            self.inventory_register_waits();
            self.inventory_publish(after.in_game);
        }
    }
}
