# FastExp source port

Fixed source: LiquidBounce ae0eb396 ModuleFastExp and the shared noncombat
RotationsValueGroup. The implementation exposes Rotate with the complete shared
rotation tree, NoWaste with both durability thresholds, Normal and Fast throw
rates, CombatPauseTime, and SlotResetDelay. It preserves the HOLD binding and
disableOnQuit. The source does not declare a HUD mode tag.

The native snapshot reads the actual offhand, hotbar, armor damage, maximum
durability and Mending enchantments. Offhand bottles take priority. The other
hand participates in repair estimates. Experience-orb checks use the actual
following-player identity and squared velocities. Throw actions use the shared
silent-slot service and genuine item-use path. The slot and alive guards are
read at every continuation and before every individual bottle use.

Every source tickUntil suspends into the shared FIFO ticker before checking its
condition. This includes existing-orb absorption, downward aiming, the ten-tick
orb-appearance timeout and final absorption. The source fractional throw
accumulator, repair-run counters, cancellation behavior and screen pause at the
start of an outer iteration are preserved. Rotation requests use priority 35
and their own shared processor identifiers. Freeze selects theoretical server
rotation. BlinkManager/Backtrack lag-state integration remains pending those
real module services; enabling FakeLag is not used as a substitute.

Simplified and Traditional Chinese display labels and source-key help live in
the nested display dictionaries and inherit the existing game-language switch.
The shared rotation descriptions are copied using their real source prefix.

The single focused smoke was build/fast-exp-smoke-01.exe:
repair_threshold_fifo_aim_throw_and_orb_smoke passed. Small subsequent slot
guard and continuation-order corrections are left for the integration compile;
the smoke has not been rerun. Official native/engine compilation and full-client
differential verification remain pending. No pristine upstream file is edited.

Apply the integrator once with --apply, then run update-fast-exp-locales.py.
