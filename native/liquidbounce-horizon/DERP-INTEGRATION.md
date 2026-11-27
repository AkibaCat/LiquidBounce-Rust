# Derp integration contract

Pinned source: LiquidBounce `ae0eb396`, unchanged `ModuleDerp.kt`,
`ArrayExtensions.kt`, `ModeValueGroup.kt`, `CoroutineTicker.kt`,
`RotationsValueGroup.kt` and `ModuleManager.kt`. Production runtime is Rust;
Kotlin/Java are development oracles only.

## Owned production files and registration

- `native/liquidbounce-behaviors/src/derp.rs`: all five Yaw modes, three Pitch
  modes, independent stored Jitter/Spin yaw and source coroutine timing.
- `engine/src/config/derp.rs`: all eleven settings/mode requests, both named
  choice groups, every inactive mode's settings and raw Gson imports.
- `derp-host.rs`, `derp-patch.py`: one read-only real-player snapshot helper.

Shared identity: `ModuleKind::Derp => ("Derp", "Fun", "derp")`.
Forward `SettingChange::Derp(DerpSetting)` to `RuntimeRequest::Derp`, then
`DerpSetting::apply(&mut modules.derp)` in original declaration order. The apply
method returns unit. Register `pub mod derp` in the behavior library and use
`serialize_derp`/`parse_derp` in the shared config dispatch. The static settings
tree has Yaw, Pitch, SafePitch, NotDuringSprint after the common settings.

Compose `derp-patch.py` after the existing slot owner helper; append its returned
`client_derp_state` helper to the native API fragment. It adds no synthetic
tick/event. Parse its actual local yaw/pitch, logical Sprint mapping state and
actual player sprinting through `derp::Snapshot::parse`.

## Tick phases

1. At the existing outer Minecraft CoroutineTicker phase, call
   `Derp::begin_tick(running, fresh_snapshot)` once. Out-of-game calls with
   running=false cancel waits; stored mode yaws remain unchanged.
2. Preserve the existing RotationManager FIRST_PRIORITY GameTick update and
   its possible real yRot reset. Do not run the new module handler before it.
3. During ordinary GameTick, capture a fresh native snapshot, call
   `start_yaw_handler(running, snapshot)` and then
   `rotation_target(running, snapshot, &mut random_unit_f32)`.
4. Submit a returned target to the shared manager with provider identity 4,
   `ROTATION_PRIORITY=-20`, and `ROTATION_PROCESSOR=7`, using the shared actual
   allowed-to-update guard. Provider 4 is live only while the Derp module runs.

ModuleManager's source order is Derp(Fun), Twerk(Fun), Sprint(Movement),
AntiAFK(Player), AutoFish(Player), AutoRespawn(Player). Derp's Jitter and Spin
handlers are registered during choice construction, before its main handler.
Keep its ordinary GameTick work before Sprint: both may submit priority -20,
and stable equal-priority insertion/order is meaningful.

CoroutineTicker is one cross-module pending FIFO. Existing local schedulers
already preserve their own waits, but fixed module resume order does not prove
full cross-module callback equivalence. In particular, AntiAFK Old's ten-tick
wait normally precedes Derp's most recent one-tick wait; its yaw effect must be
visible to the following Jitter iteration. A common global wait registration
order is required for arbitrary simultaneous continuations across modules.
The source oracle below verifies Derp in isolation and does not claim that
shared scheduler property.

Jitter captures ForwardTicks when its run starts; it reads BackwardTicks live
only after the forward phase's final wait completes. Each iteration reads live
player yaw. Zero/negative counts perform no iterations. After its final wait
completes, the same tick's GameTick can begin the next run.

Spin increments its stored float yaw by the raw integer Speed before the main
handler reads it, then waits one tick. It never wraps or clamps. Sprint guards
skip only the rotation request and its random draws, not Jitter/Spin updates.

## Rotation processor and lifecycle

Processor 7 must call `Derp::process_rotation(from, to, last_managed,
player_last, random)` both while aiming and resetting. This runs a separate
fixed default Linear smoother with vector component limits. A direct per-axis
180-degree clamp is different. Defaults are Silent movement correction,
reset threshold 2, reset lifetime 5 and considerInventory=false. Targets remain
unnormalized until the shared RotationManager normalizes them.

The source's RotationsValueGroup is private and deliberately never added to
the tree. Do not expose or import a Rotations subtree or reuse mutable
NoRotateSet/AntiAFK settings. Its processor identity is independent even though
the common implementation is reused.

Changed Yaw mode writes call `set_yaw_mode` and cancel the old wait even outside
a world; same-mode writes preserve it. Pitch mode changes do not cancel the Yaw
run. Actual CancelSequences/OnDisabled callbacks call `cancel_sequences`,
retaining both stored mode yaws. Preserve the original ClientModule distinction
between in-game lifecycle callbacks and out-of-game enabled-value writes.
Do not remove an already submitted RotationManager target on a mode change;
the provider is ModuleDerp, not an individual mode.

## Evidence

- `build/derp-02/report.json`: four production tests and Clippy with warnings
  denied, covering all angle modes, scheduler boundaries, raw settings,
  default hidden smoothing and the shared 256-vector JDK25 RNG oracle.
- `build/derp-config-03/report.json`: 40 complete production config tests and
  Clippy with warnings denied. Includes all fifteen Yaw/Pitch combinations,
  inactive settings, raw imports, error continuation, lifecycle forwarding and
  the absent hidden tree. An explicit registration-only shim is recorded.
- `build/derp-source-02/report.json`: unchanged pinned `ModuleDerp.kt` compiled
  with real Kotlin suspend continuations and actual JDK25 ThreadLocalRandom.
  Rust replays identical commands across 150 ticks, matching raw yaw/pitch
  float bits and RNG draw counts, including both SafePitch states, live repeat
  counts, same-mode writes, disable/world guards, sprint skipping and nonfinite
  angle imports. Five tests and Clippy passed. Minecraft/settings/event stubs
  are explicit fixture boundaries; hidden smoothing is tested separately.
- `build/derp-native-01/report.json`: real API generation and complete
  `horizon-client --lib` compilation passed in a separate pinned snapshot.

These fixtures do not claim shared module registration, arbitrary cross-module
continuation FIFO, or actual game execution. Those are parent integration work.
