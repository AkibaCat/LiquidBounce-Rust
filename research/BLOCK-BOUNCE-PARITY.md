# BlockBounce source parity

Pinned source: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`,
`ModuleBlockBounce.kt`, `AABB.isBlockAtPosition` in `BlockExtensions.kt`,
and the `PlayerJumpEvent` injection in `MixinLivingEntity.java`.

## Complete module behavior

The module is in Movement and declares Motion 0.42, with the original UI range
0.2 through 2.0. Imported finite values retain their exact Float value without
UI clamping. The module has no extra per-enable or per-world state.

The PlayerJumpEvent changes the final single-precision getJumpPower result.
The modifier therefore runs after the jump-strength attribute, block jump
factor and Jump Boost contribution, before the existing negligible-power
early return. Existing upward velocity, sprint impulse and sync marking keep
their original Horizon behavior. Other living entities receive an identity
modifier through the shared trait's default method.

Block detection reads the real player's current bounds at each jump. It
subtracts the Double 0.01 from minY, then scans only floor(minY) at inclusive
floor(minX) through floor(maxX) and floor(minZ) through floor(maxZ), with X
outside Z and an immediate successful return. This matches the source
extension, including exact integer edges; collision shapes and other vertical
layers do not enter this check.

The complete vanilla 26.3 matching set is the slime block, honey block and all
16 bed colors. State properties do not affect the original class checks. The
native query copies Motion into the actual window player's next simulation
snapshot; a swapped LAN guest, server copy or speculative replay cannot obtain
the hook policy. The actual jump modifier additionally requires client_side.

`block-bounce-patch.py` composes after movement and mining transforms. It copies
the same behavior source into Horizon's client crate, adds the configuration
snapshot event and jump boundary, and never edits pristine upstream files.
The engine config child supplies the complete File and Interop setting trees,
parsing, finite-value round trips, range metadata and invalid-input tests.

## Verification

```text
python -X utf8 native/liquidbounce-horizon/test-block-bounce-patch.py --out build/block-bounce-04
```

Eight tests and Clippy with `-D warnings` pass. The fixture compiles the entire
transformed `living_motion` module, including its ordinary-world test, the
actual `MotionWorld::modify_jump_power` and identity bodies, the same complete
behavior source, and the native hook. The real `horizon_mod_api_gen` creates
the Host, event and dispatcher, whose copy-out and identity checks are tested.
Source guards also verify the production snapshot call order and initialization.

`oracles/BlockBounceFacts.java` executes against the actual cached MC 26.3 JAR.
All 1,286 registered blocks agree with the SlimeBlock/BedBlock/HoneyBlock class
membership check, including the 18 positive IDs. Six actual AABB boundary cases
check complete visited coordinates, negative positions, inclusive edges,
Double values on both sides of 0.01 and the world boundary. Ninety Java Float
cases compare the original jump formula and final Motion addition bit for bit.
The arithmetic oracle is source-derived Java, not execution of the Kotlin
module. The registry classes and AABB construction are actual MC code.

Evidence is preserved in `build/block-bounce-04/report.json`, the command logs
and `mc-oracle.json`. Earlier attempts retain a fixture range-syntax correction,
bool-assert lint correction and pre-format success. Production behavior did
not change during those fixture corrections.

The explicit isolated-test boundaries are the Game/window/player shell,
world block lookup and asset registry shell. Shared engine module registration,
lifecycle, config integration and full game execution belong to the parent
integration; this report does not claim those checks have completed.

## Language support

Motion and both VehicleBoost speed settings have owned English, simplified
Chinese and traditional Chinese help. Display strings follow the loaded game
language, retaining stable English setting identifiers. The total owned help
coverage is now 221 entries per Chinese display dictionary. The theme checks,
11 Interop unit tests and two live-backend integration tests pass, and the
caption audit has zero missing simplified or traditional display captions
(recorded simplified fallbacks remain explicit for traditional Chinese).
