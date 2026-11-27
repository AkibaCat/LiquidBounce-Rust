# ItemTags handoff

Owned implementation matches the pinned LiquidBounce `ModuleItemTags.kt`
settings and rendering flow at `ae0eb396`. Shared files and formal locales have
not been changed by this subtask. Root applies the transaction once:

```powershell
python native/liquidbounce-horizon/integrate-item-tags.py
python native/liquidbounce-horizon/update-item-tags-locales.py
```

The transaction validates all anchors before changing seven shared source files:
behavior registration, engine declaration/state/callback cleanup, config
publication/parsing/application, runtime hook/world cleanup, and native generator
registration. The native extension must run after `chest-stealer-patch.py`, since
it connects that producer to the shared list renderer.

## Behavior and configuration

`native/liquidbounce-behaviors/src/item_tags.rs` implements Filter/Items,
distance Scale/ClusterEntities curves (the real common curve sampler),
RenderOffset, RowLength, PreventOverlap, all three MergeMode choices and
Shulker/MergeStacks/ShowTitle. Every background Rect setting and the inactive
Texture choice survive File/Interop configuration roundtrips. Registry lists
use actual registry ids and unknown ids resolve to air. Boolean/object list
members are rejected as non-registry primitives. Interop includes CURVE axes,
tension, VECTOR3_D `useLocateButton=false`, registry metadata and descriptions.

Entities retain actual join identity and insertion order. Clustering is seed
based, strict `< radius²`, non-transitive and uses a per-seed camera distance.
Scale `< 0.01` skips. ByComponents keys include the canonical item/component
patch and copy even singletons. ByItem discards components only for multiple
same-item stacks. None and ByItem singletons can read the still-identical live
native stack between ticks; replaced/removed objects use the tick snapshot.
Counts wrap as signed JVM Int sums, then sort by descending count and UTF-16
descriptionId. Hash-map tie order for equal counts/descriptionIds is
process-specific in upstream; this implementation keeps stable input order.

## Native item-stack list service

`item-stack-list-game.rs` is a shared native ItemStackListRenderer. Producers
submit a `List` with optional slots (holes retained), title/title color,
row length, center, scale, background and optional custom per-item Renderer.
The custom renderer receives the live Game only at draw time and may capture
owned data, allowing BedPlates to provide its distance/count/tool overlays
without changing ItemTags or the common queue.

`rearrange=false` draws immediately. `true` joins a single stable planned queue.
One flush runs after current world-HUD producers, including ChestStealer, and
uses the production `item_tags_layout.rs` dimensions/padding/downward
GuiOverlapRearranger across all queued producers. The queue is cleared before
each HUD frame and on world change; flushing takes its contents before drawing,
so font/BetterInventory callbacks do not retain a queue borrow. New planned
producers must submit before `client_item_stack_lists_flush`.

ItemTags now only collects native stacks and submits main/container lists to
this service. ChestStealer SilentScreen already had a real native slot list;
the native patch connects it to the same service and removes its scanline
rounded-panel approximation. It preserves the source's default
`draw(event.context)` / `rearrange=false`; the original SilentScreen tag does
not participate in the planned overlap set. BedPlates itself is not implemented
by this handoff, but its common rendering interface is ready.

The service uses actual resolved GUI item models, glint and native item
decorations, with original local-coordinate seeds and scaled 3D item scissors.
Container previews read the resolved `minecraft:container` component (not only
shulker item ids), keep nonempty items, optionally use the same merge function
and retain custom component titles/current-language item names. Background
uses vanilla slot sprites or the original rounded-panel SDF, three glow layers,
outline, gradient with f32 FMA, margin and radius.

The HUD vertex byte layout/stride stays unchanged. Shape metadata is encoded
in high bits of its existing `text` word, exposed as Uint32 location 5. Packed
text classification uses `text == 1`, excluding rounded primitives. The shared
HUD shader handles these shapes before texture sampling, at the normal HUD
drawing position; it does not add another GPU overlay.

The single new mod-api hook is `client_item_tags`. Its one real
`pub(super) fn native_client_item_tags(&self,input:&str,output:&mut String)`
definition resides in `game/native_hooks.rs`, matching the emitted target.
There are no additional mod-api helpers. Native code finishes its engine query
before actual drawing, so shared font NameProtect callbacks can run normally.

## Chinese display and validation

The updater supplies Simplified and Traditional Chinese ItemTags names,
setting/choice labels and all 22 descriptions to the existing language-driven
display schema. Identifiers stay unchanged. Actual item names and component
translations use the game's current language. Both updater runs on temporary
locale copies are byte-idempotent.

Owner validation is limited to read-only full native composition, transaction
anchors, true hook definition/target uniqueness, HUD and shared queue wiring,
complete source syntax parsing with rustfmt and temporary Chinese locale
checks. No Cargo/Rust/Kotlin compiler, Clippy, game, GPU or full-suite test was
run by this subtask. `item-tags-evidence.json` records owned hashes and scope.

The root can run the prepared six focused tests after applying the transaction:

```powershell
python native/liquidbounce-horizon/test-item-tags-focused.py --out build/item-tags-focused-01 --run
```

Three tests are in the actual production behavior/layout module; two are in
the actual production config module using the real extracted shared
Tree/common/scalar/number/bool/leaf parsers. The sixth parses and validates the
complete composed HUD WGSL with Naga 30.0.1. Only registration/dispatch is
narrowed. No actor/native Game implementation is substituted or claimed to run.
The fixture is prepared without compilation at `build/item-tags-focused-01`.
`--filter` supports retries restricted to a failed test. Root still needs its
normal native generator/mod-api generation and official combined checks.

Pinned sources: `ModuleItemTags.kt`, `ItemStackListRenderer.kt`,
`ItemStackListRenderState.kt`, `ItemStackListLayout.kt`,
`GuiOverlapRearranger.kt`, `ItemStackListBackground.kt`, `GuiPanel.kt`,
`CurveValue.kt`, `CleanupPlan.kt::ItemAndComponents`, `ItemStackComparators.kt`,
`Color4b.kt`, `WorldToScreen.kt`, `WorldExtensions.kt`,
`FeatureSilentScreen.kt`, `RoundedRectGuiElementRenderState.java` and
`shaders/gui/rounded_rect.{vsh,fsh}` under `upstream/LiquidBounce`.
