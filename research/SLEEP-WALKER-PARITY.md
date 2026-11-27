# SleepWalker

Baseline: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`,
`features/module/modules/exploit/ModuleSleepWalker.kt`. The stable module name is
`SleepWalker`, category `Exploit`, with search alias `BedGodMode`. The module
contains only common ClientModule settings; there are no additional choices,
timers, packets, or custom hitbox dimensions.

The original GameTick handler checks `player.isSleeping` and then performs, in
order, `player.pose = Pose.STANDING`, `player.clearSleepingPos()`, and
`mc.gui.setScreen(null)`. Official Minecraft 26.3 bytecode confirms that
`LivingEntity.isSleeping` reads the presence of `SLEEPING_POS_ID`, independently
of the current pose. `clearSleepingPos` stores an empty Optional.
`Gui.setScreen(null)` clears old focus and calls `Screen.removed`; it does not
call the screen's `onClose` callback. The distinction matters for the bed chat's
wake-up behavior and for preserving a typed chat draft.

`sleep-walker-host.rs` exposes the generated Host helper
`client_sleep_walker_tick() -> bool`. The main runtime calls it on GameTick only
while the module is running. It accepts only the window's actual player,
excluding a dedicated-server Game, a LAN guest's acting copy, a server movement
copy, speculative replay, and a missing player. If the local sleeping-position
field is present, it sets the actual motion pose to Standing, clears the sleeping
position and Horizon's cached sleeping flag, then returns the screen to gameplay.
The existing interrupted-chat cleanup removes bed-chat UI and preserves its
draft. Native screen ownership and focus are cleared and mouse capture resumes.
The helper does not invoke `wake_up`, send a stop-sleeping packet, teleport the
player, reset velocity, change the sleeping timer, release an occupied bed, or
run night-skipping logic. These are absent from the original handler.

The source comment mentions a small hitbox, but the executable handler supplies
no separate size override. The port follows the actual Standing pose and its
normal dimensions rather than inventing an additional reduction. Clearing the
cached sleeping flag is necessary because Horizon's movement pose selection
uses that cache; otherwise the next motion update would restore Sleeping.

Reproduce the isolated native check with a fresh output folder:

```powershell
python -X utf8 native/liquidbounce-horizon/test-sleep-walker-patch.py --out build/sleep-walker-04
```

The successful handoff evidence is `build/sleep-walker-03/report.json`: five
tests plus Clippy with `-D warnings`. The fixture compiles the production helper,
the real `horizon_mod_api_gen` Host implementation and original tick dispatcher,
the actual pose/dimension methods, and the original chat draft/removal methods.
It verifies the independent sleeping-position guard, correct standing dimensions,
unchanged position/velocity/timer, Chinese chat draft retention, repeated server
sleep updates, non-bed screen closure, and all identity exclusions. An enabled
fixture listener exercises the generated existing tick event and helper; the
main actor/registry integration is tested separately by the main task.

The same check inspects the official 26.3 JAR (SHA-1
`e877b6a07acd633fb3bb475002175cec036e7b87`) for the sleeping-position and screen
lifecycle contracts and records the javap output. This is bytecode inspection,
not a separately executed Java state oracle. The Rust fixture uses storage,
attribute and mouse-capture shells; it is not a complete game or a claim of
server-side permission bypass. Full client compilation and in-game sleeping
screenshots belong to the main integration run.
