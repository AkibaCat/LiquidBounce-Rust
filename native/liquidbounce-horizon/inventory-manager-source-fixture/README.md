# InventoryManager source oracle fixture

The runner copies the pinned upstream `InventoryManager`, `InventoryPlan`,
`InventoryAction`, `InventoryExtensions`, `Guards`, and `EventListenerScope`
sources into each evidence directory without changing their bytes. The real
Kotlin JVM standard library, `Random.Default`, JOML vectors, and
`kotlinx.coroutines` execute those sources. `oracle.rs` sends the same commands
through the production Rust manager and compares each observable row.

## Deterministic time

`Clock` supplies the fixture render dispatcher and delay scheduler. Main sets
the real coroutine library's `kotlinx.coroutines.main.delay=true` property before
manager initialization. This also routes delays made under the original
`EventListenerContinuationInterceptor` through Main, since that interceptor
does not itself implement `Delay`.

`patch_monotonic.py` reads the real `MonotonicTimeSource.class` from the selected
stdlib jar. It verifies the sole `System.nanoTime()J` method reference and its
exactly two callers (`read` and `<clinit>`), then changes only that reference's
owner to `fixture.Clock`. Every original method's bytecode, including Kotlin
time marks and Duration arithmetic, is retained. The fixture classes precede
the stdlib jar on the classpath. `source-clock.json` records both class hashes,
the reference, and the original invocation offsets. `sourceMillis` in every
row observes the actual standard library mark elapsed time.

## Fixture boundaries

Minecraft screens, menu contents, stack components, global inventory settings,
constraint storage, native click helpers, and packet classes are deterministic
fixture objects. In particular, the regular menu geometry is deliberately
synthetic and a stack's complete item/components identity is represented by a
string. Native click prediction covers only the inputs used by these commands;
it is not a replacement for Minecraft's complete container prediction.

Creative-add/drop helper effects follow the official MC 26.3 helper conditions:
add requires creative mode; drop additionally requires a nonempty stack and
allows no screen, a creative screen, or an unrelated screen, while rejecting
other container screens. Feature-flag negotiation is outside this fixture.
An explicitly supplied slot remains an add operation even when its server id
is negative. The manager's last-click handling is compared independently of
whether a helper produces a packet effect.

`nearestFirst` uses the inventory menu in every screen state, preserves input
ties and duplicates, and covers unresolved/negative container positions and
overflowing squared distances. The real Kotlin implementation seeds `minBy`
with the first item and compares subsequent distances through `Float.compare`;
an unresolved slot's `Float.MAX_VALUE` therefore precedes an overflowing valid
distance of positive infinity.

A passing source report confirms source execution only. A passing Rust
comparison confirms behavior against this fixture, including random state,
time, queue/continuation effects, cursor bits, stacks, and wait indicators.
Production host/runtime tests and in-game verification remain separate
evidence and must not be inferred from either result.
