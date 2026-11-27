# ItemScroller integration

Pinned sources: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Horizon-Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.

## Canonical registration

1. Export `pub mod item_scroller;` from behaviors. Add `pub item_scroller: liquidbounce_behaviors::item_scroller::ItemScroller` to `BehaviorModules`, initialized with `Default::default()`.
2. Register `ItemScroller` in category `Misc`, translation suffix `itemScroller`. Preserve common settings and the two module settings: ClickMode (only QuickMove, default QuickMove) and Delay (default 2..3, editor range 0..20, ticks).
3. Include/export `config::item_scroller::{ItemScrollerSetting, serialize_item_scroller}`. Add the setting change and runtime request variants, parse dispatch, actor application and serialization dispatch using the existing module pattern. Imported delay endpoints remain raw; setting changes do not reset the chronometer.
4. Under `behavior-hooks`, include `mod item_scroller_runtime;` and forward the actual generated events. Primitive-only events have no lifetime parameter:

```rust
fn client_item_scroller_query(
    &self,
    _host: &dyn Host,
    event: &mut horizon_client::mod_api::ClientItemScrollerQueryEvent,
) {
    self.item_scroller_query(event.shift, event.left_pressed, &mut event.allowed);
}
fn client_item_scroller_reset(
    &self,
    _host: &dyn Host,
    event: &mut horizon_client::mod_api::ClientItemScrollerResetEvent,
) {
    self.item_scroller_reset(event.now);
}
```

5. Compose `item-scroller-patch.py` once after the shared inventory-manager, BetterInventory and AutoBreak transformations. Append its returned hook names; it adds no Host helpers. The patch inserts exactly five render seams and one global mouseClicked HEAD seam.

## Behavior preserved

The original physical Shift/left-button guards short circuit before a new delay draw and wall-clock read. Each eligible query samples the inclusive delay again, including canceled clicks. The chronometer starts at zero and survives setting changes and enable/disable. The calculation retains Java signed-long wrapping and uses wall-clock milliseconds.

The render handler checks carried-empty and hovered-slot before querying. After BetterInventory's preview, it copies the actual hovered stack, calls QuickMove with official MC26.3 SDL left-button value 1, sets skipNextRelease without clearing quick crafting, updates lastClickSlot and click time/button, then resets the chronometer. Mouse cancellation precedes bound hotbar/pick handling, applies to every pressed mouse button while left is held, permits a nonempty carried stack, and does not reset the timer or click state.

Creative display extraction includes the current first-row offset. Creative trash QUICK_MOVE clears all 46 InventoryMenu slots and sends all 46 CreativeModeItemAdd equivalents, including already empty slots; it retains carried state. The native remote cache is synchronized once after sending to avoid duplicate changes.

## Verification

- `build/item-scroller-source-01/report.json`: byte-for-byte original ModuleItemScroller.kt and complete Chronometer.kt; 72 focused JVM/Rust vectors plus three Rust tests and Clippy. Actual JDK25 wall-clock and ThreadLocalRandom are used. The per-vector seed and following random draw verify short-circuit sampling order. Exact boundary/wrapping is tested separately from broad real-time bounds.
- `build/item-scroller-config-02/report.json`: unchanged complete production config and behavior snapshot with declaration-only absent registration shims; 74 tests (four ItemScroller tests) and Clippy. Covers File/Interop trees, Java Unicode choice lookup, raw ranges and integer conversion, independent leaf errors, and no timer reset.
- `build/item-scroller-consumption-02/report.json`: unchanged final Game helper in an explicit recording Game/menu/network fixture, complete transformed production container_screen.rs, and actual ItemScroller decisions; 18 tests (six focused helper tests) and Clippy. Covers the pre-click copy and reset order, callback button 1, physical input, carried/hover guards, all-button cancellation, quick-craft preservation, creative display and all 46 trash packets. Source assertions cover all five render seams and the global HEAD before hotbar bindings.
- `build/item-scroller-native-02/report.json`: real fill_hooks plus full independent horizon-client --lib check, including the final all-button cancellation seam; both commands exit 0. Validates concrete Game types, native helper bodies and generated ABI. Canonical engine registration and actual game/GPU checks remain root integration checks.

The failed config01 (test variable shadowing) and consumption01 (fixture crate alias) snapshots remain preserved. Native01 passed before the final cancellation correction and is historical evidence only. Exact production/support/report hashes are in item-scroller-freeze.json. Independent verification did not modify shared staging or pristine upstream.
