# Shared Agent land simulation

The owned `navigation/simulation` sources translate Minecraft Agent common
simulation at commit `0abf69ac35157f33348310326017f4e3fea6b95f`.
They preserve both modern and 1.8 land physics, block support, collision ordering,
step alternatives, edge backoff, block effects, and the original shared collider
cache when copying motion. `Terrain<B,L>` implements the simulation grid when
`B: SimulationState`; block facts must come from actual block state identities.

Fresh source evidence (all `passed: true`):

- `build/simulation-collision-04/report.json`: 2,590 sequential commands against
  unchanged original BlockColliders, Collisions, Support, EdgeBackOff and
  MotionTraits. Exact results and ordered shape queries match. Test and Clippy
  with `-D warnings` pass.
- `build/simulation-motion-03/report.json`: 8,190 sequential commands against
  nine unchanged original simulation sources. Modern and 1.8 motion, reset,
  copy, copyFrom, original `reset(Player)` getter order, effects, and all world
  queries match. Test and Clippy with `-D warnings` pass.
- `build/simulation-sine-table/report.json`: all 65,536 float sine values were
  extracted from the actual common artifact. `sine-table.bin` is the resulting
  complete table, avoiding host libm differences.

The intentional original quirks remain: `copyFrom` does not copy sneaking or
replace target traits/cache; reset does not clear sneaking or landingFall.
`MotionPlayer::reset_player` preserves the original getter/query sequence.

This is common **land** simulation, needed by the original jump planner.
It does not implement fluids, climbing, slow falling, or the 26.3 vanilla
SimulatedPlayer used by SafeWalk/PlayerSimulationCache. Do not register SafeWalk
against this land simulator or count SafeWalk/Parkour/AutoWalk as completed by
these shared dependency tests. Native full-client differential remains not_run.
