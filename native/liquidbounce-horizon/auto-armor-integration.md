# AutoArmor integration

Frozen owned implementation of `ModuleAutoArmor.kt` and `AutoArmorSaveArmor.kt`
from pinned LiquidBounce ae0eb396. Runtime and behavior are Rust; Python only
wires source and updates development localization assets.

Apply serially from the repository root:

```powershell
python native/liquidbounce-horizon/integrate-auto-armor.py --check
python native/liquidbounce-horizon/integrate-auto-armor.py
python native/liquidbounce-horizon/update-auto-armor-locales.py
```

The integrator validated 9 shared files without writing them. It validates every
anchor before a write and rolls back completed writes on failure. Include the
latest owned `inventory-cleaner-host.rs`: it now publishes the actual GLIDER
component as a class fact, so AutoArmor preserves both elytra and custom gliders.

Behavior includes the shared two-pass ArmorSettings evaluator, Smart and
RawDefense with Projectile Protection toggle, all 41 candidate slots, equipped
curse exception, server locks, Hotbar (including offhand), CanSwapArmor, normal
quick move and full-inventory throw. Player-life priority is 40. After a removal,
the target slot is checked live and the same piece is equipped once if empty.
The shared ItemPolicy armor settings are initialized, updated immediately by
configuration and synchronized before InventoryCleaner/ChestStealer planning.

SaveArmor includes threshold preference, InventoryOpen-only automatic GUI,
creative/spectator guards, hotbar replacement precedence, handled-screen armor
attribute loss tracking, one-tick close/open sequencing, sampled millisecond
close delay and screen rechecking after waits. GUI opening dispatches normal
GUICloser, key release, screen serial, Replenish and ChestStealer observers.

Validation completed:

- One focused smoke executable: `build/auto-armor-smoke-01/auto-armor-smoke.exe`.
  Output: `AutoArmor: shared policy, curses, locked armor, hotbar/offhand,
  removal/full inventory, gliders and SaveArmor waits passed`.
- Native patch generation dry run succeeded:
  `build/auto-armor-smoke-01/hooks-preview.patch` (no production staging writes).
- Simplified and Traditional Chinese scratch resources each have 17 AutoArmor
  descriptions. The updater is idempotent and introduces no U+FFFD characters.
- Owned behavior, runtime, configuration and smoke Rust files passed rustfmt.

No Cargo build or game session was started by this subtask. The parent performs
the formal serialized staging/check and integrated runtime verification. The
focused smoke does not establish in-game parity.
