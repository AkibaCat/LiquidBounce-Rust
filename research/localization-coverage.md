# Static Chinese configuration-caption coverage

Pinned upstream: `ae0eb396c07f22383789b28d5912d714c6ca0a86`. Audited 236 registered module roots and 696 module-directory Kotlin files.

The upper-bound inventory contains 3905 literal declarations / 2439 distinct captions.

| Locale | Translated declarations | Deliberately retained English | Simplified fallback | Missing declarations | Missing distinct captions |
| --- | ---: | ---: | ---: | ---: | ---: |
| zh_cn | 3789 | 116 | 0 | 0 | 0 |
| zh_tw | 3373 | 116 | 416 | 0 | 0 |

867 nonliteral/tree declarations remain explicitly recorded for Kotlin/runtime resolution. The per-module report includes reachable configuration source files, literal indices, dynamic indices, and ambiguous references.

This report is a lexical audit of display vocabulary. It does not prove an exact evaluated runtime configuration tree or completed native gameplay implementations. File-level helper closure can include sibling declarations; module-directory coverage also includes unattached helpers and three unregistered modules.

Regenerate: `python scripts/audit-localization.py`. Check freshness: `python scripts/audit-localization.py --check`. Require no missing simplified literals: `python scripts/audit-localization.py --check --fail-on-missing`.
