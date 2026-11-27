# AutoWalk and Minecraft Agent parity

Baseline: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`,
Minecraft Agent `0abf69ac35157f33348310326017f4e3fea6b95f`, published common
snapshot `1.0.0-20261120.103027-24`. The Agent archive, published sources and
binary are recorded in `minecraft-agent-baseline.json`.

AutoWalk is **not complete, exported to the shared behavior library, registered,
or included in the completed-module count**. A forward-only substitute or
Horizon's different existing navigation implementation is not equivalent.

## Frozen foundational units

| Unit | Evidence | Scope |
| --- | --- | --- |
| Input, Keystrokes, generic AStar | `build/navigation-core-03/report.json` | 14,301 Kotlin/Rust operations; Java priority-queue ties and double ordering |
| Geometry, regions, destinations and staging | `build/navigation-goals-02/report.json` | 7,811 operations; unchanged destination/region source and real published Agent math/Terrain.isKnown |
| Sliced AStarSearch, Limits, Outcome, Failure | `build/navigation-search-01/report.json` | 6,847 operations; six unchanged Kotlin source files, measured actual JVM compute nanoseconds supplied to the Rust clock seam |
| BackgroundSearch and Replanner | `build/navigation-background-04/report.json` | 2,467 operations; two unchanged Kotlin sources, controlled actual Executor, real published Terrain state invalidation; separate real Rust thread cancellation/ownership test |
| AutoWalk outer modes and lifecycle | `build/auto-walk-04/report.json` | 440 operations; unchanged AutoWalk source, extracted original mode lifecycle/math functions, actual MC 26.3 math and Java String.equalsIgnoreCase |
| AABB, Clip, VoxelShape and Shapes | `build/navigation-shapes-01/report.json` | 582 operations; two unchanged Agent sources, raw box ordering/normalization, ordered ray clipping, collision axes/tolerances and shape identity |
| Terrain, StandHeight, BlockShapes and BlockKinds | `build/navigation-terrain-03/report.json` | 1,923 targeted commands; five unchanged Agent sources, observable world/state/shape calls, stand/crouch/footprints/bowls, openables, hazards/climb/restitution, ordered collision and invalidation |
| Reach and complete flight route generation | `build/navigation-air-reach-03/report.json` | 743 targeted commands; 15 unchanged Terrain/Reach/flight/AStar sources, observed sampling/world calls, full sliced flight search and previews, simplification/climb fallback and failure budgets |

Each report records its source hashes, executed commands and limitations. Rust
tests and Clippy with warnings denied passed. Later additions to the navigation
directory do not extend the scope of earlier reports.

Background/Replanner freeze hashes:

```text
background.rs 338aa0b52b1ef386bd40585d4fd579ac60b296185cbf4a6f820068802360794a
replanner.rs  0f7c53eb7470b4a3a68b2f527a2c1b6274fd3434e3f3b02adbac1c843014ada4
```

The Background fixture covers pending slices, retaining previews when a new
preview is null, completed same-key reuse, the 20-poll block-change threshold,
state-equivalent and irrelevant changes, retaining changes without a level,
1024 pending changes versus overflow at 1025, exception-to-NO_PATH conversion,
staging and height/loading rules, failure cooldown, exact goal-move threshold,
reset/forget/world-change distinctions, key equality and wrapping clocks.

`cancel(false)` targets the dependent future returned by `.exceptionally`, not
the original `supplyAsync` future. Consequently, even queued upstream work still
runs after cancellation. The Rust side drops the result receiver and retains
owned snapshot/search data in that worker. `navigation-background-01` preserves
the initial wrong queued-cancellation assumption; `02` preserves a fixture-only
known-block cache mismatch. `03` passed before final formatting and the real
Rust-thread test; `04` is the final frozen evidence.

AutoWalk outer-mode freeze hashes:

```text
auto_walk.rs      179982a7f033625c2bb0671c4d6c4b9f3052a8f9f2620d7f2ae3b1a7b494119e
java_case.rs      1dbd2dd69667b3804cba9cdc32ccb67e5d25eb79af17b74ea56a6bf92e62b4dc
java_case_data.rs 26a9046fd7ce5f677107c23f602c6f3841570be144c6d5a14666c660a3a97b1b
```

The outer fixture checks every seven-key input mask with the module disabled and
enabled, direction goal caching and exact refresh thresholds, unknown-chunk
recomputation versus Goto goal retention, root-vehicle arrival and notification
before reset/halt, nonfinite floor/cast coordinates, first matching player,
last-seen retention, Java Unicode case comparison and mode/toggle lifecycle.
World standability, navigator operations and the surrounding event/config
framework are shells; actual MC direction vectors are supplied to the Rust host
trig seam. This does not validate terrain physics, navigation or native input.
The Java 25 simple case mapping contains 1,483 nonidentity code points and was
extracted by `JavaCaseFold.java`, then formatted with rustfmt. Runtime comparison
uses UTF-16 length and simple `lower(upper(codePoint))`, without full case
expansion or game-locale casing. Development failures `auto-walk-01/02` concern
the VOID_AIR fixture initialization/opt-in, and `03` a fixture Clippy lint.

The world-change hook is documented separately in `WORLD-CHANGE-PARITY.md`.
It signals lifecycle changes before sound cleanup; a future native navigation
adapter must invalidate/release immediately and must not sample a new-world
snapshot during that callback, because the host backing storage may still hold
the old world.

## Remaining implementation

- Complete all AutoWalk inherited settings and their native serialization.
- Complete Agent terrain, block-state/shape caching, collision and simulation
  semantics, terrain invalidation and live/snapshot native adapters.
- Complete walk, flight, glide/elytra, vehicle/boat route generation, route
  movement/action controllers, server confirmations and Navigator dispatch.
- Complete LiquidBounce LocalPlayerBody: simulation, rotations, silent hotbar,
  break/place/use, exact keys, sprint, vehicles, flight/elytra/recast and release.
- Wire real block/world/correction/confirmation events and route rendering;
  provide every inherited setting and its English/simplified/traditional help.
- Validate the complete module independently, then in the cumulative native
  engine, then in a real game. Only that permits registration/completion claims.

Shape freeze hashes:

```text
aabb.rs        992f52b782c1bcada81f76e6f142644981d74c7d2a9f07e89aea5933f5946eb7
voxel_shape.rs 40aafd50bddbf8da7bca783a66868e296d9e08f8fa47a62d0652943b22181ba2
```

The Terrain fixture uses a controlled block/registry/world adapter, including
identity-preserving state lookup, singleton versus equal duplicate shapes,
neighbor-dependent collision shapes, fluid/kind/block facts and optional state
properties. The original Terrain and helper sources execute unchanged against
these observable facts. The Rust constructors keep width/height/open-door facts
immutable, and snapshots/cache ownership remain the caller's responsibility.
This validates Terrain algorithms, not the actual MC/native registry extraction,
world snapshots, player simulation, reach ray tracing or route controllers.
`navigation-terrain-01` preserves a fixture-only empty-shape encoding mistake;
`02` matched every command but failed two collapsible-if Clippy warnings; `03`
includes final formatting and passes all checks.

Terrain and air/reach freeze hashes:

```text
terrain.rs        943652166de88615d6389058d90ccd89d21a54e6e58cd7737b27ce33cebda4eb
terrain_motion.rs fcd10d619dae310156a2c30d0767a65030ed5b696210d715e9fcf404e3dc1076
terrain_cache.rs  079c29bf2f893cc4d66508073420f5cb9c8d491718f685c156842e718c992349
reach.rs          8c7c956026cdf4733054b8669661c4bb9bcb8519ba68e11aa12426f089f4fb37
flight.rs         ec19905c05f82c4222d7deb41ab8d8b614c051c89a816bb4cd3f9c65e7ba8f57
```

`build/navigation-air-reach-04/report.json` revalidates the same 743 commands
after adding the non-mutating `FlyMoves::terrain_mut` borrowing accessor for
owning background searches. Tests and Clippy pass; flight arithmetic is unchanged.
The current flight freeze hash supersedes the value above:

```text
flight.rs         af1925ba3ecb15622ce6f6bd410363374a0c97a5e83c6988eb2676a015892aa1
```

The Reach fixture observes every sampled ray endpoint and its outline/collider
selection, fluid NONE and null entity. Clip answers follow explicit patterns,
so it covers Reach's sampling/visibility decisions and supported/air feet
positions, not the host ray traversal. Stable closest-first ties, eight-position
truncation, body exclusion, reach bounds and default float eye height are tested.
Flight executes unchanged abilities, 26-neighbor Moves and RouteSearch, including
diagonal intermediate cells, water and ground cost factors, actual Terrain/FMA
line samples, full AStar slices, previews, already-there and fits-above behavior,
strict goal-end length bounds, route simplification, climb fallback, incomplete
routes and failure budgets. The fixture supplies measured actual JVM elapsed
nanoseconds to the Rust clock seam. `navigation-air-reach-01` preserves a missing
compiler opt-in flag; `02` passed; `03` keeps cached constructor facts immutable
and includes final formatting. Flight input/controller/native body remain work
in progress, so this is not full flight or full AutoWalk completion.

## Walk foundations and course

`build/navigation-walk-base-03/report.json` passes 945 targeted commands against
12 unchanged Agent source files (five Terrain helpers, WalkAbilities, Costs,
Water, Nodes, Changes, Course and MotionTraits), plus Rust tests and Clippy with
warnings denied. The comparison includes the exact observable world/state/shape
query order, float ground drag and air accounting, partial floor/ladder centers,
swimming/crouching room, strict half-health survival, water/ladder landing
precedence, unloaded/deadly sides, replaceable/reserved cells, floor/liquid/
falling-block mining exclusions, finite mining costs and duplicate change lists.
Course covers direction/aim thresholds, run-up/edge/clearance/safety queries,
full ballistic estimates, braking and climb catches. PlayerMotion is only a
real JVM value capture in this fixture; full simulation is a separate unit.

`navigation-walk-base-01` preserves missing JVM 25/friend compiler flags; `02`
preserves a fixture-only `minecraft:` block-name prefix mismatch and three
shorthand-pattern lint warnings; `03` contains the final formatting and passes.
World and registry facts remain controlled native adapter seams. Full Moves,
walk RouteSearch, controllers and native bindings are still incomplete; none
of these units registers AutoWalk or counts it as complete.

Walk foundation freeze hashes:

```text
walk/abilities.rs 7dd3c4a84b56680b777dad6da5643da69f88c663f709afc012a6483c10d405c5
walk/changes.rs   c7fd1869327e58f8a68013b994f2e1c8df892c63d3414d76e9bdbb56009c7544
walk/costs.rs     f863364aeaf50590ddcaa8de842cb7147bbcdae08b2386ab54a2cd46fe254020
walk/course.rs    b15ac6ae96f1bc572736749afb91d6e6d6f5ba250bdb92c5137d77277cc8ea96
walk/nodes.rs     7aff6ca2b2ed479b107533c58485fa44c3b78fe9b52f99e4dd755aa151245ed2
walk/water.rs     d814cb66ccdc424b6fb5cf3c334ccc1000e741a4fd87aa450c1e4ba8bcb8b5e2
```

The search core now offers `AStar.advance_with_context` and the default
`SearchProblem.neighbors_with_context` hook. A walking neighbor callback must
read the actual current `cost` and `previous` maps, before any of its returned
edges are relaxed, as Kotlin's callback reads its owning `search`. No copied
or predicted search context is used. Existing callers delegate through the
same loop. `build/navigation-core-04/report.json` revalidates all 14,301 original
core commands. `build/navigation-search-02/report.json` passes 6,886 commands,
logging every node cost and predecessor inside every callback against unchanged
Agent source, including targeted duplicate/stale entries, cheaper predecessors
and zero-cost cycles; Rust tests and Clippy pass for both runs. These replace
the earlier core/search freeze hashes only; all other frozen units are intact.

```text
astar.rs  1dfa2cbff16ef906e93772271d92bd8b29371934cfdcf784deba743306d74d62
search.rs 2c5ca766d181a2f935b1f40148b858b49f1ab6ec4a793b558ec9fb4b3ffdf8f9
```

`build/navigation-straight-01/report.json` passes 215 focused commands against
unchanged StraightWalk and five Terrain source files, plus Rust tests and Clippy.
The fixture observes every side/drop/clearance query and half-block sample,
including zero-length behavior, endpoint tolerance, stepping onto/off slabs,
off-center/diagonal clearance, initial unknown sides, hazards, water and void.
This freezes only straight walkability; the full route/controller remains work.

```text
walk/straight_walk.rs b988c4703d14cd333288cc7cad2a108b174cc899db974022b1a8b337b1ef2b4c
```

## Complete walk route generation

`build/navigation-walk-route-02/report.json` passes 793 targeted sequential
commands against 37 unchanged Agent source files (Terrain, all required walk
and jump units, the AStar search layer and the complete shared land simulator).
The complete Moves, Parkour and RouteSearch translation now matches every
emitted offer, its order, Step fields, Jump plan, lazy route points and full
observable world query sequence. Cases cover flat/slab/low-ceiling movement,
ascent and falls, honey and slow floors, crouching, climbing, swim pose,
mining and placement changes, doors, stock and reserves, avoid sets, held
footprint adjustment, exhausted air, real AStar costs/predecessors, partial
bridges, failure budgets and both careful and risky parkour. The fixture feeds
the actual JVM elapsed computeNanos through the Rust clock seam. Rust tests
and Clippy with warnings denied pass. `navigation-walk-route-01` preserves a
successful behavior comparison followed by an unformatted jump-module lint.

This freezes walk route generation, not the movement controllers or AutoWalk.
The live registry/world/native body are still explicit adapter boundaries.
The shared jump chain has its own additional Leap/AssistedJump checks.

```text
walk/route.rs        cc1de12c6f83b2495bb40e80ff8bfa3e6e67cadf04a54bfdb9ca9da9f22dc629
walk/parkour.rs      c07ec684c7ceea434fb0fb16438241ca49b56ba22cca4fae617f8968476174f3
walk/moves.rs        34ab3cf3112a9d88cfe90695e31fc867847d49e4dbc58163a2e1b8dbb7ed92f9
walk/moves_surface.rs cea80b18f8e47a07c4db15cd7e1badabd5332a1ce5d00bc5bccc1a53f38271cd
walk/moves_actions.rs bbfb0bf8b6bc44a62f262b70f91f92e125b3b6c6369f83f62a0b66a0715c7d22
walk/moves_emit.rs   6b060ab0181d567bec8d47809659aeec660eb2debc9466ef6276525370d491ed
walk/route_search.rs de95d3bcc4e7251e452814f8f096cea87fcb9106f7cd994d57a14242445c2489
```

## Flight handling measurements

`build/navigation-flight-handling-02/report.json` passes 11,111 sequential
commands against unchanged Handling and FlyAbilities. Every one of the 128
input masks is exercised at nine targeted yaws, together with drag/thrust
blending, base-value resets, NaN and minimum speeds, exact movement thresholds,
collisions, skipped/duplicate/wrapping ticks, held vertical keys and coasting.
Private measurement state, all public results and the complete ordered live
Player getter trace match. Rust tests and Clippy with warnings denied pass.
The first run preserves 96 cases with Windows UCRT sine/cosine rounding
differences; the second uses Horizon's unchanged MIT-licensed jmath::exact
sine/cosine arithmetic. Its documented large-angle platform fallback remains;
the evidence establishes the listed angles, not every possible floating yaw.
Native Player binding and the flight controller remain separate work.

```text
flight_handling.rs b6bf3db12bee33a9712e1b92531ebaa32c2a38088d62be7ab09b5a3a55c960e8
flight_math.rs     dad0f05513485c8a7f573c9a4fef84f8398c22ba0d7ee18a425c0ebece2d4608
```

## Flight following and capture

`build/navigation-flight-follower-01/report.json` passes 1,909 focused sequential
commands against eight unchanged Agent source files. Full Follower leg state,
keys, targets, sprint, takeoff and landing, sharp corners, lookahead, off-path
checks, misses, stalls, direction dead angles and segment distance match.
Flight capture and ProgressWatch include exact ordered Player, Abilities,
Body, collision and press calls. Rust tests and Clippy with warnings denied
pass. The fixture uses a minimal real Body/options context holder; native
world and player bindings, flight orchestration and AutoWalk remain work.

```text
progress.rs        9eb4749b22cf758a999484c824714fdbb08b2913503f5d35d50bb776d22a51f3
flight_body.rs     ef4220afe89b2de8b729b39075427b95c7f7a11d9eda14c367b15775ad161a7c
flight_follower.rs 65f48b91ff0d9abc2cf2a97de1f187baeb0f24eb2e34fd6e2d0fd59a7f1bf1cb
```

## Flight navigation and owning search storage

`build/navigation-flying-03/report.json` passes 927 sequential commands against
11 unchanged Agent source files. Complete Flying, Follower, Handling,
Replanner, BackgroundSearch and staged destination behavior agree, including
all follower state, output keys, ordered reports and world/player queries.
The search provider is controlled to exercise searching, preview, found,
already-there, failure, exceptions, queued cancellation, retry timing,
off-path urgency, stage lead, goal/radius/control changes, missing worlds,
cache invalidation and level changes. Full route generation has its separate
unchanged-source oracle. Rust tests and Clippy with warnings denied pass.

The final fixture uses the actual production Terrain and SharedLevel snapshot,
shared_destination and SearchSlot/StoredWorker. It observes that staged goal
checks and cached block reads use the same immutable snapshot while main-thread
changes invalidate the Terrain owned by the active search. Owning searches move
through the original worker lifecycle without self references or unsafe code.
`navigation-flying-01` preserves two fixture-only Clippy failures after matching
all 927 commands; `02` passes; `03` replaces the controlled terrain seam with
actual production Terrain and shared destination storage and also passes.

Route report callbacks remain an observable context seam; listener identity
deduplication and native body/world binding remain separate work. This freezes
flight orchestration, not complete AutoWalk, which remains unregistered.

```text
flying.rs         e683fd1fa80afedf76042e285e4739662df2dc0aa6269f52641063d536363108
owned_search.rs   56e869a120401b70e8b1bb2f1637e665c581ddcb7764641f5642aea2efe462db
staged_search.rs  b7719deb00878f14eae14452ce2062a56ef4058e0cd8e6d5ea1b4960da873420
route_storage.rs  7c2c1b75f20807b81dc99b8679c796f91978725bd02ae4eb7d41e486d36b843a
```

## Glide abilities, physics and landing/cruise controllers

`build/navigation-glide-base-02/report.json` passes 3,242 sequential commands
against six unchanged Agent sources: GlideAbilities, Rockets, Steering,
ElytraMotion, Landing and Cruise. Every result bit, private pitch-sweep field,
rocket burn/count state, cruise altitude/tick/landing handover state and ordered
item getter, column and fallback query agrees. Cases cover inventory/durability
overflow, absent firework duration, minimum flight, every burn transition,
targeted yaw/pitch/motion/boost combinations, signed zero and NaN angles,
landing orbit/climb/approach behavior, persistent trajectories, unknown columns,
ceiling limits, budget estimation, fallback and cruise-to-landing transitions.
Rust tests and Clippy with warnings denied pass. The first run preserves four
NaN sign discrepancies in yaw; 02 matches HotSpot's canonical Math.atan2 NaN.

Elytra direction uses the already verified original 65,536-entry float table,
and lean sine/cosine and atan2 use the frozen JVM-compatible arithmetic. Native
equipment/component/height providers, glide route generation, Passage and
runtime Gliding remain separate; AutoWalk is still unregistered.

```text
glide/abilities.rs     7533b554b1aba8c30c997a0aa59958d30f7da7e4c304ff35b909681b2c923a97
glide/rockets.rs       11363e0eb3298758607bedf4ab263bffc519c9c5d1a2c4fc3b43ec64af5d36e5
glide/elytra_motion.rs e0a0bee51afae7af0abfeee4764225ba2034ab3562cc9fce4cbf2777e5bc27de
glide/landing.rs       04b2b4641477747afb725263bed47a9c8393a5afb041f2445b1767b8da76542f
glide/cruise.rs        7e7035066906961bc2e870f467230140a74cb046dc82b442f53501245273af98
```

`build/navigation-glide-route-02/report.json` passes 246 sequential commands
against 12 unchanged source files, including the complete glide RouteSearch,
Landing, ElytraMotion and Terrain. Exact output routes, sampled previews,
private attempt position/motion/tick/approach/rocket state and every ordered
world/shape query agree. Cases include takeoff, low durability budgets, direct
and high-approach retries, safe/unsafe touchdown, blocked starts, roofs, walls,
slabs, water, unknown cells, region bounds and multi-slice outcomes. Rust tests
and Clippy pass. Run 01 preserves fixture compiler errors (missing fastutil
classpath and the Found property name), corrected in 02. This freezes glide
route generation; Passage, Airways, runtime Gliding and native bindings remain.

```text
glide/route_search.rs 9dbc9a7715a162748d09f1b650a70bdb99be6ebcf11c9bbcec48b530ed092c24
```

## Airway search and passage steering

`build/navigation-airway-03/report.json` passes 723 sequential commands against
18 unchanged Agent sources. Complete AirwaySearch, AStarSearch, Terrain and
StageDestination agree on every edge/cost/goal/heuristic, cache, sampled line,
route, preview and ordered snapshot/world/shape query. Cases cover clearance,
26-neighbor corner restrictions, unknown edges, stage routes, simplification,
escape space, incomplete routes and search limits. Rust tests and Clippy with
warnings denied pass. Runs 01 and 02 preserve fixture-only Kotlin syntax and
internal-method reflection failures; javap confirmed `neighbors$main` for 03.

`build/navigation-passage-01/report.json` passes 694 sequential commands against
12 unchanged Agent sources. Complete Passage, ElytraMotion, Landing and Terrain
agree on all controls, route/arcs/extension state, projection/lookahead,
coarse/fine rollout costs, rocket state and ordered snapshot/world/shape queries.
Rust tests and Clippy with warnings denied pass. These freeze airway route
generation and passage control; runtime Gliding/Airways/Contacts, native binding
and full AutoWalk remain separate work.

```text
glide/airway_search.rs 0e5562c811208c3208e4f350ea6b47f262fa9644f41f9a1ed6aa3e107b74e642
glide/passage.rs       3df72e9376db144a5c2bf5d2a22a7f95997f0aef8c66201e8725d326feb11e0d
```

## Roof bounce and glide contacts

`build/navigation-glide-runtime-base-01/report.json` passes 645 sequential
commands against 11 unchanged Agent sources. Complete RoofBounce, Contacts,
TargetProgress and Terrain agree on all private bounce/contact state, reports,
input/look actions and exact live player/world/shape query order. Cases include
flat side-strip scans, exact distance/drop thresholds, unknown reach, obstacles,
mid-route goal changes, fresh/recast jump presses, every collision flag
combination, contact normals, persistent touching and recontact. Rust tests and
Clippy with warnings denied pass.

Contacts uses the original JDK HashSet spread/bucket/insertion-chain iteration
for at most six entries. Direction enum identity hashes are external process
facts, captured by the fixture from its oracle JVM. A pure Rust host assigns
six stable identities itself; the source gives no cross-process identity or
callback ordering guarantee. No Agent or JVM identity provider is required by
the production code. Gliding/Airways and native binding remain separate work.

```text
glide/roof_bounce.rs c87ad8193f4efb9fd240ca716f0d90ef04b3356d483a9a67948cb60ad2ac427c
glide/contacts.rs    64a105f06b516bfabb45b8d522a3e736ce1fb3249ca5cad2d334f5bcd13a84cd
```

## Complete gliding orchestration

`build/navigation-gliding-06/report.json` passes 841 sequential commands against
24 unchanged Agent sources, including the complete Gliding, Airways, Contacts,
GlideCapture, Terrain, BackgroundSearch and Replanner. All results, private
controller/replanner/rocket state, input/look/item actions, progress reports and
ordered live player/world/snapshot/shape queries agree. Cases include equipment
capture order, takeoff/open cooldowns, stale route starts, retry/failure paths,
falling takeover, airborne passage and stage refreshes, offhand/hotbar rockets,
ceiling/unavailable worlds, direct movement and cache invalidation. Rust tests
and Clippy with warnings denied pass. Search providers return controlled
outcomes here; complete route generation has its separate frozen oracles above.

Runs 01--03 preserve fixture compile/runtime failures. Run 04 preserves a
fixture-only nested Mutex deadlock introduced while converting the test world
from Rc/RefCell for background workers. Run 05 passes all differential checks
but fails Clippy's int_plus_one in that same fixture; run 06 fixes that expression.
No production navigation behavior was relaxed to pass these runs. Native
bindings and complete AutoWalk remain unintegrated and unregistered.

```text
glide/airways.rs b798a58af96fc851f23460a161085a3eab6e89c6fbedf2f9c4b6c0c8e882b29e
glide/capture.rs 26f7f16809d68cbcdff24587319f8aabce3be3332cefbf2b1bca0e762ac1f9c0
glide/body.rs    96a1251560d93173c672f1d786813d82b01b5d38e353149065e644b49de67105
glide/gliding.rs 084e5b8812bd10884e9adb4e29c58586fafc938cf884b65ec4c8825467052014
```

## Ridden vehicle route generation

`build/navigation-ride-01/report.json` passes 1,475 sequential commands against
17 unchanged Agent sources. Complete Vehicle, Moves, RouteSearch, Terrain and
AStarSearch agree on captured abilities and getter order, jump heights, cached
resting spots, all ordered neighbors and exact costs, line passes, routes,
previews, search state and every world/shape query. Cases cover water surfaces,
ice friction thresholds, side/corner anchors and wide vehicles, ledges and
jumps, safe falls, blocked/unknown starts, start adjustment, exhausted partial
routes, boat rider-height goals, destinations without ends and search limits.
Rust tests and Clippy with warnings denied pass. Runtime Steering/Riding/Boating
and native bindings remain separate work; AutoWalk is still unregistered.

```text
ride/vehicle.rs      511ab105f3d4157abbc0a1e6c0ce3a683fb8e3e2525a87cff28e00af7953c420
ride/moves.rs        92446c9eadf73907bd08344a3aab44f38c47cf1d4374fd48eb03eca29cf2f6e7
ride/route_search.rs 6cff2b0982fb3c07cc655d4004ae62353991d4ea66d85879caaf1f915c9519ee
```
