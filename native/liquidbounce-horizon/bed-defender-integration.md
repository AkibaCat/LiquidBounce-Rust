# BedDefender integration draft

Depends on the integrated LiquidFiller / non-support BlockPlacer and BedPlates services. Shared integration remains unapplied by this owner. No Cargo, rustc, full staging or Rust test execution was performed.

Run from the repository root:

```text
python native/liquidbounce-horizon/integrate-bed-defender.py --check
python native/liquidbounce-horizon/integrate-bed-defender.py
python native/liquidbounce-horizon/test-bed-defender.py
python native/liquidbounce-horizon/test-bed-defender-focused.py --out native/liquidbounce-horizon/packet-tests/bed-defender --run
```

The focused script prepares without a compiler unless --run is passed. It runs four actual production model tests; two production config tests remain available in the unified engine test batch (filter bed_defender_). Both test groups are written but unexecuted here.

The idempotent integrator updates nine shared source files, including requester routing in the single global PostRotationExecutor. Placement/crystal providers are 34/35, processors 512/528, requester 0x4245445f44454645. Existing native BlockPlacer target, obstruction, interaction, swing, damage and crystal services are reused; each module keeps its own queue, cooldown, rotation settings, crystal state and two placement renderers.

Implemented behavior follows ModuleBedDefender.kt: strongest hardness first (unbreakable before finite hardness), then more items, then effective selected slot/offhand/proximity; full default collision blocks and optional chest items; early canAct, RequiresSneak and slot-availability gates; exact section scan order; both bed halves including missing-opposite states; four SelfBed choices from production BedPlates; closest matching bed with stable ties; independent outward/upward layer BFS; inclusive AABB range; stable ascending layer / descending center distance; insertion-preserving placement queues. Place and SelfBed settings use the existing production serializers/parsers, preserving all inactive modes and shared renderer/rotation metadata.

Manual key tracking supplies the exact scanner order rather than a coordinate sort. SpawnLocation observes original raw PlayerPosition values even when the packet is cancelled, before accepted vanilla handling. Manual notifications are translated by the dedicated locale updater. Actual locale publication belongs to the localization owner.

Validation completed: Python syntax parsing; nine-seam integration dry run; complete current native transform stack in memory with hook/helper and pre-cancellation PlayerPosition event assertions. No generated native patch or staged checkout was written.

Known shared boundaries: the crystal object identity, section re-insertion ordering, and network-to-actor toggle timing limitations described in liquid-filler-integration.md also apply here. SpawnLocation gating likewise occurs when its queued original-packet observation reaches the actor. Native gameplay, keyboard, packets and GPU output still need the parent batch's compile/game verification.
