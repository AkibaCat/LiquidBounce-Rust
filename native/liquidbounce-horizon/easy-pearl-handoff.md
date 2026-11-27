# EasyPearl owned delivery

Pinned upstream: `ae0eb396c07f22383789b28d5912d714c6ca0a86`, `ModuleEasyPearl.kt`.

Complete Misc module and aliases PearlHelper / PearlAssist / PearlTP. Settings retain the original order: AimOffThreshold 2.0 in 0.5..10.0 (no suffix), ReachableCheck true, the full shared Rotations tree, ShowDistance true. Rotation provider 22 / processor base 300 / priority 40. No shared files, staging or canonical locales were edited.

## Integration

After AutoPearl is integrated:

```text
python native/liquidbounce-horizon/integrate-easy-pearl.py --check
python native/liquidbounce-horizon/integrate-easy-pearl.py
python native/liquidbounce-horizon/update-easy-pearl-locales.py --folder native/liquidbounce-interop/resources/localization
```

Regenerate the official behavior patch and run the grouped native+engine build. The native EasyPearl patch depends on AutoPearl's actual world-ray and feet/eye solver bridge; it does not modify AutoPearl's owned sources. The real useItem HEAD runs for local and both normal/rotated remote paths; cancellations emit PASS RETURN. For local predicted use, the HEAD observes the real view before prediction temporarily aligns item use with packet angles, and the inner item boundary skips duplicate dispatch. Every canonical player query is guarded against guest/server/replay contexts.

The model preserves the source's unusual lifecycle: disabling clears only currentTargetRotation; targetPosition persists. A missing target or failed solve leaves the previous currentTargetRotation untouched. The real Host item use runs with the raw solved yaw/pitch and delay zero, outside any engine RefCell borrow; target/current fields clear only after that call returns, preserving recursive interaction ordering. Unreachable non-MISS hits display the original localized red warning and clear only targetPosition. MISS plus no solution proceeds normally.

The real look ray is OUTLINE / Fluid.NONE at 1000 blocks, using interpolated player eye and view rather than managed rotation. Projectile solving uses actual player feet and the shared GENERIC solver, dimensions 1 x 0. World rendering uses through-wall original green/red FULL_BOX faces (alpha50), outlines (alpha200); air uses a DOWN face and four 0.7-high sides fading to alpha0. Distance is from current feet to hit, one decimal and `m`, projected at hit+eyeHeight; original theme font metrics use scaleToVanillaFont (9/height), centered/middle with shadow.

## Validation

`build/easy-pearl-static-01/report.json`: 8 shared files validated transactionally, 10 native files composed in memory (including the not-yet-staged AutoPearl dependency), and 29 EasyPearl description keys in each Chinese display locale. Original simplified/traditional warning keys checked. Python scripts parsed and owned Rust files formatted. Rust/Cargo/GPU execution deliberately deferred to the user's grouped build request.

Prepared focused production-model smoke, not executed:

```text
rustc +stable --edition 2024 native/liquidbounce-horizon/easy-pearl-smoke.rs -o <batch>/easy-pearl-smoke.exe
<batch>/easy-pearl-smoke.exe
```

It checks holding/use guards, threshold equality, MISS/unreachable policy, disable persistence, solve-null assignment behavior, missing slot, raw offhand/slot effect and reentrant clear-after-use ordering. Official native+engine compilation and actual-game validation remain outstanding. The current shared font bridge rasterizes original Inter and uses its CJK fallback; active custom-theme switching is the next separately assigned task. ab_glyph metrics can differ from original AWT/FreeType. No smoke fixture establishes actual-game pixel parity.
