// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    coroutine_ticker::WaitOwner,
    runtime::{LiquidBounceHooks, raw_options},
};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::auto_queue::{Context, Effect, Slot};
pub(crate) const GOMME_REQUESTER: u64 = 0x4151_474f_4d4d_4501;
pub(crate) const CUSTOM_REQUESTER: u64 = 0x4151_4355_5354_4f4d;
fn random(a: i32, b: i32) -> i32 {
    liquidbounce_behaviors::random::random_i32_inclusive(a, b)
}
fn language() -> u8 {
    match horizon_client::lang::code()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "zh_cn" | "zh_sg" | "zh_hans" => 1,
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => 2,
        _ => 0,
    }
}
fn text(value: &str) -> &str {
    match (language(), value) {
        (1, "Queue started") => "已开始排队",
        (2, "Queue started") => "已開始排隊",
        (1, "Already in queue") => "已经在排队中",
        (2, "Already in queue") => "已在排隊中",
        (1, "Match started") => "对局开始",
        (2, "Match started") => "對局開始",
        (1, "Match won") => "对局胜利",
        (2, "Match won") => "對局勝利",
        (1, "Match lost") => "对局失败",
        (2, "Match lost") => "對局失敗",
        (1, "Not on GommeHD.net") => "当前不在 GommeHD.net",
        (2, "Not on GommeHD.net") => "目前不在 GommeHD.net",
        (1, "Could not find Duels NPC") => "未找到决斗 NPC",
        (2, "Could not find Duels NPC") => "未找到決鬥 NPC",
        (1, "Interacted with Duels NPC") => "已与决斗 NPC 交互",
        (2, "Interacted with Duels NPC") => "已與決鬥 NPC 互動",
        _ => value,
    }
}
impl LiquidBounceHooks {
    pub(crate) fn auto_queue_enabled_info(&self, host: &mut dyn Host) {
        self.state().behaviors.borrow_mut().auto_queue.info = false;
        self.auto_queue_effects(host, vec![Effect::EnableInfo]);
    }
    fn auto_queue_running(&self) -> bool {
        let state = self.state();
        let b = state.behaviors.borrow();
        b.query_running(&state.registry, "AutoQueue")
    }
    fn auto_queue_context(&self, host: &dyn Host) -> Option<Context> {
        let v = host.client_auto_queue_snapshot()?;
        let mut c = Context {
            now: v["now"].as_i64()?,
            pause: v["pause"].as_u64()? as u32,
            selected: v["selected"].as_i64()? as i32,
            header: v["header"].as_str().map(str::to_owned),
            footer: v["footer"].as_str().map(str::to_owned),
            npc: v["npc"].as_i64().and_then(|n| i32::try_from(n).ok()),
            slots: v["slots"]
                .as_array()?
                .iter()
                .filter_map(|s| {
                    Some(Slot {
                        slot: i32::try_from(s["slot"].as_i64()?).ok()?,
                        item: s["item"].as_str()?.into(),
                        name: s["name"].as_str()?.into(),
                    })
                })
                .collect(),
            aura_running: false,
        };
        {
            let state = self.state();
            let b = state.behaviors.borrow();
            c.aura_running = b.query_running(&state.registry, "KillAura");
        }
        #[cfg(feature = "webview")]
        if self
            .browser
            .as_ref()
            .is_some_and(|b| b.lock().expect("Browser poisoned").visible)
        {
            c.pause |= 1;
        }
        Some(c)
    }
    fn auto_queue_control(&self, host: &mut dyn Host, name: &str, enabled: bool) {
        let options = raw_options(host);
        let effects = {
            let mut state = self.state();
            let id = state
                .registry
                .modules()
                .find(|m| m.definition().name == name)
                .map(|m| m.id());
            id.map(|id| {
                if let Err(e) =
                    state.with_registry(options, |r, c| r.set_enabled_sync(id, enabled, c))
                {
                    eprintln!("[LiquidBounce] AutoQueue {name}: {e:?}");
                }
                state.take_effects()
            })
        };
        if let Some(effects) = effects {
            crate::runtime::apply_effects(self, host, effects);
        }
    }
    fn auto_queue_effects(&self, host: &mut dyn Host, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Command(s) => host.client_auto_queue_send(&s, 0),
                Effect::Chat(s) => host.client_auto_queue_send(&s, 1),
                Effect::ChatOrCommand(s) => host.client_auto_queue_send(&s, 2),
                Effect::Register(id) => {
                    self.state()
                        .behaviors
                        .borrow()
                        .coroutine_ticker
                        .register(WaitOwner::AutoQueue, id);
                }
                Effect::Select { slot, owner } => {
                    if slot >= 0 {
                        let _ = host.client_select_slot_silently(
                            if owner {
                                GOMME_REQUESTER
                            } else {
                                CUSTOM_REQUESTER
                            },
                            slot,
                            20,
                        );
                    }
                }
                Effect::Use(slot) => {
                    host.client_auto_queue_use(i32::from(slot == -1));
                }
                Effect::RawUse => host.client_auto_queue_raw_use(),
                Effect::Interact(id) => host.client_auto_queue_interact(id),
                Effect::Control(name, enabled) => self.auto_queue_control(host, name, enabled),
                Effect::ResetGomme => {
                    host.client_reset_silent_slot(GOMME_REQUESTER);
                }
                Effect::Notice(message, error) => {
                    let message = text(message);
                    #[cfg(feature="webview")]self.state().events.borrow_mut().push(("notification".into(),serde_json::json!({"title":"AutoPlay","message":message,"severity":if error{"ERROR"}else{"INFO"}})));
                    #[cfg(not(feature = "webview"))]
                    eprintln!("[LiquidBounce] AutoPlay: {message} (error={error})");
                }
                Effect::EnableInfo => {
                    let lines = match language() {
                        1 => [
                            "请将服务器语言设为德语（DE），以使用自动重排。",
                            "自动重排会自动加入决斗队列，并在断线后重新加入。",
                        ],
                        2 => [
                            "請將伺服器語言設為德語（DE），以使用自動重排。",
                            "自動重排會自動加入決鬥佇列，並在斷線後重新加入。",
                        ],
                        _ => [
                            "Please set your server language to German (DE) to use AutoPlay",
                            "AutoPlay will automatically queue for Duels and re-join if you get disconnected",
                        ],
                    };
                    for line in lines {
                        host.chat(line);
                    }
                }
            }
        }
        let deadline = self.state().behaviors.borrow().auto_queue.next_wake();
        host.client_auto_queue_schedule(deadline);
    }
    pub(crate) fn auto_queue_tick(&self, host: &mut dyn Host) {
        let running = self.auto_queue_running();
        let c = self.auto_queue_context(host).unwrap_or_default();
        let effects = self
            .state()
            .behaviors
            .borrow_mut()
            .auto_queue
            .tick(running, &c, random);
        self.auto_queue_effects(host, effects);
    }
    pub(crate) fn auto_queue_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let running = in_game && self.auto_queue_running();
        let c = self.auto_queue_context(host).unwrap_or_default();
        let (keep, effects) = self
            .state()
            .behaviors
            .borrow_mut()
            .auto_queue
            .callback(id, running, &c, random);
        self.auto_queue_effects(host, effects);
        keep
    }
    pub(crate) fn auto_queue_wake(&self, host: &mut dyn Host) {
        let running = self.auto_queue_running();
        let c = self.auto_queue_context(host).unwrap_or_default();
        let effects = self
            .state()
            .behaviors
            .borrow_mut()
            .auto_queue
            .wake(running, &c, random);
        self.auto_queue_effects(host, effects);
    }
    pub(crate) fn auto_queue_message(
        &self,
        host: &mut dyn Host,
        source: u8,
        kind: u8,
        message: &str,
    ) {
        let running = self.auto_queue_running();
        let now = host.client_auto_tool_time();
        let effects = self
            .state()
            .behaviors
            .borrow_mut()
            .auto_queue
            .event(running, source, kind, message, now);
        self.auto_queue_effects(host, effects);
    }
    pub(crate) fn auto_queue_world_change(&self) {
        let effects = self
            .state()
            .behaviors
            .borrow_mut()
            .auto_queue
            .world_change();
        let state = self.state();
        let b = state.behaviors.borrow();
        for effect in effects {
            if let Effect::Register(id) = effect {
                b.coroutine_ticker.register(WaitOwner::AutoQueue, id);
            }
        }
    }
}
