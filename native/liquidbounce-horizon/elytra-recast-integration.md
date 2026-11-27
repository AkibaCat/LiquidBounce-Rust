# ElytraRecast integration handoff

Owned implementation targets the complete pinned ElytraRecast module, its LivingEntity mixin and its LocalPlayerBody query. Root owns shared registration, production build and game checks. Full original/native client differential remains `not_run`; this does not complete the 236-module objective.

## Shared registration

Register **ElytraRecast**, category **Movement**, no aliases and no custom settings. Existing `zh_cn`/`zh_tw` module descriptions use `liquidbounce.module.elytraRecast.description`.

- Add `ModuleKind::ElytraRecast` with key `liquidbounce.module.elytraRecast` and the normal query-running registration. No BehaviorModules-owned policy state is needed: the history belongs to each PlayerMotion body.
- Add/export `engine/src/config/elytra_recast.rs::serialize_elytra_recast` and use generic common setting parsing/application only. File has Enabled/Bind/Hidden; Interop has Bind/Hidden plus descriptions.
- Add `elytra-recast-patch.py` after mining identity transforms. Add hook `client_elytra_recast_running` and helper `client_recasts_elytra` to the catalog. The patch copies its own policy/chest/adapter into Horizon client_lib, so a behavior-crate export is optional.
- No disable/world callback, tick coroutine or state-reset registration.

Runtime hook:

```rust
fn client_elytra_recast_running(&self, _host: &dyn Host,
    event: &mut ClientElytraRecastRunningEvent) {
    let state = self.state();
    event.running = state.behaviors.borrow_mut()
        .query_running(&state.registry, "ElytraRecast");
}
```

Generated event `running` is a plain bool. Do not add a screen guard. The helper returns only running, matching `LocalPlayerBody.recastsElytra`, for future navigation consumers.

## Complete behavior

Original files at ae0eb396: `ModuleElytraRecast.kt`, `MixinLivingEntity.java::recastIfLanded`, `LocalPlayerBody.kt::recastsElytra`.

At actual local LivingEntity.aiStep TAIL, sample fallFlying. If module running and previousElytra and current sample false, stop `minecraft:item.elytra.flying` in PLAYERS (index 7), call recast, then reset noJumpDelay to zero even if recast was rejected. Finally record the pre-recast sample.

Recast reads CHEST first, then short-circuits flying, passenger, climbable, water, levitation, already fallFlying, GLIDER, nextDamageWillBreak and physical-jump OR input-jump. It does not exclude onGround, lava, dead/sneaking players, or gliders without EQUIPPABLE. It sets local fallFlying before ordinary PlayerCommand `{id: own entity id, action:6, data:0}`. Packet cancellation does not roll back the flag or cooldown. Public `ElytraRecast::recast_elytra` itself has no running guard.

`PlayerMotion.client_elytra_recast.previous_elytra` defaults false on a new body and is cloned with that body. Disabled tails continue updating it. Toggling or world changes do not clear history; replacement bodies start false. Server copies, guest actors and speculative replay do not execute the local callback.

## Native seam and timing

The patch preserves all old MotionWorld entry points with a no-op callback and adds separate `_with_ai_tail` methods through dry_tick, player_movement_tick_with_fluids, simulation_tick_with_fluids and local_tick. Real remote controlled movement selects the tail-aware entry. Passenger and sleep/dead branches invoke exactly one corresponding tail.

The callback executes after aiStep travel/block/freezing/spin logic and before fallFlyTicks and subsequent tick bookkeeping. The ordinary remote `started_gliding` detector suppresses its duplicate packet after recast. This synchronous seam is necessary because a deferred MotionEvent queue would change flag/tick ordering.

After `sync_player_body`, running, physical input, raw input and immutable chest components are captured before disjoint mutable player/audio borrows. Current environment, auto-jump and player flags are queried at the real tail. Audio stop and remote Session.send execute synchronously there. PlayerCommand has no carried-item sync branch, so the ordinary Session path is the actual packet-listener/codec boundary for this command.

CHEST resolves actual `ItemModels.components_view` over the actual `item_render::stack_view`, including patches/removals. Damageability requires MAX_DAMAGE and DAMAGE and excludes UNBREAKABLE; getDamageValue clamps before the Java Int threshold comparison. Count-zero and air stacks are empty.

## Verification

```powershell
python native/liquidbounce-horizon/collect-elytra-recast-facts.py
python native/liquidbounce-horizon/test-elytra-recast.py
```

**12/12 tests and Clippy `--all-targets -- -D warnings` passed.** The final full production composition contains **272 transformed Rust sources** and passes rustfmt syntax validation; this is not a full native typecheck. The test driver formats composition sources in batches of 48 to stay within the Windows command-line limit; its final SHA256 is `94607c616c8edb8998f6baf9f743ee89dcf2c3296c675319bf22a3d75bcb1abc`.

The MC collector executes actual pinned MC26.3 ItemStack methods over **362** stack variants. It binds the four relevant prototype components from prepared `items.json`; unrelated components are unbound. It then executes the **unchanged original ModuleElytraRecast.kt**, exact extracted original Java `recastIfLanded` method body and exact original `DataComponentHolder.isGlider` property for **8192 tail** and **2048 direct public recast** cases. The Kotlin copy is byte-for-byte checked and hashed; the extracted method/property and originals are hashed. No hand-translated policy oracle remains.

The original-policy harness uses a constructor-free actual LocalPlayer subclass with recording environment/equipment/abilities providers, actual MC ItemStack/Input/PlayerCommand and sound identifiers, plus module-running and Minecraft/input/sound recording shells. Synthetic policy flags use controlled empty stick/air prototypes with actual component patch methods. Rust compares every recorded ordered read/effect and final flag, cooldown and history. The real `Input.jump` record accessor and primitive `noJumpDelay` assignment are not intercepted; their resulting behavior/cooldown are checked independently. This is original source execution within dependency shells, not a complete original client.

The native fixture compiles the actual generated Host/dispatch, exact native query/helper, actual chest adapter and actual MC prepared item data, transformed full dry_tick body, exact capture callback, full composed packet_bridge and actual transformed Session.send/send_silently/send_with_path with Pipe and packet codec. The capture driver lives in a sibling remote module with minimal explicit imports and no `item_render` import, matching the real remote scope and requiring the qualified `horizon_client::item_render::stack_view` path. Storage, world providers and the outer controlled-tick driver are shells. The slot bridge is a dependency stub unused by PlayerCommand. Full registry/actor execution and original/native full-client differential remain `not_run`.

Evidence lives under `build/elytra-recast-tests`: oracle/source hashes, Java logs, test/Clippy logs, result.json, isolated complete composition and syntax log. No upstream or staging files are modified by this verification.
