# FightBot independent Navigator adapter

Pinned source: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, MC 26.3.

`fightbot-navigation-patch.py` composes an independent instance of the existing real native Navigator, host, world and search adapters. The production `auto-walk-runtime.rs` and `auto-walk-host.rs` owned sources are not edited. The patch changes only the generated copies. FightBot never calls AutoWalk enable, mode, goal or lifecycle methods.

This adapter implements the Navigator part of `KillAuraFightBot` and `NavigationBaseValueGroup`. The KillAura owner supplies its actual combat target, goal calculation, configuration, rotation processing, route rendering and sprint callbacks. Register it only with that full consumer.

The separate runtime has its own host, route view, worker/search state, configuration, input result and epoch-filtered block/packet event queue. Event capture broadcasts to both owners before either queue is drained. World replacement retires both runtimes and clears results and physical-hold metadata without restoring an old world's inventory. The next canonical input creates a fresh runtime for that epoch.

Destroy, use-item and safe-walk holds are tracked by owner. Releasing one owner cannot stop another owner's mining hold. Hotbar claims restore the most recent remaining owner's slot, and the final release restores the original slot. These ownership helpers change only sharing of real physical state; actual movement, mining, use and route search still run through NativeHost.

Navigation runs at radius 0.7 and adds FightBot's jump assist only when the actual Navigator reports plain walking. Its original exact keys are captured separately before assist. FINAL_DECISION can therefore apply the actual Navigator keys. Goal absence halts and releases the body. An enabled-to-disabled transition stops the Navigator. A disabled configuration also stops/releases immediately at the canonical live host; a recursive navigation call or noncanonical/retired host defers the reset until safe input without reborrowing Game.

The patch exports two native helpers:

```rust
Game::kill_aura_navigation_config(&mut self, text: &str)
Game::kill_aura_navigation_result(&self) -> Option<String>
```

The consumer supplies:

```rust
Game::kill_aura_fight_goal(&self, value: &serde_json::Value)
    -> Option<([f64; 3], bool)> // world goal and plain-walk jump assist
```

`kill_aura_fight_goal` runs inside `auto_walk_world::read`; it must use its provided Game reference directly and must not enter another navigation read/write scope.

Input configuration: `enabled`, `running`, actual numeric `targetId`, `willClick`, `range`, `opponentRange`, `dangerousYaw`, `runawayOnCooldown`, `leader`, `username`, `leaderRadius`, plus the shared native navigation fields `inputYaw`, `movementYaw`, `sent`, `current`, `externalRotation`, `isFlying`, `recastsElytra`, `scaffold`, `options`.

Output fields: `input`, `exactInput`, `exact`, `enabled`, `running`, `plainWalk`, `acting`, `movementRotation`, `request`, `wantsSprint`, `usingItem`, and `route` with `points`, `head`, `previousHead`, `ahead`, `goal`, `start`, `breaks`, `places`, `preview`, `state`, `since`. The consumer adds the target-center movement pitch when `acting` is false, as the source FightBot does.

The KillAura integrator owns the shared generator edit. Its final order must be:

```text
auto-walk-patch.py
... other consumers ...
kill-aura-patch.py
fightbot-navigation-patch.py
final Step
```

The FightBot patch returns helper names `client_kill_aura_navigation_config` and `client_kill_aura_navigation_result`; do not declare duplicate helper methods in the KillAura native patch. The generated patch affects six native files: `navigation_owners.rs`, `game.rs`, `auto_walk_runtime.rs`, `auto_walk_world.rs`, `auto_walk_host.rs`, `native_hooks.rs`.

`kill-aura-fight-goal.rs` supplies the actual goal method and is appended to generated `kill_aura.rs` by that consumer's patch. It reads the provided Game directly. Its nested module keeps names/imports independent of the rest of KillAura. Native collision checks use the actual player's moved bounding box, equipment, powder-snow subject and original-position collision context. Leader lookup includes the actual local profile, visible remote player bodies and placed LAN players in the current dimension. Candidate generation, strict/inclusive boundaries, FMA, promoted float RAD_TO_DEG, stable sorting and tie order follow the source. The six prepared goal tests are run separately once in the same combined batch:

```powershell
python native/liquidbounce-horizon/test-kill-aura-fight-goal.py --model-only
```

Static verification passed with six files composed only in memory, Python AST checks and rustfmt parsing of temporary generated runtime/host copies. The production AutoWalk runtime remains SHA256 `6355cbf602363f2ba212165a69aff1f7d8e7ba81620e6bb4f5d333e0a4fe1794`.

Root's next combined module batch should run the five focused owner lifecycle tests exactly once:

```powershell
python native/liquidbounce-horizon/test-fightbot-navigation.py --model-only
```

Then perform the combined official native and engine checks with the full KillAura consumer. No Rust execution, Cargo build, official compilation or in-game parity has been claimed by this delivery. `fightbot-navigation-runtime.draft.rs` is an unused historical intermediate and is not a generator input.
