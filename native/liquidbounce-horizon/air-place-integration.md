# AirPlace

Fixed source: LiquidBounce ae0eb396 / MC 26.3. Shared registry, file/interop configuration, actor, raw scrolling, real local and remote useItem HEAD, and FULL_BOX world preview are integrated by `integrate-air-place.py`.

Main and offhand item components are read from the actual component resolver. Consumables are excluded; block items use real default-state can_survive, and spawn eggs, armor stands and non-gliding fireworks are supported. Air/fluid selection respects LiquidPlace. CustomRange validates the original and adjusted targets. The local path scopes GuestUse::OnBlock, the remote path scopes the actual forced hit and invokes useItemOn; the parent useItem returns PASS for a valid target, including failures. Preview uses the existing world primitive renderer.

The single focused smoke `build/air-place-smoke-01.exe air_place_range_and_scroll --exact` passed. It covers target normalization, degenerate input, modifier handling, raw scroll conversion and clamping. Official compilation is tracked in the batch Check reports; no fresh game or full differential validation has run. The latest EXE remains the earlier 76-module build.

Remaining final integration cases: actual block/entity reach mismatch on a MISS target, custom-range fluid placement and both use-hand paths, occlusion and preview color. Simplified and traditional Chinese labels plus 11 descriptions per locale are in the owned localization resources and follow the current game language.
