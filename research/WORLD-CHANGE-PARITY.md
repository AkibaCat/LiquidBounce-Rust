# Client world changes

`MixinMinecraft.java` at LiquidBounce `ae0eb396` injects `WorldChangeEvent(level)` at HEAD of `Minecraft.updateLevelInEngines(ClientLevel, boolean)`. This includes a non-null level on login, dimension change or same-key new level; a kept same-level respawn does not call `setLevel` again. It is independent of a player existing and of modules running.

The local pinned Mojang 26.3 client jar (`build/resource-runtime/.local/minecraft/26.3/jars/client.jar`, SHA-1 `e877b6a07acd633fb3bb475002175cec036e7b87`) was inspected with JDK 25 `javap -p -c`. The output is retained in `build/navigation-core-03/Minecraft-bytecode.txt`. `Minecraft.setLevel` assigns `level` at bytecode 2 and calls the one-argument `updateLevelInEngines` at 7. That delegates to the two-argument method, whose first operation calls `soundManager.stop`; the injected event runs before that sound, camera, particle and renderer work. `disconnect` invokes the two-argument null update at bytecode 197 and only then clears `player` at 202. `clearClientLevel` invokes the null update at 78 and clears `player` at 83.

`NativeClientState.world` cannot identify this event: `NativeWorldId` contains only a save folder or server address. Dimension changes and a replacement level on the same connection keep that value. Watching `WorldState.in_game()` misses entry and replacement and can duplicate disconnection handling.

The owned `timer-patch.py` adds one generated listening hook:

```rust
fn client_world_change(
    &self,
    host: &dyn Host,
    event: &mut ClientWorldChangeEvent<'_>,
)
```

`event.dimension: Option<&str>` is the new dimension key or `None` for the null update. The native method permits only `native_window()` and no LAN actor context. `native_window` uses the immutable `ExecutionRole::Client`, so neither the integrated server's thread nor a dedicated server emits the event.

The actual non-null consumer is inside `enter_remote_dimension`'s `!same` branch, immediately before `stop_sounds_for_new_level`. Its unchanged predicate is `keep_world && heard_level == spawn.dimension`. Login passes `keep_world=false`, so a new login with the same dimension still emits. Same-level respawn with a kept level does not emit. `leave_server` emits the null update before the equivalent engine sound cleanup, after the early branch that starts a deferred single-player save. `finish_close_world` now delegates its remote branch before that cleanup and lets `leave_server` emit exactly once; its non-remote branch invokes the gated null hook before cleanup.

Horizon does not implement `Minecraft.setLevel` as one method. Its existing `client_level_load`/`client_level_unload` callbacks are later in the replacement branch, after streaming/world/entity/renderer mutations. The added event runs before those callbacks to preserve the source HEAD position relative to engine work. It does not claim identical ordering with those separate Horizon/NeoForge callbacks. At the new-level hook the payload supplies the destination key; querying the host can still see the previous world backing storage. The runtime Timer consumer only uses the client-window check and resets shared Timer fields. Future navigation consumers must use the event as an invalidation boundary rather than take a new world snapshot from this callback.

The integration owner should bind this event to `timer.world_change()` and remove both the boolean `observe_world` Timer fallback and the disconnection Timer fallback. Other world cleanup is outside this patch. Source `Timer.world_change()` clears only the original fields; it preserves speed requests and `real_ticks`.

Independent evidence is `build/world-change-04/report.json`: five tests execute the exact extracted same-level predicate, non-null/null event consumers, save-defer branch, close delegation, engine sound consumers, native method, real `ExecutionRole` and generated native API. They cover login, changed dimension, same-key reload, kept respawn, deferred save, exactly one null update, both server roles and LAN actor exclusion, an independent server thread, and the Timer reset fields. Surrounding game/session/audio objects are shells; complete lifecycle methods and in-game behavior still require aggregate build/smoke evidence.
