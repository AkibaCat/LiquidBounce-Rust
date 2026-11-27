# Timer source and native parity

The owned Timer implementation is complete and independently validated.
Shared registry/runtime/config integration, aggregate game compilation and
in-game checks belong to the integration owner. The handoff is
`native/liquidbounce-horizon/timer-integration.md`.

The fixed source is LiquidBounce ae0eb396c07f22383789b28d5912d714c6ca0a86.
Unchanged ModuleTimer.kt, Timer.kt, RequestHandler.kt, BalanceReset.kt and
CoroutineTicker.kt execute with real Kotlin stdlib, kotlinx-coroutines,
fastutil and JVM PriorityBlockingQueue. Only Minecraft, event registration,
config declarations and module lifecycle boundaries have shells.

## Modes and configuration

| Mode | Settings and defaults |
| --- | --- |
| Classic (default) | Speed 2f, editor 0.1f..20f |
| Pulse | NormalSpeed .5f, NormalSpeedTicks 20, BoostSpeed 2f, BoostSpeedTicks 20, OnMove false; speed editor 0.1f..20f, ticks editor 1..500 |
| Boost | BoostSpeed 1.3f, SlowSpeed .6f, NormalizeDuringCombat true, Balance 0..20 (editor -100..200), ResetBalanceOnFlag true |

All modes request priority 20. Pulse has exclusive real CoroutineTicker waits,
changes state before requesting a segment and resets state on enable. OnMove
controls starting a segment. Disable/mode change immediately cancels old waits.
Disable requests 1f at priority -20, subject to provider-running pruning.

Boost uses the previous input move vector, combat state and exact
InventoryScreen/ContainerScreen exclusions. Moving precedes the inventory
check. Combat normalization requests 1f; other requests retain balanced Float
arithmetic and invalid-range behavior. All eleven config leaves and three
choices roundtrip File/Interop, preserving signed zero, raw speeds, Int
truncation and reversed ranges. Chinese labels keep English config identities.

## Shared Timer and real consumers

The pure service retains provider identity, wrapping Int/Long arithmetic,
inverted signed-priority comparison, equal-priority heap behavior and
provider replacement via compact/heapify. Client-thread queries prune only
heap roots; off-thread reads do not prune. Requests expire at
resetAfterTicks + 1 against the FIRST_PRIORITY GameTick counter.

Float speed/balance arithmetic, Double realTicks, holds/history, dynamic loop
bounds and burst accounting match source. Player accounting observes final
cancellation. World change resets only source fields, retaining speed requests
and realTicks. The production adapter has independent locks. Its packet
listener observes cancelled corrections and matches immediate same-thread
versus queued off-thread render reset.

The native patch transforms the actual DeltaTracker.Timer Float delta before
residual accumulation. FixedStep retains client server-rate rules, sprint
behavior and its initial ten-tick cap. The actual game loop checks frameTicks
at every condition, including termination; bursts extend the initial bound.
Held interpolation reaches both game and camera partial-tick readers.

A mutable PlayerTick event is followed by a separate final-state event because
native Hooks have no listener priorities. Cancellation skips player cooldowns,
inventory pop, effect/sleep clocks and simulation, while retaining ambient
work and external sendChanges. Absent or chunk-unready players do not dispatch
the event. TerrainSpeed ordered-entity composition is a separate aggregate
integration check.

The pinned integrated.rs/execution_role.rs explicitly run single player on a
separate Server thread joined through the in-memory remote connection. The
patch changes only the window client's loop, never elapsed Duration or server
rate. An independent server clock still advances one 20-TPS step for 50ms
while the client accelerates.

## Evidence and boundaries

`build/timer-10/report.json`: passed=true, 4,839 Kotlin comparison commands,
29 Rust tests, Clippy with warnings denied. Raw Float/Double bits and source
state are compared for queue priority/replacement, expiry/provider pruning,
waits/mode changes, raw settings, invalid ranges, cancellation, hold/history,
bursts, world change and correction resets.

The fixture compiles the whole patched clock with its original tests, actual
frame-loop body, generated Host/API forwarding, production TimerService, real
packet pipeline, config child with copied parent codecs and selected real
remote cancellation-consumer blocks. Surrounding Minecraft/world/GUI services
are shells. Tests cover residuals, pause/freeze, ten-tick cap, burst/hold, final
cancellation, readiness, inventory/chest ownership, cancelled correction
delivery and render reset.

The report hashes unchanged Kotlin sources, owned files, host inputs and
transformed native source, and verifies upstream files remain unchanged. Its
fullGameIntegrationPassed is explicitly false. It does not compile the whole
tick_remote_level or full engine and does not claim screenshots, live timing
or registry installation. Aggregate compilation and game smoke evidence are
required before reporting installed gameplay coverage.
