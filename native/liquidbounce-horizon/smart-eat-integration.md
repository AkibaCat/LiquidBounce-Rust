# SmartEat integration handoff

Baseline: LiquidBounce `ae0eb396`, `ModuleSmartEat.kt`; pinned MC26.3 JAR SHA1
`e1a9a856d71bfb9b18a06d997a38de4086bb5fd2`. AutoEat is a child of SmartEat,
not a separate original module. This implementation is not registered yet.
The 236-module port and full original-client/Rust-client differential remain
incomplete; the latter is `not_run`.

## Owned implementation

- Behavior: `native/liquidbounce-behaviors/src/smart_eat.rs`.
- Actor snapshot decoder: `engine/src/smart_eat.rs`.
- Config child: `engine/src/config/smart_eat.rs`, including the complete original
  declaration order, toggleable groups, defaults, suffixes and ranges.
- Source transform: `smart-eat-patch.py` calls `smart-eat-return-patch.py`.
- Native host: `smart-eat-host.rs`, `smart-eat-use-result-host.rs`,
  `smart-eat-pickable-host.rs`, `smart-eat-item-ray.rs`.
- Actual Java dispatch tables: `smart-eat-use-owners.rs` (all 1658 original
  items, 27 use owners), `smart-eat-animation-overrides.rs` (23 overrides).
  Regenerate with `collect-smart-eat-facts.py`.

The parent integrator owns shared behavior exports, `ModuleKind`,
`BehaviorModules`, config dispatch, runtime registration and patch generation.
Add the engine `smart_eat` child and shared behavior `smart_eat` export.
`smart_eat_use_result.rs` is included privately by the source transform and
does not require an engine/shared export.

## Patch ordering and RETURN event

Compose SmartEat after Mining, Packets, Slots, Fishing, NoBlockInteract and
MultiActions. The fixture composes Attack, Mining, FastPlace, NoBlockInteract,
Packets, Slots, Fishing, MultiActions, then SmartEat. SmartEat recognizes
MultiActions' moved held-use query in `hand.rs` and replaces it with the logical
key query. Its original standalone path in `game.rs` is also supported.

Added native hooks:

- `client_smart_eat_wait(&mut Host, empty event)` at native client tick HEAD.
- `client_use_key_pressed(&Host, pressed: &mut bool)` for each logical use-key
  query, including native helper queries.
- `client_item_interacted(&mut Host, pass: bool)` after normal/rotated remote
  Item.use commit and local Item phase completion/restoration.

Added helpers:

- `client_smart_eat_snapshot() -> String`, parsed by `parse_snapshot` into
  `Result<Option<Context>, String>`; `null` means unavailable local player.
- `client_smart_eat_draw(batch, slot: i32, offset: i32)`.

The item result is classified from pre-use actual dispatch, effective components,
cooldowns, dedicated raw-angle OUTLINE ray and actual entity state. It never
uses Horizon's consumed-action boolean to infer PASS. FAIL and SUCCESS both
set `pass=false`. Original items have exhaustive owner coverage; unknown
mod-defined items return None unless the outer spectator/cooldown rule already
determines PASS. Nonlocal/server/replay turns emit no client event.

Local `use_hand` is split into `GuestUse::OnBlock` and `GuestUse::Item` phases,
restoring the previous value before RETURN and on errors. These phase values
do not change actor identity. A consumed/failed block phase emits no Item event.
Normal clicks skip empty stacks; Fishing's direct Item.use path includes air.
Spectator Item.use emits PASS without effect or packet. The original client's
FoodOnAStick PASS continues to offhand even if the integrated server effect
boosted its vehicle. Filled portal-frame and PlaceOnWater useOn fallthrough
corrections keep Item.use reachable at the correct phase.

Bottle's dragon-cloud owner branch is false on the original ClientLevel copy.
`audit-smart-eat-cloud-owner.py` verifies real constructors, metadata, packet
creation and every client constant pool: no cloud owner is synchronized or set
by the vanilla client. The authoritative server can collect dragon breath;
its `Breath.from_dragon` must never substitute into the client RETURN result.
Evidence is `build/smart-eat-tests/cloud-client-owner-proof.json` and
`cloud-client-owner-bytecode.txt`.

Boat scans its real expanded player AABB and tests the eye against qualifying
physical boxes inflated by the correct pick radius. Base-Entity margin hooks
apply; Projectile's radius 1 override bypasses them. It includes natural and
remote leash knots, TNT, falling blocks, living entities even at zero health,
and all native pickable categories. Removed living entities, spectator Players,
marker stands, in-ground arrows/tridents and the local player are excluded by
their actual respective rules. Entity-type tags use remote Dynamic UpdateTags
and Builtin IDs first; a missing tag in a synced registry is empty. Local
`stations().enchantments.entity_tags` is only the no-remote fallback.

## Runtime integration and lock boundaries

Take/parse the native snapshot outside the module-registry lock. Compute policy
under the state lock, then drop it before host selection/draw calls: selection
events and native callbacks can re-enter the registry. `Effects.pause_combat`
uses the shared `CombatWindow::pause_combat_for_at_least`, including `Some(0)`.
`NotDuringCombat` reads the existing shared `is_in_combat` state.

For each `Effects.selection`, use `slot.hotbar_index()`:

- `Some(index)`: call the existing native `client_select_slot_silently` with
  `selection.requester`, index, `selection.ticks` outside the state lock.
- `None` (offhand): successful no-op, preserving the prior silent-slot owner
  and emitting no selection event. Never send -1 into SlotService.

`slot.native()` encodes -1 only for the draw helper. Owner IDs are distinct:
`SILENT_OFFHAND_REQUESTER` and `AUTO_EAT_REQUESTER`.

Use `client_smart_eat_wait` as the global early CoroutineTicker dispatch entry,
before SilentHotbar's priority-1001 tick and before CombatManager decrements.
The transform inserts this call at native tick HEAD after the Slots transform,
so it runs before the slot timer. Do not independently advance SmartEat first:
its relative order with Derp, AntiAFK and other pending/running waits is the
shared ticker's FIFO order.

After every `start_run`, immediately drain `take_wait_registrations()` and
register each id as `WaitOwner::SmartEat`, before applying Host effects or
running another module continuation. The first callback belongs to next tick.
At its FIFO position take a fresh live snapshot and call
`advance_callback(id, running, context.as_ref()) -> (keep, Effects)`. Keep=true
retains this exact callback. It persists until tickUntil finishes; a new waiter
always gets a fresh checked counter id. A stale cancelled callback returns
false without touching a replacement. None context simulates player access
failure, cancelling this callback and preserving force-use. The legacy
`advance_waiter` exists only for standalone policy tests; runtime must use the
global FIFO interface.

On ordinary GameTick, call `silent_offhand_tick(running, owner, context)` and
`start_run(running, in_combat, context)`. Derive owner by querying whether the
slot is modified by `SILENT_OFFHAND_REQUESTER` outside the state lock; unrelated
owners must not refresh. `start_run` only pauses/starts the waiter; it does not
eat immediately. Existing waiter callbacks do not recheck the combat-entry gate.
Original AutoEat has no requires guard: its stopAt/eat callback runs before
the listener-running check, which occurs only when the completed coroutine
resumes. Therefore advance_callback may still select while running=false;
completion then preserves force-use instead of executing the final clear.
Explicit CancelSequences lifecycle must call cancel_auto_eat separately.

On `client_item_interacted`, call `interacted_pass` only when `pass` is true.
On every `client_use_key_pressed`, call `use_key(running, pressed)` and apply its
combat pause; never write the result into the stored physical/toggle button.

On `gui_layer_pre("minecraft:hotbar")`, take the fresh snapshot, choose
`render_food(running, context)`, release the state lock, then call the draw helper
with `food.slot.native()` and `render_offset`. It preserves the original F1
gate, integer GUI center, decorations/model/frame order, item coordinates
`center-117-offset,height-19`, and frame `center-120-offset,height-23,29x24`.

## Lifecycle and intentional original quirks

When active listeners are disabled/cancelled, end the AutoEat waiter with
`cancel_auto_eat`; it intentionally preserves force-use state. Reset only the
disabled child's own silent-slot requester. A module disable stops both child
listeners; a child setting transition invokes its onDisabled only while in-game
and its parent is enabled. The config child's `apply` mutates values only;
runtime supplies these lifecycle effects. Do not reset force-use on disable.

WorldChange alone must preserve the waiter. Its next callback without a player
aborts the waiter and preserves force-use. Do not add a WorldChange reset for
the shared CombatWindow pause value.

The estimator walks offhand then hotbar 0..8. It preserves the original
`maxWithOrNull` distance bug (farther hotbar slot wins; ties retain first),
integer inverse health priority, potion branch without FOOD fallback and
emergency apple `foodComponent!!` abort. A missing emergency FOOD ends the
waiter while preserving force-use; an absence of any food keeps it waiting.
AutoEat eats before checking hunger > minimum; equality keeps waiting.
Selection cancellation still sets force-use. SilentOffhand's first selection
uses max(delay,5); eating refresh uses the raw delay and only its own owner.

## Verification and limits

`python native/liquidbounce-horizon/test-smart-eat.py`: **47/47 passed**.
`cargo +stable clippy --manifest-path build/smart-eat-tests/Cargo.toml
--all-targets -- -D warnings`: passed with the shared test target cache;
output retained in `build/smart-eat-tests/clippy.log`.
`python native/liquidbounce-horizon/audit-smart-eat-composition.py`: passed
the complete current production chain plus SmartEat, 223 Rust files parsed with
rustfmt's skip_children option. This is syntax validation, not a native typecheck;
evidence is `full-composition.json` and `full-composition-syntax.log`.
The test compiles actual transformed RETURN wrappers, actual classifier/ray/
Boat scan, effective component conversion, shared config adapter and native
hook generator. It uses real Horizon Items, BlockFacts, BlockRegistry, protocol
Dynamic UpdateTags/Builtin IDs and geometry. Production network/effect/draw
implementations are stubbed; world shapes are explicit fixtures.

Results and scope: `build/smart-eat-tests/result.json`, `test.log`.
Actual Java evidence: `oracle.json`, `java-oracle.log`, `use-bytecode.txt`,
`can-eat-bytecode.txt` (Abilities.invulnerable, not instabuild),
`entity-owners.json`, `entity-pick-bytecode.txt`, `cloud-client-owner-proof.json`,
`cloud-client-owner-bytecode.txt` in the same build directory.

The wrappers establish the event boundary over Horizon's existing effects.
They do not establish full parity of every native item effect, world mutation,
network prediction or rendering path. Full original-client/Rust-client
differential remains **not_run**.
