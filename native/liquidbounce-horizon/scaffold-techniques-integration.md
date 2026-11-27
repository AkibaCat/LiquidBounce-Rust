# Scaffold Normal/Expand native dependency

Frozen LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`.

This is a dependency of the complete Scaffold runtime. It does not register a partly implemented module. Only static composition and temporary rustfmt parsing have run; centralized Rust and game checks remain with the root integrator.

Apply `scaffold-techniques-patch.py` after `block-placement-patch.py`. Register its returned native helper `client_scaffold_technique(request) -> String` in the same way as the other Scaffold helpers. The native math uses the existing `edge_random` Java/Kotlin random adapter. Apply `scaffold-click-inside-patch.py` after the shared placement-use dependency, before Step. It returns no exported helpers.

## Plan

```json
{
  "op": "plan",
  "technique": "Normal",
  "position": [0.5, 64.0, 0.5],
  "pose": "Standing",
  "serverRotation": [0.0, 70.0],
  "slot": null,
  "optimalLine": null,
  "aimMode": "Stabilized",
  "freeze": false,
  "height": {
    "state": {"placementY": 63, "startY": 64, "jumps": 2, "wasTowering": false},
    "sameY": "Off",
    "towering": false,
    "downRunning": false,
    "shouldGoDown": false,
    "ceiling": false
  }
}
```

Use the actual RotationManager server rotation. Supported predicted poses are Standing, Crouching, Swimming, FallFlying, SpinAttack and Sleeping (also accepts source uppercase enum names); actual player scale determines eye height. `slot` is a best hotbar index `0..8`, or null for the source's 64 sandstone planning fallback. This fallback is planning only and must not become an equipped stack.

`optimalLine` is null or `{ "position": [x,y,z], "direction": [x,y,z] }`. Aim modes are Center, Random, Stabilized, NearestRotation, ReverseYaw, DiagonalYaw, AngleYaw and EdgePoint. Normal uses FULL when Freeze runs, DOWN when shouldGoDown, otherwise NORMAL. Its face movement flag comes from actual horizontal key presses, as in PlayerLocationOnPlacement.

For Expand, set `technique` to `Expand` and supply `length` in `1..10`; `aimMode`, `freeze`, and `optimalLine` are unused. It searches the inclusive `0..Length` sequence, using actual player yaw, Kotlin Float sin/cos and truncation. It uses the zero offset, center aim and facing-away faces.

The height flags must already represent the source feature states: `towering` is the selected tower and actual physical jump; `downRunning` is Down.running; `shouldGoDown` is enabled plus actual physical Sneak; `ceiling` is Ceiling.running and canConstructCeiling(). `sameY` accepts Off, On, JumpKey, Falling and Hypixel. The native side reads actual velocity, movement, collision and input/physical jump. Height resolution keeps tower/wasTowering, Down, Ceiling, jump and SameY precedence. Each Expand candidate resolves height lazily and stops on the first target.

The result always returns a valid updated state even if target finding fails:

```json
{
  "target": {
    "interacted": [0,62,0], "placed": [0,63,0], "direction": [0,1,0],
    "point": [0.5,63.0,0.5], "minY": 63.0, "rotation": [0.0,80.0]
  },
  "heightState": {"placementY":63,"startY":64,"jumps":2,"wasTowering":false}
}
```

`target` can be null. Merge returned `heightState` into the runtime's existing height state; retain forceSneak and other unrelated state. In particular, Hypixel can reset jumps while evaluating candidates.

## Rotation and crosshair

Normal rotation request:

```json
{"op":"rotation","technique":"Normal","target":null,"requiresSight":false,
 "telly":{"enabled":false,"doNotAim":false,"resetMode":"Reverse"}}
```

Use the target returned by planning. Telly enabled+doNotAim returns the actual player yaw rounded to a multiple of 45 with ties to even and pitch at least 45 for Reverse; Reset returns null. RequiresSight performs the source trace and retains its original fallthrough behavior. Expand rotation ignores Telly/RequiresSight and aims at placed-block center from actual player eye. The response is `[yaw,pitch]` or null.

Crosshair request:

```json
{"op":"crosshair","technique":"Normal","target":null,"rotation":[0,80],"shouldGoDown":false}
```

It verifies from actual player eye over the full world with the maximum real entity/block range. Expand and Normal Down retain the planned hit on failure. A hit result is `{ "block":[x,y,z], "face":[x,y,z], "point":[x,y,z], "inside":false }` or null. Actual ray results retain the original inside bit; planned fallback has false exactly as BlockPlacementTarget.blockHitResult.

## Placement inside integration

Both `client_scaffold_action` checkPlacement and execute now require an actual bool either at `hit.inside` (preferred) or top-level `inside`. Never substitute false for a missing actual trace result. The execute operation scopes the value around client_do_placement, reaching the real UseItemOn packet, remote preview PlaceContext and local PlaceContext. The previous value is restored after completion and unwind. Existing callers of client_do_placement keep their prior behavior.

Clutch's crosshair result now has the same inside field. Its successful world trace keeps inside; the source's planned fallback is false.

## Verification prepared

`python native/liquidbounce-horizon/test-scaffold-techniques.py` parses nine composed native Rust sources plus two prepared smoke sources, without writing shared/staging files or executing Rust. `scaffold-aim-smoke.rs` covers trimming, random axes, nearest-face edges, vertex tie order, distant-origin invariance, degenerate faces, stabilized crop and segment projection. `scaffold-techniques-smoke.rs` covers Freeze/Down precedence, Telly ties to even, Float truncation, inclusive Expand search, early stop and Hypixel state mutation. Root should run them in the next centralized batch.
