# CameraClip and DankBobbing source parity

Baseline: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon pin in this project, Minecraft 26.3 merged JAR. These are two complete module declarations, separate from the translated catalog.

## Source and implementation

CameraClip follows `modules/render/cameraclip/ModuleCameraClip.kt`, its inherited `ScrollAdjustValueGroup.kt`, `ToggleableValueGroup.kt`, `PerspectiveEvent`, `EntityExtensions.cameraDistance`, and `MixinCamera.java`. The pure implementation is `native/liquidbounce-behaviors/src/camera_clip.rs`; the full config tree is `native/liquidbounce-horizon/engine/src/config/camera_clip.rs`.

Defaults are distance 4, Animation on, speed .6, Linear easing, ScrollAdjust on, left Control, sensitivity .3, RememberScrolled off, RequireFreeLook off. All seven easing choices are retained. Config input preserves raw float values beyond UI ranges. Only the actual scrolled-distance setter clamps to 1..48. Turning Animation off keeps the vanilla entity distance and still enables wall clipping, matching the original separate handlers.

The native patch adds a real PerspectiveEvent singleton cache to Camera. Each real client `Camera.tick` phase first preserves preceding final perspective/distance, then reads the raw F5 option, clears noClip, and computes the camera entity distance. It runs the module chain once and stores its result for every render consumer until the next tick. Spectator entities and living mounts are included. The module extension separately maximizes rider/mount scale and distance before multiplication; vanilla camera distance still maximizes the already scaled products.

CameraClip's normal listener runs before AutoF5 and QuickPerspectiveSwap. Its animation resets lastDistance when the preceding final perspective differs from current raw perspective. ScrollAdjust's READ_FINAL_STATE listener observes the perspective after both later modules. Therefore the actor order is CameraClip perspective, AutoF5, QuickPerspectiveSwap using the current result, and CameraClip final reset. The former per-query `client_camera_perspective` hook remains an API compatibility surface; ordinary effective-perspective reads no longer dispatch it. A test probe may observe the cached result.

Render placement suppresses collision rays when noClip is true and uses the cached last/current event distance interpolation after the ordinary generic distance adjustment phase. Equal endpoints bypass interpolation, preserving signed zero. Source-imported unusual values are not sanitized by a new distance guard.

ScrollAdjust sees the raw double vertical wheel delta before discrete scrolling, operating-system sensitivity, or screen branches. It narrows to float before multiplying sensitivity. Its gate checks the modifier, RequireFreeLook, and the **raw** F5 perspective. The raw event is not cancelled; only the later hotbar-scroll event is cancelled when the same gate passes. Zoom's later listener remains deliverable because event cancellation does not stop subsequent listeners.

Modifier names follow the actual 26.3 InputConstants table. Its keyboard codes are SDL scans: left Control is 224 and UNKNOWN is 0. `key.keyboard.0` resolves the named Digit0 key (39), rather than UNKNOWN. Named lookup precedes numeric fallback, existing numeric codes canonicalize to their known names, and non-string key JSON is rejected. The original key pressed helper calls the keyboard scan query even for mouse-category keys; this behavior is retained rather than replaced with a mouse-button query. Keyboard release resets only matching keyboard-category modifiers.

Reset runs after a final first-person perspective, modifier keyboard release, enabled-child updates when the parent enables, or manually enabling that child while its parent is enabled and the client is in game. Disable does not reset. RememberScrolled changes CameraDistance only when different; otherwise reset writes the target through the scrolled-distance clamp. Parent reset belongs in `ModuleCallback::UpdateChildren { enabled:true }`, after root OnEnabled. A true reset return publishes a full Interop CameraDistance leaf inside the original `valueChanged` event and the actor's normal config persistence records it.

DankBobbing follows the complete `ModuleDankBobbing.kt` and `MixinGameRenderer.java` replacement. Motion defaults to 5, with UI range 1..50 and raw imports retained. Its setting belongs in category Fun. The implementation preserves original walk-based translations and the deliberately **bob-based** sine/cosine rotation arguments. It uses the original PoseStack quaternion rotation arithmetic, including JOML identity, translation, and affine paths. Simplifying those rotations to a direct axis matrix changes float bits and is avoided.

The patch uses the actual camera player's AvatarBob history, including spectated players. Mobs, armor stands and mannequins do not satisfy the original player guard. NoBob, the view-bobbing option and the original tracer dependencies dominate DankBobbing. Unregistered Tracers, ItemESP, StorageESP, or FreeLook dependencies remain false instead of becoming dummy modules. NoHurtCam and the native camera shake composition are preserved. World and first-person hand matrices share the same computed bob matrix.

## Integration contract

Apply the main generator's existing `fov()` transform, then `camera-control-patch.py`, `camera-clip-patch.py`, and `dank-bobbing-patch.py` in that order. The existing camera-control helper is unchanged.

New hooks are `client_camera_tick`, `client_mouse_scroll_raw`, and `client_dank_bobbing`. The new Host helper is `client_keyboard_scan_down`. Generated mutable scalar hook arguments become owned mutable scalar event fields; callers borrow fields when passing them to pure behavior methods. No custom replacement API generator is used.

The two config adapters retain the complete declaration order. CameraClip has CameraDistance; Animation Enabled/Speed/Easing; ScrollAdjust Enabled/Modifier/Sensitivity/RememberScrolled/RequireFreeLook. This ordering matters: child-enable callbacks execute before a later RememberScrolled import. DankBobbing has Motion only. The English, simplified Chinese and traditional Chinese help keys for all settings and both CameraClip groups are owned additions, leaving pinned original locale resources byte-exact.

## Validation and limits

`python -X utf8 native/liquidbounce-horizon/test-camera-clip-dank-patch.py` passed 21 tests and Clippy with `-D warnings`. Evidence is `build/camera-clip-dank-hook-tests/result.json`, with source hashes and logs. The fixture compiles the entire transformed production Camera module, the actual singleton tick body, effective-camera consumer, all generated Host/events/dispatch, Game::camera_entity, remote mob/stand conversion, Game::camera_player_bob, and Game::bob_matrix. Shell boundaries are recorded in the report.

The Java oracle executes actual MC26.3 InputConstants, Mth and PoseStack code: 268 key rows, 42 complete easing/speed rows, and 18 complete 4x4 bob matrices, including hurt poses. Rust outputs match the animation and matrix float bits exactly. This is not a GPU image comparison or a shipping-game integration result.

`test-camera-clip-dank-config.py` snapshots unmodified production config and behavior sources without registration shims. Its first snapshot passed all 20 config tests, including six new CameraClip/Dank cases for raw-import round trips, every easing choice, declaration ordering, local invalid-key failure, canonical key identity, metadata, and nonfinite output rejection. The first Clippy run exposed a separate NoEntityInteract/NoBlockInteract duplicate data-module import; the parent owns its shared fix. The immutable first report is `build/camera-clip-dank-config-01/report.json`.

Old `test-camera-control-patch.py` (11 tests) and `test-auto-f5-patch.py` (9 tests) both passed after adding the new independent helpers. The old helpers remain testable on their own, while the combined fixture verifies the new singleton phase. The old CameraControl generated-API fixture retains two existing unused-import warnings; it is not reported as a new Clippy-clean fixture.

Chinese resources currently contain 112 owned English/simplified/traditional setting-help entries. Theme verification, static caption audit, freshness audit, all 10 Interop tests and Interop Clippy passed. NoRotateSet's dynamic keys and all MultiActions choice labels are explicitly checked. These localization catalog checks do not register or implement additional modules.

The parent integration run separately owns package generation, complete game compilation, persistent actor wiring, actual UI language switches, collision scenes and rendered screenshots. Existing Horizon spectator hurt-bobbing still obtains hurt inputs from the local player; this unrelated baseline behavior is not claimed to be corrected by the Dank player-history patch.
