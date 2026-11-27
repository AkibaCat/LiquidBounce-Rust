# AutoPearl — owned implementation, ready for the shared batch

Pinned LiquidBounce ae0eb396. Complete Trigger path; Target mode uses the explicit `AutoPearlRuntime.aura_target: Option<i32>` seam. The current engine does not implement KillAura, so this field defaults to None and must be populated from its actual target later. This dependency is not claimed as completed functionality.

Source configuration order is Mode, Rotate (full shared Rotations tree), Limits, CombatPauseTime, SlotResetDelay. Original defaults, ranges, units, CHOOSE metadata, aliases PearlFollower/PearlTarget and Combat category are retained. Both Chinese locale variants include 38 descriptions each, including the remapped complete existing shared rotation subtree.

The native bridge captures raw AddEntity packet position/movement, resolves its owner once before vanilla entity insertion, and handles the integrated-server add_projectile equivalent. A FIFO delays guest/server acting contexts until canonical local state is restored. Epochs discard stale-world facts. The source queue only adds while empty, can coexist with an active coroutine, and the actual CoroutineTicker performs the minimum one callback, twenty-tick timeout, slot guard refresh and direction-angle <=1° check. Aim requests use provider 21, processor base 280, priority 35, including final/success callbacks. Throws use actual offhand-first slots, silent selection, randomized reset delay, and GCD-normalized target rotation.

Both real and hypothetical pearls use the actual resident COLLIDER/no-fluid ray and contextual native entities. The shared exact projectile solver is reused; incoming owner rays retain real equipment, crouching and powder-snow context. The hypothetical projectile copies actual player motion unless Freeze is running, starts eyeY-0.10000000149011612, applies the initial velocity-only correction, and moves at ticks 1 through 239. The simulated impact is still required with Limits disabled. Strict destination-distance comparison and inclusive minimum-distance boundary match source.

No Rust/Cargo smoke was executed for this module, per the user's multi-module batch instruction. `build/auto-pearl-static-03/report.json` passed transactional engine anchors (9 files), in-memory native patch composition (7 files) and locale schema checks. Earlier static-01/02 directories are incomplete failed static-check attempts and remain preserved. `auto-pearl-smoke.rs` is prepared for root's one focused usability invocation alongside the official native+engine batch. Full actual-game parity remains unverified.

Root integration, serially:

```
python native/liquidbounce-horizon/integrate-auto-pearl.py --check
python native/liquidbounce-horizon/integrate-auto-pearl.py
python native/liquidbounce-horizon/update-auto-pearl-locales.py --folder native/liquidbounce-interop/resources/localization
```

The focused batch harness is a standalone Rust main, using production owned behavior, rotations and projectile math:

```
rustc +stable --edition 2024 native/liquidbounce-horizon/auto-pearl-smoke.rs -o <batch-output>/auto-pearl-smoke.exe
<batch-output>/auto-pearl-smoke.exe
```

`auto-pearl-patch.py` must compose after `auto-bow-patch.py` because it reuses the audited native pickable universe without editing AutoBow's owned source. The complete generated body is checked before patch publication. AutoBreak, ItemScroller, shared sources, staging, pristine upstream and canonical locales were not modified by this delivery.
