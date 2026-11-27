# AntiVoid handoff — MC 26.3

Pinned LiquidBounce: `ae0eb396c07f22383789b28d5912d714c6ca0a86`.
Pinned Horizon: `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`.

Integration owner, apply serially after AntiBot and the existing Blink/NoSlow/Step services:

```powershell
python native/liquidbounce-horizon/integrate-anti-void.py --apply
python scripts/update-anti-void-localization.py
```

The integrator validates 28 edits across 9 shared files and rechecks every pre-write byte snapshot. This owner has not applied shared integration, changed locale files, staged the native tree, or invoked Cargo/rustc. The parent owns regeneration, staging and the next batch compilation.

The generic AntiVoid extension runs immediately before Step. Step remains the last extension, so its constructor scan sees every adapter. The integration plan adds one explicit late call after Step rewrites remote.rs; owned anti-void-late-patch.py then wires the PRE/POST sendPosition callbacks without changing constructor order.

Implemented:

- Disabled-by-default Player module; GhostBlock default, Flag, Blink and UseItem; all eight primitive settings and original defaults, ranges, units and registry-list serialization.
- Real NativeSimulation body replay. The first falling snapshot among ticks 0–9 decides prediction. UseItem extends the same forecast through ticks 1–40, stopping at the first grounded snapshot. Prediction suppresses all synthetic shapes; UseItem's later extension retains the source's unsuppressed behavior.
- Actual loaded-world BlockCollisionScan with the live player's context, main-hand stack, powder-snow subject and moved bounding box. AABB.setMinY reorders endpoints in MC 26.3; this was verified against the official bytecode and uses Horizon's matching with_min implementation.
- Process-wide GhostBlock only changes empty shapes below floor(rescuePosition.y), including cached-empty sections. It observes shape suppression and the actual player's death state.
- Flag uses the exact float-to-double Height. Silent adds to the real sendPosition PRE Y before position-sender baseline calculation and sees POST after packet dispatch. It is excluded from passenger sendChanges. Ordinary until-listeners retain registration count, read the current Height, survive POST and self-remove on PRE or when Flag is no longer running.
- Blink contributes outgoing Queue to the existing monotonic shared BlinkManager and cancels that manager's actual queue, resetting to its first stored position. It additionally exempts Scaffold. The packet bridge refreshes the actual borrowed player's death state.
- UseItem checks the configured downward velocity and future landing, then uses the selected server slot, offhand, or nearest hotbar slot with original tie ordering. It samples the shared random reset range, selects through real SilentHotbar and uses the active rotation; an unsuccessful interaction still counts as the source rescue attempt.
- onEnabled resets only likelyFalling. No invented disable/world clearing of rescuePosition. Notifications and all 13 descriptions are provided in Simplified Chinese, Traditional Chinese and English fallback.
- Fly and Scaffold query the real registry. Shared RescueGuards exposes set_clutch_guarding, gated by the actual Clutch running state. **Clutch is not yet registered; its real canCatch writer remains a follow-up and is required before final inter-module acceptance.** It is not substituted with enabled/running state.

Checks performed:

```powershell
python native/liquidbounce-horizon/test-anti-void.py --static-only
```

Result: 15 native files composed in memory; 28 shared edits/9 files validated; all 13 descriptions and labels validated; Python AST and rustfmt parsing passed. No shared/staging writes.

Prepared for the parent's combined test batch, **not run by this owner**:

```powershell
python native/liquidbounce-horizon/test-anti-void.py --model-only
```

Seven model/config cases cover every mode, tick boundaries, first-fall decision, source lifecycle retention, float promotion, dynamic PRE/POST listeners, Blink positions and exemptions, slot priorities, eight-setting File/Interop roundtrip, atomic item registry parsing and non-finite serialization. Official compilation and Kotlin differential execution have not been claimed.
