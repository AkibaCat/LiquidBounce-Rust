// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::old_regen::Context;
pub(crate) const TIMER_PROVIDER: u64 = u64::MAX - 2;
impl LiquidBounceHooks {
    pub(crate) fn old_regen_enabled(&self, host: &mut dyn Host) {
        // This pinned runtime speaks native MC26.3, which is newer than 1.8.
        let language = horizon_client::lang::code()
            .to_ascii_lowercase()
            .replace('-', "_");
        host.chat(match language.as_str(){"zh_cn"|"zh_sg"|"zh_hans"=>"§c警告：OldRegen仅支持1.8版本的Minecraft服务器！但是，你的客户端版本可以更高（1.17+）以便绕过。","zh_tw"|"zh_hk"|"zh_mo"|"zh_hant"=>"§c警告：OldRegen僅支援1.8版本的Minecraft伺服器！但是，你的客戶端版本可以更高（1.17+）以便繞過。",_=>"§cWARNING: OldRegen is only supported on 1.8 Minecraft Server! However, your client version can be higher (1.17+) in order to bypass."});
    }
    pub(crate) fn old_regen_tick(&self, host: &mut dyn Host) {
        let (module, running, timer) = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            (
                modules.old_regen,
                modules.query_running(&state.registry, "OldRegen"),
                modules.timer.clone(),
            )
        };
        timer.set_provider_running(TIMER_PROVIDER, running);
        if !running {
            return;
        }
        let Some(c) = host.client_old_regen_context() else {
            return;
        };
        let context = Context {
            health: c[0] as f32,
            creative: c[1] != 0.0,
            dead: c[2] != 0.0,
            ground: c[3] != 0.0,
            moving: c[4] != 0.0,
            regeneration: c[5] != 0.0,
            food: c[6] as i32,
            burning: c[7] != 0.0,
            wet: c[8] != 0.0,
            longest_harmful: (c[9] != 0.0).then_some(c[10] as i32),
        };
        let count = module.tick_speed(running, context);
        if count > 0 {
            timer
                .state
                .lock()
                .expect("Timer poisoned")
                .timer
                .request_speed(module.timer, 20, TIMER_PROVIDER, 1);
            for _ in 0..count {
                host.client_old_regen_send(module.packet_type);
            }
        }
    }
}
