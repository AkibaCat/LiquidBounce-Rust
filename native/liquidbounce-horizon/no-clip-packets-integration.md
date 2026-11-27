# NoClip revision 02

Adds the independent actual PacketListener in `engine/src/no_clip_packets.rs`.
Add `mod no_clip_packets` under behavior-hooks and
`BehaviorModules.no_clip_packets: Arc<NoClipPackets>`, default it, install it in
the shared PacketPipeline at NoClip's declaration position and priority0.
Synchronize `set_running(query_running(...,"NoClip"))` in the actor's existing
packet-state synchronization path after registry changes and every tick.
It is independent of AutoDisable's running state.

Call `self.no_clip_drain(host)` after `self.auto_disable_drain(host)` in the
shared `client_auto_disable_flag` callback before accepted PlayerPosition
application, and at the beginning of GameTick after auto_disable_drain. The
reader records every original PlayerPosition while running, including cancelled
events, and never calls Host/registry. Replay/silent deliveries do not enqueue.
Queue drain rechecks NoClip running/settings/pause through actual Host state.

`no_clip_setback` now synchronously collects `state.take_effects()`, releases
state and invokes `runtime::apply_effects`, so the onDisabled collision cleanup
finishes before returning to the accepted-position handler.

Also fixes controlled mount lookup in resident singleplayer: Horizon's
`driven_mount()` intentionally only handles remote-client first_passenger state.
The helper now evaluates actual local equine saddle/steering/happy-ghast
controller predicates when no remote exists; boats use the existing first-seat
ownership predicate. Existing EntityControl is_saddled behavior remains live.

`build/no-clip-smoke-02/report.json`: one new focused Cargo compile following the
packet change, three short tests passed, including an already-cancelled original
PlayerPosition, disabled listener and replay deliveries. Actual owned patch
anchors and copied locales pass again. Complete Game/runtime compilation remains
the final aggregate check. Original smoke01 and freeze01 are preserved.
