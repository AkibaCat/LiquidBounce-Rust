# TerrainSpeed integration handoff

Pinned source is LiquidBounce `ae0eb396`: unchanged `ModuleTerrainSpeed.kt`, `FastClimb.kt`, `IceSpeed.kt`, `WaterSpeed.kt`, with the original movement helpers. The complete policy and all configuration groups are implemented. The native seams below are implemented and independently checked, but the remaining verification boundaries below still apply. Do not record full native parity or the 236-module objective as complete.

## Shared registration

- Module **TerrainSpeed**, category **Movement**, alias **FastClimb**, key `liquidbounce.module.terrainSpeed`.
- Export `liquidbounce_behaviors::terrain_speed`; store a `TerrainSpeed` settings value, without extra per-module lifecycle state.
- Register `engine/src/config/terrain_speed.rs`: `serialize_terrain_speed`, `parse_terrain_speed`, `TerrainSpeedSetting` and `SettingChange::TerrainSpeed`. Its fourteen settings preserve the original hierarchy, inactive Motion/Clip choice data and raw imported finite floats. UI ranges do not clamp imports.
- Append `terrain-speed-patch.py` after Strafe and mining identity transforms. Catalog hook `client_terrain_speed`; no helpers.
- No enable/disable/world reset or screen gate.

Hook event has `flags:u32`, `mode:u8`, seven `f32` values (`climb_motion`, `slipperiness`, `ice_motion`, `base_horizontal`, `base_vertical`, `sprint_horizontal`, `sprint_vertical`), `input:u8`, `running:bool`. Flags: `1` FastClimb enabled, `2` IceSpeed enabled, `4` IceSpeed Motion enabled, `8` WaterSpeed enabled, `16` AutoSwim, `32` SprintSpeed enabled. Mode `0` Motion, `1` Clip. The input is the **previous installed untransformed** directional flags, forward/back/left/right = 1/2/4/8, before this tick updates ClientInput. Runtime obtains normal module running and copies raw settings without a screen condition. Physical Forward/Sprint/Jump/Sneak keys are captured by the host independently of transformed input events.

Chinese localization peer confirmed all TerrainSpeed help paths have English, Simplified Chinese and Traditional Chinese entries, including `fastClimb.mode`; theme localization verification passed.

## Original behavior

FastClimb starts enabled. Motion defaults to `0.2872f` (UI `0.1..0.5`) and modifies every local PlayerMoveEvent, without a MoverType filter, when horizontalCollision and onClimbable are true. Clip checks onClimbable and physical Forward, saves the starting position and block position, checks up to eight blocks, and calls absSnapTo at **every** ladder/vine step. The first other block exits one block along the player's horizontal Direction and one block higher. Scaffolding is not LadderBlock or VineBlock.

IceSpeed starts enabled, Slipperiness `0.6f` (`0.3..1`); nested Motion starts disabled, factor `0.5f` (`0.2..1.5`). The Block.getFriction listener is global, not scoped to the querying body. ICE, BLUE_ICE, FROSTED_ICE and PACKED_ICE each return configured slipperiness and, when Motion is enabled, multiply the **local player's** current velocity by `(widened f32 factor,1.0,widened f32 factor)` on every query. Each multiplication is preserved individually, including IEEE rounding and signed zero.

WaterSpeed starts enabled, AutoSwim true, BaseSpeed horizontal/vertical `0.44f`, SprintSpeed enabled with horizontal/vertical `1f` (all UI `0.1..10`). MovementInput sets jump in water unless physical Sneak is held. PlayerTick reads the preceding input's moving/vector flags, current yaw and physical keys. Physical Sprint selects SprintSpeed when enabled; physical Jump takes vertical precedence over physical Sneak. Idle axes preserve the original helper's signed-zero and exceptional arithmetic. PlayerTick precedes the current tick's input read.

## Native seams

`terrain-speed-host.rs` captures settings once inside a window-thread TLS scope. It requires the native owner, excludes LAN guest actors, server copies and speculative replay. Integrated server simulation runs on another thread with no TLS snapshot and receives unchanged friction. The scope's Drop checks all pending callbacks/history were consumed.

`terrain-speed-motion.rs` binds the original Clip provider to actual MotionWorld storage. MotionResources gathers the full state-ID set whose BlockFacts classes contain `LadderBlock` or `VineBlock`; it does not replace this with the broader climbable tag. There are no other MotionResources struct literal initializers in the pinned source. MC26.3 blockPosition uses Math.floor followed by Java d2i; absSnapTo clamps X/Z to +/-30,000,000 and writes old-position history. Each snap records history to the TLS bridge and the Game host synchronizes `previous_position` before subsequent consumers.

Actual move_self runs Strafe then Terrain before original before-position capture/auto-jump accounting. Actual shulker_box_push also dispatches Terrain. The optimized piston path dispatches before limiting, even for zero pushes, and rereads the bounds/position after Clip. External move heads synchronize the player's current Game yaw before Direction lookup.

All eight raw friction query sites are transformed: local MotionWorld, dropped item settle, experience orb, armor stand, mob living travel, mob leash angular friction, boat ground-block averaging, boat leash angular friction. Non-player queries carry their actual entity ID. The local MotionWorld query owns its local velocity and applies directly. Other solvers record one multiplier per query and Game drains it around player-observing boundaries; no cumulative factor is computed.

`terrain-speed-order.rs` stores actual insertion order and ordered passenger lists. The production Game provider walks the actual `ClientLevel` root snapshot, tests removal and vehicle relationships at each turn, executes the selected original native solver once, then recursively walks that entity's current passenger list. New entities enter the root snapshot next frame. Frozen levels tick Player roots and roots carrying any depth of Player passengers; passenger recursion does not independently freeze them.

The old before/after classifications and precomputed flattened plans were removed. Per-entity friction multiplications drain before that entity's individual push, event callbacks and the next entity turn. Mob push uses the original `other.push(this)` direction; boat push is emitted at its own turn. Shulker events likewise execute at their actual insertion/passenger turn. Controlled mounts and boats tick at the vehicle's turn using the preceding rider input; `remote_ride` latches this tick's input without repeating vehicle physics. Own attached fireworks tick at their actual entity turn.

NoPush's original `MixinClientLevel.getPushableEntities` cancellation gates the living-entity push branch. AbstractBoat.tick uses the separate Level.getEntities call and keeps its own push even when this NoPush query is suppressed. The exact production push helper is executed by the fixture in both enabled/disabled cases, using the real impulse and boat predicates with explicit storage/team/climb providers.

`terrain-speed-passenger-patch.py` separates rider input from seating. `Entity.rideTick` performs ZERO -> full tick -> positionRider. Seating filters only the selected entity, preserves seat count/index calculations, and does not clear velocity again after the tick. Thus Water's velocity survives a cancelled passenger PlayerTick. Local old-position capture precedes its turn; block entities follow the entity walk, and sendChanges follows all entities and block entities.

When composed with Timer, `client_timer_player_tick_ready` runs at the actual local entity turn after the loaded-chunk guard. Dispatch order is `native_client_player_tick` (Timer cancellation) -> Terrain Water -> `native_client_player_tick_final(cancelled)`. Event cancellation never skips Water, matching original EventManager dispatch. The cancelled branch skips the player body/cooldowns/effects but keeps later entities and final sendChanges. Runtime helper methods become `&mut self` so Water updates the actual player before final listeners.

Required patch order: existing production transforms (including Strafe/mining identity) -> Timer -> TerrainSpeed. TerrainSpeed itself invokes order patch, then passenger patch. It remains outside the canonical catalog until root accepts the consumer validation.

## Remaining verification boundaries

The tests below do not constitute a complete original/native client differential. The 305-case oracle executes actual MC scheduler/commonTick/rideTick, with tick bodies and position storage explicitly supplied by recording shells. The Game ticker fixture executes the exact production provider/ticker and velocity reset, with physics/event-body shells. Native solvers are typechecked in a complete independent Horizon snapshot, not compared end to end against every original entity body. Non-friction ancillary visuals/projectiles remain one batch after ordered entities; they have not been asserted globally equivalent. Full native registry/actor execution and complete original/native client differential remain **not_run**.

## Verification and evidence

```powershell
python native/liquidbounce-horizon/collect-terrain-speed-facts.py
python native/liquidbounce-horizon/collect-terrain-speed-scheduler.py
python native/liquidbounce-horizon/test-terrain-speed.py
python native/liquidbounce-horizon/test-terrain-speed-native.py --out build/terrain-speed-native-06
```

Current policy/consumer checkpoint: **24/24 tests**, Clippy `--all-targets -- -D warnings`, and complete production composition syntax validation passed. Composition at this checkpoint contains **286 transformed Rust sources**. The independent complete Horizon library check also passed (actual fill_hooks, cargo check --offline -p horizon-client --lib; 220 seconds). One pre-existing unused camera-perspective helper warning remains. See `build/terrain-speed-native-06/report.json` for its actual outcome.

The original collector copies and byte-checks all four Kotlin files and DirectionalInput unchanged, and extracts the exact moving/getMovementDirectionOfInput/withStrafe/Vec3.copy helpers. It executes **7,040 original event vectors** (2,304 move, 64 input, 4,096 tick, 576 friction), comparing all output doubles by raw bits and the Clip provider's exact read/snap trace. It uses an Unsafe-created actual MC26.3 LocalPlayer subclass with controlled dependency providers, actual Vec3/Input/BlockPos/Blocks/Direction. Only Vec3 x/y/z ACC_FINAL is removed, matching the original accesswidener; method bytecode and the pinned jar remain unchanged.

The MC facts collector separately executes actual Entity.absSnapTo through a recording subclass (only setPos is a storage provider), checks **81 snap boundary vectors**, **15 Direction.fromYRot vectors**, and enumerates all registered LadderBlock/VineBlock instances. Results identify exactly minecraft:ladder and minecraft:vine. Snap tests include nonfinite values, signed zero and world clamps.

The native fixture compiles actual generated Host/dispatch and exact composed native movement-input, move_self, shulker head, optimized piston helper and MotionWorld friction method, with the owned adapter and TLS bridge. World, player storage, physics execution and outer tick scheduling remain explicit dependency shells. Configuration tests extract the real shared Tree/parser helpers and exercise all fourteen settings in File and Interop forms, raw out-of-range/signed-zero imports, inactive choices, malformed leaves with independent progress and nonfinite export rejection.

The scheduler collector also executes **305 actual ClientLevel cases** and **144 actual Entity.push vectors**, comparing trace/order/commonTick/rideTick seating history and individual velocity updates by raw bits. The production Game ticker fixture checks the cancelled passenger Water/final/seat/next-entity order.

`build/terrain-speed-tests/result.json` records implementation hashes and oracle SHA256 `7588bb361c5b09a2b99f89a3835b999a9a7fdf5335686d05e06e8a0d8207cf86`. The directory also holds original source execution metadata, minecraft-facts.json, composition coverage assertions for all eight query sites, syntax/test/Clippy logs and generated API sources. Full native registry/actor execution and full original/native client differential are **not_run**. No upstream or production staging files were edited by this work.
