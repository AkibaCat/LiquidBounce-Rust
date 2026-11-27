# AutoTool / SilentHotbar integration handoff

Baseline: LiquidBounce `ae0eb396`. This child implementation does not establish
completion of the 236-module port or full-client behavioral parity.

## Files

- Behaviors: `auto_tool.rs`, `auto_tool_inventory.rs`, `silent_hotbar_render.rs`
  under `native/liquidbounce-behaviors/src/`.
- Engine children: `engine/src/auto_tool.rs`, `auto_tool_packets.rs`,
  `config/auto_tool.rs`.
- Native generation: `auto-tool-patch.py`, `auto-tool-host.rs`, generated
  `auto-tool-menu-providers.rs`. Compose after attack, packets, slots and
  MultiActions in the shared patch generator.

The shared behavior exports, ModuleKind, config dispatch and runtime registration
are intentionally left to the parent integrator.

## Runtime contract

Retain `Arc<auto_tool_packets::InventoryPackets>` and register it with the shared
`PacketListener` pipeline. Its `outgoing_accepted` implementation observes final
accepted original packets; silent sends are ignored. Before AutoTool tick,
progress or frame decisions, drain the current `host.client_session()` outside
the registry lock:

- `UseOn(pos)`: call `host.client_auto_tool_menu_provider(pos)`; if true, observe
  `{"kind":"mayOpen"}` using `host.client_auto_tool_time()`.
- `MayOpen`, `Click`, `Close`: observe `mayOpen`, `click`, `close` respectively.
- Forward native `client_inventory_observation(payload)` into
  `InventoryActor::observe(payload, now)`.

At `client_block_breaking_progress(pos)` obtain native
`client_auto_tool_snapshot(pos, module.require_near_bed)`, parse `Snapshot`, then
call `AutoTool::progress(running, snapshot.mining_context(pos,in_combat),
&snapshot.stacks)`. Use the existing shared CombatManager state. For Some(slot),
request the AutoTool silent-slot owner with the module's `swap_back_delay`.

On each game tick use a snapshot at `[0;3]` without bed scan and call
`InventoryActor::tick`. On each `client_auto_tool_frame` call `frame`, including
when the module is disabled so global inventory closing can finish. Native menu,
close and cursor operations must run outside the registry lock. After `Swap`,
call `performed` with the snapshot taken **before** the native click; its source
record is the item the server can restore. Packet callbacks can re-enter while
the actual menu click executes. For a visible cursor, move to the executor's
final cursor target immediately before native swap as well as handling
`Operation::MoveCursor`.

After a successful or failed Swap is reported via `performed`, refresh the
snapshot, drain newly accepted packet observations outside the lock, and advance
`frame` once more in that frame. This permits immediate closing when ClickDelay
and CloseDelay are both zero. The anchored plan contains only one chain, so the
next plan is created by a later GameTick; do not run `tick` inside the frame.
Stop the frame after MoveCursor (the same timestamp would return the same
position) or after Close. No unbounded frame loop is needed.

For `client_silent_hotbar_render`, use `SilentHotbarRender::render` with the
existing slot service's clientside slot and its `modified` flag. It changes
visible slot and swap-height scale only. The transformed native `tick_hand`
retains actual server-held equipment and attack-strength reads.

The existing CancelBlockBreaking event calls
`module.cancel_breaking(running)` even if a later AbortBreaking listener cancels
the native stop operation.

## Lifecycle

- Module disabled: reset only its silent slot and call `AutoTool::disable()`.
  `frame` cancels the suspended plan when its listener stops running.
- ToolSelector mode changed: preserve breaking session and anchored controller;
  mode enable/disable defaults are empty. The inactive listener's pending plan
  is cancelled through `frame`, without resetting the anchor.
- ConsiderInventory true to false **while in game**: call
  `InventoryActor::cancel_child(&mut module)`. Its direct parent is a `Mode`, so
  no parent Toggleable enabled short-circuit applies.
- World changed: clear executor and packet queue, and call
  `module.swaps.cancel_plan()` to release the abandoned suspended plan. Preserve
  the anchored controller's request/anchor fields. Do not call its `reset()`.

`config::auto_tool::AutoToolSetting::apply` mutates settings only; the runtime
wraps it with those lifecycle rules. Obtain the live block registry with
`client_auto_tool_block_registry`. The config child retains all setting groups,
hidden MillisecondDelays marker, SwapPreviousDelay alias and legacy tick-to-ms
migration.

## Verification and limits

`python native/liquidbounce-horizon/test-auto-tool-patch.py`: **35/35 pass**.
`cargo +stable clippy --manifest-path build/auto-tool-tests/Cargo.toml
--all-targets -- -D warnings`: pass, using the existing shared test target cache.

Evidence: `build/auto-tool-tests/result.json`, `test.log`, `oracle.json`,
`java-oracle.log`, `clippy.json`. The source templates and fixture APIs expose
which menu, network and cursor side effects are stubbed.

The tests compile the real hook generator, transformed local/remote mining HEAD,
GameHands forwarding, native snapshots, effective component conversion, actual
Horizon Items/BlockFacts/BlockRegistry, actual tick_hand/AttackStrength, config,
actor, collector and protocol packets. Java runs actual MC26.3 component getters,
menu-provider ownership and a real 46-slot InventoryMenu. The component getter
oracle explicitly binds fixture prototypes; it does not load normal default
item components. Actual menu/network/cursor side effects remain fixture stubs.

The executor implements AutoTool's single-chain part of InventoryManager and
does accumulate maximum close delay across those chains. Cross-module request
prioritization, carried-stack recovery, BetterInventory-controlled visibility,
and the original 5ms cursor coroutine remain outside this implementation.
Original InventoryManager also cancels redundant outgoing container-0 Close
packets; a read-only accepted callback cannot implement that cancellation.
Generated menu-provider decisions model normal block entities by block kind;
corrupt/missing block-entity data has not been compared in a live client.

**Full original-client versus Rust-client differential: not_run.**
