# SnapTap integration handoff

Owned implementation and verification are ready. Shared module/config/runtime registration is intentionally left to root. Full original/native client differential remains `not_run`; this is one module toward the required 236.

## Sources and contract

Pinned original: `upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/movement/ModuleSnapTap.kt` (ae0eb396).

- Register `SnapTap`, category `Movement`, aliases **NullMove**, **SOCD**. There are no custom settings.
- Add behavior `snap_tap: liquidbounce_behaviors::snap_tap::SnapTap` and default initialization; export the owned behavior module.
- Add engine clock module `engine/src/snap_tap.rs` (it provides `nano_time() -> i64`). Only pairwise monotonic nanosecond ordering matters; do not replace it with milliseconds or a counter because equality prefers the positive direction.
- Add config child `engine/src/config/snap_tap.rs` and route serialization to `serialize_snap_tap`. Generic common parse handles Enabled/Bind/Hidden; no custom parser or SettingChange exists.
- Add `snap-tap-patch.py` to production composition. Its only addition is the actual native helper `client_snap_tap_key_mask(KeyCode) -> u8`; no new hook is needed. The helper reads actual current `Settings.keys` via actual `keybinds::key_name`.

## Raw keyboard ordering

In `feed_keyboard`, obtain `host.client_snap_tap_key_mask(code)` before taking state (as with other host queries). After `observe_world`, but **before** `registry.handle_keyboard_sync`, query SnapTap running and call:

```rust
modules.snap_tap.keyboard(
    running,
    snap_tap_mask,
    match action {
        KeyAction::Press => liquidbounce_behaviors::snap_tap::Action::Press,
        KeyAction::Release => liquidbounce_behaviors::snap_tap::Action::Release,
        KeyAction::Repeat => liquidbounce_behaviors::snap_tap::Action::Repeat,
    },
    crate::snap_tap::nano_time,
);
```

`MixinKeyboardHandler.java` emits KeyboardKeyEvent on all screens, then KeyEvent only with no screen. Registry module binding switches belong to the latter. Therefore SnapTap must see the old running state on a key that also toggles its binding. Do not add `screen_open` exclusion. Both native `input_key` and controller `client_control_key` already feed this shared path; do not feed the policy twice.

The mask includes every conflicting movement binding. The policy preserves Kotlin `when` priority **left, right, back, forward**, processing only the first. Press reads clock once, release changes holding only, repeat and unmatched/disabled events do not read the clock.

## Movement ordering and lifecycle

In `client_directional_input`, apply SnapTap to a `rotations::DirectionalInput` built from the event flags before AntiAFK's default-priority movement handler. Assign the four flags back, then continue existing AntiAFK, NoBlockInteract and shared rotation handling. Twerk only changes shift and may stay before it.

Original module initialization order is Sprint line 578, SnapTap line 589, AntiAFK line 593 in `ModuleManager.kt`. `EventHookRegistry.addIfAbsent` inserts equal priority after the existing equal-range. SnapTap's movement handler has default priority 0.

Only internal **both-held** state enables axis overwrite, even if another handler changed the incoming axis flags to false. Single/zero-held leaves that axis untouched. Timestamp equality chooses right/forward, signed `i64` comparison matches JVM Long.

**No disable reset and no world-change reset exist.** Disabled releases are ignored and stale state survives reenable; preserve it. The policy deliberately has no lifecycle reset method.

## Verification

Run `python native/liquidbounce-horizon/test-snap-tap.py`.

Result: **17/17 tests passed** and Clippy `--all-targets -- -D warnings` passed.

Evidence: `build/snap-tap-tests/result.json`, `test.log`, `clippy.log`, isolated generated `api-source/`, `config-fixture.rs`, `lib.rs`.

The fixture compiles the actual owned policy and clock; actual shared Tree/common and owned config child; actual Horizon `KeyBinds`/winit `KeyCode`; actual native helper and actual `horizon-mod-api-gen` Host implementation. It exercises both axis press orders, equal and signed nanosecond times, repeat/release guards, all 16 input flag combinations, disabled stale state, first-match binding conflicts, current rebinding, physical key names, mouse/unknown/unmapped keys, server execution exclusion and common localized config serialization.

Storage shells substitute Game options/settings; no full registry/actor execution, real UI screenshot, original/native full-client differential, or full native typecheck is claimed by this fixture. Production integration/build/controller verification remain root-owned.
