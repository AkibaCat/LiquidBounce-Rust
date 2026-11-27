# Display dictionary correction, revision 01

The owned NewChunks, DamageParticles, ProtectionZones and PlacementESP locale
updaters previously wrote full translation keys at the top level of display JSON.
The real native DisplayDictionary consumes only moduleNames, labels,
descriptions and englishDescriptions. Those top-level additions were ignored.

These four updaters now write through display_locale_schema.py into the actual
schema. Module names use stable registry names; setting and tagged-choice labels
use unchanged original English identifiers. Descriptions retain complete source
translation keys. English help entries cover the same settings. PlacementESP
updates both ProphuntESP and VoidESP. Only legacy top-level entries owned by these
five modules are removed. Canonical simplified and traditional dictionaries are
updated; pinned upstream language files and prior frozen evidence are preserved.

This is a display-resource correction. No gameplay smoke, server test or GPU test
is repeated or claimed. The root agent owns official aggregate builds and actual
game language-switch validation. Copied-dictionary schema/idempotency results are
recorded separately under build/display-locales-fix-01/report.json.
