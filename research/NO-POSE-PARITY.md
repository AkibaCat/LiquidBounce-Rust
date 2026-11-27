# NoPose source and consumer verification

The implementation follows `ModuleNoPose.kt`, `MixinAvatar.java` and the
`MixinEntity.setPose` HEAD injection at the fixed LiquidBounce commit
`ae0eb396c07f22383789b28d5912d714c6ca0a86`.

`NoSwim` defaults to false. When enabled and running, it cancels assignment of
`Pose.SWIMMING` only on the actual local player. It does not clear the separate
swimming movement flag, change key input, force a different pose on enabling, or
cancel sleeping, fall flying, spin attack or crouching. A retained local policy
snapshot does not cancel poses on server copies or server movement replay.

`SneakHeight` defaults to `1.15`; the exact tags and unscaled float dimensions are:

| Tag | Width | Height | Eye height |
| --- | --- | --- | --- |
| 1.8 | 0.6 | 1.8 | 1.54 |
| 1.9 | 0.6 | 1.65 | 1.54 |
| 1.15 | 0.6 | 1.5 | 1.27 |

The original `MixinAvatar` has no local-player or client-side guard. The central
avatar dimensions therefore cover remote players, mannequins and integrated
server bodies as well. Other species and poses retain their prior dimensions.
Only the window publishes this global policy; server ticks cannot reset it.

The real Minecraft 26.3 JAR oracle invokes `EntityDimensions.scalable`,
`withEyeHeight`, `withAttachments` and `scale` with the source-derived NoPose
constructor. Three choices at six scales produce 18 rows. The box and eye values
use float multiplication, while attachments use double multiplication of the
original base coordinates. In this JAR, `Player.DEFAULT_VEHICLE_ATTACHMENT.y` is
double `0.6`, with raw bits `4603579539098121011`. Both passenger and vehicle
attachment results match every oracle row bit for bit. NoPose attachments are
cached with the local body's dimensions; setting changes do not refresh an
unchanged pose. Existing non-NoPose attachment paths retain their previous
behavior.

`test-no-pose-patch.py` transforms pristine sources in memory and compiles the
complete production `pose.rs`, including collision-fit selection, crouching,
server pose ticks and passenger pose ticks. Tests also compile actual fluid
eye/threshold methods, guest pose/dimensions/bounds/eyes, the original guarded
avatar dimension branch, dismount dimensions, particle body dimensions, both
attachment entry points and the actual API generator's Host implementation and
hook dispatch. The collision fixture uses real Horizon Aabb intersections.

Camera eye height, model pose, model light probes, first-person hand submission,
travel boxes and server anticheat consumption are verified by source guards.
These guards are not GPU render tests. The Mob shell retains Horizon's existing
dynamic `awake_dims` architecture; this fixture does not establish equivalence
to every vanilla remote-avatar dimension-cache refresh phase. Full game and
image verification remain separate integration checks.

Evidence is written under `build/no-pose-hook-tests/`: `result.json`,
`dimensions-oracle.json`, `test.log`, `clippy.log`, generated Host source and the
transformed production `pose.rs`.
