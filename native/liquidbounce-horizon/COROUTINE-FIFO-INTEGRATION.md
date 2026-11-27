# Global CoroutineTicker integration

Pinned source: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`,
`event/CoroutineTicker.kt`. The queue belongs to the complete client lifetime.
It is shared by all module coroutines; it must not be recreated per tick or
advanced in fixed module order.

## Queue contract

`engine/src/coroutine_ticker.rs` supplies `CoroutineTicker: Clone + Default`.
Every clone refers to the same queue. `register(WaitOwner, callback_id)`
returns a globally increasing ticket and only appends to pending.

`begin_tick() -> Option<TickBatch>` freezes running followed by pending, clears
pending and prevents recursive dispatch of that batch. With each
`batch.next_task()`, dispatch exactly that task and call
`batch.finish_current(keep)`. `keep=true` retains its original FIFO position.
Registrations made while a callback runs remain pending until the next batch.
Dropping an unfinished batch retains its current and remaining tasks in order.
No queue mutex is held while module callbacks or native effects execute.

The batch guard only covers callback dispatch. The whole Minecraft tick needs
the separate depth scope from `coroutine-ticker-patch.py`; runtime dispatches
only when `Host.client_coroutine_outermost()` is true. The scope must remain
alive through the end of all ordinary tick handlers, because those handlers
can reenter after the FIFO batch has already finished.

## Module API and registration order

Derp, AntiAFK and AutoFish expose `take_wait_registrations() -> Vec<u64>`.
After **every** state-machine call, drain these ids into the shared queue
immediately, before releasing control to any native effect or another module.
Cancellation does not erase the registration list: the source also enqueues
callbacks that subsequently discover their continuation was cancelled.

- Derp: `advance_callback(id, running, live_snapshot) -> bool`. Every wait is
  `waitTicks(1)`, so the old callback always completes (`false`). Continuing a
  forward/backward loop allocates a new local id, queued for next tick.
- AntiAFK: `advance_callback(id, running, live_snapshot, live_server_rotation,
  rng) -> Result<(keep, Option<Effect>), &str>`. A positive countdown retains
  that same callback. Completing a random direction wait may register a new
  outer-delay callback on the same logical lane; the old id still completes.
- AutoFish: `advance_callback(id, running, live_rod_hand, live_hook_present,
  rng) -> Result<(keep, Option<FishHand>), &str>`. An unsatisfied `tickUntil`
  and unfinished countdown retain their id. Reel/recast/autocast transitions
  create new ids. After each real use, call `after_use` and drain registrations
  before performing any returned synchronous second use.
- HandDerp: `start_delay(running, || ticker.register(WaitOwner::HandDerp, 0))`
  takes its global ticket directly. Dispatch
  `advance_callback(task.ticket, running) -> (keep, Option<MainHand>)`.

Every callback checks identity **before** applying guards or decrementing.
A cancelled old task therefore cannot cancel or shorten its replacement even
if both appear in the same frozen batch. Missing ids simply complete.

Obtain each callback's guards and snapshots at its own FIFO position. Applying
all guards/countdowns in a preliminary module pass is observably different:
earlier callbacks can edit yaw, rod hand, world state or settings synchronously.
The owned `anti_afk_runtime.rs` and `derp_runtime.rs` perform these steps and
register waits automatically. Their dispatch entry points are
`anti_afk_callback(host, in_game, callback_id) -> bool` and
`derp_callback(host, in_game, callback_id) -> bool`.

The old per-module `begin_tick`/`advance_wait`/`next_ready` APIs remain for
standalone compatibility fixtures. The integrated runtime must use the
single-callback APIs exclusively.

## Evidence

`test-coroutine-fifo.py` snapshots production sources without modifying shared
staging or upstream. Each run needs a fresh evidence directory.

- `build/coroutine-fifo-03/report.json`: 23 Rust tests, Clippy with warnings
  denied, and unmodified `CoroutineTicker.kt`, `ModuleAntiAFK.kt`,
  `ModuleDerp.kt` compiled together with real kotlinx.coroutines. Ten shared
  yaw ticks match Rust production state machines exactly. The old AntiAFK
  callback resumes before the most recent Derp callback on tick 10; both
  observe 190 degrees. Fixed module order would leave Derp at 10 degrees.
- The same suite verifies opposite registration orders, synchronous swing
  callbacks changing a later live snapshot/guard, new AutoFish reel/recast
  callbacks deferred to next tick, cancelled callback identity across four
  real modules, independent random direction/outer delays, FIFO retention,
  and the verbatim native depth scope/helper after the outer batch finishes.
- `build/derp-source-03/report.json`: all 150 original-module angle/RNG ticks
  still match after the stable callback API additions.
- `build/hand-derp-source-02/report.json`: all 562 original-module command rows
  still match after adding its identity-checked FIFO callback entry point.

Minecraft, event, settings, fastutil list and native Game receiver stubs are
explicit fixture boundaries. These fixtures establish queue/state-machine
equivalence; the full Horizon build and real game trace are separate evidence.
