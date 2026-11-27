# ChestCleaner

Fixed source: LiquidBounce ae0eb396, ModuleChestCleaner.kt and both screen filter value groups.

The actual game inventory executor obtains live menu slots, stack identity/components, menu type and the displayed title in the selected Minecraft language. It submits each Throw action as a separate continuation with one shared plan ID, then closes the captured menu if requested. Inventory constraints, live owner cancellation and world cleanup use the common InventoryManager. Registry and title filters include all 16 title categories and chest boat variants; the UI exposes the actual 26.3 menu registry. File/import settings and simplified/traditional descriptions are integrated.

One focused smoke: `build/chest-cleaner-smoke-01.exe`, one test passed. Official native/engine integration check is tracked separately in the build reports. Full gameplay and upstream differential verification remain pending. Unknown menu registry values currently resolve by dropping an unknown entry rather than retaining a nullable registry object; that edge case remains for final differential review.
