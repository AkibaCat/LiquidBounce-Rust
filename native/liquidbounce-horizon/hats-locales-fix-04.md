# Hats display-dictionary correction, revision 04

The display dictionaries deserialize `moduleNames`, `labels`, `descriptions`
and `englishDescriptions`; old top-level translation keys were ignored.
The updater now writes into those real sections and removes only legacy
top-level `liquidbounce.module.hats*` entries. Rendering descriptions inherit
the complete frozen Breadcrumbs translations with Hats terminology.

Run the updater again against canonical localization resources. No gameplay
or GPU smoke was repeated. `build/hats-locales-04/report.json` records copied
dictionary schema checks. The native test provider's Host signature also now
matches mutable `Host::chat`, as required by the revision 03 production fix.

Root owns official aggregate native/engine checks and actual game visuals.
