# ReportHelper source port

Source: LiquidBounce ae0eb396 `ModuleReportHelper`, `ReportHelperAutoReport`,
`ReportHelperAutoConfirm`, `CoroutineTicker`, `MixinChatListener` and
`ContainerItemSlot`. Category Player; alias AutoReport. AutoReport and
AutoConfirm are both disabled by default. AutoConfirm choices are Hypixel
and Heypixel. All original settings serialize through file/import and Interop
trees, with Simplified and Traditional Chinese labels/descriptions.

AutoReport uses the actual accepted/delayed ChatReceive boundary already used
by AutoAccount, source zero only. Signed player chat uses its decorated line;
disguised chat uses the original undecorated component; system messages use
the packet component. Title/subtitle do not report. `world.players()` order
is maintained at real player AddEntity/replacement, RemoveEntities and world
clear, retaining player entity profile names after tab-list removal. Native
LAN guest objects supplement the legacy in-process world. No combat-target
or global Targets filter is used. FriendManager names remain exact case.

Chance is consumed before the case-insensitive reported-set duplicate check;
delay is captured after successful insertion. Each parallel sequence has an
individual pending callback in the shared FIFO CoroutineTicker. Delay zero
sends synchronously; resuming uses the current command pattern. The one String
argument supports Java Formatter argument indexing/reuse, general String,
boolean and hash conversions, width, precision, uppercase, percent/newline;
invalid or incompatible formats terminate that sequence without sending.
Session identity comes from the actual AuthSession object (Horizon installs
it once at launch); a changed object clears the reported set. Disable,
disconnect and world teardown clear pending continuations.

AutoConfirm receives the real prospective container ScreenEvent at the
existing GUICloser setScreen HEAD. It does not poll to fabricate events.
Each event gets a unique token; only the accepted actual screen replacement
binds that token as current. A cancelled screen and a later same-kind screen
therefore remain different objects. The existing inventory screen epoch still
observes every real replacement and excludes temporary mapping contexts.

The source `tickConditional(5)` first increments elapsed ticks, then checks
whether the captured object is current. An identity match on ticks 1–4 resumes
inspection; tick 5 always terminates. The captured screen's retained GUICloser
menu view supplies non-player slots and syncId, including when a new player
menu was constructed but its ScreenEvent was cancelled. Hypixel checks exactly
27 slots with the exact air/green terracotta/player head/red terracotta layout.
Heypixel checks exactly 9 and clicks the first diamond sword's actual menu slot.
The genuine native menu predictor sends ordinary Pickup input. Closing uses
`clientSideCloseContainer`, proven by `research/LocalPlayer-26.3.javap.txt`:
it resets the client menu and screen without a ContainerClose packet.

Apply once, from the integration owner:

```powershell
python native/liquidbounce-horizon/integrate-report-helper.py --apply
python scripts/update-report-helper-localization.py
```

Focused smoke: `python native/liquidbounce-horizon/test-report-helper.py`.
Full-client differential: **not_run**. Official aggregate native/engine checks
belong to the parent integration batch. No full-client or repeated broad tests
are implied by the focused behavior smoke.
