# Blink and shared BlinkManager

Pinned LiquidBounce ae0eb396: all ModuleBlink settings, real local dummy,
Ambush, strict `++ticks > ResetAfter`, reset/discard and selected-origin flush.
No source tag is added. Dummy uses an actual empty-equipment RemotePlayer Mob,
fresh entity UUID, negative local id, real profile skin and original head yaw.
Disable and world teardown remove its native identity and entity.

Criticals, Freeze, Blink and NoSlow BlockingBlink arbitrate one mixed FIFO.
QUEUE > PASS > FLUSH is monotonic. NoSlow's obsolete listener is unregistered.
The manager exposes real positions, nonempty state, origin/count flush, cancel
and timestamp operations. Packet silent sends continue to update tracking.
The actual loaded LocalPlayer turn invokes Blink before input/aiStep, separately
from GameTick; pauses and canceled player ticks do not increment ResetAfter.

Incoming forced flush inserts direct replay events onto the same reader FIFO
before the triggering packet. The game actor replays these through the existing
native replay helper, avoiding a frame-race in which teleport could precede
older queued packets. Ordinary receive probes still flush at packet-processing
HEAD. World/session changes discard stale queued packets.

AutoDodge uses an explicit real-predictor interface, gated by its actual module;
AutoDodge is not yet registered and this cross-module behavior awaits that port.
BlinkManager's five ESP modes and global render settings are a separate owned
delivery in progress. No complete-client differential or visual parity is claimed.

One focused smoke checks default direction, strict threshold, mixed FIFO count
flush and movement-only cancellation. Official integrated compilation and final
game validation remain distinct, root-owned checks. Simplified/traditional
module/settings/help/notifications follow the current game language.
