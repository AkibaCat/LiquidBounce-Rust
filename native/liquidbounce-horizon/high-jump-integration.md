# HighJump integration handoff

Pinned original: LiquidBounce `ae0eb396`, unchanged `ModuleHighJump.kt` and actual `CoroutineTicker.kt`. The implementation contains both complete modes and all settings. Full original/native client differential remains **not_run**; this is one module of the 236-module objective.

## Shared registration

- Export `liquidbounce_behaviors::high_jump`; store `high_jump::HighJump` as `BehaviorModules.high_jump`.
- Register module `HighJump`, Movement, key `liquidbounce.module.highJump`.
- Declare engine modules `high_jump_runtime` and config `high_jump`, exposing `HighJumpSetting`, `serialize_high_jump`, `parse_high_jump`, and `SettingChange::HighJump`.
- Compose `high-jump-patch.py` after AirJump, Sprint, Timer and TerrainSpeed transforms. Its returned helpers are `client_high_jump_state`, `client_high_jump_vertical`. The extra fragment `liquidbounce_high_jump.json` declares the free, synchronous `client_high_jump` hook; do not add a duplicate Game hook.
- Implement `Hooks::client_high_jump(&self,event:&mut ClientHighJumpEvent)` without Host, calling `self.high_jump_jump(&mut event.motion)`.
- Add common `WaitOwner::HighJump`, dispatching callbacks through `high_jump_callback(host,in_game,id)`.
- Call `high_jump_tick(host)` at GameTick after the common CoroutineTicker has resumed all callbacks and before entity ticking.
- On a real module disable, call `high_jump.disabled()` immediately. On Mode edits use `set_mode`; both cancel pending waits before re-enable/another mode can resume them.
- Do not reset `should_glide` on disable, mode change or world change. It is original singleton state. World-running loss only cancels pending waits when their next callback resumes.

## Complete settings

`Mode` defaults to case-sensitive `Vanilla`; the other choice is `Vulcan-2.7.5` with alias `Vulcan`. Global `Motion` defaults to `0.8f`, UI range `0.2f..10f`. Vulcan's `Glide` defaults false. Finite imported floats are not clamped by UI ranges. The original inactive choice subtree remains serializable. Glide help path is `liquidbounce.module.highJump.mode.vulcan-2.7.5.glide`.

## Original timing and actual consumer

Both modes synchronously replace PlayerJumpEvent.motion before the actual jump threshold/max/sprint body consumes it. Vulcan starts a new independent 100-tick parallel sequence on every jump, including zero or negative configured motion. The first subsequent outer Minecraft tick advances wait 1; nested Minecraft ticks do not advance it. At wait 100, set velocity Y to positive zero and `should_glide=true`. Multiple simultaneous waits remain independent FIFO registrations in the common ticker.

While Vulcan is running with Glide and `should_glide`, GameTick clears the flag on ground. Otherwise fallDistance>0 writes `-0.155` only at even actual player tickCount; nonpositive fallDistance writes `-0.1`. There is no screen guard. `should_glide` survives lifecycle changes.

The owned native transform splits horizon-world's original `jump_from_ground` into its unchanged body plus `jump_from_ground_with_power`. The actual MotionWorld head computes original jump power, calls the synchronous bridge, computes the original second jump-power expression, then passes the replacement power to the actual body. This preserves both original expression evaluations and prevents a delayed actor event from missing the jump's threshold/max operation. The tail still posts the existing jump/statistic/exhaustion events.

The free hook is declared under supported `horizon_client_lib::high_jump_bridge::native_client_high_jump`. Its callback is installed by entry before gameplay and invokes the generated `dispatch_client_high_jump`. No generator source change is needed. No Game Host is borrowed while MotionWorld owns the player.

The remote step captures the native-window/no-LAN-actor identity after `sync_player_body` and assigns `PlayerMotion.client_high_jump_owner` before actual physics. Constructor default is false. The head separately checks client_side, server-copy and replay guards. This is necessary because original `native_client_queries_allowed` alone does not check native window or client_side. Native GameTick/state helpers likewise gate owner access.

## Validation and boundaries

`collect-high-jump-original.py` copies and byte-checks the unchanged original module and actual CoroutineTicker. It executes the real Kotlin coroutines library against **48 continuous scenarios**, covering both modes and alias, seven raw motions, delayed/overlapping jumps, 0/1/99/100/101 tick lifecycle boundaries, nested ticks, glide/fallDistance and real tick-count parity. Module/player storage, registration and owner cancellation are explicit dependency shells. It is not an original complete-client execution.

`test-high-jump.py` executes eight tests, including raw-bit comparison of every original trace step, exact composed MotionWorld method with the real transformed horizon-world jump body, owner/server/replay rejection, and File/Interop configuration roundtrips. Test and Clippy `--all-targets -- -D warnings` pass. The composition syntax check contains 288 transformed Rust sources at this checkpoint. Evidence: `build/high-jump-tests/result.json`, `oracle.json`, `oracle-metadata.json`, `composition.json`.

Oracle SHA256: `4f987ec8ef46655067c9dd8006f3fd1b9ba8c75a37fd318433f097f15161e8a5`.

The independent cumulative pinned Horizon `fill_hooks` and complete `horizon-client --lib` typecheck **passed**, recorded in `build/high-jump-native-02/report.json`. It took 244 seconds including target-lock waits and reports only the pre-existing unused camera-perspective helper warning. Shared registry integration, common ticker registration, actual game execution, and complete client differential remain separate root integration work. No upstream or shared production staging file is written by these owned drivers.
