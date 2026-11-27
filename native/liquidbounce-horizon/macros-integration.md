# Macros and shared CommandManager foundation

Source is LiquidBounce ae0eb396 `features/module/modules/misc/ModuleMacros.kt`,
`SingleItemStackPickMode`, `SilentHotbar`, `InteractionUtils.useItem`,
`BlockExtensions.doPlacement`, `SwingMode`, `RefreshableRangeValue`, and the
Main dispatcher/suspend event scope. Macros is disabled by default in Misc.
Its source `doNotIncludeAlways` concerns optional autoconfig inclusion; the
autoconfig command is not yet connected and this port does not claim it is.

All five Chat and five Item groups retain their source order and settings.
TriggerDelay defaults to 0..0 ms (UI 0..15000), Trigger to the unknown key.
Chat Delay is 0..0 ms (UI 0..5000), Messages empty, UseClientCommand false.
Item defaults are UseOnly, Name/Paper/Equals, registry item minecraft:paper,
DoNotHide, and HoldTime 1..1 ticks (UI 1..20). Both interaction swing choices
and all eight exact/case-insensitive name matching choices are retained.
No source range is clamped to its UI bounds. Invalid empty random ranges
retain imported values and fail their sequence rather than panicking through
the native callback.

The actual keyboard path invokes Macros after ordinary module binds, matching
KeyboardKeyEvent before KeyEvent. Press, repeat and release are all accepted;
mouse events are not routed here. Native and browser screens prevent a new
macro event. Each event captures one monotonic timestamp and uses strict
elapsed > sampled TriggerDelay. lastTriggerTime advances before refresh and
is retained across disable. Each eligible macro samples a new trigger delay.
Each item reobserves the actual inventory, with offhand before hotbar 0..8,
including explicit air/empty slots. It selects its own silent requester and
immediately executes; there is no invented one-tick delay.

Chat sequences can overlap across events. They retain their original message
list iterator while reading current UseClientCommand, prefix and delay at the
source execution points. Every message, including the final message, is
followed by random real-time delay. A separate native Main wake resumes work
between ticks. Human text setters replace a list without altering a captured
iterator; JSON imports mutate the current list and preserve Java's
hasNext/ConcurrentModificationException boundary. A child error cancels its
same-event siblings; later events remain independent. Disable cancels tasks
and resets only the Macros requester, preserving cooldown/random state.

UseOnly invokes the existing genuine interaction utility with managed current
rotation or the player's current rotation. Empty hands still predict/send
UseItem. PlaceOrUse selects before the hit-result cast, retains a miss
BlockHitResult, skips an actual entity hit, calls useItemOn, stops on Fail,
and falls back to useItem only for Pass with an originally nonempty hand.
The direct call does not add startUseItem reach, hands-busy or block-breaking
checks. Local predicted swings are scoped per macro call; HideForClient
suppresses that animation, never sends a Punch packet, and does not change
ordinary swings. Equip height follows the successful result/changed original
stack-or-creative rule. This uses the existing native remote/local prediction
implementations, not fake packet acknowledgements.

The shared service is `client_commands::Settings` in BehaviorModules. Global
CommandManager (alias Commands for config import) exposes real Prefix `.` and
HintCount 5 (UI 0..10) with file persistence and value events. Public runtime
seams are `client_command_prefix`, `client_command_execute(host, body) ->
Result<(), String>`, and `client_command_message(host, full_message) -> bool`.
Future modules should call the same execute seam. Manual prefixed chat is
intercepted before network chat, uses the current prefix and writes body plus
newline to command_history.txt even after command errors. Direct macro
execution bypasses history and propagates errors. Old fixed-prefix
AutoAccount/AutoDisable chat interceptions are removed by the native patch;
their genuine execution functions remain available.

The shared lexer retains ASCII-space normalization outside double quotes,
case-insensitive command paths with argument case unchanged, literal word
arguments, quoted string arguments with single/double quotes and strict
backslash escapes, and greedy text. Unknown commands report an error with
Levenshtein hints; they never become server chat or a successful no-op.

Connected real execution branches:

- toggle/t changes the actual module enabled state through lifecycle callbacks.
- bind and binds add/remove/clear/list modify the actual registry binds.
  Optional action/modifiers inherit current values; none resets bind only
  through bind. bind modifiers use whitespace; binds add uses comma selection.
  MultiSelect silently filters unknown/ineligible entries and rejects an empty
  result. Lists use source page size 8 and clamp a valid requested page.
- hide hide/unhide/clear/list changes actual persisted Hidden values and uses
  the source eligibility predicate.
- value set/reset/reset-all addresses actual module/CommandManager paths,
  applies only the selected leaf or group, restores nested inactive modes and
  preserves Bind during reset-all. Text/bool/int/float/range/enum/mode/key/bind,
  text lists and item-registry lists are connected. Other component/registry
  kinds fail explicitly until their live deserializers are connected.
- friend add/remove/alias/clear/list uses the shared real FriendBook with
  exact case-sensitive names, UTF-16 order and friends.json persistence.
- autoaccount login/register uses the existing real sender and ingame guard.
- autodisable add/remove/list/clear uses the real selected module set and
  source failure boundaries, with the existing clear implementation bridged.
- say sends genuine chat even for a leading slash; macro normal messages use
  sendChatOrCommand, preserving that source distinction. help lists only the
  connected command roots.

The complete 44 built-in registrar inventory, exact root names/aliases/source
files, and each unconnected branch are recorded separately in
`macros-command-inventory.md`. This foundation is not the complete upstream
Brigadier tree. Addon registration, full command completion, structured
click/hover/error chat metadata, and the unconnected built-ins remain open;
unsupported client commands always return explicit errors. These are command
foundation limits, not placeholder behavior for the Macros item/chat paths.

Owned files: behavior `macros.rs`, `client_commands.rs`; runtime/config files
with those names; `macros-host.rs`, `macros-patch.py`, `integrate-macros.py`,
`macros-smoke.rs`, `test-macros.py`, this document, the command inventory, and
`scripts/update-macros-localization.py`. The updater supplies all ten group
names, every leaf label/description, and CommandManager labels in Simplified
and Traditional Chinese, selected by the actual game language.

Integration owner applies once:

```powershell
python native/liquidbounce-horizon/integrate-macros.py --apply
python scripts/update-macros-localization.py
```

The shared plan validates 23 changes across 7 files, depends on AutoQueue and
the now-integrated BlinkManager preserved-global anchors, and is intentionally
non-idempotent. The native patch changes 6 files in memory. Focused smoke
`python native/liquidbounce-horizon/test-macros.py` passed one combined
behavior/lexer/native-scheduler test. The host fragment parsed through
rustfmt. Only this short check was run (an assertion typo and a concurrently
changed BlinkManager anchor were corrected before its final successful run).
No Cargo, shared integration apply, staging write or official build was run
by this owner. Native/engine type checks belong to the parent batch.
Full-client differential: **not_run**.
