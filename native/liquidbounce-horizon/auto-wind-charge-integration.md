# AutoWindCharge owned implementation

Source: pinned `upstream/LiquidBounce/.../player/ModuleAutoWindCharge.kt` and Minecraft-Agent 26.3 FallingPlayer/SimulatedPlayer.

Owned files:

- `native/liquidbounce-behaviors/src/auto_wind_charge.rs`
- `engine/src/auto_wind_charge_runtime.rs`
- `engine/src/config/auto_wind_charge.rs`
- `auto-wind-charge-native.rs`, `auto-wind-charge-host.rs`, `auto-wind-charge-patch.py`
- `integrate-auto-wind-charge.py`
- `update-auto-wind-charge-locales.py`
- `auto-wind-charge-smoke.rs`, `validate-auto-wind-charge-patch.py`

The behavior preserves all source early-return state, the ten-tick ground aim grace period, sampled horizontal pitch lifetime, movement-direction yaw, enabled/disabled rotation paths, and offhand-first selection. The full noncombat RotationsValueGroup codec and live processor dispatch reuse NoRotateSet. The landing horizon starts from actualServerRotation and implements Linear/Sigmoid minimum-range calculation and Acceleration turn-speed semantics. Native prediction replays the actual cloned player against real block/entity collision shapes with jump disabled and the player's final modified input. The query returns supporting block, landing tick, and position before movement. View-angle checks use vanilla trig vectors. Throws use the existing real native item-use and silent hotbar services; Rotate=false uses mouse-GCD normalization from current managed or actual player rotation.

Rotation provider: 16. Processor base: 200. Request priority: 35. Requests occur at ordinary GameTick after FIRST_PRIORITY rotation update and are consumed on the next tick. Input jump suppression occurs among ordinary input handlers; AutoDodge's final jump decision retains its later ordering. A final input snapshot records the actual input for the next prediction and horizontal direction.

Parent integration:

```powershell
python native/liquidbounce-horizon/integrate-auto-wind-charge.py --check
python native/liquidbounce-horizon/integrate-auto-wind-charge.py
python native/liquidbounce-horizon/update-auto-wind-charge-locales.py
```

Validation already performed once in owned scratch:

- rustfmt on owned model/runtime/config/native/smoke files passed.
- integrator `--check`: 8 shared files validated; no shared writes performed.
- `rustc --edition 2024 native/liquidbounce-horizon/auto-wind-charge-smoke.rs -o build/auto-wind-charge-smoke-01/auto-wind-charge-smoke.exe`, then execution passed: ground aim grace, jump gating, pitch cache, air landing, offhand, cooldown and all calculateTicks modes.
- `python native/liquidbounce-horizon/validate-auto-wind-charge-patch.py`: native patch generation passed; scratch zh_cn and zh_tw each have 34 AutoWindCharge description entries, valid UTF-8 and idempotent updates.

No Cargo build, production staging, localization write, or game launch was performed by this owner. Parent must run the normal staged compile and actual client checks. All runtime behavior remains Rust; Python scripts only wire/validate development artifacts.
