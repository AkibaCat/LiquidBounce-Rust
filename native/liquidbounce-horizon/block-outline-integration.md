# BlockOutline owned handoff — 2026-11-26

Pinned LiquidBounce ae0eb396c07f22383789b28d5912d714c6ca0a86 / Horizon
9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0. Shared roots, dictionaries,
production staging, AutoBreak/ItemScroller and upstream are untouched.

Run `integrate-block-outline.py` once under root ownership after its optional
`--check`. All source replacements accumulate in memory and validate before
writing; concurrent changes are checked before the first write. It adds the
Render category module, exact BlockOverlay alias, canonical config/runtime,
world-change reset, and patch registration. Source has no disable override;
animation positions intentionally survive module disable.

The module retains SideOnly=true, Color=0x464677ff, Outline=0x964677ff and
Slide enabled/150ms/Linear defaults and the complete easing choices. Native Game
supplies the actual block hit, world-border bounds, camera-context outline boxes
and vanilla exposed edges. A single-box outline transitions between exact world
AABBs; complex shapes reset animation. The ShapeSurfaceMesh/PlaneMask translation
merges actual exposed faces, flood-fills the selected hit face with 1e-7 epsilon,
and emits only that connected face's perimeter. It does not outline every input
box separately. Both faces and lines render through walls, as the source does.

The world handler renders independently of vanilla F1, game-mode and build
restrictions. Its active query cancels the vanilla outline while enabled, even
when the current target produces no custom geometry. No fake ray/hit is used.

`block-outline-patch.py` adds the source-colored `LiquidBounceQuad` world primitive
and its actual native tessellator, reusable by later source geometry. Apply this
patch before any later patch that uses that variant. The only ClientHooks forward
is `ClientBlockOutlineEvent` -> `self.block_outline(host,event)`.

`build/block-outline-smoke-01/report.json` passed one Cargo compile / four tests:
the actual owned behavior/surface/config/engine runtime plus actual shared Tree
and parser bodies, with declaration-only registry/Host providers. The actual
native patch composed against the current staged source in memory. Copied zh_cn
and zh_tw dictionaries were updated in the same check. This fixture does not
compile Game or constitute visual proof; root aggregate validation owns both.
One fixture-only unused re-export warning remains.

Root applies `update-block-outline-locales.py --directory
native/liquidbounce-interop/resources/localization` once for the full Chinese
labels and option descriptions, retaining the pinned original module help.
