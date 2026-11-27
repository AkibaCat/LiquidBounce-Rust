# AutoDodge

Pinned ModuleAutoDodge/DodgePlanner/Minecraft-Agent SimulatedArrow. Registered category is Combat, matching source despite the movement path. All nested settings, default choices and File/Interop help are wired with simplified/traditional Chinese resources.

MovementInput uses actual NativeSimulation over the scoped cloned 26.3 player, owned chunks and entity collisions for 80 predicted ticks; arrows and both trident stores are merged in actual JoinSeq order. Returning player tridents are excluded by their real returning counter. Each projectile uses the source Agent drag/gravity and collider rays with null entity context. Planner probes both 0.6/1.6-height collider rays, preserves the strict padding/safe distance and 0.05 side preference, and uses source yaw arithmetic.

Blink GameTick synchronously queries that same native predictor with rigid positions for 40 ticks and expansion 0.7. It preserves the packet-distance filter and index, chooses the first safe packet or the best delayed hit, flushes the true shared queue and uses localized Blink notifications. No deferred empty predictor or fabricated geometry is used.

Rotation writes real camera/player yaw, jump is deferred to the next MovementInput after other permanent handlers, Timer uses priority40/resetAfter1 and the shared Timer provider. Inventory/using-item/Scaffold ignores and Blink suppression are real conditions. MurderMystery will publish its source disallowsArrowDodge to AutoDodgeRuntime.murder_mystery_disallows when that module is integrated; it is currently unavailable and not falsely registered.

Focused check: build/auto-dodge-smoke-01.exe, one test passed: projectile progression/order/block stop, stationary arrows, packet selection/filter, planner urgency and angle direction. Final source review corrected float-cast placement for Rotation.lookingAt yaw after that check; the formal batch Check is pending. The short fixture does not establish in-game parity. Final all-module and fresh-game tests remain not_run.
