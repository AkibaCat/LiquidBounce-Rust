# LiquidBounce render behavior for Rust integration

Source-derived pure logic from LiquidBounce commit
`ae0eb396c07f22383789b28d5912d714c6ca0a86` (GPL-3.0-or-later,
Copyright 2015-2026 CCBlueX). This crate does not register game modules. It has no
game state, graphics renderer, network connection or placeholder module list.

## Implemented behavior

- **Zoom**: original defaults/range metadata, all seven Easing variants, integer
  transition anchors, both lifecycle callbacks, all original clock-read points,
  wheel-step multiplication/rounding/clamping/cancellation, signed elapsed-time
  progress, and fractional projection correction.
- **FullBright**: Gamma option capture and additive f32 tick progression, exact
  cap behavior, mode enable/disable selection, and NightVision effect intents
  with the actual Minecraft constructor defaults. Gamma does not decrease when
  Brightness is lowered. NightVision removal does not restore an earlier effect.
- **NoFOV**: Constant/Custom integer FOV and multiplier functions with original
  defaults/range metadata and floating-point range behavior.
- **Chronometer/Easing**: signed wall-clock/JVM integer overflow, strict and
  inclusive elapsed boundaries, and original expression widths/order.

RangedValue's bounds are metadata in the original source; its setters do not
clamp them. Settings therefore accept raw values. Only source-prescribed
clamps are applied, including invalid-duration guards and the scroll target
range `1..179`. Changing DurationFactor affects the next Zoom reset, while
changing Transition affects the current transition immediately.

## Runtime integration contract

Use a real wall-clock closure (`FnMut() -> i64`) or an implementation of
`MillisClock`; the bridge supplies actual player/world/listener gates. Zoom
deliberately does not own enabled/running state. Invoke `on_enabled` at the
original callback point before enabled becomes true and `on_disabled` before
it becomes false. Feed the observed default integer FOV after applying NoFOV
when that module is running. The caller must not replace those callbacks with
resets to a synthetic default.

The camera base-FOV hook follows `MixinCamera.hookGetFov`: while Zoom is
running, use `get_fov(true, 0, clock)`; otherwise use
`get_fov(false, original, clock)`, then apply NoFOV only when it is running and
the returned value equals `original`. At the later `calculateFov` return,
apply `apply_fractional_fov` to the vanilla-calculated result. Panoramic mode
returns its original FOV without this correction. These two points and their
ordering need actual Horizon hooks. Applying only a visual camera pose does
not implement the original projection behavior.

`on_hotbar_scroll` takes the already-quantized integer hotbar wheel event;
feed its cancellation back to the real input dispatcher. Mouse slowdown needs
the original `MixinMouseHandler.turnPlayer` first-person/scoping conditions
(`original || Zoom.running`) mapped into Horizon's actual input pipeline.

FullBright's tick method takes the real parent running gate and yields an
optional NightVision intent. Execute it on the actual local player's effect
system at PlayerPostTick. `on_disabled` yields removal even when the mode is
not running, matching the parent child-state callback. `switch_mode` emits
old-mode removal synchronously before changing selected mode. Use
`brightness_override` at the real lightmap extraction point; `None` lets the
bridge retain vanilla or the later XRay fallback.

## Explicit remaining boundaries

This crate contains the pure behavior, not Fabric/Java injection points or a
complete gameplay port. Native game hooks, the general Value/Mode event bus,
config persistence, coroutine scopes, GUI exposure and all other modules are
outside this crate. Arbitrary external config listeners that change parent
running state midway through mode switching need the original ValueGroup
lifecycle in the integrating runtime; `switch_mode` receives one supplied
parent-state observation.

The convenience `on_disabled(default_fov, clock)` accepts one externally
sampled FOV value. To preserve the exact original read point after the first
clock reset, use `on_disabled_with_default_fov(provider, clock)`; it invokes
the provider between the two original resets.

Kotlin calls Java Math.pow with double operands for exponentials. This port
calls Rust's double powf with the same operands and casts at the same point;
the formulas are not approximated. All included Java-generated f32 vectors
agree bit-for-bit. Universal last-bit parity between every platform math
library and Java Math.pow has not been established.

NoFOV's invalid/empty floating range returns `FloatRangeError` instead of
throwing Kotlin's IllegalArgumentException. Valid-range behavior is preserved.
The caller handles this error where the original game would handle the
exception. Native Math libraries may choose different NaN payload bits; NaN
propagation and comparisons are retained.

Public settings' Rust PartialEq is not the original Value event comparison:
Kotlin considers all empty floating ranges equal, whereas the public limit
tuple does not. Do not substitute this structural comparison for Kotlin's
range equality when implementing the general config bus.

## Source and verification

Reference files are under `../../upstream/LiquidBounce/src/main/`:

- `kotlin/net/ccbluex/liquidbounce/features/module/modules/render/ModuleZoom.kt`
- `kotlin/net/ccbluex/liquidbounce/features/module/modules/render/ModuleFullBright.kt`
- `kotlin/net/ccbluex/liquidbounce/features/module/modules/render/ModuleNoFov.kt`
- `kotlin/net/ccbluex/liquidbounce/utils/math/Easing.kt`
- `kotlin/net/ccbluex/liquidbounce/utils/client/Chronometer.kt`
- `kotlin/net/ccbluex/liquidbounce/config/types/group/ModeValueGroup.kt`
- `java/net/ccbluex/liquidbounce/injection/mixins/minecraft/render/MixinCamera.java`
- `java/net/ccbluex/liquidbounce/injection/mixins/minecraft/render/MixinLightmapRenderStateExtractor.java`
- `java/net/ccbluex/liquidbounce/injection/mixins/minecraft/client/MixinMouseHandler.java`

Minecraft 26.3 `Mth.lerpInt`, `Mth.floor`, `Mth.lerp`, and the
`MobEffectInstance` constructor were independently verified with Java 25
`javap -c` against the locally cached deobfuscated 26.3 Minecraft JAR. The game
JAR is not copied into this crate or distributed.

Tests use fixed independently generated Java vectors and behavioral boundary
expectations, not expected values computed by the tested implementations.
Run `cargo test --offline` and
`cargo clippy --offline --all-targets -- -D warnings` in this directory.
