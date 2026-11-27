# ExtendedFirework source parity

Fixed LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`.
The native behavior implements both Normal and Custom modes. Normal uses
`(0.1, 1.65, 0.5)`; the custom vector contains the three coefficients of the
same formula, not world axes. Inactive Custom settings remain importable.

The actual local player's attached firework calls the generated native hook
immediately before the boost. Each component is computed as
`movement + (look * x + (look * y - movement) * z)`, preserving the Java
`ModifyArgs` followed by `Vec3.add` rounding order. The disabled branch also
uses the original Java grouping; Horizon's previous left-associated expression
could differ by one ULP. Non-gliding rockets do not enter the hook. Native
window, actual client player, LAN guest, server-copy and replay identity gates
prevent applying the module to another simulated player.

The shared managed-rotation correction applies independently of whether
ExtendedFirework is running, as in the original firework mixin. It uses the
current rotation only while an active target requests movement correction.
`Vec3.directionFromRotation` float angle arithmetic and the actual vanilla
sine table are retained; ordinary view direction uses `Entity.getLookAngle`.

The complete VECTOR3_D adapter preserves missing axes as zero, ignores
unknown fields, accepts Java numeric strings including hexadecimal doubles,
and accepts NaN/infinities on import. Invalid components reject that leaf.
Non-finite serialization returns an error, matching original Gson behavior.
Settings retain original names, categories and Mode/Custom tree shape. English,
simplified Chinese and traditional Chinese help describes the coefficients.

Verification:

```text
python native/liquidbounce-horizon/test-extended-firework.py --out build/extended-firework-05
```

`../build/extended-firework-05/report.json` passes 58 tests and Clippy with
warnings denied. It compiles unmodified ModuleExtendedFirework.kt and
VectorAdapter.kt with minimal settings shells. Original Java mixin argument
expressions execute through real MC Vec3.add. Actual MC LocalPlayer.getLookAngle
and Vec3.directionFromRotation produce the reference directions.

442 imports include malformed values, nulls, signed zero, NaN/infinity,
overflow, subnormals, rounding ties and 400 deterministic random hexadecimal
numbers. 576 boosts cover both modes, four custom vectors, six view angles,
three movements, managed/unmanaged look and enabled/disabled states, comparing
raw double bits. The Rust fixture compiles the complete transformed native
follow_own_player consumer and actual generated Host/event dispatch. Identity
guards and non-gliding behavior are checked separately. Upstream files are
unchanged. This standalone evidence does not claim full-game acceptance or a
complete Java-client/Rust-client differential run.
