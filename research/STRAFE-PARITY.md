# Strafe source parity

Pinned LiquidBounce commit: `ae0eb396c07f22383789b28d5912d714c6ca0a86`.
The behavior ports all of `ModuleStrafe.kt`, the directional-input helper,
`withStrafe`, and the `moving` input predicate.

## Complete behavior and settings

Strafe belongs to Movement. StrengthInAir and StrengthOnGround default to 1.0
and expose the original 0.0–1.0 Float slider range. StrictMovement defaults to
false. Imported strengths retain their raw Float values, as in the original
value deserializer. Both File and Interop trees retain original identifiers.
There is no module-specific lifecycle state.

Only SELF PlayerMoveEvent events are modified, before Entity.move handles
noPhysics, stuck multipliers or collision. A selected strength of either signed
zero returns immediately, including when StrictMovement is enabled. The moving
predicate reads the actual ClientInput move vector. Direction is calculated
from the separate untransformed directional flags, including opposing-key
cancellation, using Float yaw arithmetic followed by JDK-compatible Double
trigonometry. The modifier preserves the input vertical displacement and the
stored velocity. AutoJump observes the resulting actual displacement.

The native hook captures all three settings and untransformed flags only for
the actual native window and real client player, outside a swapped LAN guest.
The snapshot is assigned before travel. The movement modifier rechecks client
identity and reads the current ground state, yaw and move vector. Piston and
shulker-box movement remain on their original non-SELF paths.

Integration uses `client_strafe(air, ground, strict, input, apply)`; the parent
sets `input` from `modules.directional_input`, before rotation transforms, and
`apply` from the module's running predicate. The patch applies after the mining
identity hook. Settings apply through `StrafeSetting::apply(&mut Strafe)`.

## Verification

```text
python native/liquidbounce-horizon/test-strafe-patch.py --out build/strafe-02
```

The final report passes six tests and Clippy with `-D warnings`, with upstream
sources unchanged. The fixture compiles the unmodified ModuleStrafe and
DirectionalInput Kotlin files. It extracts the original entity/vector helper
bodies byte-for-byte and executes actual MC 26.3 LocalPlayer, ClientInput, Vec2,
Vec3 and MoverType classes with JDK Math. Constructor-free LocalPlayer storage,
module value delegates, handler activation and mixin-provided untransformed
storage form the environment shell.

69,125 cases compare raw Double bits for five MoverTypes, nine yaws, sixteen
directional flag combinations, six strength pairs, both ground and strict
states, four independent move vectors, and disabled behavior. Strengths cover
signed zero, tiny, negative and out-of-range values. Input vectors and flags
are deliberately independent, preserving the distinction used by the source.

The native fixture compiles the actual transformed movement modifier,
move_self, shulker-box path, identity guards and complete Entity.move
implementation. Actual horizon_mod_api_gen generates Host, events and dispatch
code for the copy-out hook. Cases cover window/guest/server/replay/missing-player
gates, noPhysics, stuck multipliers, velocity preservation and AutoJump's actual
displacement. Parent configuration/registry integration and full-game execution
are separate checks and are not claimed by this standalone report.
