# ElytraSwap integration work

Original: LiquidBounce ae0eb396, ModuleElytraSwap.kt. Player category,
ChestSwap alias, disableOnQuit, complete PlayerInventoryConstraints.

Owned policy: `native/liquidbounce-behaviors/src/elytra_swap.rs`.
`build/elytra-swap-source-04/report.json` passed: 852 scenarios against the
unchanged Kotlin module using real kotlinx.coroutines suspension/cancellation,
Rust tests and Clippy. This covers the lazy search order, captured direction,
empty chest's two pickups, interrupted chains, and disable after perform resumes.
Its stack attributes and inventoryPlan registration are explicit providers.

`elytra-swap-patch.py` is now wired into the aggregate patch generator. It adds exact
GLIDER, nextDamageWillBreak and CHEST_ARMOR facts to the existing native inventory
stack records. GLIDER/damage use the existing actual-MC-verified component helper;
CHEST_ARMOR uses the server's merged item tags, independent of EQUIPPABLE.

Canonical integration now includes complete constraints configuration, ChestSwap
alias, persistent state, onEnabled reset and disableOnQuit. The shared executor
submits the atomic pickup chain with a live owner. It consumes Completed, releases
the behaviors borrow, synchronously disables the real registry, then acknowledges
the manager effect. Cancelled clears only the old plan; its captured direction
survives. An empty selection disables immediately outside the actor borrow.

`build/elytra-swap-actor-02/report.json` passes six actual actor/manager tests and
Clippy. ElytraSwap cases cover all hotbar/main/offhand positions with occupied and
empty chest slots, exact pickup lists, empty cursor, completed-only disable and
pending cancellation retaining direction. `elytra-swap-actor-01` preserves a
fixture compile failure (OwnerState has no Default), corrected in 02.
`build/elytra-swap-config-01/report.json` passes the full production configuration
test suite and Clippy, including the complete ElytraSwap constraints tree, file
round trips, requirement parsing and legacy tick-to-millisecond migration.
Canonical behaviors Clippy with all targets and warnings denied also passes.

Still required before counting the feature complete:

- Validate with the real shared manager, actual generated native stack extraction,
  configuration, cumulative engine tests and fresh game armor/offhand transfers.

Engine actor tests now include registry alias, settings round trip, enable reset
and quit lifecycle checks. The canonical registry declares ElytraSwap; the fresh
production build and 74-module game validation are coordinated separately.
