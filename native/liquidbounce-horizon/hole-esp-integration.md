# HoleESP handoff

Pinned LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Horizon-Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.

Root applies serially:

```powershell
python native/liquidbounce-horizon/integrate-hole-esp.py
python native/liquidbounce-horizon/integrate-hole-esp.py --apply
python native/liquidbounce-horizon/update-hole-esp-locales.py --directory native/liquidbounce-interop/resources/localization
```

The transaction validates 15 edits across 7 shared files before writing. Adds exactly one module, `HoleESP` / Render / `holeESP`, with tag=null as upstream does not call tagBy, actual actor GET/File/import/value changes and both inactive Mode choices. It does not touch frozen AutoBreak or ItemScroller. `hole-esp-freeze-02.json` supersedes the original freeze's extra Mode tag integration step; production scanner/settings/runtime and the passing smoke bodies are unchanged.

Owned behavior `hole_tracker.rs` exposes source-derived `HoleTracker`, shared reference-identity `HoleManager` subscribers and `Region`/MovableRegionScanner. The native patch copies the identical pure scanner into the actual game. Native cache reads real resident block facts, AIR and explosion resistance, observes COW sections through weak snapshots, diffs classification changes, handles unloaded chunks, clears on world reset or disable, and runs at real LocalPlayer post-tick. No synthetic world entities or full world clones are retained.

Both Box and GlowingPlane are complete. Scan distances/defaults, four colors, independent outlines, GlowHeight, signed vertical distance fade, alpha truncation and integer footprint bounds follow the source. The drawing instance copies the already frozen Breadcrumbs colored quad/one-physical-pixel-line renderer, always with effect=null; no new shader. GlowingPlane draws a DOWN face, optional bottom perimeter and four vertical gradients.

Focused evidence: `build/hole-esp-smoke-03/report.json`, 5 tests passed. Actual production scanner, manager, settings, engine runtime and native bridge compile/run against declaration-only Host/registry/resident-world providers. Tests cover classification thresholds, all footprints, filling headroom, wall breaking, COW snapshots, chunk unload, disable/reset, region movement, subscriber max distances, mode geometry, signed fade, alpha replacement/truncation, inactive File/Interop roundtrip and all 16 descriptions in each copied Chinese dictionary. Historical smoke-01 (missing fixture enum Default) and smoke-02 (fixture mistook Bind.key for localization key) are preserved.

No official engine/native build, actual game parity or GPU matrix is claimed. Root owns official integration and final full validation.

Remaining native caveat for final game QA: initial subscription is picked up on the next real LocalPlayer post-tick, while upstream onEnabled can schedule its scan immediately. Live chunk block edits are observed at that same post-tick from actual final world state; the original asynchronous ChunkScanner may observe intermediate packets separately.
