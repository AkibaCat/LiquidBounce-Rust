# GUICloser integration handoff

Frozen owned implementation: 2026-11-26. The accompanying
`gui-closer-freeze-02.json` records current exact hashes and the predecessor
`gui-closer-freeze.json` digest; shared registration and aggregate
generation are owned by the root integrator.

## Source contract

The unchanged reference is LiquidBounce
`ae0eb396c07f22383789b28d5912d714c6ca0a86`,
`ModuleGUICloser.kt`, with the ScreenEvent dispatched at the head of
`MixinGui.setScreen`. Horizon is pinned to
`9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Horizon-Mixins to
`5a64af938ec86b00d53d16a408aabc0038fb3e57`.

GUICloser is a Misc module, with aliases AutoClose/ContainerCloser and suffix
guiCloser. Matches is the default and tests the complete title; Contains uses
Java regex search. The ordered, nullable list defaults to `^Vote$`. A null
before a match aborts the handler without cancellation or printing; an earlier
match short circuits it. Disabled/non-container events do nothing. Matching
prevents the prospective container Screen from opening. An unmatched title is
printed only when PrintScreenTitle is enabled. PrintScreenTitle is included in
File/Interop serialization and excluded only from Public serialization.

The configuration parser retains order, duplicates and null elements, accepts
a scalar as a singleton and replaces a filter list atomically. Invalid regex
replacement leaves the complete old list intact while independent settings
may apply. The narrow vendored Java regex API uses JDK25-observed Unicode,
case/name/property/grapheme data and UTF16 matching semantics; finite oracle
coverage does not establish universal Java Pattern equivalence. Original
crate notices and the local changes are recorded in vendor PROVENANCE.json.

## Screen and menu separation

The screen event precedes old-screen lifecycle, replacement, native-screen
initialization, capture, use-key release and recipe-screen reset. Supported
remote menu metadata and backing entities are constructed first, matching
vanilla's newly assigned player.containerMenu even when setScreen cancels.

Horizon stores both the visible screen menu and player menu on Game. The
retained ContainerView therefore moves only the old screen's owned station,
mount and native-screen objects and retains its carried/crafting/container
state and remote metadata. Its remote entity copies include both chest halves
and remain independent when the next menu reuses a synthetic block key.
Normal Game fields hold the active new player menu between UI consumers.
Remote, socket, state_id, packet session and shared player inventory are never
swapped or rolled back.

HUD, window/pad input, enchanting-book animation and cursor geometry use the
retained screen view. Network packets and inventory Host snapshot, click,
creative, use and close expose the active menu, including reentrant calls
from a native-screen UI scope. The Host snapshot keeps active menu/stacks/
carried with the visible old screen's screen/geometry metadata. Stale visible
menu IDs cannot operate the active menu; same-ID validation follows vanilla.
Two temporary inventory emulations remain outside screen cancellation and
lifecycle observation.

Capture is idempotent across repeated cancellation and nested active calls.
An abandoned constructor restores moved objects. Accepted native providers
can still fail before actual Screen assignment, so acceptance alone does not
discard the retained view. Actual lifecycle serial change invalidates it,
including replacement by the same Screen enum.

Recipe-screen UI state remains on the current Screen, while synced recipe
cache updates remain active packet state. Actual MC26.3
ClientPacketListener.handlePlaceRecipe bytecode validates the new active
container ID, then calls the current Screen's RecipeUpdateListener. The
adapter does that validation before entering the retained UI scope. Evidence
is `build/gui-closer-client-packet-listener.txt`; no additional protocol recipe
payload was invented.

Retained titles preserve their components and are reparsed through Horizon's
component reader under the current Language. Text.plain alone reads already
resolved spans and is insufficient after a language change. Local default
container titles retain translation keys. AutoF5 reads the retained visible
title; prospective GUICloser decisions read the new active menu title. The
frozen AutoF5 source files were not changed.

Notification uses the original gradient LiquidBounce tag, gray body, bold
blue RGB 0x4677ff title, CopyToClipboard action and chat.copy.click hover. The
user's Chinese-display requirement localizes its body to 容器界面标题 /
容器介面標題. Complete setting help is merged into the Simplified/Traditional
display dictionaries; the freeze records the GUICloser subset only.

## Evidence at handoff

* `build/gui-closer-source-16/report.json`: unchanged Kotlin handler against
  actual JDK25, 45,371 behavior/regex vectors, three tests and Clippy.
* `build/gui-closer-source-consistency-01/report.json`: behavior and 23 vendor
  files are byte-identical to authoritative source16 (24 compared).
* `build/gui-closer-jdk-data-02/report.json`: JDK25-observed data generation.
* `build/gui-closer-config-04/report.json`: complete config/behavior snapshot,
  81 tests and Clippy, without a registration shim.
* `build/gui-closer-config-source-03/report.json`: 23 actual Gson/JDK25 and
  unchanged serializer/ListValue/adapter vectors.
* `build/gui-closer-packaging-02/report.json`: actual staging reader and legacy
  source package retain all 24 vendor files; real package validator passes.
* `build/gui-closer-current-03/report.json`: latest cumulative transformations
  only, used by the following exact-body tests; no Cargo/game claim.
* `build/gui-closer-view-05/report.json`: exact production context source,
  actual Horizon Text/component/Language, 11 recording Game/World/menu tests
  and Clippy. Includes nested failed construction, provider failure before
  lifecycle, recipe UI/active state separation and language change.
* `build/gui-closer-titles-07/report.json`: ten tests of final title bodies,
  original complete AutoF5 title enumeration, actual EN/SC/TC resources and
  generated Host API, with explicit title/Game providers.
* `build/gui-closer-consumption-09/report.json`: eight tests and Clippy of the
  exact host helper and transformed real screen boundaries; assertions cover
  all 14 entry points and temporary inventory exclusions.

Historical evidence is preserved. Native05 compiled a complete concrete
Horizon client before retained-view isolation; it is not proof of the final
context implementation. Native06/07 failed preparation anchors; native08
failed wrapper syntax during fill_hooks; native09 passed fill_hooks then
failed in the render-generator stale check before concrete Game compilation.
View03 exposed the real stale translated-span issue and is preserved as a
failure. View01's tests passed but its recording u32 stack triggered Clippy's
clone_on_copy; that allow is fixture-only.

Root production76-02 reached concrete Game compilation and exposed E0624:
the new gui_closer_view sibling called the private native_hooks helper
client_container_menu_text. Revision02 gives that generated helper pub(super)
visibility and reruns the three focused suites above. The failed production
report and original freeze manifest are preserved; this correction awaits
root's next cumulative concrete Game build.

The final context implementation still requires root's cumulative concrete
Game/engine compilation, registry/config actor integration and actual game /
GPU / TestEngine verification. No fixture or compile check here is described
as game execution. Root has registered the config actor test
`gui_closer_aliases_atomic_nullable_filter_and_file_interop_roundtrip` and owns
the clientGuiCloserScreen runtime scenario. This handoff does not claim that
actor test has passed in the actual engine yet.
