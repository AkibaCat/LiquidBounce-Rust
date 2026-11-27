// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{inventory::InventoryActor, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    book_bot::BookBot,
    inventory_manager::{Action, Context, Input, Outcome, OwnerState, Request, Slot},
};
use serde_json::{Value, json};
pub const OWNER: u64 = 0x424f_4f4b_424f_5400;
#[derive(Default)]
pub struct BookBotRuntime {
    pub settings: BookBot,
    pub count: i32,
    pub last: i64,
    pub pending: Option<(u64, Option<i32>)>,
}
impl BookBotRuntime {
    pub fn enabled(&mut self, now: i64) {
        self.count = 0;
        self.last = now;
    }
    pub fn owner(&self, running: bool, selected: i32) -> (u64, OwnerState) {
        (
            OWNER,
            OwnerState {
                running,
                guards_pass: self
                    .pending
                    .is_none_or(|(_, capture)| capture.is_none_or(|c| c == selected)),
                constraints: self.settings.constraints.clone(),
            },
        )
    }
    pub fn outcome(&mut self, inventory: &mut InventoryActor, outcome: &Outcome) -> bool {
        let (request, completed) = match *outcome {
            Outcome::Completed { request } => (request, true),
            Outcome::Cancelled { request } => (request, false),
            Outcome::Performed { .. } => return false,
        };
        if !self.pending.is_some_and(|(id, _)| id == request) {
            return false;
        }
        self.pending = None;
        inventory.manager.end_plan(request);
        completed
    }
}
impl LiquidBounceHooks {
    pub(crate) fn book_bot_disable(&self, host: &dyn Host) {
        let options = crate::runtime::raw_options(host);
        let mut state = self.state();
        let id = state.behaviors.borrow().query_ids["BookBot"];
        if let Err(e) = state.with_registry(options, |r, c| r.set_enabled_sync(id, false, c)) {
            eprintln!("[LiquidBounce] BookBot disable: {e:?}");
        }
    }
    pub(crate) fn book_bot_plan(&self, host: &mut dyn Host, context: &Context) {
        {
            let state = self.state();
            let b = state.behaviors.borrow();
            if !b.query_running(&state.registry, "BookBot") || b.book_bot.pending.is_some() {
                return;
            }
        }
        let Some(payload) = host.client_book_bot_snapshot() else {
            return;
        };
        let Ok(snapshot) = serde_json::from_str::<Value>(&payload) else {
            return;
        };
        let selected = snapshot["selected"].as_u64().unwrap_or(0) as u8;
        let candidates = &snapshot["candidates"];
        let order = (0..9).chain(std::iter::once(40)).chain(9..40);
        let Some(book) = order
            .into_iter()
            .find(|i| candidates[*i].as_bool().unwrap_or(false))
        else {
            self.book_bot_disable(host);
            return;
        };
        let needs_swap = !candidates[usize::from(selected)].as_bool().unwrap_or(false);
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let b = &mut *b;
        let id = b.inventory.id();
        let actions = if needs_swap {
            vec![Action::Click {
                screen: None,
                slot: Slot::Player(book as u8),
                button: i32::from(selected),
                input: Input::Swap,
            }]
        } else {
            vec![]
        };
        b.inventory.manager.submit(Request::new(
            id,
            OWNER,
            0,
            actions,
            context,
            &b.inventory.manager,
        ));
        b.book_bot.pending = Some((id, needs_swap.then_some(i32::from(selected))));
    }
    pub(crate) fn book_bot_write_due(&self, host: &mut dyn Host) {
        let now = host.client_auto_tool_time();
        let (settings, root, count) = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            if !b.query_running(&state.registry, "BookBot") {
                return;
            }
            let runtime = &mut b.book_bot;
            let s = &runtime.settings;
            // Kotlin float multiplication is evaluated before Float.toLong.
            if now.wrapping_sub(runtime.last) < (s.delay * 1000f32) as i64 {
                return;
            }
            runtime.last = now;
            (json!({"mode":s.mode,"pages":s.pages,"ascii":s.ascii_only,"space":s.allow_space,"source":s.source,"cyclic":s.cyclic,"sign":s.sign,"name":s.name}).to_string(),state.config_root.to_string_lossy().into_owned(),runtime.count)
        };
        if host.client_book_bot_write(&settings, &root, count) {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            b.book_bot.count = b.book_bot.count.wrapping_add(1);
        }
    }
}
