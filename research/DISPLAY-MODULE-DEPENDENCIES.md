# Display modules requiring shared foundations

These are source findings against LiquidBounce ae0eb396, not completed native
modules. Display-name translations do not imply gameplay or render parity.

## TrueSight

`render/ModuleTrueSight.kt` has the complete Sight selection (Barriers, Lights,
Entities), body RGBA (255,255,255,100) and feature-layer RGBA
(255,255,255,120). `MixinClientLevel` changes the marker target and the
placed-state comparison independently; the original state supplies the particle
texture. Implement both barrier and light markers without creative/held-item
requirements, while preserving normal markers when disabled.

`MixinLivingEntityRenderer` changes the living body tint and supplies a
translucent render type when vanilla has no body/translucent/outline. It also
changes the extracted state's invisible flag. `MixinRenderLayer` changes only
the `renderColoredCutoutModel` helper's tint/material, rather than indiscriminately
recoloring armor, held items, eyes or all vertices. Both retain ESP's independent
`requiresTrueSight` fallback (white alpha120) and the LogoffSpot path.

Horizon separates these consumers into `game/creatures/mob_geometry.rs`,
`game/perspective.rs` (own body and inventory preview), `game/armor_stands.rs`,
`mobs/render.rs` (body, per-layer material, CPU and GPU submission),
`mobs/illusioner.rs` (images), `mobs/trader.rs` (effect invisibility),
`animate_tick.rs` and `game/render_tick.rs`. The complete port must cover all of
them and preserve AntiBlind's original HEAD cancellation priority. Revealing
only the ordinary body and ignoring the feature/render-state branches is not
full parity.

## Hitbox

`combat/ModuleHitbox.kt` exposes Size=.1 (0..1), ApplyToDebugHitbox=true and
ApplyToComponent=true. The EntityMarginEvent replaces margin with Size only
when `shouldBeAttacked()` passes. This includes Attackable, world-border,
local-player/passenger exclusions, global combat Targets, EntityTaggingManager
classification, friend handling, alive/invisible/sleeping and entity-class
filters (`utils/combat/CombatExtensions.kt`). Shared team/bot tagging listeners
must be composed rather than treating all living entities as enemies.

`MixinAttackRange` and `MixinProjectileUtil` additionally add Size to the
attack-range component margin for the local attacker when ApplyToComponent is
enabled. Debug bounds have their own toggle and margin consumer. A complete
port must distinguish replacement of entity pick radius from addition to the
component margin, and apply these paths without changing collision dimensions.

## NameProtect

`misc/nameprotect/ModuleNameProtect.kt` updates own/friend/other-player mappings
at game tick, supports custom aliases, default friend aliases, private config,
and separate static/rainbow color modes for each group. `NameProtectMappings`
implements overlap/case/matching/caching policy; it is a required dependency.

Both String replacement and FormattedCharSequence wrapping are required. The
wrapped sequence accumulates UTF-16 indices across style runs, preserves styles
outside replacements, applies replacement colors, supports early sink exit,
and respects the TextColor name-protection bypass. All rendering, font width,
line splitting and foreign text sanitization hooks need the same replacements.
A simple substring replacement of player names in chat is not full parity.

## Next batch delivered instead

CrystalView and AutoF5 have standalone behavior and Horizon patch helpers with
their full pinned settings. Their fixture reports explicitly distinguish real
transformed function/API compilation from the parent's full-client and in-game
checks. See `build/crystal-view-hook-tests/result.json` and
`build/auto-f5-hook-tests/result.json`.
