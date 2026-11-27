# Notifier handoff

Pinned source: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, ModuleNotifier and TotemPopTracker. Registered as Notifier in Misc; all fourteen setting leaves, editable message formats and both output modes are connected. The integrator and Chinese updater have already been applied by the root owner.

The native packet bridge delivers final PlayerInfoUpdate/Remove, EntityEvent and RemoveEntities packets in actor order, including cancelled packets as the source event handler does. Real player equipment snapshots carry component identity, localized hover names, use duration, active hand and elapsed use ticks. Join/leave and game mode caches preserve the source's distinct UTF-16 name guards and world/disable lifetimes. Consumption and held-item events keep independent per-player/per-hand state. Global totem counts continue while Notifier is disabled and clear on death, removal or world change; self and friend pops are filtered.

Default stored message formats remain the original English values. At display time the unchanged defaults use the game's Simplified or Traditional Chinese locale; customized formats remain unchanged. Formatter handling includes indexed/reused arguments, widths, precision and supported string/integer conversions. Formatting errors are logged locally. Enum names are represented by native strings; Java enum identity hashes cannot be reproduced by `%h`.

AntiBot is a pending shared dependency: `NotifierRuntime.bot_entities` is currently a consumer interface, awaiting the complete AntiBot service. This is not an AntiBot implementation or parity claim. Its owner has been assigned the real service and synchronization before the Notifier tick.

One focused smoke passed in `build/notifier-smoke-01.log`: name and mode guards, consumption completion, held counts, supplied bot filtering, friend/totem behavior, removal, formatter and disable. Native and engine compilation passed in `build/check-native-160-02.log` (official report `build/integrated-check-160-02.json`). Final live validation remains pending.
