# Theme font bridge handoff

Pinned LiquidBounce: `ae0eb396c07f22383789b28d5912d714c6ca0a86`.

## Root integration

Run the shared transactional integrator after the official 17002 staging boundary:

```text
python native/liquidbounce-horizon/integrate-theme-font.py --check
python native/liquidbounce-horizon/integrate-theme-font.py
```

The integrator edits two shared engine files in memory and commits only exact anchors. Then regenerate the normal Horizon staging patch and run the grouped native plus engine checks. The focused font fixture is intentionally separate:

```text
cargo +stable test --offline --manifest-path build/theme-font-static-01/Cargo.toml -- --test-threads=1
```

No Rust, Cargo, GPU, or game run was performed while preparing this handoff.

## Behavior

`target-renderer-font-registry.rs` parses each listed theme font's SFNT `FULL_NAME`, so `metadata.json` `nativeFont` selects a face name rather than a filename. Registered faces persist across reloads like `FontManager.fontFaces`; an invalid replacement leaves an already registered face available. Unknown or case-mismatched names fall back to included `Inter Medium`. Included `Inter Bold` remains the bold face for the default theme; custom faces use synthetic dilation when needed.

`ThemeFontMonitor` uses the browser's actual `theme_root`, loads on startup, checks metadata and listed font file stamps once per second, and reloads on Ctrl+R. The native `Font::begin_frame` compares the registry generation and clears only the `liquidbounce:target*` glyph sets before new HUD text is queued.

## Evidence and limits

`build/theme-font-static-01/report.json` records static validation, real SFNT names, transactional patch scope, and the focused test manifest. The smoke covers internal-name matching, custom face switch, generation/cache invalidation, CJK fallback, synthetic bold, case-sensitive fallback, invalid-font retention, malformed metadata retention, and same-reload stability.

The existing TargetRenderer reports and freezes remain historical. `target-renderer-freeze-03.json` appends this font revision without changing earlier freeze files. The original theme marketplace and selection manager are outside this bridge, and `ab_glyph` output may differ from AWT/FreeType pixels until official visual validation.

## Ownership

Production-owned font files are complete and frozen pending grouped official verification. Future rendering work must remain under an independent draft path until handed off; do not stage unfinished font or renderer edits into production-owned files.
