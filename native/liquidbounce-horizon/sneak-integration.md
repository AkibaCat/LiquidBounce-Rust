# Sneak

Native 26.3 port of pinned ModuleSneak, all three source choices and both settings.
Legit runs at the input event: true previous input move vector, fire resistance,
Resistance V, creative/spectator and Frost Walker checks; actual player body
prediction for two ticks with jumping disabled and edge backing disabled;
actual bottom collision shapes against magma. Vanilla marks ordinary outgoing
PlayerInput sneaking without altering the local input. Packet dispatch remains
separate from registry borrows and sees canceled events as source handlers do.

Switch preserves the source protocol error notification and does not emit a
legacy protocol packet: source send1_21_5Start/StopSneaking returns immediately
without ViaFabricPlus, which this native 26.3 runtime does not load. Simplified/
traditional Chinese notification, labels and descriptions follow game language.

One model smoke passed, covering input guard, immunity, packet flags and native
Switch behavior. Actual Game/engine compile and final game validation pending.
The two-tick native simulation uses the shared current player simulator; complete
LocalPlayerHooks differential behavior is part of final validation, not established
by the short model smoke.
