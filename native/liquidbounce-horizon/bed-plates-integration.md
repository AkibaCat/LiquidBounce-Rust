# Complete BedPlates handoff

Status: owned implementation and read-only static validation complete. No
Cargo/Rust/Kotlin compiler, Clippy, game/GPU, smoke/oracle or full test suite was
run by this owner. Shared source, upstream, official locales, generated patches
and staging were not modified. Root owns all integration and compilation.

## Root integration

ItemTags and its common ItemStackListRenderer are already installed. Apply each
transaction once, then update the two official Chinese display dictionaries:

```powershell
python native/liquidbounce-horizon/integrate-bed-plates.py
python native/liquidbounce-horizon/integrate-bed-plates-choice-labels.py
python native/liquidbounce-horizon/update-bed-plates-locales.py
```

The first transaction validates and wires seven shared engine/generator files;
the second validates and wires three shared theme/localization sources. Both
support read-only `--check` and reject concurrent source changes before writing.
The native extension belongs immediately after `item-tags-patch.py` in the
normal native generator. Regenerate the existing Chinese theme patch through
`scripts/make-theme-localization-patch.py` before the normal theme build. The
theme baseline remains untouched.

The single new hook is `client_bed_plates`. Its one actual declaration is
`pub(super) fn native_client_bed_plates(&self,input:&str,output:&mut String)` in
`game/native_hooks.rs`, and its emitted target names that same module/function.

## Implemented source behavior

All settings, every inactive choice, and original defaults are present:
Rect/Texture with all panel values, Outline, MaxLayers, ShowBed, TextShadow,
RenderOffset, Scale curve, MaxCount, HighlightUnbreakable, Compact,
PreventOverlap, Predefined/Custom filter (Blocks/Filter), and all four
IgnoreSelfBed modes with Slots/Loose, BedDistance, Track/Untrack, plus
IgnoreAdjacent. Interop carries the original curve axes/tension, registry type,
VECTOR3_D locate-button flag, and MULTI_CHOOSE `canBeNone=false` metadata.

The native tracker reads real loaded sections, uses palette `maybe_has`, scans
only actual BedBlock HEAD states, and retains weak section snapshots rather
than stale strong chunk references. Section changes/unloads invalidate nearby
bed defenses; changing MaxLayers rescans surroundings; disable/world change
clear the appropriate state. A dependency-free production scan module is reused
by the behavior model and generated native source. It performs the two separate
outward/up/left/right BFS walks in the original direction order. Counts are by
block identity and layer. Full-mode order is layer/count/unbreakable-first
hardness/UTF-16 descriptionId. Water/lava use bucket icons. The predefined
filter reads the actual default state's CONDUCTOR flag and original ladder,
water, glass and stained full-glass whitelist; panes are not added.

Color identification reads resolved DYED_COLOR components from selected native
armor slots, including component defaults/removals. Exact matching uses opaque
texture-diffuse RGB; loose matching uses the original sixteen-dye order.
SpawnLocation receives raw PlayerPosition packet coordinates before packet
dispatch and preserves the source's strict old-player distance `<16` test and
head-integer bed radius. It does not apply relative flags. Mode changes reset
the exited mode's runtime state. Manual tracks the nearest real bed half in
the eye-centered inclusive cuboid, accepts only pressed keyboard events, works
through both mutually exclusive real-window and direct-control key entries,
and reproduces zero-position, no-bed, untrack and world-change notifications.

Tick distances use the original render center above the two halves. Native
projection rejects points behind the camera but does not cull screen x/y.
MaxCount is consumed only after successful projection/submission, including
empty source lists. Adjacent-bed filtering inspects all tracked beds. The
original compact layer-zero behavior is retained. Upstream compact block order
is reference-hash/process-specific; this port retains first input order rather
than claiming bitwise agreement for that arbitrary order.

Native rendering submits the original custom renderer to the shared
ItemStackListRenderer queue after ItemTags and before the single common flush.
It uses real resolved GUI item models, distance labels with vanilla shadow,
layer Roman numerals, counts including one, configured TextShadow, and red
tool warnings computed from all nine actual hotbar items. It does not add item
durability/cooldown decorations. Bed outlines use DyeColor's map color rather
than the bed state's wood map color. Rect/Texture, scaling, model scissors and
overlap avoidance reuse the already validated common service, so BedPlates and
ItemTags share one rearrangement set.

## Chinese display

Both display dictionaries receive the module name, settings/choices, all 36
descriptions and five notification strings (41 entries per locale). Notification
title/message use the game's current Locale; coordinate placeholders survive.
The updater is byte-idempotent. Raw module/setting/choice/key identifiers remain
unchanged.

The existing global `Chest` caption means a storage container. A scoped choice
caption in `labels[setting.key + "." + choice]` lets this armor choice display
`胸部` without renaming container captions. The second transaction adds this
generic fallback-aware helper to the two actual theme integration sources and
routes MultiChooseSetting's display expression through it. Value arrays,
`includes(choice)` and `handleChange(choice)` remain the original raw strings.

## Validation ready for root

Passed final owner command:

```powershell
python native/liquidbounce-horizon/test-bed-plates.py --native
```

It checked seven engine plus three scoped-caption transaction anchors, parsed
13 complete Rust sources with rustfmt in a temporary directory, composed the
actual full native generator in memory, checked the real hook declaration and
emitted target, checked both keyboard entries/raw position ordering, validated
coexistence with ItemTags/common queue, composed the actual theme component in
memory, and verified temporary Simplified/Traditional dictionaries, placeholders,
container-caption preservation and updater idempotence. No shared output was
published.

Six focused tests are prepared at `build/bed-plates-focused-01`. Root can run:

```powershell
python native/liquidbounce-horizon/test-bed-plates-focused.py --out build/bed-plates-focused-01 --run
node native/liquidbounce-horizon/test-bed-plates-choice-labels.mjs
```

Four tests are inside the exact production behavior/scan model; two are inside
the exact production config module using the extracted actual shared Tree,
common, scalar, bool, number and config parsers. Only module registration and
dispatch are narrowed. The real dependency enum/ModifierKey implementations are
extracted verbatim with the actual key tables. This fixture does not simulate
or claim execution of native Game, actor or GPU rendering. `--filter` permits a
narrow retry. The Node test is prepared for execution after root integration
and official locale update; it has not been run by this owner.

Root still needs its normal native/mod-api generation, combined official checks
and native execution validation. `bed-plates-evidence.json` records owned and
pinned-input hashes with the exact owner validation scope.
