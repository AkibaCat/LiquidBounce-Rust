# AntiCheatDetect

Fixed source: LiquidBounce ae0eb396, ModuleAntiCheatDetect.kt, AntiCheat.kt and the required ServerObserver fields.

An always installed packet observer captures login, the first five ping/keepalive samples, original transaction IDs and accepted channel registration. Eight evidence algorithms preserve source ordering, signed wrapping and probability calculations. Completion reports only when the module is enabled. Enabling also reports existing evidence. The native result is styled local client chat with automatic English, simplified or traditional Chinese wording; it is not sent to the server.

One focused smoke: `build/anti-cheat-detect-smoke-01.exe`, one test passed. Full gameplay and upstream differential verification remain pending. Configuration-stage channel transfer and the unrelated ServerObserver TPS/hosting/reconnect fields are not claimed as implemented by this module.
