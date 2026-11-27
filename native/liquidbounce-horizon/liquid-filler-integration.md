# LiquidFiller / non-support BlockPlacer integration

Frozen integration draft; shared edits have not been applied. No Cargo, rustc, staging, native patch output, or Rust smoke execution was performed by this owner.

## Integrate

Run from the repository root:

```text
python native/liquidbounce-horizon/integrate-liquid-filler.py --check
python native/liquidbounce-horizon/integrate-liquid-filler.py
python native/liquidbounce-horizon/test-liquid-filler.py
```

The integrator edits nine shared files and is idempotent. Its native patch is appended after Scaffold special techniques, following NoSlow and KillAura. It adds the processed rotation request to RotationManager, one global PostRotationExecutor, placement/crystal providers 32/33 and processors 480/496, config/registry/runtime/render hooks, and native host/packet seams. Requester is 0x4c49515549444649.

The native patch fixture applies the current full native source transform stack in memory only. RemoveEntities is observed on the original receive path before cancellation and queued to the game actor before the accepted packet. The callback ignores removal while LiquidFiller or DestroyCrystals is stopped.

## Owned files

- behaviors: liquid_filler.rs, block_placer.rs
- native: liquid-filler-native.rs, block-placer-native.rs, block-placer-crystal-native.rs, block-placer-host.rs, block-placer-patch.py
- engine: liquid_filler_runtime.rs, block_placer_runtime.rs, config/liquid_filler.rs, config/liquid_filler_schema.json
- tooling: integrate-liquid-filler.py, test-liquid-filler.py, liquid-filler-smoke.rs, update-liquid-filler-locales.py

Actual locale publication is assigned to the localization owner. The locale updater provides 210 descriptions and 221 scoped labels, including inactive modes and renderer/rotation settings.

## Evidence and verification

Checks completed before handoff: integration dry run; Python AST parsing; full native transform fixture in memory; temporary-copy locale updates repeated for idempotence. Rust tests are written but unexecuted: liquid-filler-smoke.rs and config tests matching liquid_filler_config. Parent should include these in its unified build/test batch.

Behavior covers source-only fluids, exact scan/order semantics, item filter/default registry ordering, source sponge BFS depth/count/directions, insertion-preserving placement queue, cooldown/sneak/reset timing, all Normal/None executor modes, actual native target/click/interaction paths, both placement renderers, and crystal attack delay/rotation/vanilla client damage. The source quirk that a planned air sponge target uses the normal filler is retained.

## Concrete parity boundaries still open

1. Crystal target retention currently stores entity id and distance rather than the original EndCrystal object. If the server removes the crystal while DestroyCrystals is disabled, or the world changes while the parent remains enabled, later operations query the current live entity by id. A missing old object stalls and a reused id can address a new object. Source retains the old object. This needs an incarnation token plus retained position/bounds snapshot for complete edge parity.
2. Crystal collision enumeration sorts native live crystals by Minecraft packed section order and original vector insertion order. A crystal moving between entity sections is not appended to a new per-section insertion order. Ordinary stationary crystals match; crossing-section tie ordering needs entity movement tracking.
3. Removal event is marshalled from the original network PacketEvent phase to the game actor before accepted handling, matching the existing AntiBot observation pattern. Its module-running gate is evaluated on actor delivery; a concurrent toggle between receive and delivery is a narrow timing difference from a direct listener.

These are specific unresolved edges; this draft should not be advertised as complete source parity until they are addressed.
