// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoBreak from LiquidBounce ae0eb396. The caller owns KeyMapping.click.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoBreak;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyQuery {
    pub attack: bool,
    pub miss_time: i32,
    pub block_hit: bool,
    pub block_state_available: bool,
    pub air: bool,
    pub destroying: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effect {
    pub pressed: bool,
    pub click: bool,
}

impl AutoBreak {
    pub fn query(self, running: bool, query: KeyQuery, original: bool) -> Effect {
        if running
            && query.attack
            && query.miss_time <= 0
            && query.block_hit
            && query.block_state_available
            && !query.air
        {
            Effect {
                pressed: true,
                click: !query.destroying,
            }
        } else {
            Effect {
                pressed: original,
                click: false,
            }
        }
    }
}
