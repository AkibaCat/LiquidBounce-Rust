// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{inventory::InventoryActor, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_deposit::AutoDeposit,
    inventory_manager::{Action, Context, Input, Outcome, OwnerState, Request, Screen, Slot},
    no_rotate_set::{RotationsSettings, SmoothMode},
    rotations::{DefaultRotationTarget, Rotation},
};
use serde_json::{Value, json};
use std::collections::VecDeque;
pub const OWNER: u64 = 0x4155_544f_4445_504f;
pub const REQUESTER: u64 = 0x5055_4e43_4844_4550;
pub const ROTATION_PROVIDER: u64 = 11;
pub const PROCESSOR_BASE: u64 = 120;
pub struct AutoDepositRuntime {
    pub settings: AutoDeposit,
    pub rotations: RotationsSettings,
    pub pending: Option<u64>,
    slots: VecDeque<u8>,
    screen: Option<Screen>,
    deposited: i32,
    closing: bool,
    pub target: Option<Value>,
    pub wait: i32,
    pub waiting: bool,
    pub callback: u64,
}
impl Default for AutoDepositRuntime {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            rotations: Default::default(),
            pending: None,
            slots: Default::default(),
            screen: None,
            deposited: -1,
            closing: false,
            target: None,
            wait: 0,
            waiting: false,
            callback: 0,
        }
    }
}
impl AutoDepositRuntime {
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn clear(&mut self) {
        self.pending = None;
        self.slots.clear();
        self.screen = None;
        self.deposited = -1;
        self.closing = false;
        self.target = None;
        self.waiting = false;
        self.wait = 0;
        self.callback = self.callback.wrapping_add(1);
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
    fn next(&mut self, inventory: &mut InventoryActor, context: &Context, capacities: &[i64]) {
        let (Some(id), Some(screen)) = (self.pending, self.screen.as_ref()) else {
            return;
        };
        if self.closing {
            self.pending = None;
            self.screen = None;
            inventory.manager.end_plan(id);
            return;
        }
        while let Some(slot) = self.slots.pop_front() {
            if capacities.get(usize::from(slot)).copied().unwrap_or(0) > 0 {
                inventory.manager.submit(Request::new(
                    id,
                    OWNER,
                    0,
                    vec![Action::Click {
                        screen: Some(screen.clone()),
                        slot: Slot::Player(slot),
                        button: 0,
                        input: Input::QuickMove,
                    }],
                    context,
                    &inventory.manager,
                ));
                return;
            }
        }
        if self.settings.inventory.auto_close && self.deposited == screen.container_id {
            self.closing = true;
            inventory.manager.submit(Request::new(
                id,
                OWNER,
                0,
                vec![Action::CloseScreen {
                    container_id: screen.container_id,
                }],
                context,
                &inventory.manager,
            ));
        } else {
            self.pending = None;
            self.screen = None;
            inventory.manager.end_plan(id);
        }
    }
    pub fn outcome(
        &mut self,
        inventory: &mut InventoryActor,
        outcome: &Outcome,
        context: &Context,
        capacities: &[i64],
    ) {
        match outcome {
            Outcome::Performed { request, action } if self.pending == Some(*request) => {
                if matches!(action, Action::Click { .. }) {
                    self.deposited = self.screen.as_ref().map_or(-1, |s| s.container_id);
                }
                if matches!(action, Action::CloseScreen { .. }) {
                    self.deposited = -1;
                }
            }
            Outcome::Completed { request } if self.pending == Some(*request) => {
                self.next(inventory, context, capacities)
            }
            Outcome::Cancelled { request } if self.pending == Some(*request) => {
                let id = *request;
                self.pending = None;
                self.slots.clear();
                self.screen = None;
                self.closing = false;
                inventory.manager.end_plan(id);
            }
            _ => {}
        }
    }
}
fn capacities(v: &Value) -> Vec<i64> {
    v["capacity"]
        .as_array()
        .map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0)).collect())
        .unwrap_or_default()
}
impl LiquidBounceHooks {
    pub(crate) fn auto_deposit_plan(&self, host: &mut dyn Host, context: &Context) {
        {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            if context.screen.is_none() {
                b.auto_deposit.deposited = -1;
            }
            if !b.query_running(&state.registry, "AutoDeposit") || b.auto_deposit.pending.is_some()
            {
                return;
            }
        }
        let Some(screen) = context.screen.as_ref() else {
            return;
        };
        let Some(v) = host
            .client_auto_deposit_context()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return;
        };
        let translated: Vec<Vec<String>> =
            serde_json::from_value(v["translated"].clone()).unwrap_or_default();
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let b = &mut *b;
        let s = &mut b.auto_deposit;
        if !s.settings.inventory.accepts_screen(
            context.inventory_screen,
            v["menu"].as_str(),
            v["title"].as_str().unwrap_or_default(),
            &translated,
        ) {
            return;
        }
        let items: Vec<_> = context
            .player_stacks
            .iter()
            .map(|s| {
                let item = serde_json::from_str::<Value>(&s.components)
                    .ok()
                    .and_then(|v| v[0].as_str().map(str::to_owned))
                    .unwrap_or_default();
                (item, s.empty())
            })
            .collect();
        s.slots = s.settings.matching(&items, true).into();
        s.screen = Some(screen.clone());
        s.closing = false;
        s.pending = Some(b.inventory.id());
        s.next(&mut b.inventory, context, &capacities(&v));
    }
    pub(crate) fn auto_deposit_outcome(
        &self,
        host: &mut dyn Host,
        outcome: &Outcome,
        context: &Context,
    ) {
        let v = host
            .client_auto_deposit_context()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or(Value::Null);
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let b = &mut *b;
        b.auto_deposit
            .outcome(&mut b.inventory, outcome, context, &capacities(&v));
    }
    fn auto_deposit_request(&self, host: &mut dyn Host) -> Option<Value> {
        let raw = host.client_inventory_snapshot();
        let snapshot = serde_json::from_str::<Value>(&raw).ok()?;
        let state = self.state();
        let b = state.behaviors.borrow();
        let s = &b.auto_deposit;
        if !s.settings.punch.running(
            b.query_running(&state.registry, "AutoDeposit"),
            state.combat.in_combat(),
            snapshot["using"] == true,
        ) {
            return None;
        }
        let items: Vec<_> = snapshot["stacks"]
            .as_array()?
            .iter()
            .map(|s| {
                let item = serde_json::from_str::<Value>(s["components"].as_str().unwrap_or(""))
                    .ok()
                    .and_then(|v| v[0].as_str().map(str::to_owned))
                    .unwrap_or_default();
                (item, s["count"].as_i64().unwrap_or(0) <= 0)
            })
            .collect();
        let slot = *s.settings.matching(&items, false).first()?;
        let p = &s.settings.punch;
        let rotation = b
            .rotations
            .lock()
            .expect("RotationManager")
            .server_rotation(false, false);
        Some(
            json!({"range":p.range,"walls":p.wall_range,"blocks":p.blocks,"rotate":p.rotate,"slot":slot,"swing":p.swing,"target":s.target,"rotation":[rotation.yaw,rotation.pitch]}),
        )
    }
    pub(crate) fn auto_deposit_prepare(&self, host: &mut dyn Host) {
        let Some(request) = self
            .auto_deposit_request(host)
            .filter(|v| v["rotate"] == true)
        else {
            return;
        };
        let result = host
            .client_auto_deposit_target(&request.to_string())
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        b.auto_deposit.target = result.clone();
        let Some(result) = result else {
            return;
        };
        let (Some(yaw), Some(pitch)) = (
            result["rotation"][0].as_f64(),
            result["rotation"][1].as_f64(),
        ) else {
            return;
        };
        let r = &b.auto_deposit.rotations;
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
    pub(crate) fn auto_deposit_punch_tick(&self, host: &mut dyn Host) {
        if self.state().behaviors.borrow().auto_deposit.waiting {
            return;
        }
        let Some(request) = self.auto_deposit_request(host) else {
            return;
        };
        if !host.client_auto_deposit_punch(&request.to_string()) {
            return;
        }
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let s = &mut b.auto_deposit;
        if request["rotate"] == true {
            s.target = None;
        }
        s.wait = s.settings.punch.delay;
        s.waiting = true;
        s.callback = s.callback.wrapping_add(1);
        let callback = s.callback;
        b.coroutine_ticker
            .register(crate::coroutine_ticker::WaitOwner::AutoDeposit, callback);
    }
    pub(crate) fn auto_deposit_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        id: u64,
    ) -> bool {
        {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            if b.auto_deposit.callback != id {
                return false;
            }
            if !in_game || !b.query_running(&state.registry, "AutoDeposit") {
                b.auto_deposit.waiting = false;
                return false;
            }
            let s = &mut b.auto_deposit;
            s.wait -= 1;
            if s.wait > 0 {
                return true;
            }
            s.waiting = false;
        }
        self.auto_deposit_punch_tick(host);
        false
    }
}
