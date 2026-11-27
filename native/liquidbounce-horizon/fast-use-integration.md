# FastUse

Pinned specification: LiquidBounce `ae0eb396`, `player/ModuleFastUse.kt`.
Immediate, ItemUseTime and Crossbow preserve their separate settings, mode tags,
FastEat alias and runtime ordering. Conditions read the real consumable component,
ground state, movement and regeneration; StopInput runs at critical input priority.
Immediate suspends on the existing FIFO coroutine ticker, cancels on condition
loss/disable/world change and reads speed and packet type at send time. Its timer
request uses the shared Timer service at priority 20. ItemUseTime checks actual
ticks held. Crossbow uses inventory SWAP and the actual item interaction path twice.
Release uses the LivingEntity release path without adding a release network packet.

Simple verification: `fast-use-smoke-01.exe`
`immediate_timed_crossbow_and_cancellation_smoke`: 1 passed. Full integrated
build and game verification are recorded separately; this is not a game pass.
