# GodBridge ledge replay source audit

Reviewed against the pinned LiquidBounce and Minecraft-Agent source, without
compiling or running a new oracle.

## Ledge selection

`features/ScaffoldLedgeFeature.kt` first calls `isCloseToEdge()`. If close and
`blockCount <= 0` or `calculateTicks(rotation) >= 1`, return only
`sneakTime=max(1,ticks)`. This preempts the selected technique extension.

The GodBridge extension reads snapshot **1**, after one simulated movement, and
acts only when `hasBackedOffEdge` is true. This flag records the hypothetical
sneak backoff even when ordinary input ultimately walks off the edge. The
existing native `navigation_physics::with_prediction` returns the appropriate
probe flag separately from the completed movement result; do not infer it from
the final displacement or from `onGround`.

Trace from `snapshotOne.pos + (0,current-player-eyeHeight,0)` in the supplied
rotation direction. Return no action if target is absent, or if both the target's
crosshair requirements and Scaffold's crosshair validity pass. A missing target
does **not** force a safety action.

Low block count is a strict `< ForceSneakBelowCount` comparison. Otherwise choose
uniformly from the configured nonempty enum set. If the chosen mode is Jump and
the discrete jump apex is at least 2 blocks, remove Jump from the configured set
and choose from the remaining modes, falling back to Sneak. The apex starts from
the player's final `jumpPower.toDouble()` and iterates `height += motion` then
`motion=(motion-DEFAULT_BASE_GRAVITY)*BASE_VERTICAL_AIR_DRAG` while positive.

## Input and tick phase

`LocalPlayerHooks.kt` constructs the prediction with the cached directional
input, current input jump, `isSprinting || keyPresses.sprint`, and
`player.isShiftKeyDown`. GodBridge must not reuse `isCloseToEdge`'s explicit
`jump=false,sneaking=false` input.

`PlayerSimulationCache` refreshes directional input at CRITICAL_MODIFICATION,
normal, and MODEL_STATE priorities, then Scaffold's ledge handler runs at
SAFETY_FEATURE. A native bridge should retain that pre-safety directional input
for the forecast; the ledge action itself changes the outgoing event afterwards.

`SimulatedPlayer.fromClientPlayer` passes
`PlayerSimulationCache.isLocalPlayerMoving`. It is true during the original
movement-input event. In that case the first forecast tick finishes the current
tick through `aiStep` and pose update, omitting `commonTick` and initial crouch
refresh; subsequent forecast ticks run the complete tick. The existing
`falling_forecast::visit_with_keys` always starts a complete native `local_tick`.
Integration must account for the actual phase at which Horizon dispatches the
event, rather than skipping native phases without first having executed them.

`isCloseToEdge` uses the same phase-aware simulation with jump and sneaking
explicitly false, then tests the swept edge ray and current/two-tick collision
bounds. Both uses require the actual world collision and source module hooks.

## Persistent technique state

GodBridge's `isOnRightSide` and Breezily's `lastSideways`, `lastAirTime`, and
`currentEdgeDistanceRandom` have no source enable/disable reset. Preserve them
across mode changes. Breezily initially uses the **double literal 0.45**; later
thresholds are sampled from its FloatRange and promoted to Double.

Breezily's `lastAirTime` is wall-clock milliseconds. Update it whenever the
block directly below the current player block position is air; on solid support
continue for at most 500 ms. Opposing edge tests are independent `if`s, and the
second wins if thresholds ever overlap. Resample only on a nonzero sideways
change; keep the prior direction in the center region. Replace left/right while
preserving the event's current forward/backward input.
