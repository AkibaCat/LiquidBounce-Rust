# AutoWalk dependency assessment

AutoWalk is not implemented or registered. Its Forward, Direction, Goto and
Follow modes depend on the complete Navigator and Minecraft Agent integration;
a straight-line Forward approximation cannot represent the original module.

The pinned source uses `minecraft-agent-common:1.0.0-SNAPSHOT` and adapter/Fabric
`1.0.0+26.3-SNAPSHOT` from the CCBlueX snapshots repository. Agent navigation
requires simulated terrain, standability, path search, block/door actions,
vehicle bodies, flight/elytra, rotations, movement keys and route rendering.
Vanilla Horizon mob pathfinding does not provide those semantics.

Direction chooses a goal 1024 blocks ahead, recomputing after 512 blocks or a
20-degree turn, with radius 2. Goto resolves destination standability and tests
rootVehicle arrival within radius 1 before success notification/disable.
Follow searches other players case-insensitively, retains lastSeen and uses
the configurable distance (default 3, editor 1..10).

Shared settings include AllowBreak/AllowPlace false, AllowParkour true,
RiskyJumps false, MaxFall 3 (0..20), PathRange 64 (16..128), AllowFly and
AllowElytra/ElytraRockets true, AllowBoat/AllowEat false, plus Scaffold bridge
interaction. These dependencies remain an explicit implementation boundary.
