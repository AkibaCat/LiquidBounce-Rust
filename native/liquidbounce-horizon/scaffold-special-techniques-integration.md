# GodBridge, Breezily and movement replay handoff

Owned code is complete; it is not registered independently from Scaffold.
The complete existing native generator, with pending Scaffold fragments before
Step, composed successfully and six native Rust sources plus the focused test
source parsed through rustfmt. No compiler or test was invoked.

The root adapter exposes `client_scaffold_special_technique(request)` through
the actual `native_client_scaffold_special_technique` wrapper. Requests retain
the documented `plan`, `crosshair`, `rotation`, `breezilyInput`, and
`godbridgeLedge` operations. `plan` and `crosshair` extend the existing complete
Normal/Expand planner with the source center target and default click verifier.

Apply `scaffold-replay-patch.py` after FallingForecast and ElytraRecast. Apply
`scaffold-special-techniques-patch.py` after techniques and features. Step stays
last so every added MotionWorld constructor receives its real sender field.

The replay retains all old full-tick entry points and adds
`visit_with_keys_probe(game,ticks,add_y,keys,continues_tick,callback)`, passing the
independent hypothetical sneak-backoff result into every callback. Only an
explicit first-tick continuation omits the base health/fluid/crouch phases.
The actual Horizon directional-input callback precedes its native base tick,
so GodBridge correctly requests a full tick. Its keys preserve cached directional
input, actual jump, actual crouch and current-or-key sprint; its geometry reads
the real world, current scaled eye height, first replay snapshot and block/entity
interaction ranges.

GodBridge side state and Breezily side/time/random distance persist across
enable/disable, as in the source objects. Low-block sneak uses strict `<`.
Jump-mode fallback uses actual native jump power and the original discrete
gravity/float-drag apex. Random draws occur only in the original branches.
Breezily uses wall-clock milliseconds, inclusive 500 ms support grace, separate
edge tests, and resamples only on nonzero direction changes.

Five production-rule checks are prepared, not executed:

```powershell
cargo +stable test --offline --manifest-path build/scaffold-special-focused-01/Cargo.toml -- --test-threads=1
```

They cover rotation rounding and persistent side, mode selection/random
consumption, center retention and threshold resampling, grace/input guards,
and cardinal direction boundaries. Full module schema/runtime and localization
are owned by the complete Scaffold integration; native/API compilation and
actual gameplay remain part of its later grouped validation.
