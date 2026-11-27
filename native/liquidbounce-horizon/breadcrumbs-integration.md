# Breadcrumbs — complete owned handoff

Pinned LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`. Category Render; alias PlayerTrails. AutoBreak and ItemScroller unchanged.

Root integration:

1. `python native/liquidbounce-horizon/integrate-breadcrumbs.py --check`
2. `python native/liquidbounce-horizon/integrate-breadcrumbs.py`
3. `python native/liquidbounce-horizon/update-breadcrumbs-locales.py --directory native/liquidbounce-interop/resources/localization`
4. Root's ordinary native patch generation/aggregate check. The extension runs after Chams and BlockOutline. No staging mutation was performed by the owner.

`integrate-breadcrumbs.py` accumulates edits to seven shared files, validates all anchors and checks original contents before writing any file. It adds the registry/default/alias, behavior module, config enum and actor forwarding, native event forwarding, disable/world-change clear, and patch-list entry.

## Source behavior

All original settings are implemented: OnlyOwn, ARGB Color, Rainbow, Height, Temporary.Enabled/Alive/Fade and Render.Plain/Shader. Shader contains all nine styles, their independent inactive settings, original declaration order and source defaults (all Fill=0.15). The config roundtrip yields 76 changes, preserves imported values outside UI ranges, and retains case-sensitive mode selection.

Trail identity uses `Mob.join.get()` rather than network IDs or relocatable pointers. Native players use separate lifetime serials keyed by native role and entity ID, advancing on tick-counter reset or dead-to-live replacement. Snapshot interpolation uses the actual current/previous positions. A removed entity retains its last snapshot and becomes not alive. `lastPositions` survives trail removal until disable/world change, matching source. OnlyOwn deliberately keeps other dead entities' trails and removes other alive entities' trails. Expiration uses strict `<`; fade is point-specific; the last point is replaced with live interpolation every frame. Height zero draws gradient lines, otherwise gradient quads. Shader silhouette alpha starts opaque before age fade. Rainbow uses the original Float HSB conversion and ten-second period with a monotonic process origin.

## Native renderer

Owned Game snapshots route tick and world-frame inputs through `ClientBreadcrumbsEvent<'_>`. Renderer drawing is independent of F1 and camera perspective, has no depth test, and runs after world entities and before the first-person hand. Pending Breadcrumbs postpones Horizon's optimized in-world hand pass. The actual world-post target size is used.

Plain retains Float endpoint color and alpha; one-pixel lines are clipped in homogeneous space and expanded into triangles. Shader mode captures RGBA8 silhouettes, then runs the source field_rows, field_columns, and selected style GLSL unchanged except explicit bindings/version/includes. Original RGB565 packing, 8-bit distance quantization, tie order, uniforms, all noise functions, and all nine style equations are retained. A separate RGBA8 style target precedes straight-alpha scene blending. Internal GL row orientation is retained, with one flip at final composite. Aura receives actual camera axes, lens, camera block/fraction and shader game time. Final colors decode only for Horizon's linear float frame. GLSL is compiled through Horizon's existing vendored Naga; no new runtime dependency is added.

## Evidence

`build/breadcrumbs-smoke-03/report.json`: passed. One focused Cargo invocation compiled actual owned behavior, config, runtime, WGPU renderer and all 11 source fragment shaders. Four tests passed: lifecycle/fade/OnlyOwn cleanup, runtime geometry/uniform dispatch, complete inactive-style File+Interop roundtrip, and actual GPU Plain+9 styles draw/readback with nonzero output. Adapter: NVIDIA GeForce GTX 1060 3GB, Vulkan. The fixture uses actual shared Tree/parser bodies, declaration-only Host/registry providers, WGPU 30.0.1 and Horizon vendored Naga/WGPU-HAL. Actual native patch anchors composed in memory; copied zh_cn/zh_tw dictionaries updated. Initial failed evidence is preserved in smoke-01/02 (temporary Rust borrow and WGSL entry-point call, both corrected).

The native Game module and aggregate generated event signatures still require root's aggregate compile. GPU pixels prove isolated renderer usability, not final in-game appearance or complete source visual parity.
