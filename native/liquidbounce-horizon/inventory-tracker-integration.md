# InventoryTracker and CommandInvsee

Source baseline: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`;
Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`.

## Apply in the root's serial integration batch

```powershell
python native/liquidbounce-horizon/integrate-inventory-tracker.py --check
python native/liquidbounce-horizon/integrate-inventory-tracker.py
python native/liquidbounce-horizon/update-inventory-tracker-locales.py
```

The integrator registers the World module, SavePlayers tree/actor route,
disable/world reset, pending native cleanup, invsee command/help entry and the
callback. It appends `inventory-tracker-patch.py` immediately after
`logoff-spot-patch.py` in the native generator. Do not independently add the
fragment twice. No shared or staging files were written by this handoff.

## Owned files

- `native/liquidbounce-behaviors/src/inventory_tracker.rs`
- `native/liquidbounce-horizon/engine/src/inventory_tracker_runtime.rs`
- `native/liquidbounce-horizon/engine/src/config/inventory_tracker.rs`
- `native/liquidbounce-horizon/integrate-inventory-tracker.py`
- `native/liquidbounce-horizon/update-inventory-tracker-locales.py`
- `native/liquidbounce-horizon/test-inventory-tracker.py`
- `native/liquidbounce-horizon/inventory-tracker-{native,screen,preview,layers,chat,host}.rs`
- `native/liquidbounce-horizon/inventory-tracker-patch.py`
- `native/liquidbounce-horizon/test-inventory-tracker-native.py`
- `native/liquidbounce-horizon/inventory-tracker-native-evidence.json`

## Behavior and native boundaries

Equipment is heard per actual client `setItemSlot` HEAD, including equal-valued
replacement objects and packets received while the module is disabled. Only
RemotePlayer facts pass the runtime and AntiBot gates; animal armor is ignored.
Offhand history updates before mainhand. History deduplicates item type and
enchantments, retains 36 entries and keeps object-identity timestamps after
history eviction. Equipping armor removes its corresponding history row.
Projection preserves the real main hand in slot zero and follows the original
empty-mainhand offset. SavePlayers changes clear saved object membership without
clearing history. Disable/reset clears tracked non-equipment slots 1–35 by UUID
and preserves CommandInvsee.viewedPlayer. World invalidation uses a Cell because
the existing world callback accepts `&self`; the next client observation consumes
invalidation before reading stale objects.

The existing `ItemStack::object_identity()` is the stack handle; `Mob.join` is the
player object token. Time is UNIX-epoch milliseconds. Equipment events deliver
complete facts to avoid native/actor re-entry while behavior State is borrowed.
Queries are `world`, `open`, `reset` and `clearSaved`; the native pending screen
open runs after command execution returns.

`.invsee <name>` first resolves a case-insensitive current tab-list name, then
prefers a current world object over its saved object. It accepts quoted Reader
strings and works with InventoryTracker disabled. It does not resolve arbitrary
offline saved names. Completion also works while disabled, reads the actual
configured command prefix, uses only current tab-list names and preserves pinned
SharedSuggestionProvider prefix matching after `.`, `_` and `/` boundaries.
Input cursor offsets count Unicode characters; returned replacement ranges are
UTF-8 byte start/length, validated by native before constructing suggestions.

The 176×166 native screen displays all 46 player inventory slots, hovered normal
tooltips, armor placeholders, carried item and a mouse-following full player
preview using the real avatar layers, armor, held items, cape and elytra pipeline.
It is read-only and does not issue ClickSlot. Inventory/Escape closes it. Lore is
appended only inside the viewed screen, in gray, with minutes modulo one hour.
English, simplified Chinese and traditional Chinese follow the game language.

## Focused validation for the next combined batch

No Rust compiler, Cargo, Clippy, game or broad suite was run for this handoff.
The model defines 10 focused `inventory_tracker` tests: equipment order,
identity/deduplication, armor removal, capacity and empty hands, bot/local/body
gates, saved/reset behavior, lore timing, prefix matching, Unicode ranges and
invalid extra/quoted completion arguments. Config has two focused cases:
File/Interop roundtrip and malformed-leaf isolation. Run their filters once in
the root's established combined environment. Static preparation uses AST,
transactional shared anchors, full native composition, temporary Rust parsing,
callback uniqueness and Chinese updater idempotence; it does not establish
actual-game parity.
