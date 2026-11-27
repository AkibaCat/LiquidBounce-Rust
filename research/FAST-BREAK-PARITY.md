# FastBreak source parity

Pinned LiquidBounce commit: `ae0eb396c07f22383789b28d5912d714c6ca0a86`.
The owned implementation ports the complete `ModuleFastBreak.kt` module,
including None and Grim-2.3.48 modes and the former AbortAnother alias.

## Behavior and integration

FastBreak belongs to World. BreakDamage defaults to 0.8 and exposes the original
0.1..1.0 Float slider. OnlyTool defaults to false. The parser preserves raw
imported strengths, and File/Interop trees retain all original identifiers and
both mode choices. No new module-specific settings are introduced.

Each GameTick removes the client game-mode destroy delay. If current progress
is strictly greater than BreakDamage, it becomes exactly 1.0. OnlyTool permits
the actual main-hand item only when it is in one of the current axes, pickaxes,
shovels or hoes tags. Empty stacks fail the condition, matching actual
ItemStack.typeHolder behavior. Disabling restores the client game-mode delay
to five ticks regardless of OnlyTool. Server-side game-mode state is untouched.

The native helper uses the real client inventory slot, independent of acting
offhand state, and rejects native-window, LAN guest, server-copy and replay
contexts. It writes remote.destroy_delay / remote.destroying progress or the
local click game-mode state. The complete existing remote_mining implementation
then consumes the resulting progress and sends its normal completion packet.

Grim-2.3.48 observes normal outgoing PacketEvent at priority zero. For every
STOP_DESTROY_BLOCK satisfying OnlyTool, it synchronously sends an ordinary
ABORT_DESTROY_BLOCK at the position above, preserving direction and using the
original three-argument constructor's zero sequence. It still observes an
event cancelled earlier in the pipeline. Silent delivery bypasses this normal
listener. The Game.send head refreshes actual main-hand tags, so a selection
change followed by a packet in the same tick cannot use the earlier item.
Snapshots are scoped to a nonzero matching network session. The behavior mutex
is released before nested packet dispatch, and no Game or registry lock crosses
a callback.

Parent integration loads fast_break.rs and fast_break_bridge.rs into Horizon;
applies fast-break-patch.py after packet, HandDerp scoped sender, mining identity
and hotbar-slot support; installs FastBreakService at priority zero; and calls
fast_break_tick at GameTick. FastBreakSetting applies under the service's
behavior mutex. The normal disable lifecycle queues client_fast_break_reset
and executes it after dropping actor locks.

## Verification boundaries

Final evidence: `build/fast-break-09/report.json` passes all seven tests and
Clippy with `-D warnings`. Both original upstream trees remain unchanged.

```text
python native/liquidbounce-horizon/test-fast-break-patch.py --out build/fast-break-09
```

The standalone test compiles unmodified ModuleFastBreak.kt and byte-exact
original item categorization helper bodies against actual MC 26.3 classes.
Official recursive item-tag resources populate the real item registry. The
fixture binds empty component prototypes because components do not participate
in the source tag predicate; actual ItemStack count, isEmpty and typeHolder
methods execute normally. The ClientModule values, event activation, interaction
fields and network collection form a declared environment shell.

28,800 cases cover ten main-hand items; twelve threshold/progress combinations
including signed zero, out-of-range imports, NaN and infinity; both OnlyTool,
enabled and mode states; five packet actions; all six directions; and Java
BlockPos Y overflow. Raw Float progress bits and actual vanilla packet
constructor results are compared with Rust. All 1,658 registered items are also
checked with count one and zero, while the Kotlin negative-count oracle verifies
that those stacks are empty (native inventory counts are unsigned).

Native tests compile the transformed production helpers, generated Host helper
dispatch, complete remote_mining, Game.send, Session send methods, production
packet bridge and pipeline, FastBreak listener, real Pipe transport and packet
codec. Window/world/resource storage and prediction side effects form the native
environment shell. A no-op fixture hook satisfies the generator's requirement
for a nonempty hook catalog; the FastBreak Host helpers themselves are generated
from the production native helper signatures. Parent registry/config wiring,
disable-effect lifecycle and full-game execution require separate integration
checks and are not claimed by this standalone suite.

The fixture keeps remote and native_hooks as sibling modules. The two mining
field declarations are extracted from the transformed Remote struct, including
visibility, and the original Game.send and remote_mining methods remain in the
remote module. Thus the native helper must pass the same sibling privacy checks
as the full client. The configuration suite extracts the production parent
Tree/parser helpers and executes the owned complete child parser and serializer.
