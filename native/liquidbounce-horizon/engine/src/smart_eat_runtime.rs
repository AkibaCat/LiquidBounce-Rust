// SPDX-License-Identifier: GPL-3.0-or-later
//! SmartEat actor bridge. Host callbacks never run under the registry lock.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{Host, HudBatch};
use liquidbounce_behaviors::smart_eat::{Context, Effects, SILENT_OFFHAND_REQUESTER};

fn snapshot(host: &dyn Host) -> Option<Context> {
    match crate::smart_eat::parse_snapshot(&host.client_smart_eat_snapshot()) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("[LiquidBounce] SmartEat snapshot: {error}");
            None
        }
    }
}

impl LiquidBounceHooks {
    fn smart_eat_running(&self) -> bool {
        let state = self.state();
        state.behaviors.borrow().query_running(&state.registry, "SmartEat")
    }
    fn smart_eat_effects(&self, host: &dyn Host, effects: Effects) {
        if let Some(ticks) = effects.pause_combat {
            self.state().combat.pause_combat_for_at_least(ticks);
        }
        if let Some(selection) = effects.selection {
            // The original offhand overload succeeds without changing slots.
            if let Some(slot) = selection.slot.hotbar_index() {
                let result =
                    host.client_select_slot_silently(selection.requester, slot, selection.ticks);
                #[cfg(feature = "webview")]
                self.state().probe("smartEatSelection",
                    serde_json::json!({"requester":selection.requester,"slot":slot,"ticks":selection.ticks}),
                    serde_json::json!({"accepted":result.as_ref().ok(),"error":result.as_ref().err()}));
                if let Err(error) = result {
                    eprintln!("[LiquidBounce] SmartEat selection: {error}");
                }
            }
        }
    }

    pub(crate) fn smart_eat_tick(&self, host: &dyn Host) {
        if !self.smart_eat_running() { return; }
        let Some(context) = snapshot(host) else {
            return;
        };
        let owner = host
            .client_silent_slot_modified_by(SILENT_OFFHAND_REQUESTER)
            .then_some(SILENT_OFFHAND_REQUESTER);
        let effects = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules.smart_eat.silent_offhand_tick(
                modules.query_running(&state.registry, "SmartEat"),
                owner,
                &context,
            )
        };
        self.smart_eat_effects(host, effects);
        let effects = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "SmartEat");
            let effects = modules
                .smart_eat
                .start_run(running, state.combat.in_combat(), &context);
            for callback in modules.smart_eat.take_wait_registrations() {
                modules
                    .coroutine_ticker
                    .register(crate::coroutine_ticker::WaitOwner::SmartEat, callback);
            }
            effects
        };
        self.smart_eat_effects(host, effects);
    }

    pub(crate) fn smart_eat_callback(&self, host: &dyn Host, in_game: bool, callback: u64) -> bool {
        let context = snapshot(host);
        let (keep, effects) = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = in_game && modules.query_running(&state.registry, "SmartEat");
            modules
                .smart_eat
                .advance_callback(callback, running, context.as_ref())
        };
        self.smart_eat_effects(host, effects);
        keep
    }

    pub(crate) fn smart_eat_interacted(&self, host: &dyn Host, pass: bool) {
        if !pass || !self.smart_eat_running() {
            return;
        }
        let Some(context) = snapshot(host) else {
            return;
        };
        let effects = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules.smart_eat.interacted_pass(
                modules.query_running(&state.registry, "SmartEat"),
                state.combat.in_combat(),
                &context,
            )
        };
        self.smart_eat_effects(host, effects);
    }

    pub(crate) fn smart_eat_use_key(&self, pressed: &mut bool) {
        let mut state = self.state();
        let effects = {
            let modules = state.behaviors.borrow();
            modules
                .smart_eat
                .use_key(modules.query_running(&state.registry, "SmartEat"), pressed)
        };
        if let Some(ticks) = effects.pause_combat {
            state.combat.pause_combat_for_at_least(ticks);
        }
    }

    pub(crate) fn smart_eat_render(&self, host: &dyn Host, batch: &mut HudBatch) {
        if !self.smart_eat_running() { return; }
        let Some(context) = snapshot(host) else {
            return;
        };
        let food = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules
                .smart_eat
                .render_food(modules.query_running(&state.registry, "SmartEat"), &context)
                .map(|food| (food.slot.native(), modules.smart_eat.render_offset))
        };
        if let Some((slot, offset)) = food {
            host.client_smart_eat_draw(batch, slot, offset);
        }
    }
}
