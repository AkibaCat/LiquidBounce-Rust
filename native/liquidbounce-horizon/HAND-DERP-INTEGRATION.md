# Complete HandDerp, LiquidBounce ae0eb396

Owned production files: behavior `hand_derp.rs`, config `config/hand_derp.rs`,
`hand_derp_packets.rs`, `hand-derp-{bridge,host}.rs`, `hand-derp-patch.py`.
The cumulative patch must apply after packet/slot patches, adding its two helper
names to the existing behavior fragment. It writes a new Horizon
`hand_derp_bridge.rs` through `edit`; no pristine source is modified.

## Shared registration

- Export `pub mod hand_derp;` from behaviors.
- Engine `mod hand_derp_packets;`, `Arc<HandDerpService>` in BehaviorModules;
  register `HandDerp` / `Fun` / `handDerp` with its actual enabled lifecycle.
- Config module/reexport: `HandDerpSetting`, `serialize_hand_derp`.
  `ModuleKind::HandDerp => ("HandDerp", "Fun", "handDerp")`;
  `SettingChange::HandDerp(HandDerpSetting)` and RuntimeRequest of same type.
  Parser is `hand_derp::parse_hand_derp`, Runtime forwarding copies the setting.
- Serialize the behavior mutex's state; apply settings under that mutex.
  Mode uses `set_mode`: changed mode cancels, same mode preserves captured wait.
- Install the service as ordinary PacketListener **priority 1**. Publish running
  on registry/world changes before packet dispatch. Both Silent incoming and
  Swing outgoing execute despite cancellation by earlier ordinary listeners.
  NoSwing's connection injection hides Punch before event creation, so no flip.

## Initialization and real option

`hand-derp-patch.py` publishes actual current Settings::client_information at
the window's **native_client_start before dispatch**, before a world or enable.
The bridge preserves this first main_hand forever, even across worlds and later
options changes. Current options refresh at frame/tick and immediately before
Game.send. `HandDerpService::initialize()` initializes its pure behavior from
that captured original and can be called at startup/first handler without
recapturing the current changed option.

`client_hand_derp_hand() -> i32` returns actual option 0 left / 1 right, and
refreshes the bridge. `client_hand_derp_send(hand:i32) -> bool` builds complete
current actual information and changes **only** main_hand. It never changes
Settings.main_hand_left or normal broadcast's sent_info cache.

The packet keeps current language, raw renderDistance cast to protocol byte,
chat visibility/colors, model layer mask, text-filtering, server-listing and
particle values. Settings::client_information is Horizon's authoritative
current ClientInformation builder; its account text filtering currently follows
Horizon's ordinary connection value. No independent guessed fields are added.

## Tick phases and lifecycle

At CoroutineTicker phase, under behavior lock call `initialize()` first,
then `begin_tick(running)` to decrement/cancel. Obtain `ready_ticket()` for
the global FIFO. When that ticket is selected, call `resume(ticket)` under
lock, drop lock, then `host.client_hand_derp_send(hand.id())` if Some.

At normal GameTick priority0, **Derp registers before HandDerp**, followed by
Twerk, Sprint, AntiAFK, AutoFish/AutoRespawn (the pinned ModuleManager array and
object initialization order). Under lock call
`start_delay(running, || global_wait_registration_ticket())` and execute any
returned hand through Host after unlocking. Delay0 flips immediately with no
ticket; negative raw Delay registers a one-tick wait. Captured Delay survives
live leaf changes until resume. Resuming completes the exclusive sequence;
the later normal handler may register again in the same tick.

On actual **in-game** disable, read current real option through Host, call
`initialize()` then `on_disabled(MainHand::from_left(actual==0))`; unlock then
send returned flip. Preserve the source's condition: only when real option
differs from the startup original. This is an additional currentHand.opposite
flip; it is not assignment to originalHand and does not restore the real option.
Disable outside a world does not invoke the source onDisabled callback; waits
are invalidated by guards/cancel at the next appropriate phase. Changed mode
cancels immediately even outside the world. Current/original hand never reset.

## Scoped synchronous packet sending

The owned cumulative patch adds PacketEvent's optional scoped sender and
`send(packet)->bool`, plus `packet_bridge::outgoing_with_sender`. Existing
`outgoing` remains compatible for callers without a Session sender.
Actual Session.send_with_path passes `|nested| self.send(nested)` before any
writer lock. The listener flips under its behavior mutex, drops it, then calls
event.send. This executes nested ordinary ClientInformation listeners/write
before continuing the original Punch's later listeners/write. Earlier Punch
cancellation does not suppress the HandDerp callback or nested information.
Silent sends bypass this listener through the existing explicit silent path.

Silent incoming matches metadata index **0/1 equal to current real option id**,
across **every entity id**, and cancels the whole packet. Do not substitute
the normal main-arm metadata index or filter to local player.

## Evidence

- `build/hand-derp-packets-01/report.json`: 5 tests + Clippy -D warnings;
  complete pure behavior and actual original listener, scoped sender and exact
  patched Session bodies through real Horizon framed Pipe encoding.
- `build/hand-derp-config-01/report.json`: 43 whole production config tests +
  Clippy; explicit registration-only shim where shared integration is absent.
- `build/hand-derp-source-01/report.json`: byte-for-byte original ModuleHandDerp
  compiled with real Kotlin suspend continuations and minimal environment
  stubs; **562 command rows** compare both initial hands against production Rust,
  covering both modes, captured/live/raw delays, mode/lifecycle/world guards,
  cancelled/NoSwing Punch, Silent predicate and unusual disable behavior.
- `build/hand-derp-native-01/report.json`: complete independent pinned Horizon
  API generation and horizon-client --lib check passed (150.609 s library
  check; one pre-existing camera perspective dead-code warning).

These independent tests do not claim full game/actor integration. Shared
CoroutineTicker global FIFO must be composed before full cross-module timing
equivalence is claimed; registration tickets are explicitly supplied externally.

## Read-only final packet observations

`HandDerpService.accepted_packets()` returns cumulative ordinary accepted
ClientInformation/Punch counts plus the complete final ClientInformation and
its session id. These values come from `PacketListener::outgoing_accepted`,
after all cancellation/edit handlers. They are independent of pure hand-flip
intentions. Observe deltas around a controlled operation, since normal options
broadcasts contribute ClientInformation too. Acceptance precedes transport and
is not a server acknowledgement.

`build/hand-derp-packets-02/report.json` verifies these observations against the
real Session send bodies and framed Pipe decoding: a cancelled Punch is absent
from the count while its nested information is present; NoSwing and silent
Punch bypass the ordinary accepted observation. Full packet field values match.
