# Agent jump planning and assisted movement

The owned `navigation/walk_jump` sources translate Minecraft Agent common at
commit `0abf69ac35157f33348310326017f4e3fea6b95f`: Jump, Jumps, JumpTargets,
Airtime, RunUp, Leap, AssistedJump, and Landings. They use the separately verified
common land simulator through the actual `Terrain<B,L>` queries and shared
collider cache.

Fresh `build/jump-chain-04/report.json` has `passed: true`: 3,881 sequential
commands against unchanged original Kotlin sources, exact numeric results,
state, and ordered terrain queries. Test and Clippy with `-D warnings` pass.
Report SHA-256: `A946D958886859169D179B6B529CF9137E7740A918A5BBD5BEB340D0BB250B5B`.

Coverage includes successful and failed plans and refinements, fromHere,
takingOffNow, lastFromHere, takeOff, air steering, candidate callbacks and early
termination, Leap decision state, assisted jump target/arc/landing lifecycle,
and 1,064 exact Java math commands including NaN, infinities, signed zero, and
random vectors. Source snapshots and their hashes are preserved by the fresh
evidence script.

`Jumps<B: SimulationState>` accepts `Terrain<B,L>` explicitly to preserve query
and callback ordering. `LandingRef = Arc<Landing>` retains target identity.
`Leap::tick` accepts `MotionPlayer`; `Leap::decide` accepts copied inputs.
`JumpTargets::for_each_options` calls its callback immediately with mutable
terrain, allowing the original nested plan/short-circuit behavior.

Two original yaw expressions intentionally use different rounding: Airtime
multiplies the double atan2 before casting to float; Vec3 yRot casts atan2 to
float before multiplying the float RAD_TO_DEG. The local Java atan2 translation
retains its Sun license and avoids host libm discrepancies.

The evidence directly covers planned hops=0. Leap also covers explicit hops=1;
successful Jumps takeOff with hops=1/2 and the public Leap tick wrapper have not
yet received dedicated source-oracle scenarios. This is shared dependency
evidence, not module registration or a completed Parkour/AutoWalk/SafeWalk.
Native full-client differential remains `not_run`.
