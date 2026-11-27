# AutoBreak integration

Pinned sources: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Horizon-Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`. Independent snapshots preserve pristine upstream and shared staging.

## Production files and canonical registration

- `native/liquidbounce-behaviors/src/auto_break.rs`: exact original guards and pressed/click result. `AutoBreak` is a unit struct with no module-specific settings.
- `native/liquidbounce-horizon/engine/src/auto_break_runtime.rs`: reads actual running state and forwards the key-query result.
- `native/liquidbounce-horizon/auto-break-host.rs`: actual Game hit/air lookup, current key binding, native input queues, and all existing vanilla consumeClick phases.
- `native/liquidbounce-horizon/keymapping-clicks.rs`: mapping-identity click queues with Java signed overflow. A click never changes held or toggle state.
- `native/liquidbounce-horizon/auto-break-patch.py`: cumulative transformations; compose once after MultiActions, SmartEat, shared inventory-manager, and BetterInventory.

Root wiring:

1. Export `pub mod auto_break;` from behaviors. Add `pub auto_break: liquidbounce_behaviors::auto_break::AutoBreak` to `BehaviorModules`, initialized as `AutoBreak`.
2. Register `AutoBreak` in category `Player`, translation suffix `autoBreak`, with the ordinary common settings and an empty module-specific setting list.
3. Under `behavior-hooks`, include `mod auto_break_runtime;` in the engine.
4. Forward the real generated event:

```rust
fn client_auto_break_key(
    &self,
    _host: &dyn Host,
    event: &mut horizon_client::mod_api::ClientAutoBreakKeyEvent,
) {
    self.auto_break_key(
        event.attack, event.miss_time, event.block_hit,
        event.block_state_available, event.air, event.destroying,
        &mut event.pressed, &mut event.click,
    );
}
```

5. Apply the owned patch once after its dependencies and append its returned hook list to the normal behavior fragment. Its only generated callback is `client_auto_break_key`; it adds no Host helpers. The independent native verifier demonstrates marker-based duplicate-application protection.

## Native semantics

The original handler responds only to the attack KeyMapping, missTime <= 0, a real block hit with an available non-air state, and module running. It forces pressed=true and requests a click whenever destruction is inactive, including when the original physical pressed value was already true.

Official MC 26.3 bytecode in `build/auto-break-research-01` shows that attack/use/pick consumeClick loops run before the late attack.isDown query. Thus a synthetic first click queues for the next tick. The late query retains screen and instant short circuiting; mouse grab is checked after isDown, so an ungrabbed query can still queue a click before continueAttack receives false. Horizon's agent/control virtual pointer follows the same existing input eligibility as physical input.

The remote InteractionInput seam and local hand seam use their actual missTime/destruction values. The local path passes the mem-taken `Clicks` state directly, avoiding the temporarily default `self.clicks`. Host mapping reads also use the original isDown callback semantics.

KeyMapping.click resolves the current attack binding and increments every mapping sharing that key. Already queued clicks retain the mapping identity through rebinding and module disable. Unconsumed movement/screenshot/debug mappings retain their counts without a physical press. The unknown-key exclusion reproduces pinned LiquidBounce MixinKeyMapping; vanilla official KeyMapping alone lacks this exclusion.

The prefix consumes mappings in official handleKeybinds order: perspective, smooth camera, GUI, advancements, social, chat, command, spectator shader toggle, hotbar, inventory, quick actions, offhand, drop, attack/use/pick, then the eligible spectator hotbar phase. Actual existing screen/menu/actions handle their side effects. Ordinary screen openings, pointer release, player replacement, and world close clear queued clicks. Screen assignment wrappers evaluate the complete right-hand expression before checking the resulting screen, and compose after inventory lifecycle wrappers. The patch exposes only the remote fields needed by this Game sibling adapter as `pub(super)`.

## Verification

- `build/auto-break-source-03/report.json`: byte-for-byte original ModuleAutoBreak.kt; 960 Cartesian behavior vectors and 52 actual official MC26.3 KeyMapping lifecycle operations, four tests and Clippy with warnings denied. Real KeyMapping, HitResult, BlockHitResult, BlockPos, and Direction are used; minimal event/module/environment fixtures supply the surrounding application. The subclass only exposes the protected key field already widened by LiquidBounce. MC26.3 SDL mouse-left code 1 is used. Kotlin handler source remains unchanged.
- `build/auto-break-consumption-03/report.json`: 19 tests and Clippy with warnings denied. Complete transformed production InteractionInput, complete Hand/HandWorld/Hit/UseResult/Destroying/Clicks declarations and methods, unchanged real production KeyBinds and ClickQueues. Covers next-tick queue consumption, held=true first click, disable/rebind identity, GUI/instant short circuit, mouse-grab ordering, item-use draining, unknown key, and mem-taken local state.
- `build/auto-break-native-02/report.json`: real fill_hooks and full independent horizon-client --lib check passed, including final pointer-release clearing and Remote input visibility. This validates Game helper bodies, lifecycle wrappers, local/remote seams, and generated ABI; canonical engine wiring and actual game/GPU checks belong to root integration. Exact report and production hashes are recorded in `auto-break-freeze.json`.

Failed source02 (SDL test initial binding), consumption01 (fixture dependency), consumption02 (test initializer Clippy), and native01 (Remote.input visibility) are preserved as failed evidence. They are not passing reports. No shared production staging was modified by these independent verification runs.
