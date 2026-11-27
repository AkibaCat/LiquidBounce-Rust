# Shared placement kernel — MC 26.3

Pinned LiquidBounce: `ae0eb396c07f22383789b28d5912d714c6ca0a86`.
Pinned Horizon: `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`.

Integration owner applies the single generator transaction serially:

```powershell
python native/liquidbounce-horizon/integrate-block-placement.py --apply
```

The extension is independent of placement-use-patch.py. It must precede consumers
(Extinguish and Scaffold) and the final step-patch.py constructor scan. No module is
registered by this shared primitive. It does not execute interactions.

Native API in `game::block_placement`:

```rust
let location = block_placement::capture_location(game, predicted_position, server_rotation)?;
let mut options = block_placement::Options::center(&stack, location);
let target = block_placement::find(game, target_pos, &mut options)?;
let planned_hit = block_placement::planned_hit(&target);
let hit = block_placement::verify(game, &target, rotation, range,
    block_placement::ClickTrace::World, block_placement::FailedClick::Nothing)?;
```

Options also provide Position/Line priority, original offset sets through
`offsets(1|2|4)`, custom face sampling and backface policy. Center defaults to no
offset, unrestricted planning range and rejection of faces pointing away.

The core preserves stable descending offset ranking; vanilla Direction order;
neighbor before target; per-box normal-axis and Y ranking; strict best-facing
replacement; predicted-eye inside testing; and original hit-height epsilon.
Native queries use actual player outlines and the server-selected main-hand
stack, dynamic shulker outlines, random outline offsets, true CENTER support,
BlockPlaceContext replacement and ScaffoldingBlockItem context movement. The
planning resolver intentionally stops before state-for-placement, survival and
entity obstruction; those belong to actual interaction execution.

World verification clips every intervening block and preserves GhostHand's
selected-block pass. Target-shape verification uses the camera eye and the actual
player's contextual shape, including vanilla interaction-shape overrides.

Completed checks:

```powershell
python native/liquidbounce-horizon/test-block-placement.py --static-only
```

Result: four native files composed in memory; one shared edit in one generator;
Python AST and rustfmt parsing passed; no shared or staging files written.

Prepared for the integration owner's next combined batch, not run by this owner:

```powershell
python native/liquidbounce-horizon/test-block-placement.py --model-only
```

Nine focused cases cover face ranking versus nearest sample, stable offset
priority, solid versus partial targets, multi-box minimum Y, half-open inside
tests, range/backface boundaries, hit-height tolerance, original offset order and
line priority. Official native compilation and Kotlin differential execution
have not been claimed.
