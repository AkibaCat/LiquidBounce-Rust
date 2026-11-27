# ResetVL integration contract

Owned production source is limited to `native/liquidbounce-behaviors/src/reset_vl.rs`,
`engine/src/config/reset_vl.rs`, `engine/src/reset_vl_runtime.rs`,
`reset-vl-host.rs` and `reset-vl-patch.py`. Shared registration, aggregate
patching, build/staging and locale dictionaries remain root-owned. AutoBreak,
ItemScroller and the GUICloser revision02 freeze are untouched by this module.

## Source and behavior

Pinned LiquidBounce ae0eb396 ModuleResetVL is a deprecated but present builtin,
category EXPLOIT, name ResetVL, no aliases and key liquidbounce.module.resetVL.
Times defaults 10 with UI range 1..50; Timer defaults 2f with UI range 0.1..10f.
The ranges constrain the UI, not imported raw values. There is no suffix.

Every started GameTick handler requests Timer at IMPORTANT_FOR_USER_SAFETY
(60), resetAfterTicks=1. It requests even while airborne. If grounded, the
source sets **all** velocity axes to (0,0.1,0), waits one outer Minecraft tick,
then sets (0,-0.1,0) and increments the singleton Int count with JVM wrap.
The coroutine is Exclusive(DROP); nested ticks and additional GameTick events
cannot overlap its wait. The global FIFO continuation runs before GameTick,
so a nonfinal down-step may be followed by the next up-step on the same tick.

Count survives disable, re-enable and world changes. A cancelled waiter cannot
resume after re-enable. The test is equality `count == times`, including zero,
negative imported values and overflow; do not change it to >=. Settings are
read dynamically. Success executes enabled=false synchronously, then the
SUCCESS notification while count still contains the completed value, then
sets count=0. CancelSequences must call cancel_sequences and preserve this
synchronous remainder. HighJump's Y-only Host setter cannot implement ResetVL.

## Root shared wiring

1. Export behavior reset_vl. Add BehaviorModules.reset_vl with default state,
   register active query name ResetVL, and handle its CancelSequences by calling
   reset_vl.cancel_sequences(). Do not recreate/reset the state on lifecycle.
2. In sync_packet_modules publish timer.set_provider_running(
   crate::reset_vl_runtime::RESET_VL_TIMER_PROVIDER,
   self.query_running(registry,"ResetVL")). This must run for config, bindings,
   world transitions and synchronous self-disable before frame Timer queries.
   The dedicated provider is u64::MAX-1; ModuleTimer retains u64::MAX.
3. Add config module/export, ModuleKind::ResetVl, SettingChange::ResetVl and
   RuntimeRequest::ResetVl with name/key/category above. Dispatch serialize,
   parse and apply its two independent leaves in Times then Timer declaration
   order. Applying settings must not reset count or restart its suspended run.
4. Include reset_vl_runtime under behavior-hooks. Add WaitOwner::ResetVl and
   dispatch shared FIFO callbacks to reset_vl_callback(host,in_game,id). Call
   reset_vl_tick(host) after Timer FIRST_PRIORITY and global callbacks, before
   high_jump_tick. Original builtin initialization registers ResetVL before
   Derp, SkinDerp, HandDerp, HighJump, AntiAFK, AutoFish, AutoRespawn and SmartEat
   (ModuleManager.kt490,514-517,563,593-618). Keep the registration order visible
   when integrating global waiters; the focused oracle does not prove the
   complete client's ordering with all modules enabled.
5. Add reset-vl-patch.py to aggregate composition and collect its helper_names:
   client_reset_vl_ground() -> Option<bool> and client_reset_vl_velocity(f64).
   It inserts exact native bodies at the existing unique Player FOV doc anchor;
   no new native event hook is required. Regenerate actual Host API.
6. At the next safe shared-resource integration run update-reset-vl-locales.py.
   It merges only ResetVL descriptions/help and the Times label. Original
   module descriptions are copied byte-for-text from pinned EN/SC/TC resources;
   every setting has explicit help. Consumption tests run it on copies only.

The success notification keeps original title ResetVL and severity SUCCESS.
English remains “Successfully reset violations”; the user's Chinese-display
requirement changes its emitted body to 已成功重置违规计数 / 已成功重設違規計數.

## Evidence and boundaries

`build/reset-vl-source-03/report.json` copies byte-identical original ResetVL,
CoroutineTicker, SuspendHandlers, SuspendHandlerBehavior, EventListenerScope,
Guards, Timer, RequestHandler and Priority, then executes actual Kotlin 2.4.21,
coroutines 1.10.2 and JDK25. Explicit settings/module/player/event providers and
an immediate Main dispatcher surround those bodies. The atomicfu API provider
uses actual JDK AtomicReference; it is not the original atomicfu implementation.

`build/reset-vl-tests-03/report.json` compares every action with the oracle:
count, enabled/running/busy, raw float Timer bits, raw double velocity bits,
ordered velocity/disable/notification effects and request tick. Production
ResetVL and Timer bodies are unchanged; the shared FIFO body has only a
ResetVl enum declaration supplied when absent. Config tests extract actual
shared Tree/parsers and provide only pending declaration registration.

`build/reset-vl-consumption-02/report.json` compiles exact runtime and native
helper bodies with actual core synchronous ModuleRegistry/CancelSequences,
TimerService implementation, shared CoroutineTicker and actual glam DVec3.
Explicit State/registration, Host and Game/player providers test lock release,
airborne requests, expiry, immediate provider disable, notification, complete
velocity writes and query guards. Webview/no-webview tests and Clippy pass.
Locale help is tested in copied dictionaries only. This is focused consumer
evidence, not actual complete Game/engine or gameplay evidence.

Preserved failures: source01 lacked the required JVM25 compiler target;
tests01 had an ambiguous fixture float type; consumption01's tests passed but
Clippy found an import made unused by omitted PacketListener and the fixture's
unit RawOptions. Those are provider fixes, not production behavior changes.

Pending root checks are cumulative shared configuration/registry wiring,
complete native Game/engine build, source package and TestEngine/actual game
scenarios. No oracle, test provider or compiler result is claimed as game proof.
