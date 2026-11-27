// SPDX-License-Identifier: GPL-3.0-or-later
//! Native AutoTool operations are executed without holding the registry lock.
use crate::{auto_tool::Snapshot, auto_tool_packets::Observation, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::auto_tool::inventory::{Operation, Random};

pub(crate) const REQUESTER: u64 = 0x4155_544f_544f_4f4c;
struct InventoryRandom;
impl Random for InventoryRandom {
    fn int(&mut self, range: [i32;2]) -> Result<i32,String> {
        if range[0] > range[1] { return Err("empty inventory delay range".into()); }
        Ok(liquidbounce_behaviors::random::random_i32_inclusive(range[0],range[1]))
    }
    fn float(&mut self) -> f32 { liquidbounce_behaviors::random::random_unit_f32() }
}
fn snapshot(host: &dyn Host, pos: [i32;3], scan_beds: bool) -> Option<Snapshot> {
    match Snapshot::parse(&host.client_auto_tool_snapshot(pos,scan_beds)) {
        Ok(snapshot) => snapshot,
        Err(error) => { eprintln!("[LiquidBounce] AutoTool snapshot: {error}"); None }
    }
}

impl LiquidBounceHooks {
    pub(crate) fn reset_inventory(&self) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        modules.inventory.world_change();
        modules.auto_tool.swaps.cancel_plan();
        modules.inventory_packets.world_change();
    }
    fn drain_inventory_observations(&self, host: &dyn Host) {
        let packets = self.state().behaviors.borrow().inventory_packets.clone();
        let Some(session) = host.client_session() else { packets.world_change(); return; };
        for observation in packets.drain(session) {
            let kind = match observation {
                Observation::UseOn(pos) => {
                    if !host.client_auto_tool_menu_provider(pos) { continue; }
                    "mayOpen"
                },
                Observation::MayOpen => "mayOpen",
                Observation::Click => "click",
                Observation::Close => "close",
            };
            self.auto_tool_observe_payload(host, &serde_json::json!({"kind":kind}).to_string());
        }
    }
    fn auto_tool_observe_payload(&self, host: &dyn Host, payload: &str) {
        let now = host.client_auto_tool_time();
        if let Err(error) = self.state().behaviors.borrow_mut().inventory.observe(payload,now) {
            eprintln!("[LiquidBounce] inventory observation: {error}");
        }
    }
    pub(crate) fn auto_tool_observe(&self, host: &dyn Host, payload: &str) {
        self.drain_inventory_observations(host);
        self.auto_tool_observe_payload(host,payload);
    }
    pub(crate) fn auto_tool_tick(&self, host: &dyn Host) {
        self.drain_inventory_observations(host);
        let Some(snapshot) = snapshot(host,[0;3],false) else { return; };
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        let running = modules.query_running(&state.registry,"AutoTool");
        let modules = &mut *modules;
        modules.inventory.tick(&mut modules.auto_tool,running,&snapshot,state.combat.in_combat());
    }
    pub(crate) fn auto_tool_progress(&self, host: &dyn Host, position: [i32;3]) {
        self.drain_inventory_observations(host);
        let scan_beds = self.state().behaviors.borrow().auto_tool.require_near_bed;
        let Some(snapshot) = snapshot(host,position,scan_beds) else { return; };
        let switch = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry,"AutoTool");
            modules.auto_tool.progress(running,snapshot.mining_context(position,state.combat.in_combat()),&snapshot.stacks)
                .map(|slot| (slot,modules.auto_tool.swap_back_delay))
        };
        if let Some((slot,ticks)) = switch {
            if let Err(error) = host.client_select_slot_silently(REQUESTER,i32::from(slot),ticks) {
                eprintln!("[LiquidBounce] AutoTool slot selection: {error}");
            }
        }
        #[cfg(feature="webview")]
        self.state().probe("clientBlockBreakingProgress",serde_json::json!({"position":position,"block":snapshot.block}),serde_json::json!({"slot":switch.map(|s|s.0)}));
    }
    pub(crate) fn auto_tool_frame(&self, host: &mut dyn Host) {
        // A completed click may immediately close with a zero delay. A new
        // chain still starts only on GameTick, and cursor travel yields a frame.
        for step in 0..2 {
            self.drain_inventory_observations(host);
            let Some(snapshot) = snapshot(host,[0;3],false) else { return; };
            let (operation,cursor,in_combat) = {
                let state = self.state();
                let mut modules = state.behaviors.borrow_mut();
                let running = modules.query_running(&state.registry,"AutoTool");
                let modules = &mut *modules;
                let in_combat = state.combat.in_combat();
                let operation = modules.inventory.frame(&mut modules.auto_tool,running,&snapshot,in_combat,&mut InventoryRandom);
                (operation,modules.inventory.executor.cursor,in_combat)
            };
            match operation {
                Err(error) => { eprintln!("[LiquidBounce] inventory operation: {error}"); return; },
                Ok(None) => return,
                Ok(Some(Operation::MoveCursor(cursor))) => { host.client_auto_tool_move_cursor(cursor); return; },
                Ok(Some(Operation::Close)) => { host.client_auto_tool_close(); return; },
                Ok(Some(Operation::Swap(action))) => {
                    if let Some(cursor) = cursor { host.client_auto_tool_move_cursor(cursor); }
                    let succeeded = host.client_auto_tool_swap(i32::from(action.source),i32::from(action.hotbar));
                    let state = self.state();
                    let mut modules = state.behaviors.borrow_mut();
                    let modules = &mut *modules;
                    if let Err(error) = modules.inventory.performed(&mut modules.auto_tool,&snapshot,in_combat,action.source,succeeded,&mut InventoryRandom) {
                        eprintln!("[LiquidBounce] inventory completion: {error}");
                        return;
                    }
                    if step != 0 { return; }
                },
            }
        }
    }
}
