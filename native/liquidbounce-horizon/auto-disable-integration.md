# AutoDisable integration contract

Source: LiquidBounce `ae0eb396` ModuleAutoDisable and CommandAutoDisable.

Owned files: `behaviors/src/auto_disable.rs`, `engine/src/config/auto_disable.rs`,
`engine/src/auto_disable_runtime.rs`, `auto-disable-patch.py`, `auto-disable-smoke.rs`.

Shared wiring (root-owned):

1. Export `pub mod auto_disable;` from behaviors and `mod auto_disable_runtime;`
   behind behavior-hooks in engine. Add BehaviorModules fields `auto_disable:
   liquidbounce_behaviors::auto_disable::AutoDisable` and `auto_disable_packets:
   Arc<crate::auto_disable_runtime::AutoDisablePackets>` with defaults. Register
   AutoDisable as ordinary World module, category key `autoDisable`.
2. Add config submodule and public exports `AutoDisableSetting,
   serialize_auto_disable`; add `AutoDisable(AutoDisableSetting)` to
   SettingChange and RuntimeRequest. Parse with `auto_disable::parse_auto_disable`;
   return the cloned RuntimeRequest and apply settings via `setting.apply` on
   the actor. Serialize `b.auto_disable`. This setting enum is Clone, not Copy.
3. PacketPipeline `add_listener(0, modules.auto_disable_packets.clone())`.
   In sync_packet_modules call its set_running with current AutoDisable running.
4. At the beginning of client_tick_pre call `self.auto_disable_drain(host)`.
   Add Hooks implementations using generated events:
   `client_auto_disable_flag(&self,host:&mut dyn Host,_event:&mut ClientAutoDisableFlagEvent)`
   calls drain;
   `client_auto_disable_death(&self,host:&mut dyn Host,_event:&mut ClientAutoDisableDeathEvent)`
   calls drain then event(Reason::Death);
   `client_auto_disable_command(&self,host:&mut dyn Host,event:&mut ClientAutoDisableCommandEvent<'_>)`
   sets `event.handled |= self.auto_disable_command(host,event.message)`.
5. Add `client_auto_disable_lifecycle(&self,host:&mut dyn Host,event:&mut
   ClientAutoDisableLifecycleEvent)`: drain first, then dispatch reason 2 to
   WorldChange and 3 to Disconnect. Do not call AutoDisable from the existing
   read-only client_world_change/client_disconnecting callbacks. The owned
   patch emits the new mutable lifecycle at the actual replacement/unload
   sites immediately before those general hooks. Each disabled module takes
   and applies its pending native cleanup before the next selected module.
6. Add auto-disable-patch.py to make-behavior-patch. It adds actual local
   PlayerCombatKill dispatch before show_death_screen; death screen opening is
   not this event. A hidden death screen still triggers. It also drains before
   an accepted PlayerPosition and consumes .autodisable chat commands locally.
7. GUI registry-list needs `/api/v1/client/registry/client_module`: return a
   name-keyed map with the actual registry's canonical module names. Existing
   interop registry-list expects this endpoint, never an invented module list.

Source behavior retained: module_names registry-list and selected reference
membership are independent. Deserializing Modules modifies only module_names.
The original clear/add/remove methods update both; .autodisable implements all
four management actions. Built-in names are unique and immutable, so canonical
name is an identity token for the resolved registry entry. Defaults retain
Fly/Speed/NoClip/KillAura, including modules not yet registered. Runtime resolves
them at event time. ReferenceOpenHashSet's unspecified identity-hash iteration
order is not reproduced; selected membership uses deterministic insertion order.

Thread boundary: ordinary original PlayerPosition packets are recorded in
arrival order, including a packet cancelled by another listener. Accepted flags
drain synchronously before teleport application; cancelled flags drain at the
next client tick or native event. Registry locks are never acquired by the
socket callback. This is an explicit actor delivery boundary, not proof of full
client differential parity.

Validation: one standalone Rust test compiled and passed for trigger masks,
default membership, add/remove identity, and configuration/reference separation.
Native patch anchors were consumed exactly once, including hidden-death path.
The root native compile remains the required actual generated-hook check.
