# AutoMobHeal integration

Pinned source: `upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/world/automobheal/AutoMobHeal.kt` and `MobFoodOption.kt`.

- `integrate-auto-mob-heal.py --check` prepares and validates the eight shared files without writing them. The script is idempotent; running without `--check` publishes the complete module registration/config/runtime/native-patch wiring.
- Owned engine files: `engine/src/auto_mob_heal_runtime.rs` and `engine/src/config/auto_mob_heal.rs`.
- Owned native files: `auto-mob-heal-native.rs`, `auto-mob-heal-host.rs`, and `auto-mob-heal-patch.py`.
- `test-auto-mob-heal.py` is the focused static check. `auto-mob-heal-smoke.rs` is a separate pure-model executable for the parent's next Rust batch.

The native adapter reads live mob health/attributes, render order, owner/tame/age/sitting state, actual body armor components and damage, offhand/hotbar item components, reloadable food tags, line of sight, and eye-to-AABB distance. The model retains the nine source group order and slot/food tie rules. Native targeted interaction uses the real client mob result and its swing answer, with no fallback item use. A consumed interaction alone starts the sampled CoroutineTicker delay. Disable resets this module's silent slot; cancellation and world changes invalidate its waiter.

Local checks run: Python syntax and focused static fixture. No Cargo, rustc, Rust smoke execution, full staging, or full compilation ran in this subtask; the parent owns those batch checks.
