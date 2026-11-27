# TriggerBot / AutoClicker

Source: LiquidBounce ae0eb396 `ModuleTriggerBot`, `ModuleAutoClicker`,
`CrosshairAttacker`, `Clicker`, and `CriticalsJump`. Reuses the already verified
shared Clicker dependency and SuperKnockback CombatCooldown observer.

## Owner integration

Run once from project root; this checked Plan intentionally is not idempotent:

```powershell
python native/liquidbounce-horizon/integrate-crosshair-clicker.py --apply
python scripts/update-crosshair-clicker-localization.py
```

The Plan wires behavior export, both module registrations and config trees,
packet observer lifecycle, FIRST_PRIORITY planning, ordinary module ticks,
coroutine owners and dispatch, physical input tracking, attack-key isDown,
INPUT/MOVEMENT_TICK sprint suppression, and menu initialization before config
loads. The native extension runs after its movement dependencies and before
Step's final MotionWorld rewrite. Native client exports the Java math functions
used by the scheduler, avoiding a new engine crate dependency.

## Behavior

- Source defaults, ranges, aliases and declaration order for TriggerBot and
  AutoClicker Attack/Use. Native door/fence-gate/trapdoor registry supplies the
  default ignored blocks; imported lists remain authoritative.
- Genuine KeyMapping queues, with no direct attack/use invocation. Attack and
  Trigger coordinate on one mapping; Use remains independent. Physical mouse
  state is separate from synthesized key-down, matching InputTracker.
- Actual target filters and hit results, weapon/enchantment predicates,
  AutoWeapon onTarget before critical gating, item use Wait/Stop/Ignore,
  broken-block 300 ms delay, named special items, item/block ignore sets, and
  two-tick DelayStart continuation.
- Smart, Ignore and Always critical selection. Smart prediction uses cloned
  real PlayerMotion, loaded chunks and entity collisions, transformed input
  with jumping disabled, real effects and movement kernels. LocalPlayerHooks
  move listeners read the source body in Strafe/TerrainSpeed/Anchor order;
  deliberate source callback side effects are retained, speculative movement
  is discarded. LiquidWalk standing on liquid blocks critical selection.
- Simulated isDown applies to attack only, before AutoBreak's listener.

## Validation

- Integration `--check`: passed after checking all current shared anchors.
- `python native/liquidbounce-horizon/test-crosshair-clicker.py`: one focused
  smoke passed (one test). Checks listener construction/time sampling order,
  shared-key click coordination, physical presses replacing the next planned
  click, attack/objective/weapon gates, Wait/Stop/DelayStart continuation,
  Smart/Always critical branches and exact prediction parameters, and imported
  ignore-list preservation. An initial smoke assertion incorrectly expected
  `pressedByPlayer` to consume an already-due click; the assertion was corrected
  against the source's removal of the next future click. Behavior code did not
  change for that correction.
- Rustfmt parsed the added Rust files. No Cargo or client launch by this owner.
- Parent aggregate compilation: pending owner integration.
- Full-client differential: **not_run**. No claim of complete differential
  parity; current unported Criticals module uses source default Jump height
  0.42 and Blink state false. NoSlow is not currently registered, so native
  baseline item/sneak multipliers are used.

Do not repeat the earlier Clicker or SpearKill smokes. Parent performs the next
aggregate native check; full validation remains deferred until development ends.
