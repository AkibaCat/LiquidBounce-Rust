# TargetLock owned handoff — 2026-11-26

Pinned LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon
`9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Horizon-Mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`.

Production owned files: `liquidbounce-behaviors/src/target_lock.rs`,
`engine/src/config/target_lock.rs`, `engine/src/target_lock_runtime.rs`,
`target-lock-host.rs`, `target-lock-patch.py`. Shared files and locale dictionaries
were not changed. AutoBreak and ItemScroller remain frozen.

## Root shared registration

1. Export `pub mod target_lock` in behaviors/lib. Add `TargetLock::default()` as
   `BehaviorModules.target_lock`. Register `TargetLock`, category `Misc`, base key
   `liquidbounce.module.targetLock`, no aliases. Source init order is immediately
   after ReportHelper and before AutoPearl; Misc precedes Movement/Player.
   No onEnabled/onDisabled/world handler clears its lock map.
2. Add `mod target_lock_runtime` under behavior-hooks in engine/lib. Add config
   `TargetLockSetting`, `serialize_target_lock`, ModuleKind::TargetLock and
   SettingChange/RuntimeRequest variants; dispatch parse_target_lock, apply the
   setting to `modules.target_lock` on the existing actor. Mode order is
   Temporary, Filter. Enum leaves use JDK equalsIgnoreCase; active mode names
   remain case-sensitive. All seven settings, ranges and suffixes are provided.
   The **entire module** has doNotIncludeAlways: Public export must omit it;
   File and Interop exports include all settings.
3. Add `target-lock-patch.py::apply` to aggregate after Hitbox (it extends the
   actual common `native_client_targeting_entity` JSON). Merge helper_names:
   `client_target_lock_player(i32)->String`, `client_target_lock_dead()->Option<bool>`.
   `native_client_target_lock_player` handles local, remote level-profile players
   including tab-list-removed NPCs, and same-dimension placed LAN guests. It does
   not infer AbstractClientPlayer from a generic player type alone. Distance uses
   raw IEEE f64 bits to retain infinity/NaN. The helpers only read real game state.
4. Extend `targeting::Snapshot` with `abstract_client_player:bool` decoded from
   native JSON (missing fixture field may default false). `player_name` is now
   the actual game-profile name, including the local player. Inside
   `State::target_info`'s **cache miss closure**, construct
   `target_lock::Player{id,name}` only when abstract_client_player and player_name
   are present, call `target_lock_runtime::apply_tag(&modules.target_lock,
   modules.query_running(&self.registry,"TargetLock"),player.as_ref(),&mut info)`
   after FriendManager's tag. Leave `is_friend` untouched. Do not clear TagCache
   on attacks, imports, mode changes or enable toggles. Existing FIRST_PRIORITY
   GameTick clear remains the only invalidation.
5. Call `self.target_lock_attack(host,event.target as i32)` at the Misc handler
   position in AttackEntityEvent, after earlier handlers and before later
   Movement/Player handlers. **Do this even if event.cancel is already true**:
   original EventManager continues dispatching cancelled attack events.
   This module changes tags and never directly cancels an attack.
6. Call `self.target_lock_tick(host)` at the Misc GameTick handler position,
   after FIRST_PRIORITY cache/wait processing and before later Movement/Player
   handlers. It queries `isDeadOrDying` only if Temporary handler is running.
7. Execute `update-target-lock-locales.py` on the integrated locale dictionaries.
   This work tested the updater on copies only. It provides all seven setting
   descriptions and Simplified/Traditional Chinese labels. Notifications use
   original English/SC/TC wording and INFO severity.

## Semantics retained

- First lock notification precedes clock read; MaximumTime is read after clock;
  refresh preserves insertion order and emits no second notification. Long sum
  wraps like JVM. Cancellation of AttackEntityEvent is ignored by this handler.
- Cleanup reads clock once even with an empty map; death clears without a clock
  read. Missing/non-client players disappear silently. Removed/range checks
  precede strict `expiry < now`. Notifications observe entries before removal.
- MaximumRange is a raw imported f32 converted to f64, then distance squared is
  compared with strict `< range*range`. No UI clamp is applied on imports.
- Usernames retains null, order, duplicates and Gson primitive-to-String
  conversion. Scalar becomes singleton. Array replacement is atomic; invalid
  entries fail only that leaf. Nullable names do not match. Case folding uses
  the existing actual JDK25 case table.
- Every native query, clock and notification executes outside state/module
  borrows. A running synchronous cleanup continues after notification callbacks.

## Focused evidence

`build/target-lock-source-01/report.json`: byte-identical TargetLock and ListValue,
actual fastutil linked map/Gson/JDK25; only System constant redirected to Clock.
Eight short scenarios, 72 steps, including cancelled attacks, notification-time
settings/clock mutation, insertion order, expiry equality, mode/enable retention,
death, null/primitive/atomic list imports, Unicode I and signed long wrap.

`build/target-lock-tests-01/report.json`: one focused Cargo compile, **10 tests
passed**, no warnings. Includes per-step original oracle, exact config/runtime/
native helper bodies, cache/classification integration, real profile identity,
distance and guards; explicit State/Host/Game scalar registration providers.
Owned patch is applied to actual Hitbox helper copy; locale updater only copies.
No large oracle expansion, Clippy sweep or full native build was run.

Complete engine registration, concrete Game compilation, cumulative package and
actual game validation remain root-owned final checks, not claimed by fixtures.
