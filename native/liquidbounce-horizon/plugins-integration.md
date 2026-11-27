# Plugins integration contract

Source: LiquidBounce `ae0eb396` ModulePlugins and ServerObserver.

Owned files: `behaviors/src/plugins.rs`, `engine/src/config/plugins.rs`,
`engine/src/plugins_runtime.rs`, `plugins-host.rs`, `plugins-patch.py`,
`plugins-smoke.rs`, `update-probes-locales.py`.

Root-owned shared wiring:

1. Export behavior `plugins`; add `mod plugins_runtime` behind behavior-hooks.
   Add `plugins: Arc<crate::plugins_runtime::PluginsService>` to BehaviorModules
   and initialize with Default. Register Plugins as Exploit, key `plugins`,
   disable_on_quit=true.
2. Add the service to PacketPipeline at FIRST_PRIORITY (`i32::MAX`) and update
   its set_running in sync_packet_modules. Incoming callbacks only queue original
   COMMAND_SUGGESTIONS data, including already-cancelled PacketEvents.
3. Use ModuleCallback::EnabledEffect(Plugins), enqueue OneShotEffect::PluginsStart,
   then call `self.plugins_start(host)` from apply_effects. Do not call Host from
   OnEnabled. CancelSequences Plugins calls service.cancel(); actual disconnect
   calls service.disconnected() so ServerObserver capture and roots are cleared.
4. client_tick_pre calls plugins_tick after AutoDisable drain. Add generated hooks:
   `client_plugins_game_message(&self,host:&mut dyn Host,event:&mut ClientPluginsGameMessageEvent<'_>)`
   calls plugins_game_message(host,event.message).
   `client_plugins_server_commands(&self,_host:&dyn Host,event:&mut ClientPluginsServerCommandsEvent<'_>)`
   calls plugins_server_commands(event.names).
5. Add config submodule/export serialize_plugins; ModuleKind::Plugins with
   Exploit category/key plugins; SettingChange::Plugins(u32), RuntimeRequest::Plugins(u32).
   Parse via plugins::parse_plugins. Apply try_methods on service.behavior under
   a short lock. Serialize the same behavior with common settings.
6. Add plugins-patch.py to make-behavior-patch. It adds actual accepted SystemChat
   and Commands heads and the four Host helpers. No player/disguised/client-added
   chat line can satisfy the GAME_MESSAGE probe. Capture server command root
   children before Horizon appends local commands.

Behavior: Slash then ColonSlash each wait 5s followed by a 1s delay on timeout.
TabComplete requests /version, then /bukkit:version if empty/timeout, each 1s.
Version result spelling wins case-insensitive ties. Server command namespace
prefixes join the sorted set; if no prefixes exist, request / with a 5s timeout.
Result reports preserve source anticheat colors and disable the module. Captured
ServerObserver plugins persist across cancellation and clear on disconnect.

Validation: two focused standalone Rust tests passed, covering method order,
timeouts, successful exact-case chat match, suggestion IDs, case deduplication,
namespace fallback and capture cleanup. Native generated-hook compilation is
owned by root. Timeouts advance on actor ticks; queued suggestion timestamps
retain arrival time. This explicit delivery boundary does not establish full
client timing or differential parity. Valid Rust Unicode uses Java25 simple
case data; isolated UTF-16 surrogate strings are outside this runtime surface.
