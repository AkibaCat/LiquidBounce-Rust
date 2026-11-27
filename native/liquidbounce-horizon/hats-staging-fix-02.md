# Hats staging correction, revision 02

Official aggregate Check 133-01 found that the behaviors crate's relative
`src/navigation/simulation/sine-table.bin` was absent: the staging builder
installs that binary only in the native game's navigation overlay.

Hats now includes the byte-identical table from the behaviors crate's
`vendor/minecraft-math/` directory, which the existing package reader already
includes recursively. Source table SHA256 is
`cdfaec6870788e193dff3f1ddfa3ca7d3f1a897042d78365a1fe3a433be3627d`.

No arithmetic or rendering changed. No smoke was repeated. Historical
`hats-freeze.json` and smoke01–04 are retained; `hats-freeze-02.json` records
this source/package correction and the original successful focused evidence.
Root performs the next official aggregate Check.
