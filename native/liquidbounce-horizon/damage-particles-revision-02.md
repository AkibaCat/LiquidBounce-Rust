DamageParticles revision02 — 2026-11-26

LAN ServerPlayer has a separate knownMovement override. Tick snapshots now use actual g.known_movement, and the actual guest actor-turn swap publishes/restores its movement into the setter capture thread-local context. Ordinary mobs, local client-player exclusion and other living actors retain the prior adapter behavior. No staging files were modified.

Revision01 freeze remains historical. Final movement check is build/damage-particles-smoke-03/report.json; it additionally checks the captured actor movement survives the real setter callback. Root native compilation/visual stage remains aggregate.
