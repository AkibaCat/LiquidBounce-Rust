// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    coroutine_ticker::WaitOwner,
    runtime::{LiquidBounceHooks, raw_options},
};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::auto_leave::KickMode;

#[derive(Clone, Debug)]
pub(crate) enum OneShotEffect {
    CaptureNames,
    OldRegenEnabled,
    BookBotEnabled,
    SneakSwitchWarn,
    PluginsStart,
    AntiCheatDetectEnabled,
    VehicleControlEnabled,
    BlinkEnabled,
    BlinkDisabled,
    LogoffSpotDisabled,
    InventoryTrackerReset,
    AutoRodDisabled(bool),
    KillAuraDisabled,
    AutoQueueEnabled,
    AutoChatGameEnabled,
    NoClipDisabled,
    PotionSpoofDisabled(liquidbounce_behaviors::potion_spoof::PotionSpoof),
    Damage(liquidbounce_behaviors::damage::Damage),
    Clip(liquidbounce_behaviors::clip::Clip),
    Kick(KickMode),
}

fn chinese() -> u8 {
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
impl LiquidBounceHooks {
    fn one_shot_notification(&self, title: &str, message: String, severity: &str) {
        #[cfg(feature = "webview")]
        self.state().events.borrow_mut().push((
            "notification".into(),
            serde_json::json!({"title":title,"message":message,"severity":severity}),
        ));
        #[cfg(not(feature = "webview"))]
        eprintln!("[LiquidBounce] {title} {severity}: {message}");
    }
    pub(crate) fn apply_one_shot(&self, host: &mut dyn Host, effect: OneShotEffect) {
        match effect {
            OneShotEffect::OldRegenEnabled => self.old_regen_enabled(host),
            OneShotEffect::SneakSwitchWarn => self.sneak_protocol_warning(),
            OneShotEffect::BookBotEnabled => {let now=host.client_auto_tool_time();self.state().behaviors.borrow_mut().book_bot.enabled(now);},
            OneShotEffect::PluginsStart => self.plugins_start(host),
            OneShotEffect::AntiCheatDetectEnabled => self.anti_cheat_detect_enabled(host),
            OneShotEffect::VehicleControlEnabled=>self.vehicle_control_enabled(host),
            OneShotEffect::BlinkEnabled=>self.blink_enabled(host),
            OneShotEffect::BlinkDisabled=>self.blink_disabled(host),
            OneShotEffect::LogoffSpotDisabled=>self.logoff_spot_disabled(host),
            OneShotEffect::InventoryTrackerReset=>self.inventory_tracker_flush(host),
            OneShotEffect::AutoRodDisabled(release)=>self.auto_rod_disabled(host,release),
            OneShotEffect::KillAuraDisabled=>self.kill_aura_disabled(host),
            OneShotEffect::AutoQueueEnabled=>self.auto_queue_enabled_info(host),
            OneShotEffect::AutoChatGameEnabled=>self.auto_chat_game_enabled(host),
            OneShotEffect::NoClipDisabled => self.no_clip_disabled(host),
            OneShotEffect::PotionSpoofDisabled(settings) => {
                crate::potion_spoof_runtime::disabled(host, &settings)
            }
            OneShotEffect::Damage(settings) => crate::damage_runtime::run(host, settings),
            OneShotEffect::Clip(settings) => self.clip_old(host,settings),
            OneShotEffect::CaptureNames => {
                let names = host.client_name_collector_names();
                let title = match chinese() {
                    1 => "名称收集",
                    2 => "名稱收集",
                    _ => "NameCollector",
                };
                if names.is_empty() {
                    self.one_shot_notification(
                        title,
                        match chinese() {
                            1 => "未找到玩家",
                            2 => "未找到玩家",
                            _ => "No players found",
                        }
                        .into(),
                        "ERROR",
                    );
                    return;
                }
                let root = self.state().config_root.clone();
                let directory = root.join("name-captures");
                let path =
                    directory.join(format!("{}.txt", crate::behaviors::current_time_millis()));
                if let Err(error) = std::fs::create_dir_all(&directory)
                    .and_then(|()| std::fs::write(&path, names.join("\n")))
                {
                    self.one_shot_notification(title, error.to_string(), "ERROR");
                    return;
                }
                let count = names.len();
                let message = match chinese() {
                    1 => format!("已收集 {count} 名玩家"),
                    2 => format!("已收集 {count} 名玩家"),
                    _ => format!("Captured {count} players"),
                };
                self.one_shot_notification(title, message.clone(), "INFO");
                let absolute = std::path::absolute(&path).unwrap_or(path);
                let message = match chinese() {
                    1 => format!("{message}。已保存至 {}。", absolute.display()),
                    2 => format!("{message}。已儲存至 {}。", absolute.display()),
                    _ => format!("{message}. Saved to {}.", absolute.display()),
                };
                host.chat(&message);
            }
            OneShotEffect::Kick(mode) => {
                if host
                    .client_auto_leave_context()
                    .is_some_and(|context| context[2] != 0.0)
                {
                    host.chat(match chinese() {
                        1 => "§c单人游戏中无法开启Kick。",
                        2 => "§c單人遊戲中無法開啟Kick。",
                        _ => "§cKick cannot be enabled in single player.",
                    });
                    return;
                }
                self.kick_now(host, mode);
            }
        }
    }
    fn kick_now(&self, host: &mut dyn Host, mode: KickMode) {
        let (a, b) = if mode == KickMode::IllegalChat {
            liquidbounce_behaviors::random::with_thread_random(|random| {
                (random.next_i32(), random.next_i32())
            })
        } else {
            (0, 0)
        };
        host.client_kick(mode.code(), a, b);
    }
    fn auto_leave_break(&self, host: &dyn Host) -> bool {
        let threshold = self.state().behaviors.borrow().auto_leave.settings.health;
        host.client_auto_leave_context().is_none_or(|context| {
            context[1] != 0.0
                || context[2] != 0.0
                || context[3] != 0.0
                || context[0] as f32 >= threshold
        })
    }
    fn auto_leave_finish(&self, host: &mut dyn Host) {
        let mode = self.state().behaviors.borrow().auto_leave.settings.mode;
        self.kick_now(host, mode);
        let options = raw_options(host);
        let mut state = self.state();
        let id = state.behaviors.borrow().query_ids["AutoLeave"];
        if let Err(error) = state.with_registry(options, |registry, callbacks| {
            registry.set_enabled_sync(id, false, callbacks)
        }) {
            eprintln!("[LiquidBounce] AutoLeave disable: {error:?}");
        }
    }
    pub(crate) fn auto_leave_tick(&self, host: &mut dyn Host) {
        let delay = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            if !modules.query_running(&state.registry, "AutoLeave") || modules.auto_leave.busy() {
                return;
            }
            modules.auto_leave.settings.delay
        };
        if delay[0] > delay[1] {
            eprintln!("[LiquidBounce] AutoLeave empty delay range");
            return;
        }
        let delay = liquidbounce_behaviors::random::random_i32_inclusive(delay[0], delay[1]);
        let condition = if delay == 0 {
            self.auto_leave_break(host)
        } else {
            false
        };
        let (wait, finish) = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let running = modules.query_running(&state.registry, "AutoLeave");
            modules.auto_leave.start(running, delay, || condition)
        };
        if let Some(id) = wait {
            self.state()
                .behaviors
                .borrow()
                .coroutine_ticker
                .register(WaitOwner::AutoLeave, id);
        }
        if finish {
            self.auto_leave_finish(host);
        }
    }
    pub(crate) fn auto_leave_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let running = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            in_game
                && modules.query_running(&state.registry, "AutoLeave")
                && modules.auto_leave.waiting(id)
        };
        let condition = if running {
            self.auto_leave_break(host)
        } else {
            true
        };
        let (keep, finish) =
            self.state()
                .behaviors
                .borrow_mut()
                .auto_leave
                .advance(id, running, || condition);
        if finish {
            self.auto_leave_finish(host);
        }
        keep
    }
}
