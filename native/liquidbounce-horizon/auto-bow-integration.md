# AutoBow integration handoff

Owned implementation is ready. The parent performs shared integration and official staging/build serially.

Run `python native/liquidbounce-horizon/integrate-auto-bow.py --check`, then invoke without `--check` to publish its eight shared files transactionally. Run `update-auto-bow-locales.py` for the production display locales. The normal patch generator then picks up `auto-bow-patch.py`.

The shared TargetRenderer implementation and its config module must also be integrated. This implementation uses `TargetRendererSettings`, `TargetRendererRuntime`, and `TargetRendererSetting.apply(&mut settings)` (no return value). AutoBowRuntime exposes `target_renderer` and `target_render_runtime`. The separate TargetRenderer integrator owns the native render event, GPU path, render callback and its locale subtree, using prefix `liquidbounce.module.autoBow.bowAimbot.targetRendering` and target `behavior.target`. ArmorStand `liquidbounce_tick_count` and Dragon `ticks` visibility are supplied by auto-bow-patch.py; do not duplicate those fields.

Rotation provider is **19**, smoothing processor base is **240**. Provider 18 belongs to ProjectilePuncher. Category is Combat, aliases are BowAssist and BowAimbot. AutoBow is ticked before AutoBuff and its use-key handler precedes SmartEat, following pristine ModuleManager registration order.

Implemented source behavior:

- AutoShoot's bow/trident/crossbow branches, precharged crossbows in both hands, persistent Gaussian charge cache, strict chronometer, global disabled-module bow-stop observation, active managed-rotation gate, forty-tick hypothetical arrow simulation and force-uncharged use-key event. Using another item prevents the alternate-hand precharged fallback.
- BowAimbot's ordered Type/Health/Distance/Direction/HurtTime/Age tracker, stable sorting, Java Float/Double comparison, recognized below-name health objective, hurt/FOV filtering, real global target policy, source registration order, full Rotations tree and per-owner target renderer state.
- Both Polynomial and Cydhranian solvers, native player motion replay (30-tick nearest snapshot clamp for aim; 40-tick forecast for hypothetical hits), nonplayer deltaMovement FMA, silhouette max-cos calculation, 128-point perspective grid, JOML cosFromSin and nested f32 matrix arithmetic, final-five-block Arrow-context visibility, and optional 300-tick actual collider/entity verification with fluid drag, shared vehicle exclusion and Freeze-aware inherited velocity.
- FastCharge's exact feature defaults, all guards, packet types, repeat-time isUsingItem checks and actual native item updates. An RAII update guard bypasses right-key/screen release checks solely for this direct updatingUsingItem call; AutoShoot's use-key release remains in the later input phase. Remote item identity and hotbar changes reset the bow chronometer using the old active item.

Owned Rust files are `auto_bow.rs`, `auto_bow_math.rs`, `auto-bow-native.rs`, `auto-bow-host.rs`, `auto-bow-ray.rs`, `auto-bow-forecast.rs`, `engine/src/auto_bow_runtime.rs` and `engine/src/config/auto_bow.rs`; developer helpers are auto-bow-patch.py, integrate-auto-bow.py, update-auto-bow-locales.py, validate-auto-bow-patch.py and auto-bow-smoke.rs. Runtime logic is entirely Rust.

Validation completed:

- rustfmt edition 2024 with skip_children=true on owned Rust files.
- integrate-auto-bow.py --check: eight shared files validated; no shared files changed.
- Exactly one focused rustc/executable smoke passed: release gates, ordered priorities and tie stability, NaN comparison, cached/clamped Gaussian charge, strict timers, disable persistence, fast-charge guards, AABB segment endpoints, point-grid visibility and both projectile solvers.
- Native patch generation succeeded into `build/auto-bow-smoke-01/hooks-preview.patch`. Earlier owned anchor failures were corrected (slot bridge selection and unique Dragon field context).
- Locale scratch files each contain 54 AutoBow descriptions, including all 25 existing rotation descriptions, with valid UTF-8 and idempotent updates. Final locale-only retry corrected an overestimated count assertion; native patch generation was not repeated for that assertion.

No Cargo, Clippy, oracle, full suite or gameplay run was executed by this agent. No production staging, shared source or production display locale was changed by this agent. Official compilation and in-game validation remain with the parent.
