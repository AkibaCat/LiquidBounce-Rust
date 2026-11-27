# BetterInventory integration

This handoff uses LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, and Horizon-Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`. Its independent snapshots leave pristine upstream and shared staging untouched. The older shared-inventory freeze remains a historical handoff and is not rewritten here.

## Production files

- `native/liquidbounce-behaviors/src/better_inventory.rs`: all behavior decisions and defaults.
- `native/liquidbounce-behaviors/src/better_inventory_locale.rs`: generated decimal/zero-digit symbols for all 1,157 available JDK 25 locales; private child module of `better_inventory`.
- `native/liquidbounce-horizon/engine/src/config/better_inventory.rs`: complete File/Interop setting tree.
- `native/liquidbounce-horizon/engine/src/better_inventory_draw.rs`: pure JSON draw adapter preserving floating-point raw bits.
- `native/liquidbounce-horizon/engine/src/better_inventory_runtime.rs`: live module state and shared manager `last_clicked_slot` reader.
- `native/liquidbounce-horizon/better-inventory-host.rs`: real renderer, component/menu extraction, and system FORMAT locale bridge.
- `native/liquidbounce-horizon/better-inventory-locale-platform.rs`: safe public platform wrapper around the Windows locale API, composed into existing `horizon-system`. The client retains `forbid(unsafe_code)`.
- `native/liquidbounce-horizon/better-inventory-patch.py`: cumulative native transformations.

`generate-better-inventory-locales.py`, `better-inventory-locale-fixture.java`, and `better-inventory-locale-oracle.json` reproduce and verify the generated locale table. Regeneration requires the pinned JDK 25 and writes a numbered output snapshot before replacing the generated production table.

## Canonical wiring

1. Export `pub mod better_inventory;` from the behavior crate. Do not export `better_inventory_locale` separately.
2. Add `pub better_inventory: liquidbounce_behaviors::better_inventory::BetterInventory` to `BehaviorModules`, initialized with `Default::default()`.
3. Add `BetterInventory` to the normal registry/query list. Its category is `Render`; translation suffix is `betterInventory`.
4. Under `behavior-hooks`, add `mod better_inventory_draw;` and `mod better_inventory_runtime;` to the engine.
5. Forward the real generated event:

```rust
fn client_better_inventory_render(
    &self,
    _host: &dyn Host,
    event: &mut horizon_client::mod_api::ClientBetterInventoryRenderEvent<'_>,
) {
    self.better_inventory_render(event.query, event.draw);
}
```

The event fields are `query: &str` and `draw: &mut String`. The helper finishes the behavior borrow before returning the result; no Host call occurs while borrowing the registry/module state.

6. Replace the temporary `false` cursor gate in canonical `inventory_runtime.rs::inventory_snapshot` with the actual value from:

```rust
modules.better_inventory.should_move_cursor(
    modules.query_running(&state.registry, "BetterInventory"),
)
```

Compute it within the normal module-state borrow and pass it as `Snapshot::parse`'s third argument. The native visible-cursor action already belongs to the shared inventory executor. This is the final BetterInventory gate that the old inventory handoff intentionally left false.

7. Import and apply `better-inventory-patch.py::apply(edit, replace_once)` once after shared `inventory-manager` and append its `hook_names`/`helper_names` to the normal behavior fragment. Its only public event is `client_better_inventory_render`; there are no Host helpers. `test-better-inventory-native.py` shows the duplicate-application guard and complete cumulative patch composition.

## Config and actor wiring

Use the registration-only shim in `test-better-inventory-config.py` as the concrete config example:

- Add `mod better_inventory;` and export `BetterInventorySetting` / `serialize_better_inventory`.
- Add `ModuleKind::BetterInventory => ("BetterInventory", "Render", "betterInventory")`.
- Add `BetterInventory(BetterInventorySetting)` to `SettingChange` and `RuntimeRequest`.
- Dispatch `parse_better_inventory`, forward the parsed setting as the runtime request, serialize `modules.better_inventory`, and apply with `value.apply(&mut modules.better_inventory)` (returns unit).
- Include the new kind in canonical module-state/UI serialization and config actor dispatch.

All 14 leaves are implemented: HighlightClicked.Enabled, HighlightClicked.Mode.active and Border.Color, TextCooldownProgress.Enabled/Mode/Scale/Color, ContainerItemView.Enabled/SkipEmptyStack/Scale/RelativeToMouse/RenderOffsetX/RenderOffsetY, and MoveCursor. Keep inactive Border.Color, `CHOICE` versus `CHOOSE`, signed ARGB, independent leaf errors, raw Float values, and editor ranges without import-time clamping. The parser's numeric Color path intentionally reproduces Gson JsonTreeReader/LazilyParsedNumber truncation and low-bit overflow; it is verified against unchanged original ColorAdapter and the exact original Color4b.fromHex function.

## Native behavior

The patch inserts highlight at actual slot-item HEAD, including empty slots, creative display slots, and the creative trash slot. Menu index resolution uses the shared real menu mapping. Cooldown text runs after the cooldown overlay and before item count for every item decoration, including nested container previews. Actual cooldown `(endTick,currentTick)` is read without mutation, and text uses the current source pose origin when scaling.

Handled-screen preview runs after slots for inventory, creative, beacon, stations, and merchant. It uses carried-empty and actual hovered-slot gates, prototype-aware components, actual item codecs, sparse slot holes, nine-column 18-pixel cells, 16-pixel icons, and the independent list/text scales. Official 26.3 bytecode inspection in `build/better-inventory-component-research-01` confirms `ItemContainerContents.fromSlots` allocates through the largest slot index and that `ItemStackTemplate` rejects air and zero count (its JSON codec restricts count to 1..99). Thus valid sparse component entries preserve the same length, including leading and internal holes.

Fractional duration seconds use the system FORMAT locale, independent of the game's resource language. Windows reads and caches `GetUserDefaultLocaleName`, matching the initial JVM default in the tested environment (`zh-CN`). Only the `%.1f` branch localizes decimal symbols and zero digits; percentages, ticks, and whole-second concatenation stay ASCII as in the Kotlin source. Non-Windows environment locale fallback is implemented but has not been exercised on another operating system. Runtime Java locale overrides are outside the native initial-default bridge.

## Verification

- `build/better-inventory-source-05/report.json`: three byte-for-byte unchanged production Kotlin files, 1,033 command vectors, all settings/modes/order, raw floats and integer overflow, preview/filter/scale boundaries; an additional differential test checks all 1,157 JDK locales against actual `Formatter("%.1f")`. Tests and Clippy with warnings denied pass.
- `build/better-inventory-config-04/report.json`: complete production configuration snapshot, 69 tests and Clippy; 28 original ColorAdapter/fromHex JVM cases. Only absent module registration is supplied by recorded test-only wiring.
- `build/better-inventory-consumption-06/report.json`: five tests and Clippy over unchanged production native helper bodies, actual behavior decisions, and a recording HudBatch with the production vertex-scaling body. This covers menu/creative/trash mapping, real entry duration, nested pose/scale, component default/removal/holes, carried gate, and actual Windows locale compared with the JVM. The helper fixture retains `forbid(unsafe_code)` and uses the unchanged production locale source in a separate platform crate.
- `build/better-inventory-locales-03/report.json`: JDK 25 generates 1,157 decimal/zero-digit rows and Unicode-correct UTF-8 formatter samples.
- `build/better-inventory-native-04/report.json` runs real `fill_hooks` and a complete independent `horizon-client --lib` check, including the final safe-client/platform-locale split.
- `build/better-inventory-runtime-01/report.json` is historical production-engine compilation evidence against native02's generated ABI. It includes the real shared inventory actor/executor and lifecycle plus explicitly recorded absent-BetterInventory registration declarations, and predates the final locale implementation. It is not a final-current-engine check.
- `build/better-inventory-runtime-04/report.json` records a deliberately cancelled queued check, with `passed: false`. At the root's request, only this run's cargo process was stopped while waiting for the shared target lock. Its frozen baseline and current overlay hashes remain preserved; this report is not passing evidence. Runtime03's failure from combining the newer Timer engine with native04's earlier ABI also remains preserved.
- Root's production TestEngine68 will cover the complete canonical cumulative engine integration, including Timer and BetterInventory. Its result is pending at this freeze and is not claimed here.

The frozen runtime source also contains the root's subsequently integrated `webview` probe logging. The independent runtime snapshots predate that instrumentation; TestEngine68 covers the canonical source including those probes.

Final reports and production source hashes are listed in `better-inventory-freeze.json`.

The native compiler log retains an ordinary unused-code warning. The isolated Clippy claims apply to the explicitly listed source/config/consumption suites. Actual game execution, GPU output, canonical cumulative integration, and final smoke remain parent integration checks.
