# AntiAFK integration contract

Pinned source: LiquidBounce `ae0eb396`, `ModuleAntiAFK.kt`, `EventManager.kt`,
`EventListener.kt`, `CoroutineTicker.kt`, `ModeValueGroup.kt`, and `SwingMode.kt`.
Runtime implementation is Rust. Java/Kotlin files are development oracles only.

## Owned production files

- `native/liquidbounce-behaviors/src/anti_afk.rs` — all three modes, six
  interactions, independent Custom coroutines, event-once lifecycle.
- `native/liquidbounce-behaviors/src/random.rs` — additive JDK25 APIs;
  `next_i32_bound` differs from origin/bound, and `next_unit_f64` uses two
  `mix32` draws through `Random.next(26)`/`next(27)`.
- `engine/src/config/anti_afk.rs` — full tree, raw settings, structural reuse of
  the existing complete noncombat Rotations codec. No runtime NoRotateSet state
  or fake module registration is used by the adapter.
- `anti-afk-host.rs`, `anti-afk-patch.py` — five real native Host helpers.

Shared registry identity: `ModuleKind::AntiAfk`, name `AntiAFK`, category
`Player`, translation key `antiAFK`. Forward requests through
`SettingChange::AntiAfk(AntiAfkSetting)` and `RuntimeRequest::AntiAfk`.
`AntiAfkSetting::apply` returns a result because imported MULTI_CHOOSE may mutate
a valid prefix before an invalid Gson entry throws. Apply in declaration order.

## Actual tick and effect sequencing

Use the existing actual client CoroutineTicker/GameTick phases, including
screens. Do not create an additional synthetic tick or crosshair poll.

1. Capture current module `running`, call `AntiAfk::begin_tick(running)` exactly
   once, before the new GameTick event's normal handlers.
2. Drain `next_ready(snapshot, server_rotation, random)` until `Ok(None)`.
   Capture a fresh actual snapshot for each call. Execute each returned effect
   outside the module state lock, then drain its `after_effect` tail, also with
   fresh snapshot/settings. Existing waits resume in registration order.
3. At GameTick, invoke `start_swing_handler(running, snapshot)` first and execute
   its effect/tail. It is Custom's independent earlier registered handler.
4. Invoke `start_main_handler(running, snapshot, server_rotation, random)` once
   and execute its effect/tail. Busy runs DROP the event. Zero waits complete
   in this call; they do not repeat an event handler within the same GameTick.

Use `random::with_thread_random` only around pure state processing. Release the
module mutex and the thread RNG borrow before native calls. A normal Punch
packet can reenter the engine and change settings or disable the module.

For every effect, execute its native action, then call `after_effect`. If that
returns an additional effect, process it the same way. In particular:

- Random swing samples its outer delay after the real Punch callbacks.
- Custom `ForwardKey` resumes the source's Jump and Rotate tail after setting
  the real key. The key is not a synthesized MovementInput override.
- Custom swing/rotate check their toggles only when starting. A toggle change
  during the captured delay does not suppress the pending action.
- Captured Custom Rotate delay is fixed, while Angle, serverRotation,
  IgnoreOpenInventory and all rotation settings are read after it resumes.

Source cancellation inside a synchronous random effect still evaluates the
outer `Delay.random()`, then immediately cancels a nonzero suspension. The
generation token in `after_effect` preserves that behavior.

## Effects and RotationManager

`client_anti_afk_state()` yields actual swinging, onGround and current player
angles. Parse with `anti_afk::Snapshot::parse`. Supply the shared manager's
current `server_rotation` independently; player look is not serverRotation.

- `Effect::ForwardKey` → `client_anti_afk_forward()`.
- `Effect::SwingHand` → `client_anti_afk_swing()`.
- `Effect::SelectedSlot(slot)` → `client_anti_afk_select_slot(slot)`.
- `Effect::Yaw(yaw)` → `client_anti_afk_rotation(Some(yaw), None)`.
- `Effect::Pitch(pitch)` → `client_anti_afk_rotation(None, Some(pitch))`.
- `Effect::Rotation(request)` → shared RotationManager request, priority **20**,
  provider **AntiAFK**, captured processor `request.processor_id()`.

Processor IDs **4/5/6** are AntiAFK's separate Linear/Sigmoid/Acceleration mode
instances. Processing and resetting a captured ID reads AntiAFK's current
settings for that particular mode, even if AngleSmooth's selection changes.
NoRotateSet uses 1/2/3 and Sprint's hidden default is 0. Reuse the exact existing
`RotationsSettings::process` arithmetic with the correct owner settings.

The request already contains captured reset parameters and movement correction;
`consider_inventory` is `!IgnoreOpenInventory`. It does not force visible player
rotation. A mode change does not proactively remove a manager request: the
source request provider is ModuleAntiAFK, not its Custom mode.

## Lifecycle and movement events

`AntiAfkSetting::Mode` cancels the old mode's sequences when the actual mode
changes, including outside a world, matching ModeValueGroup.setAndUpdate.
Unchanged mode writes do not cancel. Actual module CancelSequences/OnDisabled
callbacks call `cancel_sequences`. Out-of-game enabled-value writes retain
the original ClientModule callback behavior.

No cancellation resets `random_direction`, pending once jumps or the forward
key. EventManager skips inactive once owners without unregistering them; pending
RandomInteraction/Custom jump events are consumed when their mode next runs.
A small real Kotlin compiler fixture in `build/anti-afk-receiver-01` confirmed
the nested RandomInteraction enum's `once` receiver is RandomInteraction.

At the actual MovementInputEvent, convert the four booleans to `Direction`, call
`movement_input(running, &mut direction, &mut jump)`, then copy them back at the
module's source order. Old only sets forwards. RandomInteraction replaces all
four direction flags, including its default NONE; Custom only consumes its
pending jump. Preserve sneak/sprint and unrelated event fields.

## Native behavior boundaries

Helpers guard the actual local native client and exclude acting LAN/fake-player
swaps. `ForwardKey` writes the real logical Forward mapping and relies on the
existing ordinary release/screen/focus handlers. No automatic module-disable
release is added. ChangeSlot assigns raw selected and adds no immediate carried
sync. Swing reads the main hand's real `minecraft:attack_animation` components,
uses main hand even after offhand use, then sends ordinary Punch independently
of local swing acceptance. Singleplayer without a remote session retains the
ordinary Horizon `send` behavior rather than inventing a remote transport.

## Evidence

- `build/anti-afk-03/report.json`: production pure state/settings/random source,
  seven tests, 256 real JDK25 mixed random-sequence vectors, Clippy `-D warnings`.
- `build/anti-afk-config-04/report.json`: 37 production config tests and Clippy,
  including the final Mode setter cancellation lifecycle; explicit
  registration-only shim when shared engine has not integrated it.
- `build/anti-afk-native-02/report.json`: real API generation and complete
  `horizon-client --lib` compilation passed in an independent pinned snapshot.
  Includes final public Punch packet path and cumulative AutoTool visibility fix.

These independent fixtures do not claim shared engine registration or actual
gameplay execution; those are the final root integration checks.
