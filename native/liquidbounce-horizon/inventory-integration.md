# Shared InventoryManager and Vomit integration

This handoff is based on LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, and Horizon-Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`. No pristine upstream or shared staging file was changed by this work.

## Files to integrate

- Behavior modules: `native/liquidbounce-behaviors/src/inventory_manager.rs`, `vomit.rs`.
- Shared engine actor and native executor: `native/liquidbounce-horizon/engine/src/inventory.rs`, `inventory_runtime.rs`, `inventory_packets.rs`.
- Config codecs: `engine/src/config/vomit.rs`, `engine/src/config/inventory_manager.rs`.
- Native cumulative patch: `inventory-manager-patch.py`, `inventory-manager-host.rs`, `vomit-block-registry.rs`. The registry has its own reproducible exporter, `export-vomit-blocks.py`.

The existing `auto_tool.rs` behavior/snapshot adapter is still used. Its old `InventoryActor` is superseded by `inventory::InventoryActor`. The old `auto_tool_runtime.rs` and `auto_tool_packets.rs` must no longer compile in parallel with the replacements because their methods/types overlap.

## Engine registration

1. Export `pub mod inventory_manager;` and `pub mod vomit;` from the behavior crate.
2. Under `behavior-hooks`, add `mod inventory;` in the engine. The smallest compatible mapping is `#[path="inventory_packets.rs"] mod auto_tool_packets;` and `#[path="inventory_runtime.rs"] mod auto_tool_runtime;`. This preserves existing references in the canonical runtime and packet service. Alternatively rename all those references consistently.
3. Change `BehaviorModules.inventory` to `crate::inventory::InventoryActor`; add `pub vomit: liquidbounce_behaviors::vomit::Vomit`, initialized with `Default::default()`.
4. Add `Vomit` to the registry/query ID list. Its real category is `Fun`, translation suffix is `vomit`, and all common module values are handled through the usual config path.
5. Add `WaitOwner::InventoryManager` and dispatch it to `self.inventory_callback(host, in_game, task.callback)`. The existing global FIFO coroutine ticker remains the only tick-wait scheduler. The callback returns `false` and registers its own next wait when needed.
6. Implement native `client_inventory_wake(&self, host: &mut dyn Host, _: &mut ClientInventoryWakeEvent)` by calling `self.inventory_frame(host)`. Existing `client_auto_tool_frame` continues calling `auto_tool_frame`, which now delegates to the same shared executor.
7. Existing tick invocation `auto_tool_tick(host)` now needs a mutable Host. It submits both AutoTool and Vomit before draining the one shared manager. Remove no existing AutoTool progress observer calls.
8. Existing world-change cleanup is compatible: reset actor, cancel AutoTool swap plan, clear packet service; `reset_inventory()` is provided too. A disabled AutoTool inventory child calls `inventory.cancel_child(&mut auto_tool)`. A disabled Vomit calls `inventory.cancel_vomit()`. These withdraw pending work; an already active atomic chain is allowed to finish, as in Kotlin.

Every Host call is outside the module registry/behavior borrow. Keep that boundary when wiring lifecycle or config code because ordinary packet listeners can synchronously reenter the module state.

## Vomit config

The compile-test registration shims are recorded in `test-vomit-config.py` and are concrete examples for the canonical config changes:

- Add `mod vomit`, export `VomitSetting` and `serialize_vomit`.
- Add `ModuleKind::Vomit => ("Vomit", "Fun", "vomit")`.
- Add `Vomit(VomitSetting)` to `SettingChange` and `RuntimeRequest`.
- Dispatch parsing to `vomit::parse_vomit` and apply forwarding to `RuntimeRequest::Vomit(value.clone())`.
- Serialize actual runtime `modules.vomit` using `serialize_vomit`; apply runtime requests using `value.apply(&mut modules.vomit)`, handling its returned `ConfigError` through the existing path.

The entire `Constraints` tree is included: StartDelay, ClickDelay, CloseDelay, MissChance, SlotMultiplier, file-only MillisecondDelays and Requires. Keep the codec's raw Gson-compatible values and old tick-to-ms migration. Editor ranges are metadata and do not clamp imported values. In particular `Requires` mutation can partially apply before a later malformed entry errors, matching the upstream mutable EnumSet behavior.

## Global Inventory config

The real Settings child-group is named `Inventory`, not `InventoryManager`.

Export these symbols from `config.rs`:

```rust
mod inventory_manager;
pub use inventory_manager::{
    GlobalInventoryPatch, GlobalInventorySetting,
    parse_global_inventory, serialize_global_inventory,
};
```

`serialize_global_inventory(&modules.inventory.manager, format, describe)` returns the **Inventory child-group** to append to the Settings root. Its real defaults are ResponseTime 400 ms (editor range 50..2000), LockAfterRejections 3 (editor range 1..10).

`parse_global_inventory(settings_root)` returns `Result<GlobalInventoryPatch, ConfigError>`; process `patch.issues` individually and apply each `patch.changes` via `setting.apply(&mut modules.inventory.manager)`. This apply returns unit. `setting.name()` supplies the leaf for the normal valueChanged event. Exclude `Inventory` from preserved opaque Settings groups after implementing the live group so that imports/exports do not duplicate it. Include this path in GET, POST, settings-file load/save, and Interop metadata handling. Existing Targets remains independent.

## Packet pipeline and native patch

Use the new `InventoryPackets` service. Its ordinary outgoing method cancels redundant original container-0 closes while the server-side inventory is closed. Register this listener at the source's exact `READ_FINAL_STATE = -1000` priority, after ordinary mutators/cancellers (`utils/kotlin/Priority.kt:83`). Accepted original packets are recorded only from `outgoing_accepted`, which the packet pipeline already invokes after all ordinary handlers; silent delivery remains excluded. The service uses atomic flags and a queued session-tagged observation list and never borrows the module registry or calls Host.

`inventory-manager-patch.py` exports `apply(edit, replace_once)` for the normal cumulative patch generator; it returns `hook_names` and `helper_names`. Apply it once after the existing behavior patch transformations, then append its reported helpers/hook to the behavior fragment before fill_hooks generation. `test-inventory-manager-native.py` demonstrates the exact cumulative order, duplicate-application guard and independent workspace generation. Do not manually apply it again to already patched staging.

The native layer provides snapshots with all 41 player inventory stacks, actual menu state and carried stack, stable object serials for real screen replacements, real geometry, exact item/components payloads and a native monotonic time. Normal click operations use Horizon's verified menu predictor and sender. Creative adds use the official send-only semantics; creative throws use the real dropSpam counter/feature gates and separate RNG sources. Empty-hand targetless UseItem still sends in remote play. Silent-slot cancellation does not suppress the subsequent UseItem call, matching the source. Offhand UseItem invokes no native silent-selection operation: the real helper returns early for its null hotbar index. The source fixture's `select:40:1` record describes the helper invocation, rather than a claimed native slot-change side effect.

Native accepted incoming observations include all full menu-content packets and only container-0 slot updates. Existing AutoTool observe callback names remain valid and now feed the shared actor. The native `client_inventory_schedule` deadline integrates with winit idle/control-flow and wakes the executor between rendered frames; it is required for millisecond timing.

BetterInventory's cursor-motion option remains false until that module is implemented and wired. The generic visible-cursor path is tested against the source, but this integration does not claim a BetterInventory implementation.

## Frozen verification evidence

- `build/vomit-source-03/report.json`: unchanged real ModuleVomit and MC LegacyRandomSource/Kotlin RNG, 2,580 vectors.
- `build/vomit-config-03/report.json`: 55 config tests and Clippy with warnings denied.
- `build/vomit-registry-02/report.json`: real official MC bootstrap, 1,286 blocks, 78 air-item entries, 1,658 item-feature rows.
- `build/inventory-manager-source-05/report.json` and `source-vectors.json`: six unchanged Kotlin production files, real coroutines/Duration and deterministic MonotonicTimeSource, 479 command observations. The fixture README describes synthetic MC boundaries and the exact two Methodref owner replacements used for its clock.
- `build/inventory-manager-13/report.json`: actual production Rust manager, 14 tests plus Clippy, matching all source05 commands. Includes active atomic chains after owner stop, synchronous cancellation tail order, priorities/FIFO, requirements, live delays, tick waits, resync/rejection locks, expected-menu guards, nearest-slot Float overflow, visible cursor and creative action gates.
- `build/inventory-actor-03/report.json`: four tests plus Clippy, covering fresh-source guards on completed AutoTool tails, pending cross-owner invalidation, and the complete 72-action zero-delay Vomit chain after its owner stops.
- `build/inventory-config-01/report.json`: 59 production config tests and Clippy, including actual manager use of configured timers.
- `build/inventory-manager-native-05/report.json`: real fill_hooks/generator plus complete independent `horizon-client` library check, including latest frozen FastBreak and Elytra interfaces.
- `build/inventory-runtime-03/report.json`: passed actual production shared actor/executor compilation against native05 in 282.391 seconds, with explicit temporary integration shims captured in `integration-shims.json`. This is a full API compile, not a runtime Clippy claim; non-webview unused-code warnings from the copied shared engine remain visible in its compiler log.

These reports establish source behavior and native/engine compilation. Canonical full-engine integration, real ClickGUI configuration, actual game clicks/drops/timers and multiplayer behavior still require the parent's integration/build/game smoke before marking the module complete.
