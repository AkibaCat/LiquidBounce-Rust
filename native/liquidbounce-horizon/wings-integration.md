# Wings complete source integration handoff

Pinned LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon
`9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Horizon-Mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`.

Root applies serially, then uses the normal official native/engine aggregate:

```powershell
python native/liquidbounce-horizon/integrate-wings.py
python native/liquidbounce-horizon/update-wings-locales.py --directory native/liquidbounce-interop/resources/localization
```

The transactional integrator validates every anchor and original before
writing seven shared files. `--check` passed. No shared, upstream or staging
files were changed by this handoff. AutoBreak and ItemScroller were untouched.
The native patch composes eight files in memory, including the guest swing
accessor and a separate renderer. Integration follows Hats and uses its
packaged byte-identical Minecraft sine table in `vendor/minecraft-math/`.

## Implemented source behavior

- The pinned source contains one mode, Lines. Its geometry is procedural;
  no image, mesh, external asset or replacement geometry is needed.
- Position Height/BackOffset/EquipmentOffset, Mode.Lines inherited
  ShowDamage/ShowInFirstPerson/FriendsOptions.ViewOnFriend/Distance, Length,
  Width, FadeStartRatio, LinesCount, Angles and Color have source defaults,
  ranges, suffixes and tree structure. Render lives inside Mode.Lines.
  The module list tag is Lines.
- Each line index has the exact two translated/rotated wing pairs, asymmetric
  quarter-block length, solid segment and transparent endpoint segment.
  One line produces eight quads; the default four produce 32 quads and 192
  triangle-list vertices. Single-line interpolation, short lengths, local
  crouch affecting every selected player's tilt, red damage color and
  preserved plain alpha follow source branches.
- Strict three-dimensional friend range uses current positions and the real
  FriendBook. Geometry uses interpolated positions, native body rotation,
  actual `minecraft:scale`, bounding-box height and CHEST equipment. The local
  body is passed through the existing native RotationManager model hook.
  FreeLook checks enabled state. LAN guests use real position/yaw/swing/vehicle
  inputs and the same tickHeadTurn path as Horizon's player model, retaining
  previous/current body rotation for interpolation and removing departed IDs.
- Minecraft direction uses the original Float angle and collected sine-table
  arithmetic. Position offset uses double FMA; body yaw takes the wrapped
  shortest interpolation path. Scale applies to the mesh and back offset;
  the original unscaled Height setting remains unscaled.
- Plain plus all nine original Shader styles reuse an independent derivative
  of frozen Breadcrumbs. Every Wings style defaults Fill to 1 (`solid=true`),
  with inactive style settings retained through File/Interop. Shader
  shapeColor makes the base color opaque before explicit endpoint fading.
- Scene depth clips both Plain and Shader silhouettes. The independent mesh
  fragment shader samples the real reversed scene depth; the Shader capture
  accounts for the source texture's vertical orientation. No culling or depth
  writes are introduced. Frozen Breadcrumbs sources remain unchanged.
- Simplified/traditional dictionaries receive real nested moduleNames/labels/
  descriptions/englishDescriptions entries. Each language currently has 87
  Wings descriptions; fill descriptions correctly say default 1. Legacy
  ignored top-level Wings keys are removed if present.

## Focused evidence

`build/wings-smoke-03/report.json` passed five tests in one Cargo invocation:

1. Source geometry counts/coordinates, endpoint fading, selection, scale,
   equipment offset, crouch and damage alpha.
2. Complete File/Interop settings and inactive nine-style roundtrip, exact
   Position/Mode.Lines.Render tree structure and int-range values.
3. Production native Game bridge and runtime using own/remote/LAN snapshots,
   native model callback, shortest body interpolation, scale and chest slots.
4. Actual production `localization.rs` resolves both Chinese copied display
   dictionaries, labels and descriptions.
5. Actual WGPU draws/readback for Plain plus nine original GLSL styles,
   with a depth-only top-half occluder verifying both Plain and Shader
   silhouette vertical orientation.

Smoke01 records a closure lifetime inference compile error, fixed by explicit
argument types. Smoke02 passed geometry, native bridge, localization and GPU;
its roundtrip test incorrectly edited accent colors in styles with no accent
setting. Smoke03 corrects that test input to the two source styles that expose
color. Historical reports/logs are preserved.

Native Game/registry/Host declarations are fixture providers; this evidence
does not establish actual game visual parity. Official aggregate native/engine
compile and in-game parity remain root-owned.
