# BlockWalk source parity

Pinned LiquidBounce commit: `ae0eb396c07f22383789b28d5912d714c6ca0a86`.
Sources are `ModuleBlockWalk.kt`, `MixinBlockCollisions.java`, `ShapeFlag.java`,
the registry-backed block adapter and `ListValue` import implementation.

## Complete behavior and settings

BlockWalk belongs to Movement. Its only setting is the Blocks registry list,
defaulting to cobweb and snow. Selected blocks replace the result of
CollisionContext.getCollisionShape with a full cube inside BlockCollisions.
The replacement occurs after the original context shape query and after the
native cursor, large-shape, moving-piston and suffocation filters. It does not
change physical block metadata, direct shape queries, outlines or render models.
There is no module-specific enable, disable or world-change state.

The block set is a process-wide snapshot, matching the Kotlin module singleton.
Only the real native window, outside a swapped LAN guest, synchronizes it after
the ordinary pre-tick engine update; a missing player does not prevent clearing
disabled policy. The shared world scanner and SceneCollisions, RegistryCollisionWorld
and LookupCollisions providers apply the same policy. EntityCollisions forwards
both shape-event hooks. Empty collision worlds retain their missing-chunk behavior.
The ShapeFlag bypass is process-wide, with scoped restoration on nesting and unwind.

Horizon's cached empty block/section and particle fast paths are bypassed while
shape changes are active, because cobweb, one-layer snow and even air can gain a
collision. Metadata filters remain intact. This is necessary for selected blocks
whose ordinary collision is empty, rather than a cosmetic shape replacement.

The original MobWorld supporting-block optimization used direct AABB intersections
instead of BlockCollisions. An actual MC case with a top slab and a thin box
spanning y=1 by 1e-7 exposed a false support in that optimization. The patch
routes mob support through the same actual ground_support/BlockCollisionScan
implementation used by player support. It preserves the nearest-center and
Y/Z/X tie policy and the native voxel-intersection tolerance, while delivering
the same BlockShapeEvent boundary to both consumers. Direct physical block
collision getters remain unchanged.

The setting child preserves File and Interop trees, the registry:block metadata
and Identifier path-first ordering. Block tokens accept strings and numbers;
unknown or invalid identifiers resolve through the defaulted registry to air.
ListValue resolves an entire replacement before replacing the old set, so an
invalid later JSON token preserves the previous value. Empty lists clear it.
Both formats and transactional failure are covered by the config child test
for the parent's engine integration.

## Verification

```text
python native/liquidbounce-horizon/test-block-walk-patch.py --out build/block-walk-05
python native/liquidbounce-horizon/test-block-walk-patch.py --with-airjump --out build/block-walk-combined-07
```

Eleven tests and Clippy with `-D warnings` pass. The fixture compiles the entire
actual transformed BlockCollisionScan plus the actual world_movement and
ground_support modules. It extracts production SceneCollisions and
EntityCollisions trait implementations, the actual fast-empty method, the
actual mob support method and the native hook methods. The real
horizon_mod_api_gen generates the client Host, events and dispatch code.

`oracles/BlockWalkFacts.java` executes the cached MC 26.3 registry and shape
methods, supplying 1,286 registered blocks and 20 representative block states.
It runs 1,200 complete actual BlockCollisions scans over six bounding boxes,
five policies and both suffocation filters. Cases include cobweb, eight snow
layers, slabs, fluids, a fence, cactus, scaffolding, barrier, selected air,
negative/cross-chunk coordinates and zero-extent bounds. The shape-return
modifier is derived from the pinned mixin; this does not execute Kotlin or
install a Java mixin. Thirteen registry import cases execute the actual Gson
adapter, actual registry and actual fastutil sorted set using source-derived
ListValue control flow.

Rust comparisons check hit count, cursor order and exact voxel geometry for
all 1,200 scans. For the 600 non-suffocating cases, both real player and patched
mob support are compared with the actual MC collision candidates and the
source-derived nearest/tie rule. An actual world_movement case falls through
an ordinary cobweb and lands on its full cube when enabled. Additional checks
cover the wrapper modifier, particle empty-cache gate, suppression, generated
policy dispatch and native-window/LAN-guest exclusions.

The composed AirJump scanner retains both modules' empty-section/block gates.
Both original event listeners assign Shapes.block(), so their ordering commutes.
ShapeFlag suppresses both, including the preexisting AirJump replacement.
Twelve composed tests and Clippy pass, with an additional 2,880 actual MC scans
covering ghost floor values, selected blocks, suppression and suffocation.
Standalone verification is refreshed in `build/block-walk-08/report.json` after
this compatible composition change.

Controlled boundaries are resident chunk/palette inputs supplied from MC facts,
the Game/window shell and ordinary direct physical-shape lookup. For
onlySuffocating=true, a metadata wrapper supplies MC's actual block predicate;
the production SceneCollisions provider retains Horizon's existing predicate.
The entire inherited scanner has a fixture-only type_complexity lint allowance
for its unchanged upstream generic signature. Parent registry/config integration,
production provider compilation and full game execution remain the parent's
responsibility. The isolated report does not claim full game integration.

Evidence is `build/block-walk-05/report.json`, command logs and `mc-oracle.json`.
Upstream hashes remain unchanged. Earlier failed reports retain fixture syntax,
chunk-local addressing and the native support discrepancy that led to the
common-scanner correction.

## Chinese display

Blocks has independent owned English, simplified Chinese and traditional
Chinese help. It explains the collision-scan boundary. Display follows the
actual game language and keeps the original registry identifiers. The shared
theme verifier passes with 235 owned setting/group descriptions in this batch;
the parent records final provenance, theme build and in-game language switching.
