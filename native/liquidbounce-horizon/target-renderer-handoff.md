# Shared TargetRenderer handoff

Pinned LiquidBounce: `ae0eb396c07f22383789b28d5912d714c6ca0a86`. Eight modes, five HeightMode choices, AnimatedValueGroup rotation and TextureMode are implemented in owned files. Initial owner is AutoBow; no standalone module registry count increase.

## Root integration

1. Apply AutoBow ownership wiring first. Its native patch supplies armor stand tick count and exposes the dragon's existing ticks.
2. Run `python native/liquidbounce-horizon/integrate-target-renderer.py --apply`. The default invocation validates five transactional changes across five shared files.
3. Run `python native/liquidbounce-horizon/update-target-renderer-locales.py --directory native/liquidbounce-interop/resources/localization`. Default owner prefix is `liquidbounce.module.autoBow.bowAimbot.targetRendering`; `--prefix` supports other owners.
4. Regenerate the behavior patch and run the official native + engine check. The native patch adds eleven files/edits; its dispatch method is deliberately in `game/native_hooks.rs`, where the API generator scans.

The native GPU uses actual scene Depth32Float attachment: Ghost and covered Hearts test/write reversed-Z depth; other modes use Always with depth writes disabled. Original Marker1, Marker2 and Glow PNG bytes are embedded. Custom PNG/JPEG decode is asynchronous, cancels stale result ownership on a valid path change, preserves the last successful image on failure/nonfile, and retries after the path changes away and back.

## Owner API

Behavior exports `TargetRendererSettings`, `TargetRendererRuntime`, `TargetRendererSetting(pub String,pub Value)`, `Entity`, `Camera`, `Frame`. Setting `apply` returns unit; runtime `render(settings,entity,camera,wall_epoch_ms,client_elapsed_ms)` returns Frame. Config exports `serialize_target_renderer(settings,format,describe,key_prefix)` and `parse_target_renderer(value,path) -> (changes,issues)`.

Engine runtime has the unique anchor `        // TARGET_RENDERER_OWNERS_END`. Another owner's transactional integrator inserts, for example, `        owner!("AutoRod", auto_rod, true);` immediately before it. Owner fields are `behavior.target: Option<i32>`, `target_renderer`, `target_render_runtime`. Targets are deduplicated and geometry is combined. Runtime collector ownership is handed to root for these additional owner registrations.

## Focused evidence

- `build/target-renderer-smoke-02/report.json`: five checks passed. Full File/Interop roundtrip and inactive settings; partial parse errors; all modes/heights/curve/damage state; actual engine owner runtime; original Inter glyph rasterization using native Region atlas placement; GPU compilation/draw/readback of original images, annulus, partial heart and real depth.
- `build/target-renderer-custom-smoke-01/report.json`: one additional targeted check passed for custom PNG async decode, failure retention, nonfile retention and changed-path retry. Five prior checks filtered out.
- Both compose native integration in memory only and confirm all 109 metadata description keys in both Simplified/Traditional Chinese. No frozen renderer matrix repeated.
- `build/target-renderer-smoke-01` preserves the initial wgpu 30 optional depth field diagnostic; only those typed fields were repaired before the passing run.

Native Game/overlay full compile and real-game visual parity remain official/root work. Pure-owner declarations are fixtures. The Text2D bridge uses original theme Inter Medium/Bold 43px TTF bytes and the native atlas, with Windows CJK fallback. ab_glyph hinting/metrics can differ from the original AWT/FreeType rasterizer; switching the active custom theme font is not yet supported. Do not label the font smoke or GPU fixtures as actual-game pixel parity.
