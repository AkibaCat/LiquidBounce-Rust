# Replenish at LiquidBounce ae0eb396

The runtime owns one continuation id in the actual shared InventoryManager. It
does not send synthetic inventory packets. Native inventory/menu prediction,
player inventory slot mapping, menu guards, cancellation, atomic action chains,
click/start/close delays, requirements and server rejection locks remain in that
manager. The config includes all PlayerInventoryConstraints and legacy tick to
millisecond conversion, plus ItemThreshold=5 (editor 0..63), Delay=40 ms (editor
0..1000), ReplenishEmpty=true, Features={CleanUp}, InsideOf={}. Refill is an alias.

OFFHAND is scanned before slots 0..8. Sources are slots 9..35 with equal item and
effective components. The stable descending count sort is reversed for CleanUp,
including its equal-count order. UseSwap takes precedence over UsePickupAll and
requires the largest source count to exceed both the desired count and threshold.
PickupAll emits pickup/pickup-all/pickup as one chain. Normal refill reads each
captured source slot's current count at the continuation and emits a two or three
click chain. The pinned source's overflow third click is literally on the target
hotbar slot again, and this port preserves that behavior.

Each slot's completion records item memory, resets its chronometer and registers
a FIFO game-tick waiter for positive Delay. Its deadline uses real elapsed
milliseconds, never a tick-count conversion. Delay<=0 resumes synchronously.
The initial check uses Chronometer's strict `<`, while waiting uses `>=`.
Pending plan ids persist across waits and chains; expected menu/locked-item
guards also run at each waiter callback. The clock survives enable/world resets,
while enable and world changes clear tracked items. A real AbstractContainer GUI
replacement clears tracked items through an existing synchronous observation
hook; temporary internal inventory-menu switches do not produce ScreenEvent.

Effective native components include max_stack_size. A missing component defaults
to 1, confirmed against pinned 26.3 ItemInstance.getMaxStackSize bytecode. Item
memory keeps effective identity and a count of one; no runtime component registry
mutation is introduced by the port.

Owned locale updater writes simplified/traditional labels, module names and
descriptions into the actual nested display schema. The engine's existing game
locale selection controls when Chinese labels are rendered.

Validation is limited to one short state-machine smoke and read-only integration
and native transformer preparation. Production native and engine compilation is
performed by the root agent's serial official build. No in-game or server plugin
behavior is claimed by the smoke result.
