# InventoryMove integration

Pinned LiquidBounce ae0eb396. All eleven setting leaves are implemented: four Behavior policies, PassthroughSneak, client/server sprint, client sneak, Timer and Blink. Real remapped keyboard/mouse bindings are read through native container screens; book, sign, dialog and EditBox input suppress movement. The original theme's existing `/client/typing` state supplies search focus without an added frontend route.

Safe intercepts container clicks before local prediction and closes an actual server inventory on the source keyboard events. StopOnAction intercepts all five source container packet classes at FIRST_PRIORITY, clears the six movement inputs, and silently dispatches scheduled packets at the following idle loop. Disabling preserves already scheduled continuations; world changes clear stale sessions. Sprint controls preserve client/server ordering, and force sprint requires directional movement.

Timer requests use the shared Timer service at priority 30. Blink uses the real shared Criticals/Blink manager: Queue dominates Pass and Flush, so another module's queue cannot be overridden. MaximumTime uses the same monotonic native clock as screen opening. Simplified and Traditional Chinese cover every setting and local notification.

One combined focused smoke passed in `build/inventory-move-smoke-01/report.json`: policies, config File/Interop round trip, sprint precedence, Safe, five packet classes, scheduling, disable and Blink deadlines. The accumulated native patch and both locale files passed that same focused check. Official batch native/engine compilation is pending; packet signatures and live behavior are not inferred from the classification stub used by the smoke. No game or full regression test has been run.
