# Criticals

Source: pinned LiquidBounce ae0eb396 `ModuleCriticals`, all six nonempty modes,
`CriticalsJump`, `BlinkManager` and Freeze Queue. This implements the seven mode
choices None / Packet / NoGround / Jump / Blink / Timer / Rewound.

## Integration owner

Run once from the project root. This Plan deliberately rejects repeated apply:

```powershell
python native/liquidbounce-horizon/integrate-criticals.py --apply
python scripts/update-criticals-localization.py
```

The checked Plan has 38 transformations across nine shared files. The native
extension runs after crosshair prediction and before Step's MotionWorld rewrite.
It adds a real borrowed player snapshot to ordinary nested packet dispatch,
actual movement packet generation, target candidates, critical/magic tracking
emitters and a receive probe at `poll_remote` entry.

The AttackEntity order is WhenSprinting at priority 500, AutoWeapon selection
and synchronization, Criticals Packet/Visuals, SuperKnockback, then MaceKill.
The ordinary Criticals context is sampled again after AutoWeapon. Packet sends
and native Host calls occur with actor and Criticals locks released.

## Behavior implemented

- Every source setting, default, mode tree, enum alias and UI range, including
  all Packet offsets and all four MovePacketTypes. Non-position packets omit
  the y modifier and retain their source onGround flag behavior.
- NoGround ordinary mutation observes already canceled events. Rewound is a
  separate mutating listener at READ_FINAL_STATE (-1000), also observing
  canceled events; it resets only on activation and preserves hasPos behavior.
- Jump uses real MovementInput and PlayerJump events, exact f32 normal-jump
  comparison, source module checks, attackable entity boxes and optional
  COLLIDER/no-fluid line of sight. Optimization reuses the real cloned-body
  FallingPlayer kernel, with the source 0.42 initial velocity for this check.
- Smart crosshair selection reads configured Jump height even while Criticals
  is disabled, the pending automatic-jump bit, and live Blink state.
- Timer requests the shared Timer provider at priority 20 for one tick.
- INPUT / MOVEMENT_TICK / NETWORK sprint sources and OnAttack's wasSprinting
  behavior. Native OnAttack sends stop-sprinting and clears wasSprinting only.
- Native particle emitters, including actual entity following for mobs, LAN
  guests, armor stands and the dragon. Packet and generic Visuals listeners
  each emit independently when both qualify.
- Blink shares one timestamped, session-tagged incoming/outgoing queue with
  Freeze Queue. Criticals' normal Blink action runs first; Freeze's
  SAFETY_FEATURE (-50) action can override it. Manager priority is -500, before
  Rewound. Chat exceptions, packet PASS classes, canceled-event guard, frame
  probes, incoming state clearing, delayed nextDelay sampling and the source
  timeout early-return state are retained. Queued packet clones intentionally
  precede READ_FINAL_STATE mutation, matching the source event order.

Freeze's previous private Queue branches are removed by the integrator and its
ordinary Cancel/Stationary listener is registered at priority 0. Its existing
notification/hold/disable lifecycle remains active. Incoming replay uses the
existing game-actor replay facility; incoming flush readiness is drained before
`poll_remote`. Transport-owned configuration/transfer packets remain excluded,
as in the existing replay contract. Reader-thread flushes can wait until the
next receive probe; exact cross-thread packet-order parity has not been proven.
Outgoing callbacks lacking a nested sender retain ready packets for frame flush
rather than dropping them.

## Validation

- `integrate-criticals.py --check`: passed, 38 changes / nine shared files.
- `test-criticals.py`: one focused Rust test passed. It covers all Packet
  offsets, sprint/ground/entity gates, Rewound activation and tick reset,
  automatic jump adjustment and honey jump preservation, cooldown/landing
  optimization, sprint source differences and Blink early return/session data.
- The same smoke checked the seven native extension files in memory against
  the staged anchors. It wrote no staging files. Rustfmt parsed the new Rust
  files and Python parsing passed.
- Aggregate compilation: parent integration owner performs the official Check.
- Full-client differential: **not_run**. Runtime/network ordering and rendered
  particle parity remain for the final unified validation. KillAura and Fly
  are not registered yet; their source checks currently read false. NoSlow is
  not registered, so prediction keeps current baseline item/sneak multipliers.

Do not repeat Clicker, SpearKill or the Criticals smoke without a new failure
that requires it. No Cargo or client launch was performed by this owner.
