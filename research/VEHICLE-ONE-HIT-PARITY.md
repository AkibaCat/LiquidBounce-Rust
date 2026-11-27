# VehicleOneHit source parity

The pinned LiquidBounce commit is `ae0eb396c07f22383789b28d5912d714c6ca0a86`.
The owned implementation ports the entire `ModuleVehicleOneHit.kt` handler and
the `SwingMode.HIDE_SERVER` attack-swing path. The module has only the original
common settings and belongs to Exploit.

Actual MC 26.3 runtime class hierarchy determines eligibility: `Boat` and
`Minecart` qualify. Chest boats, rafts, chest rafts, and specialized minecart
classes do not. This includes the target version's poplar boat. Current protocol
777 emits 19 ordinary Attack-then-local-Swing pairs. The source's <=1.8 branch
emits three Swing-then-Attack pairs; the native transport itself supports only
the pinned protocol. A canceled AttackEntityEvent still runs this handler.

Host calls run outside the registry lock. Each Attack uses the ordinary
`Game.send` / `Session.send` / typed packet-interceptor route, preserving packet
cancellation, mutations and accepted observations. HIDE_SERVER resolves the
effective main-hand item's actual attack-animation component, then calls
`swing_hand` without a Punch packet. NoSwing client cancellation remains active.

The client pre-attack seam covers the remote entity/object and mob routes and
the local boat/cart entries. Local entries re-resolve the target id after a
handler, because a handler can remove or reorder the collection. The normal
startAttack swing is retained even when the game-mode event is canceled.

Horizon `game/integrated.rs` runs every playable single-player world on an
independent server thread and joins its client through a memory connection.
Those ordinary Attack packets are consumed by the same `lan_guest_packet` /
`guest_attack` server route used for LAN players. `Game.send` with no Remote is
not a playable single-player network: legacy/headless direct-world fixtures
have separate local attack paths. This distinction was checked against the
actual fixed source instead of assuming that single player bypasses packets.

`../build/vehicle-one-hit-07/report.json` passes three tests and Clippy with
`-D warnings`. The unmodified Kotlin module and SwingMode compile against actual
MC 26.3 classes, ItemStack components and ServerboundAttackPacket. The oracle
covers 161 entity types and 15,360 cases, including target-id extremes, protocol
branches, disabled state, event cancellation and three swing animations.
Constructor-free actual entities and the observed player swing are explicitly
listed environment boundaries. The Rust fixture compiles production behavior,
byte-exact native helpers, real protocol packet types and both transformed
local entry functions. Its resource/game/damage environment is a shell; it does
not establish full client parity or server destruction.

The real integrated-server boat/cart destruction scenario is in
`scripts/verify-native-integration.py`. A successful fresh run is required
before reporting that scenario as passed. The full Java/Rust client
differential and complete 236-module integration remain separate work.
