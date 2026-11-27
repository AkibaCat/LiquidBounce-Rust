// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use liquidbounce_behaviors::auto_break::KeyQuery;

impl LiquidBounceHooks {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn auto_break_key(
        &self,
        attack: bool,
        miss_time: i32,
        block_hit: bool,
        block_state_available: bool,
        air: bool,
        destroying: bool,
        pressed: &mut bool,
        click: &mut bool,
    ) {
        let effect = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules.auto_break.query(
                modules.query_running(&state.registry, "AutoBreak"),
                KeyQuery { attack, miss_time, block_hit, block_state_available, air, destroying },
                *pressed,
            )
        };
        *pressed = effect.pressed;
        *click |= effect.click;
    }
}
