# TrueSight, PotionSpoof, NoClip — owned handoff, 2026-11-26

Pinned LiquidBounce ae0eb396c07f22383789b28d5912d714c6ca0a86, Horizon
9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0. Shared roots, dictionaries,
AutoBreak/ItemScroller and upstream sources are untouched.

Each module has behavior, config adapter, runtime adapter, native host and owned
patch. TrueSight additionally has `true-sight-render.rs`; PotionSpoof has generated
`potion_spoof_effects.rs` and `generate-potion-spoof.py`.

## Shared wiring

Export `potion_spoof`, `no_clip`, `true_sight` in behaviors/lib; add default states
with those exact names to BehaviorModules. Register PotionSpoof in Player,
NoClip in Movement, TrueSight in Render, no aliases. Add corresponding ModuleKind,
SettingChange, RuntimeRequest and config serializer/parser dispatch following the
owned config adapters. Add runtime modules under behavior-hooks in engine/lib.
Source module order: Movement NoClip after NoWeb; Player PotionSpoof after PortalScreen;
Render TrueSight after Tracers. Preserve global registration/event ordering.

PotionSpoof:

- `potion-spoof-patch.py::apply` adds helper_names
  `client_potion_spoof_duration(&str)->Option<i32>` and
  `client_potion_spoof_apply(&str,i32,u8)`. No hook event.
- PlayerTick: call `self.potion_spoof_tick(host)` at PotionSpoof's handler position.
  It clones settings under a short borrow and performs all real host effects
  after releasing it. onDisabled: call `self.potion_spoof_disabled(host)`.
- 40 effect groups in actual pinned registration order; English names come from
  official MC26.3 client JAR en_us.json, not generated title case. Each child has
  Enabled=false, Level=1 UI1..10. Imports do not clamp Level; Minecraft constructor
  wraps level-1 then clamps amplifier0..255 (verified actual JAR javap).
- Enabled: addEffect then explicit addAttributeModifiers, even if a stronger
  effect rejects replacement. Disabled child: remove only duration0, followed by
  explicit modifier removal. Module disable: only enabled child duration0, with
  the ordinary removeEffect lifecycle and no extra explicit modifier removal.
- Actual effect status/particles/icons/attributes and modifier IDs are used.
  Server-client attribute lifecycle difference is preserved.

NoClip:

- `no-clip-patch.py::apply` adds helpers `client_no_clip_state()->Option<[f64;4]>`,
  `client_no_clip_flags(bool)`, `client_no_clip_velocity(bool,bool,[f64;3])`.
  Apply after mining/Strafe and EntityControl. State is actual yaw, controlled
  vehicle, jump and sneak. Uses first-passenger boat or actual driven mount.
- GameTick: `self.no_clip_tick(host)` at Movement handler position. Disable:
  `self.no_clip_disabled(host)` (unconditional original onDisabled). Incoming
  ClientboundPlayerPositionPacket PacketEvent: `self.no_clip_setback(host)`,
  including earlier-cancelled events. It emits original English/SC/TC error chat
  before synchronous registry disable; that disable must execute lifecycle.
- Settings are Speed=.32 UI.1...4, OnlyInVehicle=false,
  DisableOnSetback=true. Pause restores collision without disabling module.
  Jump wins over sneak. Direction uses existing source-derived movement_direction
  and Java sine/cosine. Horizontal old*0 preserves IEEE NaN only while moving;
  no input explicitly sets both horizontal components0. No velocity reset on disable.
- Adds an actual client_no_physics field to PlayerMotion, Mob and Boat. Player
  EntityMoveWorld, ridden living do_move and boat movement bypass collision.
  Does not set spectator/flying. Clears only the current controlled vehicle,
  retaining original behavior if a previous vehicle was left with noPhysics.
- VehicleControl is not yet implemented; runtime currently passes false. When
  that module is registered, replace that one flag with its real running query.
  This dependency is explicit and there is no placeholder VehicleControl module.

TrueSight:

- Apply `true-sight-patch.py::apply` **after Chams and SilentHotbar**, using the
  same edit/once contract. Event names: `client_true_sight_entity` fields entity
  i32, body u32, feature u32, apply bool; `client_true_sight_markers` field mask u8.
  In NativeHooks dispatch to `self.true_sight_entity(host,event)` and
  `self.true_sight_markers(host,event)`. No helper_names.
- Sight multi-choice order Barriers/Lights/Entities, all enabled by default;
  EntityColor=white alpha100; EntityFeatureLayerColor=white alpha120. ARGB config
  colors convert to Horizon ABGR at the renderer boundary. Multi-choice supports
  JDK folding and successful prefix mutation. Color import uses original
  numeric low32 and six/eight-digit hexadecimal semantics.
- Uses common targeting's actual living/invisible identity, including own player,
  remote mobs and armor stands. Only invisible living objects enter a scoped
  render tint; RAII restores scopes on early returns. CPU path is used to retain
  per-vertex tints and OIT. Body and colored-cutout models have separate colors;
  equipment, held items, eyes, energy swirl and breeze wind retain their render types.
  Existing outline behavior remains.
- Block markers stay within actual random animateTick sampling. Active barrier
  or light choice forces the original BARRIER sentinel; selected lights compare
  as that sentinel while particle sprites use actual block states. This preserves
  original expression's consequence: barriers can still show in Lights-only mode.
- ESP dependency is an explicit bool input in the pure module (white alpha120
  fallback). Runtime passes false until actual ESP.requiresTrueSight exists;
  no fake ESP registration.

## Chinese and short checks

Execute `update-sight-potion-clip-locales.py --directory
native/liquidbounce-interop/resources/localization` after integration. It adds
SC/TC labels and help for all settings and all40 effect groups; potion labels
come from the prepared official MC26.3 Chinese resources. It was tested on
copies only, so shared dictionaries remain root-owned.

Per user speed request, each module received one focused offline Cargo compile
and two short tests (configuration roundtrip + behavior smoke): all passed.
Owned patches applied to current actual staged sources in memory, and locale
updater ran against copied dictionaries. Reports:

- build/potion-spoof-smoke-01/report.json
- build/no-clip-smoke-01/report.json
- build/true-sight-smoke-01/report.json

One unused re-export warning per fixture is declaration scaffolding, not the
production adapters. These checks are not concrete Game execution, screenshots,
or proof of all native visual paths. Root final aggregate Build/Check and actual
game validation still need to compile the complete native host/runtime wiring.
