# AutoRod integration handoff

Owned implementation is ready for the parent's next batch. No per-module rustc/Cargo smoke was run, following the user's request to compile and test several completed modules together.

Apply `python native/liquidbounce-horizon/integrate-auto-rod.py` transactionally, followed by `python native/liquidbounce-horizon/update-auto-rod-locales.py`. `--check` validated eleven shared files without publishing changes. AutoBow and the shared TargetRenderer are prerequisites; the native patch must run after auto-bow-patch.py. The integrator inserts the AutoRod owner at the unique `TARGET_RENDERER_OWNERS_END` marker.

Rotation provider is **20**, processor base **260**, requester `0x4155_544f_524f_4400`. AutoPearl is reserved as provider 21/base 280. Category Combat; no source aliases. The normal tick follows AutoBuff; FIRST_PRIORITY caches follow AutoBuff's first handler, while existing coroutine continuations still run before FIRST_PRIORITY like pristine Minecraft.tick.

Implemented source behavior:

- Every source setting, ordered target criteria, strict player/target health comparisons, entire-world eligible-enemy count, actual player-to-target range, line of sight and held-item exclusion. Physical attack-binding and mouse input history reproduce InputTracker, including the unpopulated keyboard history map. Consumable and custom-name checks use actual effective stack components; sword/axe checks use real item tags, MaceItem uses the pinned class index and Knockback uses the stack's enchantment level. The pinned 26.3 protocol qualifies axes regardless of Horizon's purely visual legacy-combat option.
- Cached offhand-first rod selection, retaining a still selected rod slot, successful-cast range resampling, manual-hook skip, double point calculation in rotation-update and tick phases, silent hotbar duration, managed/server rotation angle gate, real native item interaction, per-hand interact_animation and HideForClient without an attack/punch packet.
- True FishingHook id/Hooked state/deltaMovement, exact zero-vector equality, tickConditional timeout/hit/stationary/range conditions, captured target/ranges and slot-only suspended guards. Existing hook references update without adopting a newly spawned hook before the next FIRST_PRIORITY cache; removed target references retain their last position. Reeling registers the next cooldown callback into pending FIFO, and world/disable invalidates callback epochs. Disable schedules releaseUsingItem rather than reeling and resets this module's silent selection.
- Full AimPoint/PointTracker: one-tick actual player motion forecast, nonplayer deltaMovement FMA, real hitbox/pick radius, perspective projected points, three body-part excludes, best-vector cylindrical exclude, Delay then Lazy then Gaussian. PointInsideBox offsets translate both point and box. Delay preserves the old-return-on-expiry behavior, Lazy compares position only against a float-squared threshold, Gaussian keeps independent per-axis interpolation and the upstream PointInsideBox-versus-LivingEntity dynamic-input behavior. All inactive settings and processor state survive disable. Default construction and changed range callbacks preserve when sampling occurs.
- Both source projectile solvers with FISHING_ROD's velocity/gravity/drag/radius and actual collision-context visibility; full shared Rotations tree and independent TargetRenderer state.

Owned files are the behavior `auto_rod.rs`, reusable `point_tracker.rs`, `auto-rod-native.rs`, `auto-rod-host.rs`, `auto-rod-patch.py`, engine `auto_rod_runtime.rs`, config `auto_rod.rs`, transactional integrator, locale updater, `validate-auto-rod-patch.py` and the prepared `auto-rod-smoke.rs` harness. auto-rod-patch.py only changes AutoBow Entity field visibility in the generated staging patch; it does not edit validated AutoBow source.

Completed static validation:

- rustfmt edition 2024, skip_children=true, on owned Rust sources.
- Transactional integration anchors: eleven shared files, read-only check passed.
- Python developer helpers parse successfully.
- Complete native patch generation succeeded into `build/auto-rod-static-01/hooks-preview.patch`; no production patch or staging directory was changed. Subsequent static review corrected the native KeyBinds::get return type, local destroying-field lookup and purely visual legacy-combat preference; these do not alter patch anchors and will be included in official regeneration.
- Both scratch locale files contain **194 AutoRod descriptions**, including all 25 rotation descriptions and all 109 TargetRendering descriptions. UTF-8, exact rotation subtree copy and idempotence passed. Production display locale files were not changed by this agent.

Batch verification to run: `rustc --edition 2024 native/liquidbounce-horizon/auto-rod-smoke.rs -o build/<batch>/auto-rod-smoke.exe`, then execute that harness in the batch alongside official staged native/engine Cargo checks. The prepared harness covers slot retention/offhand order, strict guards, captured ranges, waiter/cooldown transitions, cancellation, manual/stationary hook behavior, input timing boundaries, persistent Delay/Lazy/Gaussian state, box translation and the fishing solver. Official compile, config round-trip and in-game behavior validation remain with the parent batch.
