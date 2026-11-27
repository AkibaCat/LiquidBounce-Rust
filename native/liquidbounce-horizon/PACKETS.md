# 原生网络包接口与已移植的包模块

来源固定到 LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`，目标为 Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0` 的 MC 26.3 play 协议。此接口处理 Minecraft 数据包；Horizon 控制 socket 和界面 REST 仍是独立的控制通道。

`packet-patch.py` 只提供累计变换 `apply(edit, replace_once)`。主补丁生成器将变换施加到构建副本，并把返回的新增文件写入补丁。原始 upstream 和三份固定源码 ZIP 不会被此文件修改。

| 路径 | 调用位置 | 事件语义 |
| --- | --- | --- |
| 原始接收 | `receive_play` 解码成功后、KeepAlive/Cookie/Brand/Transfer 等处理前 | `Incoming + Original`，可修改字段、取消；录制器保留收到的原始字节 |
| 原始发送 | `Session.send_with_path`，以及网络线程 play 回复在 writer 锁之前 | `Outgoing + Original`，可修改字段、取消 |
| 静默发送 | `Session.send_silently` / Host `native_send_packet_silently` | `Outgoing + Silent`，按 rotation、position_separator、packet_logger 顺序调用明确提供的可选 sink，绕过普通 listener；和上游一样不检查这些跟踪服务的取消结果 |
| 接收重放 | Host `native_replay_client_packet` / `IncomingReplay.apply` | `Incoming + IncomingReplay`，校验 session 代际并直接处理，不触发普通 listener 或旧的只读 `native_client_packet` |

三个静默跟踪服务目前只是可选 sink 契约。此提交没有实现 RotationManager、PositionPacketSeparator 或 PacketLogger，默认值均为 `None`。模块接入者必须另外移植这些服务；不能把空 sink 视为它们已经完成。

`PacketPipeline` 先复制有序 `Arc` listener 列表再释放列表锁；调用方不能持有 registry 或网络 writer 锁。原始事件取消后仍继续按优先级通知后续普通 listener，由 listener 自己检查取消状态，与 LiquidBounce 的事件模型一致。NoSwing 是原版 `MixinConnection` 的事件前特判，因此它取消 Punch 后不会向普通 listener 发布该包。

每个 Session 已有的 `serial` 同时用于发送与 reader 线程。reader 使用不可跨线程移动的 RAII scope；延迟包带着原 session id，换服后会得到明确的 stale-session 错误。网络回调只能访问线程安全快照，不能等待游戏 actor、直接访问 Game 或回调 registry。

Host 重放支持游戏 play handler、KeepAlive、Cookie、资源包通知、ServerLinks 和 Brand。`StartConfiguration`、Transfer、Disconnect 涉及连接状态事务，far-terrain payload 涉及其工作队列；当前 helper 对这些输入返回明确的 unsupported 错误，不会伪造完成。握手、登录和配置阶段不在当前 play 接口范围内。游戏 handler 重放失败会向调用者返回真实错误。

NoSwing 的 `HideFor` 默认同时包含 `Client` 和 `Server`，可分别设置或清空。服务器目标只取消原始 `Serverbound::Punch`；客户端目标在窗口玩家 `swing_hand` 开始前返回，不作用于服务器玩家或其他实体。行为状态以一个原子快照保存，由游戏 actor 根据真实 registry 的 running 状态更新。静默 Punch 保留上游绕过特判的语义。

接入接口：

```rust
let no_swing = Arc::new(NoSwing::default());
let packets = Arc::new(PacketPipeline::with_no_swing(no_swing.clone()));
packets.install(); // 创建 native hook adapter 时一次安装
no_swing.set_running(registry.running(no_swing_id));
no_swing.set_hide_for(HideFor::BOTH);
```

设置代码使用 `liquidbounce_behaviors::no_swing`：`parse_no_swing_hide_for(&Value)` 和 `serialize_no_swing_hide_for(value, interop, description)`。多选使用原版 `MULTI_CHOOSE` metadata，标签查找不区分大小写，未知标签忽略；文件格式不包含协议 metadata。

验证命令：

```powershell
python native/liquidbounce-horizon/prepare-packet-fixture.py
python native/liquidbounce-horizon/prepare-pitch-fixture.py
cargo +stable test --manifest-path native/liquidbounce-horizon/packet-tests/Cargo.toml
cargo +stable clippy --manifest-path native/liquidbounce-horizon/packet-tests/Cargo.toml --all-targets -- -D warnings
```

包管线和传输夹具覆盖 NoSwing、变更与取消、重入、静默路径及重放。生成夹具直接提取实际补丁后的 `Session.send`、`send_silently`、`send_with_path` 和完整 `receive_play` 函数体，并记录变换后 net.rs 的 SHA-256。夹具用 Horizon 官方 `Connection`/`Pipe` 进行真实编码和解码，确认出站 slot 修改、Punch 取消、静默 Punch、入站 KeepAlive 字段修改及取消。配置握手的外围服务在夹具中被替换，因此该测试不证明配置事务或实际游戏集成通过。

## MoreCarry 与 NoPitchLimit

`engine/src/packet_modules.rs` 为两个原版处理器提供 typed adapter。MoreCarry（别名 XCarry）取消所有原始 `Serverbound::ContainerClose`，不筛选容器 id，没有附加设置。

NoPitchLimit 的 `ServerSide` 默认为 `true`。模块运行时，客户端 `Entity.turn`、实体 rotation storage、`setXRot` 等价路径保留原始 pitch，不钳制或取模，旧帧 pitch 仍按原版增加。只有 `ServerSide=false` 时，其普通发包 listener 才将 Rot/PosRot 的 pitch 钳制为 ±90；Pos/Status 没有编码的 pitch 字段。静默发包绕过此处理器，与原版普通 PacketEvent handler 一致。

`pitch-patch.py` 按固定原版源码修改构建副本，提供 `apply(edit, replace_once)` 和新增 hook fragment。共享 packet bridge 查询已注册 listener 的 `bypass_pitch_limit()`，读取模块的原子运行状态。原版 MixinEntity 对所有实体生效，因此该查询也对这些实体旋转 storage 生效。`PositionMoveRotation.calculateAbsolute` 的独立传送包前置钳制不在原版 mixin 目标中，保持原来行为。

游戏 actor 注册一次 `MoreCarryListener(Arc<MoreCarry>)` 和 `NoPitchLimitListener(Arc<NoPitchLimit>)`，优先级均为原版默认 `0`；根据 registry 更新 `set_running`，根据设置更新 `NoPitchLimit.set_server_side(bool)`。禁用 NoPitchLimit 时调用 Host `client_reset_pitch()`，将真实客户端当前 pitch 钳制为 ±90，保留旧帧 pitch，不发包。没有可用玩家时 helper 返回 `false`。

`prepare-pitch-fixture.py` 提取实际变换后的 turn、rope old rotation、Mob.set_rotation、remote_rotation、禁用还原函数体，编译到独立测试中；直接使用原始 Horizon mouse.rs，因此同时执行它现有的九项 mouse oracle 测试。新增测试覆盖越过 360°、两个有 rotation 的移动包变体、普通/静默路径、实体 setters 和禁用还原。总计 13 项测试，其中四项是本移植的验证入口。这个记录仅证明编译的源代码与协议夹具，完整游戏验证由主集成报告另行确认。
# SilentHotbar service and NoSlotSet

The global main-hand audit is verified by `build/hotbar-access-01/report.json`
and `build/slot-modules-02/report.json`. The first compiles exact transformed
getter/mutation bodies and command branches against real Horizon ItemStack,
including drop, swap, totem consumption, shelf exchange, nautilus bucket/saddle,
steering-stick break, enchantment and native item APIs. The second reruns all
17 packet/pitch/slot regression tests with the strengthened identity guard.
Both passed tests and Clippy with warnings denied. These are independent source
fixtures; full-client and full-game claims remain in the integration report.

The effective slot applies only to the window's actual player: `native_window`,
no swapped `lan_acting_uuid`, and `PlayerMotion.native_client_queries_allowed()`
(`!server_copy && !replaying`, supplied by the cumulative mining patch). A
standalone actual player need not have `client_side=true`. Main-hand setters
cache the effective index before borrowing mutable fields. Raw selected reads
retained for input, menus/debug, persistence, guest state swaps, server-only
fake players, replay hand reset and `Inventory.add` placement order are listed
in the hotbar-access report. `MixinInventory` targets only
`removeFromSelected/tick/getSelectedItem/setSelectedItem`, so overriding
`Inventory.add`'s field reads would change the pinned source semantics.

`engine/src/slots.rs` is the original SilentHotbar **service**, separate from the
cosmetic module of that name. The runtime holds `Arc<slots::SlotService>`, calls
`install()` once, and synchronizes `no_slot_set.set_running(bool)` with the
registered NoSlotSet module. NoSlotSet has no settings. Add `pub mod slots;` to
the engine library. Apply `slot-patch.py` cumulatively after `packet-patch.py` and
include its returned new files in the source patch.

The policy keeps four separate values: the actual `Game.selected` field, the
enforced SilentHotbar state, the carried-item cache used by vanilla's
ensureHasSentCarriedItem, and the last successfully written ordinary/silent wire
slot. Wire observations never become `serversideSlot`. This getter uses the
enforced slot or the caller's freshly read real field, with zero as the absent
player fallback. An accepted valid server SetHeldSlot updates the real field;
the enforced state continues until its owner resets it, it expires, or the
world changes.

`Requester::allocate()` creates an identity for each owner; reuse it for that
owner. Zero represents the original nullable requester. `SelectSlotEvent`
provides readonly requester/slot accessors and cancellation. Listeners execute
in descending priority and stable registration order, with no service/listener
locks held. Validation precedes events; cancelled requests preserve both state
and timer. The service's closure-based API reads the real field after events.
The HotbarItemSlot equivalent ignores `None` (offhand/non-hotbar). Replacement
captures the previous `clientsideSlot`, preserving the original visible slot.
Kotlin's `isSlotModifiedBy(null)` returns true even when no state exists; the
Rust service preserves this null identity detail.

`native_client_tick_pre` advances the timer before other tick callbacks, as the
original GameTick priority 1001 handler does. Duration one survives the first
tick and resets on the second; zero/negative durations reset on the first tick.
This also runs while paused or in menus. Level load/unload/disconnect clear the
state and its timer. The source patch redirects the local main-hand/selected
inventory reads in held-item, interaction, visual and equipment paths while
preserving the raw input/UI selected field. Server and guest Games retain their
own selected field.

NoSlotSet runs at the actor's original packet entry, after the reader's mutable
packet event. It cancels every SetHeldSlot, including invalid indices, and sends
the effective slot through the actual ordinary `Game.send` / `Session.send`
path. Ordinary outgoing listeners can mutate or cancel the reply. Incoming
replay enters the direct handler and bypasses this original-packet hook. A
reader-cancelled packet never schedules a reply. Successful ordinary/silent
SetCarriedItem writes notify the service only after releasing the writer lock.

Native host helpers:

- `client_real_selected_slot() -> Option<i32>`
- `client_serverside_slot() -> Option<i32>`
- `client_clientside_slot() -> Option<i32>`
- `client_select_slot_silently(requester: u64, slot: i32, ticks: i32) -> Result<bool, String>`
- `client_reset_silent_slot(requester: u64) -> bool`
- `client_silent_slot_modified() -> bool`
- `client_silent_slot_modified_by(requester: u64) -> bool`

`prepare-slot-fixture.py` compiles the exact transformed Session send/reader,
Game send/carried/original-packet, accepted SetHeldSlot branch and native helper
bodies against real Horizon protocol Pipe connections. `slot_service.rs` tests
owner/null identity, replacement, event reentry/cancellation/priority and timer
boundaries. `build/slot-modules-01/report.json` records independent evidence;
full-client compilation and in-game verification are separate integration work.

# RotationManager packet tracking

`rotation_packets::RotationListener(pub Arc<Mutex<RotationManager>>)` implements
the original RotationManager READ_FINAL_STATE packet handler. Install the same
listener last in ordinary dispatch (`add_listener(i32::MIN, listener.clone())`)
and as the first explicit silent sink (`SilentSinks.rotation`). The manager is
the shared state used by the actor's rotation request/update phases. Release its
mutex before sending a packet; the listener acquires it only for the state
update and performs no callback or game-thread request while holding it.

Outgoing MovePlayerRot, MovePlayerPosRot and UseItem read the final packet after
ordinary listeners finish mutating/cancelling it. They use the original server
normalization (wrap yaw; wrap then clamp pitch). A cancelled ordinary packet
updates only theoretical rotation. An explicit silent write updates both actual
and theoretical rotation; cancellation by silent tracking services cannot
suppress that write. Position/status-only movement and UseItemOn have no encoded
rotation and leave both values unchanged. The listener never rewrites packets.

Incoming PlayerPosition and PlayerRotation always update both values, including
cancelled corrections. Relative yaw/pitch resolve against **actual** server
rotation, with no yaw wrap and pitch clamped to ±90. PlayerPosition reads the
pinned Y_ROT=8 and X_ROT=16 flags, checked against the real protocol constants in
the codec fixture. Other position/delta flags do not become rotation flags.
IncomingReplay bypasses the original pipeline and tracker.

`test-rotation-packets.py` generates an independent crate that compiles the
production listener, manager and pipeline together with the exact patched
Session send/receive bodies and actual Horizon Pipe framing/codecs. The three
tests verify final enum replacement, accepted/cancelled outgoing writes,
explicit silent writes, both correction packet types and all relative-flag
combinations, and raw packet payload preservation. Evidence is recorded in a
fresh `build/rotation-packets-*/report.json` directory.

## AutoRespawn

`liquidbounce_behaviors::auto_respawn::AutoRespawn` implements the pinned
parallel, undispatched ScreenEvent sequence. `Delay` is an unmodified `i32`,
default zero; `0..20` is UI metadata, as `RangedValue`/`ConfigSystem` do not
clamp imported Ints. Negative/zero delays act immediately; positive values
capture their current wait length, including values above 20.

Apply `respawn-patch.py` cumulatively and include its returned hook fragment.
On `client_death_screen_opening`, call `screen_event(running, true)` and perform
`host.client_respawn()` if it returns true. The native event is at the actual
death-screen opening before the screen field assignment. Like MixinGui's
setScreen HEAD event, it does not cancel the outer operation: a zero-delay
respawn can clear the old screen before the outer DeathScreen assignment
continues. A saved death installs the actual player before this opening event.
Hidden death screens and repeated health packets while already dead do not
manufacture new screen events.

At each actual client game tick, before new sequences are started, call
`game_tick(running)`. For each returned continuation, freshly query
`host.client_is_death_screen()` and call `host.client_respawn()` only while it
is true; one same-tick continuation can close the screen before the next one
resumes. Keep advancing waits while in menus/out of game. With no `requires`,
the original checks `running` when a wait resumes: `game_tick(false)` discards
only waits due on that tick. Call `on_disabled()` only for the registry's real
`CancelSequences` lifecycle callback. ClientModule.onToggled returns before
ToggleableValueGroup outside `inGame`, so an out-of-game enabled-value edit does
not immediately remove its coroutine scope. A non-death screen does not cancel
waits early.

`build/auto-respawn-01/report.json` passed two tests and Clippy with warnings
denied. The fixture compiles production behavior, exact death-opening and
identity/query/respawn bodies, actual `respawn_player`, `remote_respawn`,
`Game.send` and transformed `Session.send` with real framed Horizon Pipe IO.
It covers raw Int imports, captured/parallel waits, resume-time cancellation,
manual screen changes, ordinary mutation/cancellation of the respawn packet,
saved deaths, hidden death screens and local-player identity guards. UI/world
plumbing is stubbed; full-client/game integration is recorded separately.

## AutoFish

`auto_fish.rs` ports ModuleAutoFish's exclusive DROP tick coroutine. The
dynamic `requires` refreshes the main/offhand fishing rod each waiting tick;
no original rod identity is retained. The trigger always suspends at least one
tick. Reel, recast and autocast ranges retain raw imported Ints, including
negative values and empty ranges. Zero waits continue immediately, negative
waits resume next tick, and an empty range terminates only that coroutine.
Recast settings are read after actual item use, allowing synchronous callbacks
to change them. A zero-delay recast retains the last required hand even if
the first use consumed the rod. Autocast does not recheck the hook after its
delay. Guard interruption preserves caughtFish; actual CancelSequences clears
the stage, and actual OnDisabled also clears caughtFish. Out-of-game enabled
edits skip these lifecycle callbacks, matching ClientModule.onToggled.

`fishing-patch.py` applies cumulatively and returns its production hook
fragment and `fish_bridge.rs`. The reader-safe snapshot commits after actual
local and remote hook creation, movement and removal. It rejects guest,
server-copy and replay state without requiring client_side. Native temporary
hook/player ownership changes never replace the real local snapshot. Remote
snapshots include a nonzero active session and the loaded Builtin registry.
Disconnect and level transitions clear them. AutoFish's priority-zero ordinary
Sound PacketEvent listener reads the final mutated packet even if cancelled;
silent/replay deliveries do not trigger it. The reader holds no Game pointer,
actor request or registry lock. Sound coordinates use the protocol's eighths.

Sound identity follows MappedRegistry's identity-key lookup. A direct inline
holder does not match a registered sound with the same name. An unknown
imported registry identifier becomes the original comparator's null key and
can match any unregistered inline sound. `auto_fish_sounds.rs` handles this
selection, accepts only Gson string/number tokens, resolves empty namespaces
to minecraft, and serializes with Identifier's path-before-namespace ordering.
Null/bool tokens reject the complete leaf. Float distance multiplies as f32
before the f64 distance comparison and retains imported negative/nonfinite
runtime values. Original Gson still rejects nonfinite serialization.

The shared runtime registers AutoFish in Player, synchronizes its running
snapshot during registry/world lifecycle, advances waits before GameTick
rotation updates, releases every state lock before actual item use, resumes
after-use code, and only then starts the new tick handler. Each use reads the
current managed rotation or actual player angles. Native use follows the
ordinary prediction/send path, temporarily aligning player rotation for
prediction, restoring it, then sending the UseItem packet; no Punch is added.
The actor exposes the original settings order and ranges. Browser requests
snapshot actual sound_event names through Host outside the state lock,
including initial config loading and each subsequent request; sounds and
sound_event registry endpoints expose these loaded names. Sound registry
icons currently use null because item-texture delivery is not implemented.

`build/auto-fish-03/report.json` passed five tests and Clippy with warnings
denied. It compiles production AutoFish, registry adapter, packet listener and
snapshot; exact native identity/queries/use/rotation; exact copied remote item
prediction and committed remote-hook bodies; actual Horizon item stacks,
cooldowns and framed Pipe transport. It consumes and validates the production
returned hook fragment and records hashes for all production extras. Terrain,
local fishing physics, item resources and builtin file loading are explicit
fixture stubs; full client/game checks are separate. The config crate also
passed 12 tests and Clippy after shared integration. The actor adds original
UI/file roundtrip, raw import, Gson token and lifecycle regression coverage.

## NoRotateSet

`no_rotate_set.rs` ports the default SilentAccept and ResetRotation modes.
Accepted corrections run vanilla and send their original acknowledgements
before restoring the previous angles plus `0.000001f32`. Position packets
check module/loading state at return. Rotation packets capture that state
before vanilla, retaining the restoration if an acknowledgement callback
disables the module. Native local-player/server/replay guards remain in the
actual entry helpers. Restoring calls the finite angle setters and synchronizes
the actual `rope_rot_old` and model old angles, matching Entity.setOldRot.

ResetRotation requests a normalized target with inventory consideration and
NOT_IMPORTANT priority -20. The full noncombat Rotations tree includes Linear,
Sigmoid and Acceleration, MovementCorrection, ResetThreshold and TicksUntilReset.
All nested acceleration toggles, raw ranges, defaults and the historical
`PitchAccelelation` alias are retained. A target captures the selected processor
object's identity. Its later steps and reset read that captured processor's live
settings, even after AngleSmooth selects another mode. The shared rotation
manager preserves the existing default Sprint API and adds `request_processed`
and `update_with` for these persistent processor identities.

ChangeLook runs before actual frame camera extraction. It uses float Math.fma
interpolation without wrapping yaw, then normalizes against current managed
rotation. Applying it snapshots both old angles and yBob before changing the
player. MouseRotationEvent runs only on the actual mouse path after sensitivity
and zoom scaling and before Entity.turn; controller turns retain their existing
path. It shifts current, last and last-model endpoints, using modern float
Entity.turn multiplication and clamping managed pitch to +/-90.

`no-rotate-patch.py` returns a separate production fragment with four unit
dispatch hooks and one read-only old-rotation helper. The actual correction and
frame write helpers are distinct from the API targets so the generator sees
every event parameter and mutable output. `build/no-rotate-03/report.json`
passed five tests and Clippy with warnings denied. It uses the real
`horizon_mod_api_gen` parser and compiles the exact host dispatch bodies against
the generated API. Tests also compile actual remote teleport/rotation and
player_sync bodies with framed Pipe acknowledgement transport. The arithmetic
oracle runs actual JDK RandomGenerator and Kotlin stdlib range operations for
five steps of each smoother and acceleration deceleration. Actor execution,
terrain and peripheral services are fixture stubs; full client/game integration
is validated separately.

`build/no-rotate-config-02/report.json` passed 14 settings tests and Clippy,
including original tree ordering, all mode choices, finite raw range imports,
numeric strings, exact mode versus case-insensitive enum tags, old aliases,
partial import continuation and nonfinite serialization rejection. That report
uses production config source snapshots with an explicit registration shim
where the shared engine had not yet registered the new module.

## Twerk

`twerk.rs` ports the complete Fun module's sole original Delay value: default
2, UI range 1..20, empty suffix, with signed raw imported Int values retained.
Its priority-10 MovementInputEvent handler directly overwrites sneak using
`tickCount % (delay * 2) < delay`. The product wraps as Java Int and the signed
remainder follows Java, including negative ages. A wrapped zero divisor
returns a per-handler error before changing sneak; later handlers still run.
The default-priority NoBlockInteract handler therefore follows Twerk and can
force sneak true. Disabling Twerk retains physical input and owns no clock.

`twerk-patch.py` adds only the read-only `client_player_input_tick_count`
helper. It reads the actual current local player each time and rejects LAN
guests, server copies and prediction replays. Horizon constructs directional
input and the packet flags before its ordinary local tick advances the signed
entity age. The helper returns `health.tick_count.wrapping_add(1)` to match
the imminent entity tick's KeyboardInput phase. Dead/riding branches likewise
advance the clock after input; unloaded chunks return before input. The pinned
Horizon sleeping branch does not advance this clock, a source limitation of
that branch rather than a separate Twerk timer.

`build/twerk-02/report.json` passed nine tests and Clippy with warnings denied.
It compares 312 actual JVM arithmetic vectors for both incoming sneak values,
compiles the exact native helper/identity, damage-clock body and remote input
bit assembly, and uses actual full `horizon_mod_api_gen::generate` output for
both the Host helper implementation and event dispatcher. Priority composition
uses production Twerk and NoBlockInteract with a fixture runtime. Player/world
storage and registry orchestration are explicit stubs; full client/game
integration is checked separately.

`build/twerk-config-02/report.json` passed 23 settings tests and Clippy. Its
production source snapshot verifies declaration order, original interop and
file shapes, integer truncation/wrapping, numeric strings, raw out-of-range
values, first duplicates, per-leaf failure continuation and RuntimeRequest
forwarding. It applies an explicit registration shim only when shared config
has not registered Twerk yet. English, Simplified Chinese and Traditional
Chinese Delay help are supplied by the existing localization layer.

## AutoWeapon

`auto_weapon.rs` ports all six values in their original declaration order:
Preferred (ten weapon types), Priority (Default/Knockback/Damage/AttackSpeed),
AutoShieldBreak, AutoMace, SwitchBack and ChangeOn (OnAttack/OnTarget).
The UI exposes the original defaults, choices, ranges and suffix. Imported
SwitchBack retains the original signed Int without clamping. Gson-style
multi-choice imports clear the live set first and retain a successfully read
prefix on failure; unknown enum choices are ignored. Invalid Priority leaves
its previous value and does not stop later settings from loading.

Weapon selection preserves generic-before-specialized facet order, duplicate
facets, the asymmetric MaceItemFacet comparison, Sharpness 5..99's SharpAxe
branch and the GodAxe branch at 100. Mace smash takes precedence over shield
breaking. Priority first finds the best raw stack by object identity; if that
stack supplies no WeaponItemFacet the result stays empty. It does not fall
back to the next-best item. Floating-point comparisons use Java's NaN and
signed-zero ordering. The actual player base attributes, component modifier
declaration order, slot groups, sanitization, enchantment registry and levels,
item tags, durability and enchantability supply the native item snapshots.

`item-identity-patch.py` gives each newly constructed ItemStack an opaque
allocation identity. Rust snapshots and moves retain it; menu splitting,
inventory distribution, creative copying and bundle/container copying
allocate another identity. Derived value/component equality, wire reuse and
NBT/protocol fields exclude it. The final identity-hash tie-break is arbitrary
per process, as JVM identity hashes are; Rust and JVM allocations do not share
one process and cannot promise the same final tie-break number.

`auto-weapon-patch.py` dispatches `client_attack_entity(target, living,
would_block_hit, cancel)` before the ordinary carried sync and attack packet,
or before local LivingEntity damage. The older post-attack hook remains a
separate event. Remote spears' piercing-attack operation does not call this
MultiPlayerGameMode.attack event. Local nonliving attack paths are not all
covered by this seam; AutoWeapon itself rejects those entities. The shield
query reads the actual used stack, use delay, damage-type bypass and reduction
components, head yaw and target-relative horizontal angle. Remote equipment
retains the full offhand stack so custom shield components reach this query.

The runtime must compute a Switch while holding its module state and release
that lock before `client_select_slot_silently` and `client_ensure_carried`.
Carried synchronization sends a real packet and can reenter PacketEvent.
OnAttack always calls ensure after a selection request, including a cancelled
selection event. OnTarget requests the slot without immediate sync, and runs
only when another combat module calls that service; no crosshair tick polling
is invented. Disable resets only AutoWeapon's SilentHotbar requester.

`client_attack_speed(original, speed)` wraps the real attack-delay attribute
expression once on each local/remote path. It reads a snapshot without
materializing or editing lazy attributes. The original Reach-running bypass,
OnAttack/cooldown guards and the absence of a busy guard are retained.
`Snapshot::context` accepts the actual AutoBuff slot-ownership and MaceKill
enabled inputs. These modules are not yet registered; those inputs therefore
remain false until their implementations are connected. AutoArmor's unported
SkipCursed setting has its original empty default; its future shared policy
must supply the armour exclusion when that module is added. These boundaries
are not presented as completed implementations of those dependent modules.

`build/auto-weapon-04/report.json` passed six tests, 384 JVM arithmetic vectors
and Clippy with warnings denied, including native snapshot decoding and
effective-hand/policy input preservation. `build/auto-weapon-config-03/report.json`
passed 32 production config tests and Clippy. The config fixture applies an
explicit registration shim only if the shared engine has not registered the
new module. Full native API and client compilation are validated separately.

`build/auto-weapon-native-04/report.json` passed the actual Horizon API generator,
the complete `horizon-client --lib` compilation and the real `horizon-items`
identity/NBT test in an independent pinned-source workspace. This includes the
final builtin shield classification and single-dispatch attack-speed fallback.
This fixture does not claim runtime module registration or game execution.

The complete AntiAFK implementation covers Old, RandomInteraction's six
interactions and independent Custom Swing/Rotate sequences, actual logical
Forward key state, pending mode-owned once jumps, and its full noncombat
Rotations tree. `build/anti-afk-03/report.json` passed seven production tests,
256 real JDK25 mixed random-sequence vectors and Clippy with warnings denied.
`build/anti-afk-config-04/report.json` passed 37 production config tests and
Clippy, including changed-mode cancellation outside a world.
`build/anti-afk-native-02/report.json` passed actual API generation and full
`horizon-client --lib` compilation in an independent pinned-source workspace.
The fixture confirms native helpers compile; shared registration and actual
game execution remain separate integration checks.

Derp's five Yaw and three Pitch modes retain both stateful coroutine modes,
both Sprint guards, SafePitch, raw setting imports and its private fixed
rotation processor. `build/derp-source-02/report.json` compares 150 scripted
ticks against the unchanged Kotlin module using real continuations and JDK25
randomness, matching raw angle bits and draw counts. `build/derp-02/report.json`
and `build/derp-config-03/report.json` passed production behavior/config tests
and Clippy. `build/derp-native-01/report.json` passed the real API generator and
complete Horizon client library compilation. `DERP-INTEGRATION.md` records the
remaining shared scheduling/registration and game-execution boundaries.

