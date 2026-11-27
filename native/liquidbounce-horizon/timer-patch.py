#!/usr/bin/env python3
"""Real client DeltaTracker, dynamic runTick bound and cancellable LocalPlayer seam."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

SNAPSHOT_HELPER='''    /// Previous KeyboardInput.moveVector and the exact two vanilla screen classes.
    pub(super) fn native_client_timer_context(&self) -> Option<[bool; 2]> {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return None; }
        let player = self.player.as_ref().filter(|p| p.native_client_queries_allowed())?;
        let moving = player.move_vector[0] != 0.0 || player.move_vector[1] != 0.0;
        let inventory = self.remote_menu().0 == 0 && self.container.is_none() && self.station.is_none()
            && !self.mount_active() && self.craft_size == 2;
        let excluded = self.screen == Screen::Inventory && (inventory || self.chest_rows().is_some());
        Some([moving, excluded])
    }

'''

HELPERS='''    /// Minecraft.updateLevelInEngines at HEAD: nullable client level replacement.
    /// The native role and actor gates exclude integrated/dedicated server turns.
    pub(super) fn native_client_world_change(&self, dimension: Option<&str>) {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return; }
        mod_api::dispatch_client_world_change(self, mod_api::mods(), dimension);
    }
    /// DeltaTracker.Timer's Float field assignment, before residual accumulation.
    pub(super) fn native_client_timer_scale(&self, delta: &mut f32) {
        mod_api::dispatch_client_timer_scale(self, mod_api::mods(), delta);
    }
    /// Re-evaluated before every loop condition, including the terminating one.
    pub(super) fn native_client_timer_frame(&self, ticks: i32, index: i32, end: &mut i32, held: &mut bool) {
        mod_api::dispatch_client_timer_frame(self, mod_api::mods(), ticks, index, end, held);
    }
    /// Before the actual local player's simulation. Final listeners see cancellation.
    pub(super) fn native_client_player_tick(&self, cancelled: &mut bool) {
        mod_api::dispatch_client_player_tick(self, mod_api::mods(), cancelled);
    }
    pub(super) fn client_timer_player_tick(&self) -> bool {
        let mut cancelled = false;
        self.native_client_player_tick(&mut cancelled);
        self.native_client_player_tick_final(cancelled);
        cancelled
    }
    pub(super) fn native_client_player_tick_final(&self, cancelled: bool) {
        mod_api::dispatch_client_player_tick_final(self, mod_api::mods(), cancelled);
    }
    pub(super) fn client_timer_player_tick_ready(&self) -> bool {
        let Some(player) = self.player.as_ref() else { return false; };
        let (cx, cz) = ((player.position.x.floor() as i32) >> 4, (player.position.z.floor() as i32) >> 4);
        self.streaming.is_lit(cx, cz) && self.client_timer_player_tick()
    }

'''

def patch_clock(text,once):
    text=once(text,'    frozen: bool,\n','    frozen: bool,\n    native_held: bool,\n')
    text=once(text,'            frozen: false,\n','            frozen: false,\n            native_held: false,\n')
    text=once(text,'''        self.delta_ticks = (current_ms - self.last_ms) as f32 / target_mspt(self.ms_per_tick);
        self.last_ms = current_ms;
        self.delta_tick_residual += self.delta_ticks;
''','''        let delta = self.prepare_game_time(current_ms, target_mspt);
        self.finish_game_time(delta)
    }

    pub fn prepare_game_time(&mut self, current_ms: i64, target_mspt: impl FnOnce(f32) -> f32) -> f32 {
        self.delta_ticks = (current_ms - self.last_ms) as f32 / target_mspt(self.ms_per_tick);
        self.last_ms = current_ms;
        self.delta_ticks
    }

    pub fn finish_game_time(&mut self, delta: f32) -> i32 {
        self.delta_ticks = delta;
        self.delta_tick_residual += self.delta_ticks;
''')
    text=once(text,'    pub fn game_time_delta_partial_tick(&self, ignore_frozen_game: bool) -> f32 {\n','    pub fn game_time_delta_partial_tick(&self, ignore_frozen_game: bool) -> f32 {\n        if self.native_held { return 1.0; }\n')
    text=once(text,'    pub fn advance(&mut self, elapsed: std::time::Duration) -> u32 {\n','''    pub fn advance(&mut self, elapsed: std::time::Duration) -> u32 {
        let frame = self.begin_advance(elapsed);
        self.finish_advance(frame, frame.delta_ticks)
    }
    /// Split around the actual Float field injection; UI time and server rate stay untouched.
    pub fn begin_advance(&mut self, elapsed: std::time::Duration) -> NativeTimerFrame {
''')
    text=once(text,'            self.timer.advance_game_time(now_ms, |d| d);\n','            let delta_ticks = self.timer.prepare_game_time(now_ms, |d| d);\n')
    text=once(text,'            return (remaining + 1).min(SPRINT_PER_FRAME) as u32;\n','            return NativeTimerFrame { delta_ticks, sprint_ticks: Some((remaining + 1).min(SPRINT_PER_FRAME) as u32) };\n')
    text=once(text,'            .advance_game_time(now_ms, |default| match (runs, client) {\n','            .prepare_game_time(now_ms, |default| match (runs, client) {\n')
    text=once(text,'        ticks.clamp(0, 10) as u32\n    }\n','''        NativeTimerFrame { delta_ticks: ticks, sprint_ticks: None }
    }
    pub fn finish_advance(&mut self, frame: NativeTimerFrame, delta: f32) -> u32 {
        let ticks = self.timer.finish_game_time(delta);
        frame.sprint_ticks.unwrap_or_else(|| ticks.clamp(0, 10) as u32)
    }
    pub fn set_timer_held(&mut self, held: bool) { self.timer.native_held = held; }
''')
    return once(text,'#[derive(Clone, Debug)]\npub struct FixedStep {\n','''#[derive(Clone, Copy, Debug)]
pub struct NativeTimerFrame { pub delta_ticks: f32, sprint_ticks: Option<u32> }

#[derive(Clone, Debug)]
pub struct FixedStep {
''')

def apply(edit,once):
    edit('crates/horizon-client/src/clock.rs',lambda text:patch_clock(text,once))
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',HELPERS+SNAPSHOT_HELPER+'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    edit('crates/horizon-client/src/game.rs',lambda text:once(text,'''                    let steps = self.clock.advance(elapsed);
                    for _ in 0..steps {
''','''                    let timer_frame = self.clock.begin_advance(elapsed);
                    let mut timer_delta = timer_frame.delta_ticks;
                    self.native_client_timer_scale(&mut timer_delta);
                    let steps = self.clock.finish_advance(timer_frame, timer_delta);
                    let mut timer_index = 0i32;
                    while {
                        let mut end = steps as i32;
                        let mut held = false;
                        self.native_client_timer_frame(steps as i32, timer_index, &mut end, &mut held);
                        self.clock.set_timer_held(held);
                        timer_index < end
                    } {
'''))
    edit('crates/horizon-client/src/game.rs',lambda text:once(text,'''                    }
                    self.extras_frame(elapsed);
''','''                        timer_index = timer_index.wrapping_add(1);
                    }
                    self.extras_frame(elapsed);
'''))
    def remote(text):
        start=text.index('    fn tick_remote_level(');end=text.index('\n    fn ',start+10)
        source=text[start:end]
        source=once(source,'''        self.cooldowns.tick();
        // Inventory.tick's ItemStack.inventoryTick: the pop time runs down.
        if !hold {
''','''        let client_player_cancelled = self.client_timer_player_tick_ready();
        if !client_player_cancelled { self.cooldowns.tick(); }
        // Inventory.tick's ItemStack.inventoryTick: the pop time runs down.
        if !hold && !client_player_cancelled {
''')
        source=once(source,'            self.tick_client_effects();\n','            if !client_player_cancelled { self.tick_client_effects(); }\n')
        source=once(source,'        self.sleep_timer = super::tick_sleep_timer(self.sleep_timer, self.sleeping.is_some());\n','        if !client_player_cancelled { self.sleep_timer = super::tick_sleep_timer(self.sleep_timer, self.sleeping.is_some()); }\n')
        source=once(source,'''        let stick = self.input_move();
        if self.player.is_none() {
''','''        if client_player_cancelled {
            // sendChanges belongs to Minecraft.tick, outside the cancelled LocalPlayer.tick.
            if let Some(remote) = self.remote.as_ref() && remote.loaded_sent {
                self.remote_send_state(remote.last_input);
            }
            return Ok(());
        }
        let stick = self.input_move();
        if self.player.is_none() {
''')
        return text[:start]+source+text[end:]
    edit('crates/horizon-client/src/game/remote.rs',remote)
    edit('crates/horizon-client/src/game/remote.rs',lambda text:once(text,'''        if !same {
            // A new ClientLevel: Minecraft.setLevel stops every sound.
''','''        if !same {
            // Minecraft.updateLevelInEngines HEAD, before its engine side effects.
            self.native_client_world_change(Some(&spawn.dimension));
            // A new ClientLevel: Minecraft.setLevel stops every sound.
'''))
    edit('crates/horizon-client/src/game/remote.rs',lambda text:once(text,'''        // updateLevelInEngines(null): every sound stops and the music manager
''','''        self.native_client_world_change(None);
        // updateLevelInEngines(null): every sound stops and the music manager
'''))
    edit('crates/horizon-client/src/game.rs',lambda text:once(text,'''    fn finish_close_world(&mut self) -> Result<()> {
        self.ambience.clear(&mut self.audio);
        // updateLevelInEngines(null): every sound stops; the music manager
        // lets go of its track.
        self.music.stop_playing(None, &mut self.audio);
        self.audio.stop_all();
        if self.remote.is_some() {
            self.world_icon = None;
            return self.leave_server(None);
        }
''','''    fn finish_close_world(&mut self) -> Result<()> {
        if self.remote.is_some() {
            // Its actual null update is in leave_server, after any deferred save.
            self.world_icon = None;
            return self.leave_server(None);
        }
        self.native_client_world_change(None);
        self.ambience.clear(&mut self.audio);
        // updateLevelInEngines(null): every sound stops; the music manager
        // lets go of its track.
        self.music.stop_playing(None, &mut self.audio);
        self.audio.stop_all();
'''))
    return {'hook_names':['client_timer_scale','client_timer_frame','client_player_tick','client_player_tick_final','client_world_change'],'helper_names':['client_timer_context']}
