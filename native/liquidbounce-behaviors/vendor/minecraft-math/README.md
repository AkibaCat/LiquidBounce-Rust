# Minecraft sine table

This is an exact copy of `src/navigation/simulation/sine-table.bin`, collected
from the pinned original Minecraft artifact with JDK 25 by
`native/liquidbounce-horizon/collect-simulation-sine-table.py`.

SHA256: `cdfaec6870788e193dff3f1ddfa3ca7d3f1a897042d78365a1fe3a433be3627d`.

The package reader includes `vendor/` recursively. Hats consumes this copy so
the behaviors crate compiles independently of the game's navigation overlay.
