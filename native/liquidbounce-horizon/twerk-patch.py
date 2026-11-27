#!/usr/bin/env python3
"""The actual local player's signed entity tick for MovementInputEvent."""

HELPER_NAMES=['client_player_input_tick_count']

HELPERS='''    /// Horizon builds the network input before controlled_simulation_tick
    /// advances Entity's clock. KeyboardInput.tick in vanilla follows baseTick,
    /// so this input belongs to that immediately following wrapped entity tick.
    /// Query the current player every time; never keep a module-owned clock.
    pub(super) fn native_client_player_input_tick_count(&self) -> Option<i32> {
        if !self.native_window() || self.lan_acting_uuid.is_some() { return None; }
        let player = self.player.as_ref()?;
        if !player.native_client_queries_allowed() { return None; }
        Some(player.health.tick_count.wrapping_add(1))
    }

'''

def apply(edit,replace_once):
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:replace_once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HELPERS+'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    return {'hook_names':[],'helper_names':HELPER_NAMES}
