# DamageParticles handoff — 2026-11-26

Pinned LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`.

Owned files are complete. No upstream, staging, shared registration or canonical locale files were modified. AutoBreak and ItemScroller remain frozen.

## Root integration

1. `python native/liquidbounce-horizon/integrate-damage-particles.py --check` (validated 7 files).
2. `python native/liquidbounce-horizon/integrate-damage-particles.py`.
3. `python native/liquidbounce-horizon/update-damage-particles-locales.py --directory native/liquidbounce-interop/resources/localization`.
4. Include `damage-particles-patch.py` in the next normal aggregate staging/build. The integration script inserts it before semantic-pointer after Breadcrumbs.

Integration validates all anchors and original contents before any shared write. The event hook must remain in native_hooks.rs for generated API scanning.

## Behavior and interface

Render module DamageParticles. Original declaration order, defaults and import behavior:

- TimeToLive 3, UI 0.5..5 s; Scale 2, UI 0.25..4.
- ScaleTransition QuadOut; Displacement x=0,y=1.5,z=0 (`VECTOR3_D`, locator enabled); DisplacementTransition QuadOut.
- TrackMode OnTick default / OnUpdate.
- Colors Damage and Death ARGB red, Heal and MaxHealth ARGB green.
- All 7 existing easing modes; settings imports retain raw finite values without UI clamping. File/Interop roundtrip covers all 10 settings.

OnTick uses real Mob and ArmorStand JoinSeq lifetime identities plus serials for dragon and LAN player replacement. It skips tickCount zero and the actual local client player, tracks only changes above 0.05, treats previous 0 as the map's absent sentinel, and removes entries for absent or dead entities. Expiration is strict `< earliest`, evaluated before ordinary client-tick listeners, including in OnUpdate mode. Disable and world change clear both collections.

OnUpdate has a distinct true-setter bridge: native Mob/ArmorStand/Dragon health setters emit captured old/new/max plus the bounding-box center, movement and actual capture time. Actual PlayerAttributeHealth/LivingHealth setter calls use a thread-local player context and a before-storage callback, covering damage, equipment continuations, explicit health changes, peaceful healing and hunger. The original client player's events are skipped by its thread context; server-side actors remain distinct as in the source mixin. Incoming metadata storage remains direct and produces no synthetic setter event. A shared mutex queue preserves individual transitions across worker threads; it is drained on client tick and overlay extraction. Native health storage behavior itself is retained.

Rendering uses the actual current NativeCamera matrices, camera subtraction in double before narrowing, finite clip coordinates and positive w, with no offscreen/z clipping. Actual HudBatch native font draws shadowed text centered by integer font width / 2, then scales and translates the emitted vertices. It is inserted at extractHotbarAndDecorations HEAD equivalent, after health rows and before contextual bars, so F1 and HUD layer ordering match this source event. Birth position is bounding-box center + known movement. Displacement uses double fused multiply-add, and easing progress is not clamped. Numeric formatter uses exact promoted f32 values, HALF_EVEN decimal rounding and removes trailing .0; its decimal separator defaults to the task's Chinese/English dot convention and is an explicit behavior field.

Native module files: behavior `damage_particles.rs`, engine config/runtime, game, capture, dragon helper, Host snippet, and transactional patch. There are no new Cargo dependencies. Both Chinese dictionaries have explicit native Simplified/Traditional translations.

## Focused evidence

Final `build/damage-particles-smoke-02/report.json`: passed; 4 tests.

Production behavior, config, runtime, and native capture compile. The fixture extracts actual shared Tree/parser bodies and patched actual living_health setter, with declaration-only Host/registry/native entity providers. It checks particle lifetime/format/animation, actual runtime tick/overlay responses, 10-setting File/Interop roundtrip, and proves direct data writes do not emit setter events while true setters do. The native extension composes 30 actual files in memory; Chinese updater runs on dictionary copies.

Historical smoke-01 reported 4 passing Rust checks, but review found that the first cfg(test) import caused some native setters to be skipped by its patch transformation. The corrected transformation is validated by explicit Mob/Dragon setter anchors in smoke-02. Historical evidence remains intact; smoke-02 is the only final acceptance evidence.

Actual complete game compilation and visual comparison are the root aggregate stage; this focused fixture makes neither claim.
