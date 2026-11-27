# FreeLook source parity

Pinned source: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`,
`ModuleFreeLook.kt`, modern `RotationUtil.mouseTurnDelta`,
`MixinMouseHandler.java`, and `MixinCamera.java`.

## Implemented owned surfaces

- `native/liquidbounce-behaviors/src/free_look.rs` has the full declaration:
  Perspective Front/Back (original enum order, default Back), SenseBoost 1.0
  (UI range 0.1..2.0), NoPitchLimit true, camera yaw/pitch initially zero.
  Imported finite floats are not clamped to the UI range.
- Every `on_enabled` immediately copies the real player's current yaw/pitch.
  Setting edits leave accumulated angles intact. The module declares the
  original HOLD bind and disable-on-quit lifecycle constants.
- The existing mouse event is extended with a mutable canceled flag. The
  original sensitivity, smoothing, inversion, tutorial callback and Horizon
  zoom remain before the event; cancellation skips only the final player turn.
  Independent listeners, including RotationManager, still receive the event.
  Controller/agent direct turns continue through the original `look_turn`.
- Modern cursor double values are narrowed to float, multiplied by `0.15f`,
  then multiplied separately by SenseBoost before accumulating. Pitch is
  clamped only on a mouse input when NoPitchLimit is false, including when a
  just-enabled player starts outside the range.
- The final camera hook supports both local and spectated entities. Front
  adds 180 to yaw and negates pitch. It starts at the interpolated entity eye,
  skips SmoothCamera entirely, sets detached and exits before ordinary
  detached/sleeping alignment. The stored F5 choice remains independent.
- Collision uses the existing actual Camera.getMaxZoom implementation and its
  eight visual-shape rays. CameraClip's noClip branch has zero rays.

## Source quirks deliberately preserved

`EntityExtensions.cameraDistance` already includes entity/mount scale. The
FreeLook camera injection multiplies the PerspectiveEvent distance by the
camera entity's living scale a second time; this is retained. A mount's scale
is not substituted for this second factor.

The noClip distance interpolation annotation targets `alignWithEntity` calls
to getMaxZoom. FreeLook's getMaxZoom call is in the separate injected callback,
so it does not receive that expression modifier. FreeLook therefore uses the
current event distance times the living scale, while ordinary third person
continues using the existing event distance interpolation.

## Verification

Command:

```text
python -X utf8 native/liquidbounce-horizon/test-free-look-patch.py --out build/free-look-03
```

Result: 9 tests and Clippy `-D warnings` passed. Evidence is in
`build/free-look-03/report.json`, `test.log`, `clippy.log` and
`java-oracle.json`. The earlier `-01` and `-02` evidence retains fixture
expectation/lint corrections; production behavior did not change between them.

The fixture compiles the complete transformed Camera module, complete actual
MouseLook/Rotation input module, actual Game mouse-handler and look-turn
bodies, and native hook bodies at their real module boundaries. It runs the
actual `horizon_mod_api_gen::generate(Target::Client)` and compiles its Host,
event and dispatcher output. It checks cancellation copy-out, array borrowing,
an independent listener after cancellation, excluded native identities, screen
gating, sensitivity/inversion/zoom, front/back rotation, sleeping early return,
collision, secondary living scale and the distinct noClip distance paths.
Sixty source-derived Java cases compare exact Float bits for both angles.

The Java oracle executes the source formulas rather than the original Kotlin
object. Game/window/player identity, tutorial notification and zoom-factor
providers are explicit fixture boundaries. The renderer's eye/eye_rotation
fields are Horizon metadata; their existing entity-eye meaning is retained.
Engine registry/config persistence/lifecycle integration and full EXE playtests
belong to the parent integration and are not claimed by this isolated report.

`config/free_look.rs` additionally supplies original file/interop trees,
complete parsing, enum order/range metadata and import/roundtrip/error tests.
Those tests run with the parent engine's actual shared config module.
