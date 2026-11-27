# Complete SkinDerp integration

Pinned source: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`,
`features/module/modules/fun/ModuleSkinDerp.kt`.

Owned production files:

- `native/liquidbounce-behaviors/src/skin_derp.rs`
- `native/liquidbounce-horizon/engine/src/config/skin_derp.rs`
- `native/liquidbounce-horizon/engine/src/skin_derp_runtime.rs`
- `native/liquidbounce-horizon/skin-derp-host.rs`
- `native/liquidbounce-horizon/skin-derp-patch.py`

## Settings and exact behavior

Sync defaults false, Delay defaults 0 with UI range 0..20 ticks. Imported raw
integers remain unclamped. Parts defaults to all seven choices in this enum
order: Hat, Jacket, LeftPants, RightPants, LeftSleeve, RightSleeve, Cape.
That selection order differs from the actual protocol masks:
64, 2, 16, 32, 4, 8, 1. Mutable EnumSet imports are case insensitive and keep a
successfully decoded prefix if a later Gson entry throws. Empty is allowed.

The exclusive DROP tick coroutine first captures Delay and waits. Zero runs
synchronously; negative raw values still wait one tick. After waiting it reads
Parts, Sync and the actual stored options live. Sync=true flips every selected
part. Sync=false samples Kotlin Random.Default.nextBoolean once for each
selected part in enum order. Empty selections consume no random values.

`onEnabled` copies actual model parts only when the in-game ClientModule
lifecycle invokes it. `onDisabled` clears all current parts and restores that
captured set. The initial saved set is empty. Enabled-value edits outside a
world skip both callbacks, so entering a world with the setting already on
must not invent a new capture. Repeated same-value writes also skip callbacks.
CancelSequences cancels the waiting coroutine without erasing the saved set.

The real MC 26.3 `Options.setModelPart` bytecode only adds/removes from the
option set; it does not send or broadcast a packet. Thus the native helper
changes real `modelPart_*` options. Normal Horizon options-change detection
and normal local-player rendering then see the new set. Do not substitute
HandDerp's advertisement-only packet bridge or emit a packet per part.
The source loop contains no reentrant events in these simple option writes,
so applying its final bit mask in one native call preserves its observable
behavior. No options-file save is introduced.

## Shared integration contract

Register `pub mod skin_derp` in behaviors, the owned config codec and
`mod skin_derp_runtime` in engine. Add a default `modules.skin_derp: SkinDerp`.
Config uses `SkinDerpSetting: Clone`, `serialize_skin_derp`,
`SettingChange::SkinDerp` and `RuntimeRequest::SkinDerp`; call `apply` and report
its possible import error after retaining any already applied selection prefix.

The helper patch supplies:

- `Host.client_skin_derp_parts() -> i32`: actual current seven-bit set,
  readable in a menu.
- `Host.client_skin_derp_set_parts(parts: i32) -> bool`: changes actual options,
  accepts 0..127 and emits no packet itself.

Capture `RawOptions.model_parts` via the first helper at the same point as
other actual options. On the source OnEnabled callback call
`skin_derp.on_enabled(options.model_parts)`. OnDisabled queues the mask returned
by `skin_derp.on_disabled()` for the second helper **outside all state locks**.
CancelSequences calls `skin_derp.cancel_sequences()`.

Root adds `WaitOwner::SkinDerp` when composing its exhaustive runtime match.
The owned runtime functions automatically drain local registration ids into
`modules.coroutine_ticker` before effects:

- `skin_derp_callback(host, in_game, callback_id) -> bool` dispatches one global
  FIFO task, checks identity first, reads a live guard/options snapshot and
  returns whether the same callback survives.
- `skin_derp_tick(host)` runs the normal priority GameTick handler. In the
  source ModuleManager initialization order it belongs **after Derp, before
  HandDerp**, then Twerk. The global FIFO always precedes these new handlers.

No existing WaitOwner variant or shared/staging source was changed by the
owned module. A pending delay completing at CoroutineTicker and a new zero
delay normal handler may both mutate parts in the same actual tick, as source.

## Evidence

- `build/skin-derp-source-01/report.json`: 5 tests + Clippy, **844 command
  rows** from byte-for-byte original ModuleSkinDerp and CoroutineTicker,
  real kotlinx.coroutines and JDK25 Random.Default. Production Rust compares
  actual options, saved options and exact RNG seed for every command.
  All 128 selections run in both Sync and Random modes, with raw delays,
  captured/live settings, world guards, cancellation and lifecycle changes.
- `build/skin-derp-config-01/report.json`: **50 complete production config
  tests** + Clippy. Only registration is shimmed when shared SkinDerp is absent;
  the actual codec and runtime forwarding are used. All 128 selections,
  interop/file metadata, raw imports and partial-invalid EnumSet mutation pass.
- `build/skin-derp-native-01/report.json`: independent complete Horizon API
  generation and `horizon-client --lib` check both passed (47.25s + 139.187s).
  The full pinned source, cumulative helper patch and actual fill_hooks compile
  together; shared staging was untouched.

Source fixtures use explicit Minecraft/settings/event stubs. The native API
check and final full-client/game execution provide separate hosting evidence.
