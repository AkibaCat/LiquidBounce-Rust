# Hats — complete six-mode integration handoff

Pinned LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Horizon-Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.

Root applies these two commands serially, then performs the normal official native/engine aggregate Check:

```powershell
python native/liquidbounce-horizon/integrate-hats.py
python native/liquidbounce-horizon/update-hats-locales.py --directory native/liquidbounce-interop/resources/localization
```

The transactional integrator validates all anchors and all originals before writing seven shared files. `--check` passed. No shared, upstream or staging files were modified by this handoff. AutoBreak and ItemScroller were untouched.

## Implemented behavior

- All six modes in original order: Cone (128 triangles), Halo (144×12 torus quads), Orbs (eight triangles per orb), Flower and Star (point count×32×12 torus quads), and Image.
- All common settings independently retained for every mode: FollowRotation, EquipmentOffset.ArmorOffset, ShowDamage, FriendsOptions.ViewOnFriend/Distance and FirstPersonView. All mode-specific settings, defaults, ranges, suffixes and original sibling group structure are preserved. Cone RadiusSettings and Orbs Wave are siblings of HatSettings.
- HeightOffset contains the original cubic Height curve, Period and Symmetric. Time comes from engine startup, rather than module activation. Six-mode inactive settings and the complete Plain/Shader subtree roundtrip through File and Interop. Module mode tag is included in the module list.
- Real own, remote and LAN players are supplied by the native Game bridge. Current positions determine strict three-dimensional friend distance; interpolated positions and rotations determine the mesh. The local player uses actual `rope_rot_old` yRotO/xRotO, remote entities their previous yaw/pitch, and LAN guests a ClientTick.Pre snapshot of their real rotations. Fresh entities skip old rotation interpolation. Head-slot stacks, native dimensions, eye height and hurt time are read directly.
- Friend selection uses the actual engine FriendBook, and FreeLook checks enabled state as upstream requires. FollowRotation rotates both the equipment/height offset and the mesh around the actual eye height. The source color lookup table, double sine path, Float FMA/truncation, damage alpha and shaded shapeColor alpha are preserved.
- Image is always Plain, with original UV order, X/Z rotation, two-axis scale and ColorModulator. PNG/JPEG decoding uses Horizon's existing decoder bodies. File reads/decoding occur on a worker; GPU upload occurs in prepare. New valid file selection replaces the pending request, so outdated results cannot overwrite it. Invalid files and decode errors preserve the previous valid texture. Errors are delivered once through native chat. Texture objects are released on successful replacement.
- Shape rendering independently derives from the frozen Breadcrumbs ShaderLayer core; all nine original GLSL styles remain available without editing that core. All six modes use no depth test and no face culling. Image has a separate alpha-blended texture pipe.
- Simplified and traditional Chinese updater provides 228 Hats keys per language on current dictionaries, including complete rendering styles inherited from the frozen Breadcrumbs translations. Only copied dictionaries were changed in the focused test.

## Focused validation

`build/hats-smoke-04/report.json` passed four tests in one Cargo invocation:

1. Every mode's mesh counts, finite vertices, source coordinates, armor height, damage, Shader alpha, strict friend selection, FreeLook and height animation.
2. File/Interop roundtrip of all inactive mode settings and all shader styles, image metadata and sibling groups.
3. Production native Game bridge and engine runtime, including own/remote/LAN rotation and position interpolation.
4. Actual production PNG/JPEG decoder bodies, async file loading, WGPU texture upload/draw/readback, color/alpha modulation and retained valid texture after decode failure.

The actual native patch composed seven files in memory. Game/actor/Host declarations are fixture providers. Official aggregate native/engine compilation and in-game visual parity remain root-owned; isolated fixtures are not a claim of game parity. The shader core reuses the existing Plain plus all-nine-style GPU evidence.

Historical smoke01 records a color-mask type-inference compile error. Smoke02 records one WGSL reserved-word failure and an overly strict test expectation for the source Float rotation result. Smoke03 passed before the final async lifecycle refinement; smoke04 validates that final refinement. All evidence is retained, not overwritten.
