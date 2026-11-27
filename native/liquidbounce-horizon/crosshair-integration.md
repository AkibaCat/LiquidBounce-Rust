Crosshair handoff, 2026-11-26

Root applies serially:

```powershell
python native/liquidbounce-horizon/integrate-crosshair.py
python native/liquidbounce-horizon/update-crosshair-locales.py --directory native/liquidbounce-interop/resources/localization
```

The transactional integrator validates all anchors and concurrent contents before
writing seven shared files. No pristine upstream or staging edits were made.
The native patch composes four staged files in memory, after CrosshairClicker:
its existing nearest-entity helper only becomes pub(super). Frozen AutoBreak
and ItemScroller code is untouched.

Implemented pinned source:

- Both tagged Circle and CS2 modes; independent inactive settings retained.
- Complete ranges/defaults/types and original hierarchy: ShowInThirdPerson,
  Circle Radius, CS2 Crosshair, shared Color, CooldownBar/Sources/ReadyFlash,
  and HitFlash. Toggle Enabled fields precede their children.
- Exact source color interpolation truncation/FMA, packaged Minecraft sine
  table, sync/two-color/negative or zero spin and per-angle gradient.
- Circle evaluates the original 256-angle LUT at physical pixel centers,
  including atan(p.x,p.y), repeat/linear sampling, 32767 inner-radius
  quantization, quad-derivative fwidth and one-sided smoothstep AA. It feeds
  the existing native HUD alpha pipeline through native_fill, preserving
  ARGB conversion. No new GPU shader/style matrix was introduced.
- Actual attack-strength partial ticks, main/offhand cooldown components,
  own used-item ticks and Quick Charge duration. Charge > Item > Attack
  source priority, exact-ready flash and no flash after early release.
- Actual same-target hurtTime tracking with mob/stand object joins, dragon
  and LAN guest state. Fresh targets do not start a hit flash. Disabling a
  toggle immediately resets its transient state.
- Custom HUD draws at extractHotbarAndDecorations's existing layer, uses
  source center divisors, first/third-person mode visibility and original
  InventoryScreen/ContainerScreen hiding. Other station/creative/mount
  screens preserve source behavior. The enabled module cancels the entire
  vanilla crosshair/attack-indicator layer independently of running state.
- Complete simplified/traditional nested DisplayDictionary translations,
  forty Crosshair descriptions each, verified by actual localization.rs.

Validation: build/crosshair-smoke-03/report.json passed four focused tests in
one Cargo invocation: source behavior, all settings File/Interop roundtrip,
production Runtime/native HUD bridge, and actual localization source. The
bridge test uses actual AttackStrength, native_fill and charge_ticks bodies;
Game/Host/registry declaration providers stand in for a running game. Native
patch composition and transactional integrator --check passed.

Smoke01 passed before immediate onDisabled clearing and toggle ordering were
completed. Smoke02 passed that small refinement. Final source review found
the native HUD packs channels differently from ARGB, so Smoke03 now uses the
actual native_fill body and asserts LiquidBounce blue is passed correctly.
All historical evidence is retained. No GPU suite was repeated.

This is focused usability evidence. Official full native/engine compilation,
real game rendering and differential visual parity remain root-owned.
