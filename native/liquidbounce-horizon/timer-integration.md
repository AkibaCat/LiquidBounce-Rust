# Timer integration handoff

Owned implementation passes `build/timer-10/report.json`: 4,839 commands
against five unchanged Kotlin sources, 29 Rust tests and Clippy with warnings
denied. This is not a full Horizon compile or in-game smoke. The integration
owner retains shared-file, canonical/staging, aggregate patch and EXE ownership.

## Shared modules and configuration

- Export `pub mod timer;` from the behaviors library, add `mod timer_runtime;`
  to the engine, and add `pub timer: Arc<crate::timer_runtime::TimerService>`
  initialized with Default to BehaviorModules.
- Register `Timer`, World category, key `liquidbounce.module.timer`,
  `disable_on_quit=true`. Its displayed tag is the active mode's original tag
  (`Classic`, `Pulse`, `Boost`), localized by the existing presentation layer.
- Add config child `mod timer; pub use timer::{TimerSetting,serialize_timer};`,
  `ModuleKind::Timer`, and `Timer(TimerSetting)` in SettingChange/RuntimeRequest.
  Include Timer in the common settings modules, dispatch parsing to
  `timer::parse_timer`, serialization to `serialize_timer`, and apply_setting
  to `RuntimeRequest::Timer(*value)`.
- Apply runtime values with
  `timer.update_settings(|settings|value.apply(settings))`, which immediately
  cancels the old mode coroutine. Serialize `timer.state.lock().settings`.
  Release the service mutex before Host calls or callbacks.

All eleven leaves and all three choices are implemented. Raw imported values
retain signed zero, editor-outside speeds, Int truncation and reversed Balance
ranges. A reversed Balance produces the original invalid-range result on use.

## Lifecycle and event order

1. Add `timer.set_running(query_running(registry,"Timer"))` to existing
   packet/lifecycle synchronization. Sync immediately on module transitions,
   including disconnect. Disable calls set_running(false), which emits the
   original low-priority 1f request once.
2. Timer's ModuleCallback::CancelSequences calls
   `timer.state.lock().module.cancel_sequences()`. Pending old callbacks then
   return false without effects.
3. Add WaitOwner::Timer and dispatch to the provided
   `self.timer_callback(host,in_game,task.callback)`. The provided timer_tick
   delegate already registers actual waits in the common CoroutineTicker.
4. At the beginning of client_tick_pre, call timer.first_game_tick() once,
   before early returns. The native smart-eat-wait callback resumes the
   CoroutineTicker before this handler. Required order is continuations,
   FIRST_PRIORITY request expiry, then ordinary GameTick handlers. Expiry runs
   at the title screen and while Timer is disabled too.
5. Call `self.timer_tick(host)` after observing the world, before the
   client-absent early return. It reads actual previous input, exact
   inventory/chest screen mapping, CombatWindow and registry running state.
6. Actual world changes and disconnect cleanup call timer.world_change().
   Do not replace the service with Default or clear speed requests: source
   deliberately retains request ticks, realTicks and other fields.
7. Register `pipeline.add_listener(0,modules.timer.clone())`. The ordinary
   listener observes cancelled incoming PlayerPosition corrections and resets
   immediately on the known render thread or queues until the next render
   action. It calls neither Host nor the engine actor lock.

Provider `u64::MAX` is reserved for Timer. Future shared consumers publish
actual EventListener.running with set_provider_running and different stable
IDs. Holds, bursts and balanced requests already exist without fake modules.

## Hooks and patch order

Apply timer-patch.py after coroutine-ticker-patch.py (which matches the original
for-loop), and before TerrainSpeed and its ordered entity scheduler. The
extension returns four hook_names and one helper_names for API generation.

The main LiquidBounceHooks implementation delegates these generated events:

```rust
fn client_timer_scale(&self,_:&dyn Host,event:&mut ClientTimerScaleEvent) {
    event.delta=self.state().behaviors.borrow().timer.scale(event.delta);
}
fn client_timer_frame(&self,_:&dyn Host,event:&mut ClientTimerFrameEvent) {
    let (end,held)=self.state().behaviors.borrow().timer.frame(event.ticks,event.index);
    event.end=end;event.held=held;
}
fn client_player_tick_final(&self,_:&dyn Host,event:&mut ClientPlayerTickFinalEvent) {
    self.state().behaviors.borrow().timer.player_tick_final(event.cancelled);
}
```

Timer does not cancel ClientPlayerTickEvent. The native wrapper dispatches all
mutable handlers before a separate final-state event; listener registration
order cannot express READ_FINAL_STATE on this native API.

The real Float delta changes before residual accumulation. The real loop
calls frameTicks on every condition, including terminating/zero-step checks.
Held interpolation reaches both partial-tick readers. Player cancellation
gates cooldowns, inventory pop, effect/sleep clocks and simulation, while
retaining ambient work and the external sendChanges path.

TerrainSpeed's order patch extracts the local tail to run at the player's
actual entity turn. It must carry client_player_cancelled into that helper or
move the player-owned prefix there; leaving the earlier local variable in an
extracted body will not compile. Cancellation must return Some(last_input) to
the external sendChanges phase, not send early or drop the packet. This is
coordinated with behavior_adapter and needs aggregate ordering checks.

Single player uses an independent Server thread and the window joins through
the in-memory remote connection, as pinned game/integrated.rs and
game/execution_role.rs state. The patch changes only the window client loop,
never server pacing or elapsed Duration. Existing server/freeze/sprint clock
tests and an independent server-thread clock test pass; they do not constitute
an in-game server pacing measurement.

## Remaining integration evidence

World resets now have a real `client_world_change` event, with generated
`ClientWorldChangeEvent<'_>` and `dimension: Option<&str>`. Bind that event to
`modules.timer.world_change()` using the window check, and remove the Timer
reset in both boolean `observe_world` and `client_disconnecting` fallbacks.
The owned patch emits once on actual non-null client-level replacement and
once on actual null update, before the engine cleanup. Full source timing,
native load/unload ordering limits and independent tests are documented in
`research/WORLD-CHANGE-PARITY.md` (`build/world-change-04/report.json`).

Re-run the owned runner with a new build/timer-NN directory after owned
changes. Its fullGameIntegrationPassed is intentionally false. Generate/apply
the aggregate patch, compile the native engine/EXE, and exercise all modes,
mode changes, disable-on-quit, combat normalization, stationary inventory/chest,
correction reset and independent server rate in game. Useful probes are
original/scaled Float delta, ticks/index/end/held, mode, running and balance.

The Chinese dictionaries already include Timer/TerrainSpeed help (270 owned
descriptions total). Rebuild both theme and native resources and check actual
game language switching; labels change without altering setting identities.
