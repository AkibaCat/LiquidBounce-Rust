# Chinese display support

The game selects the language. The native bridge reads
`horizon_client::lang::code()` and `generation()` from the actually loaded
Horizon language resources. Chinese variants use simplified Chinese by default;
`zh_tw`, `zh_hk`, `zh_mo` and `zh-Hant` variants use traditional Chinese. Other
languages preserve the English LiquidBounce display. No separate port language
preference is stored.

`GET /api/v1/client/localization` supplies a display snapshot. A real language
generation change broadcasts `languageChange` and the existing
`clientLanguageChanged` event. The Web theme refreshes after a language change
and after reconnecting its socket. The snapshot includes the generation and the
actual game language so integration checks can inspect the real selection.

The simplified display dictionary contains all 236 upstream registered module
names. This dictionary never registers gameplay modules: only functioning native
implementations belong in the actual module registry. Module names, category
keys, setting names, mode/choice values, binds, REST parameters and saved paths
retain their original English identifiers. Search indexes both Chinese display
names and the original English names/aliases.

The three original language JSON files under
`native/liquidbounce-interop/resources/localization/` are byte-exact copies from
LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`. Original module descriptions
take priority. The pinned files have no descriptions for the implemented
settings; 270 explicitly owned English/simplified/traditional help entries fill
those missing keys. The source and hashes are recorded in `provenance.json`.

Traditional Chinese uses independent entries when available, then simplified
Chinese, then English. The fallback is deliberately recorded; missing entries
are not claimed to be independently reviewed traditional translations. A native
label query is available through
`liquidbounce_interop::localization::display_label(language, english_label)` and
`display_module_name(language, stable_module_name)`. Native descriptions use
`describe(language, translation_key)` and `module_description(language, name)`.

The theme is an explicit derivative, not a byte-identical original build. Its
controlled patch is `patches/liquidbounce-theme-chinese.patch`; the two added
TypeScript sources live under `native/liquidbounce-theme/src/`. The generator
changes presentation expressions and leaves identifiers and bound values intact.
It adds system CJK font fallbacks while retaining Inter for Latin glyphs. It also
fixes an inherited Svelte type inference issue in the derived connecting screen
using an explicit `$state<GameWindow | null>` generic. The original source is
unchanged and still has its three original type-check errors.

Build and inspect the derivative with:

```powershell
python scripts/update-localization-resources.py
python scripts/make-theme-localization-patch.py
python scripts/build-localized-theme.py --build
node scripts/verify-theme-localization.mjs
cargo +stable test --manifest-path native/liquidbounce-interop/Cargo.toml
cargo +stable clippy --manifest-path native/liquidbounce-interop/Cargo.toml --all-targets -- -D warnings
```

The staged source is `build/themes/chinese/src/`; the production output is
`build/themes/chinese/dist/`. `localization-build.json` records the pinned
baseline verification, exact patch and overlay hashes, input/output manifests,
Node/npm/Vite/Svelte/TypeScript versions and the type-check/build exit codes.
Baseline verification checks all 561 tracked theme files against the fixed
official source archive (or pinned Git blob hashes during source restoration).

The frontend verification covers simplified/traditional labels, English
restoration, word spacing fallback, search in both languages, original module
descriptions, already-open setting descriptions and unchanged configuration
objects. Native unit tests cover locale selection, resource fallback, display
snapshots and stable keys. These checks are not evidence of a completed
in-game language-switch test; the parent integration run records its own actual
`zh_cn` → `zh_tw` → `en_us` UI snapshots and screenshots.

The static caption audit scans all 236 upstream registered module roots and
their literal configuration vocabulary, including tagged enum choices. It
records dynamic declarations and ambiguous references separately; it does not
execute Kotlin or claim an exact evaluated configuration tree. Regenerate with
`python scripts/audit-localization.py`, then check freshness and missing
simplified literals with `python scripts/audit-localization.py --check
--fail-on-missing`. The reports are `research/localization-coverage.json` and
`research/localization-coverage.md`.
