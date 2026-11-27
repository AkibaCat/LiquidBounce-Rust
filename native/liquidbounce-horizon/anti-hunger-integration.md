# AntiHunger integration handoff

Pinned original: LiquidBounce `ae0eb396`, untouched `ModuleAntiHunger.kt` and `Chronometer.kt`. This is one complete module policy and its native packet scope for the 236-module objective; full original/native client differential remains **not_run**. Shared runtime registration and actual game execution remain root integration work.

## Shared registration

- Export `liquidbounce_behaviors::anti_hunger`; declare engine modules `anti_hunger_packets` and `anti_hunger_notifications`.
- Store one persistent `Arc<AntiHungerService>` in `BehaviorModules.anti_hunger`. Add it to the shared `PacketPipeline` at priority **0**, preserving registration order among equal-priority original module handlers. Ordinary outgoing events run this handler even if an earlier handler cancelled the packet. Silent events bypass it.
- Register `AntiHunger`, category `Exploit`, key `liquidbounce.module.antiHunger`; declare config module `anti_hunger`, exposing `AntiHungerSetting`, `serialize_anti_hunger`, `parse_anti_hunger`, and `SettingChange::AntiHunger`.
- Copy current actual module-running state to `AntiHungerService::set_running` in the same registry synchronization used by packet modules. The packet callback never takes a registry lock.
- Serialize the service's `behavior` Mutex guard; apply each setting through the same guard. Settings edits, module disable/enable and world changes must retain `last_notification`, initially epoch zero.
- At both actual INPUT and NETWORK SprintEvent phases, run the existing Sprint policy first (critical priority 500), then `anti_hunger.sprint(swimming, Source::Input/Network, value)` (default priority 0). The native helper `client_anti_hunger_swimming` supplies the actual client-player state. Do not apply AntiHunger at MOVEMENT_TICK.
- Compose `anti-hunger-patch.py` after slot, weapon, Sprint and HighJump transforms; return helper `client_anti_hunger_swimming`. It declares no extra hook.

## Complete configuration

Declaration order is `NoSprint` toggle group (`Enabled` true, `WhileSwimming` false), then `KeepFloating` true. NoSprint is inactive while swimming unless WhileSwimming is enabled. WhileSwimming is retained and serialized when the group is inactive. A malformed NoSprint group records its own issue while allowing the later independent KeepFloating leaf to import.

`update-anti-hunger-locales.py` merges all setting descriptions into simplified/traditional display resources and English fallback descriptions. It verifies all six original warnSprint/warnBreak translations against pinned upstream without rewriting their text. Generic translated setting labels already exist. Resources are changed only by that owned script under coordinated root ownership.

## Safe synchronous packet scope

Original `Game::send(&self)` cannot safely expose mutable player state to packet listeners. The transform changes it to `&mut self`, splits the actual Session, PlayerMotion, World and resource borrows, and creates a temporary `AntiHungerPacketPlayer` around the real `MotionWorld`. `PacketEvent.client_player` borrows this scoped object. Session recursive ordinary sends retain the same scope; no Game pointer, unsafe cast, TLS raw pointer, or delayed actor jump is used.

Every ordinary Game-origin send uses this scope, including carried-slot and spear sends that formerly bypassed Game::send. The owner check requires native window, no LAN actor, client_side player and native_client_queries_allowed. Nonowner sessions retain the original send path and expose no local player. Direct Session-only traffic without a player scope is explicitly skipped by AntiHunger; network keepalive/custom transport traffic is outside its packet cases. This boundary must not be described as arbitrary background sends gaining a local player scope.

The transform also changes five callers to mutable receivers: `native_reject_slot_packet`, generated-helper target `native_client_ensure_carried`, `send_perf`, `send_book_settings`, and `remote_station_button`. Actual fill_hooks and complete horizon-client library typechecking passed with these signatures.

START_SPRINTING has MC26.3 wire action **1**. When NoSprint is running and the shared strict cooldown elapsed, this handler queues the original INFO notification, reads the clock again to reset the Chronometer, drops every service lock, then calls the actual MotionWorld.jump_from_ground **before the next packet listener**. This immediately consumes HighJump's synchronous jump bridge as well. START_DESTROY_BLOCK is wire action **0**: it queues warnBreak and resets the same cooldown, without jumping and independently of NoSprint/KeepFloating.

Chronometer's comparison is `last.wrapping_add(500) < now`, including the original short-circuit clock evaluation order and signed Long overflow. This is a strict boundary; exactly 500ms does not trigger. It is not a tick counter.

KeepFloating handles all four real protocol variants (`MovePlayerPos`, `MovePlayerPosRot`, `MovePlayerRot`, `MovePlayerStatus`), clearing only onGround bit 0. It requires not passenger, not destroying, not in water, not swimming, not underwater, packet onGround, and fallDistance<=0. Other flags and packet fields stay intact. No packets are cancelled by this module.

## Original notification channel

`NotificationQueue::publish` captures the game language **at emission** with the warning key. Connect `BehaviorModules.anti_hunger.notifications.drain` to root's normal WebSocket event-consumption path, extending the same pending events vector before it is returned/published. Resolve each row using `liquidbounce_interop::localization::describe(language,key)` with its stored language, rather than the current language at polling time.

Each drained event is exactly `("notification", {"title":"AntiHunger","message":<original translated text>,"severity":"INFO"})`. Drain takes the vector out under its Mutex, then localizes outside the lock. A module toggle or world change must neither discard nor retranslate previously emitted notifications.

## Evidence

`collect-anti-hunger-original.py` copies and byte-verifies the untouched Kotlin module and actual Chronometer source. Only the compiled Chronometer System class constant is redirected to an explicit clock provider; original method bytecode remains otherwise intact. **2280 scenarios** compare each step's cooldown value, exact clock-call/notification/jump trace, sprint result and movement onGround result. This includes 32 fluid/passenger/destroying combinations, signed zero/negative/positive/NaN/infinite fall distance, all source/settings/running cases, already-cancelled packets, 499/500/501 boundaries, shared sprint/break cooldown, disable/re-enable and Long overflow. Packet storage, lifecycle registration, notification recording and clock input are explicit dependency providers; this is not full original-client execution.

`test-anti-hunger.py`: **8/8 total tests pass**, including five AntiHunger tests (original oracle, notification language/shape, two config tests, actual packet pipeline) plus three reused real jump support tests. Clippy `--all-targets -- -D warnings` passes. The actual pipeline test covers cancelled ordinary commands, nested send reentrancy, notification/reset before immediate jump before later listeners, all four real move variants across 32 guard states, actual transformed MotionWorld + horizon-world jump body consumption, no-jump block notifications, silent bypass, absent owner scope, and cooldown preservation across live toggles. Composition syntax evidence is in `build/anti-hunger-tests/composition.json`; the current cumulative transform contains 794 Rust sources.

Complete pinned Horizon actual `fill_hooks` plus `cargo check --offline -p horizon-client --lib` **passed** in `build/anti-hunger-native-03/report.json`; its check took 184 seconds including package-cache waits. The sole warning is the pre-existing unused camera-perspective helper. Native02's five immutable-wrapper compile failures are retained separately. No upstream, production staging, shared generator, or shared runtime source was modified by these drivers.

Evidence files: `build/anti-hunger-tests/{result,composition,oracle,oracle-metadata}.json`, `build/anti-hunger-native-03/{report,source-snapshot}.json`, and their command logs. Original oracle SHA256: `6e5cb66d8564b57059b8645fa8991f20c0f0e3945d8712660c270f628d0ffc74`.
