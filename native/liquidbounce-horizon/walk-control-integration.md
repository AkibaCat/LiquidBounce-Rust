# Walking controller support integration

The owned `liquidbounce-behaviors/src/navigation/walk_control/` directory translates
the original Minecraft Agent common walking support from pinned commit `0abf69ac`.
It is deliberately not exported by `navigation/mod.rs` yet; the integration owner
can export it after connecting the native controller.

## Frozen source evidence

`build/walk-control-support-05/report.json` passed 12,298 sequential commands. All
26 original walking Kotlin files were compiled without changes against the pinned
common JAR, Java 25 and real registry/block/shape interfaces. The Rust snapshot
passed its oracle test and Clippy with `-D warnings`.

The evidence covers exact support state, StepProgress, Bridge/BridgeLine,
PathPoints geometry, Setbacks, StuckSpots, Stride, Overshoot, Swimming, Climbing,
Hops, StepCheck, jump intent, stair intent, Prediction, floor click/braking loops,
movement correction, public Leap lifecycle and NavigationContext.leap. Comparisons
include raw numerical bits, ordered player getter calls and ordered actual world
queries. The precise compiled navigation files are hashed in the report, after
the fixture adds its `walk_control` export.

The exact Body simulation tests use a scripted simulation to verify the control
protocol. The fallback prediction uses the actual original common PlayerMotion
for both modern and 1.8 mechanics. These are separate scopes.

## Native contract

Implement `ControlState`, `WalkPlayer`, `WalkBody`, and `WalkSimulation` in
`walk_control/body.rs`. `live_level()` must create a fresh live wrapper, whose
is-known check follows the real level height bounds and `hasChunkAt` calls.
`StepCheck` raw shape/state queries intentionally bypass the Terrain cache.

`WalkSimulation` is the host's real cloned 26.3 SimulatedPlayer interface. The
common land model does not implement it. Preserve ordered reads, including
`Hops::starts`' lazy movement-yaw callback and Stride's level/below query order.

## Remaining scope

Follower, Crossing, Telly, BlockActions, BlockTargets, Planning orchestration and
the native 26.3 simulation/world adapter are not established by this frozen
support report. New source units added later need independent source-oracle
evidence. Complete AutoWalk registration and full-client differential remain
`not_run`; this report does not mark a whole module complete.

## Additional frozen bridging and target evidence

`build/walk-control-targets-08/report.json` passed 40,533 sequential commands,
including all previous support cases. The original walking files and original
Bridging, Turning, ViewRotation, VectorExtensions and BlockSight were compiled
without changes. Rust test and Clippy with `-D warnings` passed.

This extends source evidence to complete ViewRotation and provided Turning
factories, all eight Bridging choices/cadences, Telly/Crossing control sequences,
BlockTargets tool/inventory/placement geometry, java.util.Random click points and
the full seenPoint grid with view-step margin sampling. It compares raw numbers,
ordered reads/shape calls, actual original Actions.place state and placement
identity/counts/fallback state. The exact Body simulation and clip hosts are
scripted for controller protocol checks. Crossing's nudge path is not exercised
in these sequences; BlockTargets is tested independently, including aimAt.

The native owner should implement `BridgeHost`, `TargetHost`, `SightHost`,
`TargetInventory` and `PlaceableBlocks` without substituting simplified raycasts,
snapshot states or computed inventory summaries for their original call sites.
`WalkPlayer::y` is a separate live getter: it must not read `position` first.

The new source files are in the same owned `walk_control` directory. BlockActions
world-facing ticks, Follower, Planning orchestration and real 26.3 adapters remain
outside this evidence. These added controller units do not establish complete
AutoWalk or full-client parity.

## Additional frozen BlockActions evidence

`build/walk-control-actions-12/report.json` passed 55,147 sequential commands,
including the earlier support/bridge/target cases and 14,614 added action
commands. All 27 original Kotlin sources remain unmodified. Rust test and
Clippy with `-D warnings` passed.

The new evidence exercises the actual original BlockActions aim/tick/answer/
expiration lifecycle against raw states, collision shapes, inventory and clips:
mine selection/start/continue/empty-shape tracking, action transitions, closed
door halves and cooldown, held/planned/ready/offhand placement, own-box guards,
failed clicks, replacement placement identity and retained verdict mutation,
logger/listener ordering, strict timeout and latency integer boundaries, and
levelChanged preserving ticks/cooldown/total rejection count.

Implement `ActionHost` in `walk_control/block_actions.rs` with live body,
player-box, latency, collision, use/destroy operations, listeners and the two
original warning callbacks. `PlacedBlock` uses shared Arc identity so observers
see later verdict updates on the same object.

`BlockActions::new` deliberately requires `PlacementOrder`. The fixture provides
an explicit pending-key order and the Kotlin fixture checks it against its real
HashMap. Production must supply actual JDK bucket/resize/tree-bin identity order;
no such adapter is implemented here. Rust HashMap traversal cannot substitute
for that contract. Follower/Planning, actual 26.3 world/clip/simulation bindings,
AutoWalk registration and full-client differential remain outside this evidence.

## Additional frozen StraightWalk and Shortcut evidence

`build/walk-control-shortcut-15/report.json` passed 67,524 sequential commands,
including all earlier cases plus 12,377 commands for StraightWalk and Shortcut.
All 28 original Kotlin files are unchanged; this run also compiles original
pathfinder/walk/StraightWalk.kt. Rust test and Clippy with `-D warnings` passed.

`walk_control/shortcut.rs` preserves complete side-drop checks, safe-drop/water
order, sample spacing and stop distance, ground-level step scans, body clearance,
known/border/hazard gates, real Terrain cache behavior, and farthest-first
plain-route candidate selection. The tests compare actual original world and
shape query order across flat, slab, wall, gap, deep, water, fire, unknown,
border, ceiling, diagonal and randomized worlds. Production world and exact
26.3 body bindings remain unimplemented here; Follower/Planning orchestration
and full-client parity remain outside this evidence.
## Additional frozen Follower evidence

`build/walk-control-follower-18/report.json` passed 75,714 sequential commands.
All 28 original Kotlin source files are unchanged. Rust test and Clippy with
`-D warnings` passed. Report SHA256:
`BCB81436C428DD11623DC594B9A9A672D23E0FE770A28725D7B357CFFBF554E5`.

`walk_control/follower.rs` implements route lifecycle, advance/skip and ordered
stepReached callbacks, off-path/progression helpers, real BlockActions mine,
place, open and unconfirmed-floor orchestration, external bridge floors and
persistent bridge lines, fall/stall limits, swimming, climbing, leaping, and
plain movement. Crossing is bound to the real common BlockActions, BlockTargets
and StepCheck implementations. Original start/fallback/hop logger callbacks are
included in the new source evidence and exposed by both host contracts.

`FollowerHost` supplies the live body/action contract, options, reached/fall and
Crossing logging callbacks, and partial-tick player position. `Follower::tick`
borrows the shared BlockActions instance explicitly. Routes use Arc identity;
feet-origin tags preserve Kotlin's `aim !== step.feet` behavior even when two
steps have equal coordinates. The placement tracker retains the previous Arc
and uses a monotonic identity, preventing allocator address reuse from looking
like the same placed object.

The fixture's Follower scenes explicitly initialize bridge options/rotation;
this fixed the failed follower16 fixture without changing common action logic.
The 18 report covers the helper provenance refactors added after shortcut15.
Crossing's target-nudge path is still not exercised by bridge sequences. Exact
simulation and ray-clip hosts are scripted; actual native 26.3 bindings,
production JDK pending-map order, Planning, AutoWalk registration and full-client
differential remain outside this evidence. Navigation remains unexported by the
shared production navigation root.

The earlier handoff's original-source counts were corrected by counting numbered
frozen Kotlin source files: actions12 has 27, shortcut15/follower18 have 28.

## Native integration contract after Follower18

The new `walk_control/ability_capture.rs`, `break_times.rs`, `planning.rs` and
`search_factory.rs` passed one isolated `cargo check --offline` on 2026-11-26.
They are outside the frozen Follower18 oracle. The shared navigation root and
AutoWalk registration remain owned by the main integration agent.

`Planning<B, T, S>` owns the original Replanner and StuckSpots. `B` implements
`ControlState + BreakState`, `T: Send + 'static`, and
`S: OwnedTerrainSearch<T, Route = Route<B>> + Send + 'static`. Its constructor
accepts `Arc<dyn Executor>` and a `WalkSearchFactory<B, T, S>` callback with
signature `(T, Vec3, SearchGoal, WalkAbilities<B>, Limits) -> S`. For the real
search, use `search_factory::walk_search` with
`Terrain<B, SharedLevel<L>>`; the staged destination uses that same snapshot.

`PlanningHost: CaptureHost + FollowerHost` adds four callbacks:
`max_length`, `following_path_debug`, `route_found`, and `route_searching`.
`CaptureHost: WalkBody` supplies copied-tool stacks/inventory, live options,
safe-fall distance, health and ladder behavior. Capture options preserve the
original live placement/break policies; `BreakPolicy` and copied tool stacks
must be `Send + Sync` for worker use. The avoid/unbreakable sets are copied;
the reserved set retains its Arc identity. `CaptureStack` adds count over
`BreakStack`, whose operations include copied stack, efficiency, harvest and
mining speed plus an empty-hand value.

The walking branch calls `planning.tick(goal)` first; when not walking straight,
call `planning.collect(host, follower, level, position)`, tick the follower with
the one shared `BlockActions`, then call
`planning.plan_if_needed(host, follower, level, &actions.unbreakable, goal,
radius, position)`. The exact follower call is
`follower.tick(host, actions, keys, input_y_rot) -> FollowState`. Feed stuck
positions into `planning.stuck_at`. Feed world changes into Planning and
BlockActions; process own placement answers through `BlockActions::answer`
before dispatching ordinary terrain changes. A straight walk drops planning,
clears the follower and idles actions; a reset also clears transient planning
state while stuck spots remain until explicitly forgotten, as in the source.

`FollowerHost` extends the live `WalkBody` and `ActionHost` contracts. It adds
bridge options, sneaking speed, partial-tick position, step/fall listeners and
Crossing start/fallback/hop logging. `ActionHost` extends `TargetHost`, which
extends `SightHost`; these files enumerate the native clip, inventory, block,
collision, view, use/destroy, slot-selection and latency operations. Production
must implement `PlacementOrder` for the original Java pending-map iteration
semantics. This contract is still distinct from actual native world binding.


## Capture and Planning targeted smoke19

`build/walk-control-capture-smoke-19/report.json` passed on 2026-11-26:
156 commands in three AbilityCapture/BreakTimes usability scenes matched the
unchanged original Kotlin sources. The same Rust invocation passed the
Planning constructor/stuck/drop/reset smoke (two tests total). This was a
single targeted run without Clippy or full vector replay. Report SHA256:
`C1B8BC86FF894E19445691309978A10A202AD691FCF64D881D03B39212D5FB54`.

The scenes cover disabled/enabled options, offhand/hotbar block counts, copied
tools and sets, mining hardness and retained break-policy updates. Planning's
search-result acceptance and actual native world/clip/body bindings remain
outside this smoke. Earlier Follower18 evidence stays separate. Full-client
differential and AutoWalk runtime registration remain `not_run` here.
