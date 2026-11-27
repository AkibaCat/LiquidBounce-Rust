// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_crafter::{Action, Context, Effect, Recipe},
    random::random_i32_inclusive,
};
use serde_json::{Value, json};
fn sample(range: [i32; 2]) -> i32 {
    random_i32_inclusive(range[0], range[1])
}
impl LiquidBounceHooks {
    fn auto_crafter_context(&self, host: &mut dyn Host) -> Option<Context> {
        let targets = json!(self.state().behaviors.borrow().auto_crafter.items()).to_string();
        let value = host.client_auto_crafter_snapshot(&targets)?;
        let v: Value = serde_json::from_str(&value).ok()?;
        let strings = |v: &Value| -> Vec<String> {
            v.as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        };
        Some(Context {
            menu: serde_json::from_value(v["menu"].clone()).ok()?,
            kind: v["kind"].as_u64()? as usize,
            book_open: v["open"] == true,
            recipes: v["recipes"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|r| Recipe {
                    display: r["display"].as_i64().unwrap_or(-1) as i32,
                    outputs: strings(&r["outputs"]),
                    uses: strings(&r["uses"]),
                    craftable: r["craftable"] == true,
                })
                .collect(),
            result_count: v["result"].as_u64().unwrap_or(0) as u32,
            capacity: v["capacity"].as_u64().unwrap_or(0) as u32,
        })
    }
    fn auto_crafter_effect(&self, host: &mut dyn Host, mut effect: Effect) {
        // A malformed imported zero delay has the original immediate continuation.
        loop {
            if let Some(action) = effect.action {
                match action {
                    Action::Place { display, stack } => {
                        host.client_auto_crafter_place(display, stack);
                    }
                    Action::QuickMove => {
                        host.client_auto_crafter_result(false);
                    }
                    Action::Throw => {
                        host.client_auto_crafter_result(true);
                    }
                    Action::Close => {
                        host.client_inventory_close(0, true);
                    }
                    Action::Disable => {
                        let effects = {
                            let mut state = self.state();
                            let id = state.behaviors.borrow().query_ids["AutoCrafter"];
                            if let Err(error) = state
                                .with_registry(crate::runtime::raw_options(host), |r, c| {
                                    r.set_enabled_sync(id, false, c)
                                })
                            {
                                eprintln!("[LiquidBounce] AutoCrafter: {error:?}");
                            }
                            state.take_effects()
                        };
                        crate::runtime::apply_effects(self, host, effects);
                    }
                }
            }
            let Some(callback) = effect.register else {
                return;
            };
            let immediate = self
                .state()
                .behaviors
                .borrow()
                .auto_crafter
                .waits_immediately();
            if !immediate {
                self.state()
                    .behaviors
                    .borrow()
                    .coroutine_ticker
                    .register(WaitOwner::AutoCrafter, callback);
                return;
            }
            let context = self.auto_crafter_context(host);
            effect = self
                .state()
                .behaviors
                .borrow_mut()
                .auto_crafter
                .resume(callback, context.as_ref(), &mut sample)
                .1;
        }
    }
    pub(crate) fn auto_crafter_tick(&self, host: &mut dyn Host) {
        let running = {
            let state = self.state();
            let b = state.behaviors.borrow();
            b.query_running(&state.registry, "AutoCrafter")
        };
        if !running {
            return;
        }
        let Some(context) = self.auto_crafter_context(host) else {
            return;
        };
        let effect = self
            .state()
            .behaviors
            .borrow_mut()
            .auto_crafter
            .start(&context, &mut sample);
        self.auto_crafter_effect(host, effect);
    }
    pub(crate) fn auto_crafter_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        callback: u64,
    ) -> bool {
        let running = {
            let state = self.state();
            let b = state.behaviors.borrow();
            b.query_running(&state.registry, "AutoCrafter")
        };
        let context = if in_game && running {
            self.auto_crafter_context(host)
        } else {
            None
        };
        let (keep, effect) = self.state().behaviors.borrow_mut().auto_crafter.resume(
            callback,
            context.as_ref(),
            &mut sample,
        );
        self.auto_crafter_effect(host, effect);
        keep
    }
}
