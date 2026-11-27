# KillAura integration handoff

Pinned LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon
`9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Minecraft 26.3.

The owned implementation is frozen for the root's next combined batch. No
shared source, upstream, production locale or native staging was changed by this
handoff. `integrate-kill-aura.py --check` passed against shared count 177.

## Serial application

From the repository root:

```powershell
python native/liquidbounce-horizon/integrate-kill-aura.py --check
python native/liquidbounce-horizon/integrate-kill-aura.py
python native/liquidbounce-horizon/update-kill-aura-locales.py
```

Then apply SwordBlock and the other owned integrators in the same batch, regenerate
the complete behavior patch and stage through the root's official workflow. The
KillAura integrator edits 13 shared files transactionally, including the generator.
The final generator tail is KillAura, block input, FightBot, Step; SwordBlock may
insert immediately after block input. `kill-aura-extra-patch.py` is invoked inside
the base KillAura patch and must not be added separately to the generator.

Provider/processor: KillAura `24 / 340`, between Extinguish `23 / 320` and
AutoShoot `25 / 360`.

## Owned implementation

- `native/liquidbounce-behaviors/src/kill_aura.rs`, `kill_aura_rotation.rs`.
  Their lib registration already exists; do not register them twice.
- `engine/src/kill_aura_runtime.rs`, `engine/src/config/kill_aura.rs`.
- `kill-aura-native.rs`, `kill-aura-host.rs`, `kill-aura-local-forecast.rs`,
  `kill-aura-patch.py`, `integrate-kill-aura.py`.
- `kill-aura-extra-candidates.rs`, `kill-aura-extra-attack.rs`,
  `kill-aura-extra-patch.py`, `kill-aura-extra-evidence.json`,
  `test-kill-aura-extra.py`.
- `kill-aura-block-input-host.rs`, `kill-aura-block-input-patch.py`,
  `test-kill-aura-block-input.py`, `kill-aura-block-input-evidence.json`.
- `fightbot-navigation-patch.py`, `fightbot-navigation-state.rs`,
  `fightbot-navigation-runtime.rs`, `navigation-owners.rs`,
  `fightbot-navigation-integration.md`, `fightbot-navigation-evidence.json`,
  `test-fightbot-navigation.py`.
- `kill-aura-fight-goal.rs`, `kill-aura-fight-goal-smoke.rs`,
  `test-kill-aura-fight-goal.py`, `update-kill-aura-locales.py`,
  `test-kill-aura.py`.

Paths above are relative to `native/liquidbounce-horizon` except the two explicit
behavior paths. `fightbot-navigation-runtime.draft.rs` is unused.

## Implemented boundaries

TargetSelector, item attack ranges and pick margins use live entity bounds,
AimPoint, scan ordering, shared managed rotations and all rotation timing modes.
Attack scheduling includes cooldown and weapon selection, Criticals,
KeepSprint, FailSwing, inventory closing, release/reblock, main/off hand changes,
and the same Attack-key click plans as Crosshair/ProjectilePuncher. AutoShoot owns
Use and is deliberately independent. The AutoPearl consumer reads the actual
KillAura target.

AutoBlocking owns one Arc service for packet/Blink and pose snapshots. Native
input/render callbacks read its snapshot directly and never borrow actor State.
The input patch protects release only; physical input still owns initial and
repeated use. Its display pose supports first and third person without changing
the player's real use state. SwordBlock consumes the visual seam after this patch.

Native attack dispatch keeps a single client pre-attack hook. Ordinary local and
remote targets receive one client attack notification. The guest/dragon parent
helpers each own their own one client notification plus authoritative server
attack hook. LAN guests are category 12, resolved again by entity id to their
connection and routed through the native PVP strike/land/tail. A dragon parent is
attack-only category 13, retains the parent's network id and living predicate,
and delegates its local hurt to BODY (2). Natural rays retain real dragon parts
and reject every dragon parent, including the remote mob mirror. Trace Enemy
does not alias a part to the living parent's eligibility.

Special remote entity interactions (guest, crystal, dragon part) send the actual
Interact packet and return PASS. Local interactions use `interact_entity`, which
preserves specific/v1/v2 hooks. All chosen entities retain their actual block
position world-border gate. Forced pick, ignore-entity flag, attack reach,
implicit swing and tick hit caches are restored after each scoped action.

FightBot has independent Navigator state and event queues, with owner-aware
physical holds/hotbar claims. It uses real collision, goal search, terrain and
player forecasting, applies its own navigation settings, and renders the real
route. It does not enable or reconfigure AutoWalk.

Settings preserve File and Interop trees, inactive branches, navigation/target
rendering, rotation/AI fallback and finite-number checks. The legacy absolute
Range/WallRange/ScanExtraRange format migrates to the new increases. Simplified
and traditional Chinese updater covers descriptions, settings, choices,
FightBot navigation/rendering and target rendering and is idempotent. AI notices
use the game language and a five-second throttle.

Source-specific details retained: failed-hit box fading advances per render
frame (`50 * Fade`), although the original suffix says seconds. Minecraft 26.3
has no serverbound OPEN_INVENTORY command; the corresponding reopen action cannot
send a nonexistent command. Existing real screen/inventory state is preserved.

## Verification and next batch

`test-kill-aura.py` passed: 13 transactional shared anchor changes, seven complete
generated native sources parsed by rustfmt in temporary files, unique callback
registration, hit-cache scopes, direct Arc snapshot and Chinese updater
idempotence. One subsequent robustness edit rejects remote parent mob mirrors;
the changed native file was rustfmt-parsed. Block-input static composition,
FightBot static composition, six goal static checks and extra action/API static
checks also passed. These checks make no production or staging writes.

No Rust compilation or fixture execution was run by this handoff. Run the new
fixtures once alongside the root's official combined batch:

```powershell
cargo test --manifest-path native/liquidbounce-behaviors/Cargo.toml kill_aura
cargo test --manifest-path native/liquidbounce-horizon/engine/Cargo.toml --features behavior-hooks kill_aura
python native/liquidbounce-horizon/test-fightbot-navigation.py --model-only
python native/liquidbounce-horizon/test-kill-aura-fight-goal.py --model-only
```

The behavior filter has four focused cases; config has four focused cases
(roundtrip, malformed leaf isolation/aliases, nonfinite/bounds and legacy range).
Use the root's established manifest/features if its combined native environment
requires additional existing engine features. Do not rerun the broad prior
module test suite solely for this handoff.
