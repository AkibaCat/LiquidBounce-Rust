# LogoffSpot

Pinned LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, Horizon-Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`. Complete Render module `LogoffSpot`, locale key `logoffSpot`, tag=null, +1 module. Only source setting `SendInChat=true`, with File/Interop roundtrip. No frozen AutoBreak/ItemScroller, pristine upstream or staging writes.

The native bridge captures actual Player mobs before RemoveEntities erases the original profile/name/texture maps, and also covers direct client removals. It excludes existing LogoffSpot ghosts and Blink's dummy, and suppresses only the second capture of the same entity join. Each ghost is a freshly initialized native player Mob: actual feet position, yaw/pitch/head rotation, known player inventory equipment including component-bearing main hand/armor, exact skin/slim/3D skin/cape data, and getActualHealth's case-sensitive below_name scoreboard override. The original entity UUID is the record key; each fake has a fresh random entity UUID and collision-checked negative local entity ID. Its original game profile remains available for native player skins, names, rendering and targeting. A timestamp is retained as upstream does. No custom shader or proxy model is used.

All playable windows are clients even in single player/LAN: pinned `game/world_session.rs::open_world` starts the independent server thread; the window uses the ordinary remote protocol. Creation requires both `native_window()` and `remote.is_some()` and excludes `lan_acting_uuid`. Thus fake entities never enter the authoritative Game, persistence, simulation, or LAN broadcast. There is no additional server Guest hook.

Each real client GameTick checks the ghost's actual loaded chunk first, then looks for an actual live Player with the original entity UUID. Unload takes priority over reappearance. Removal erases the fake native entity, random UUID and remote profile maps. Disable uses a one-shot Host cleanup immediately; native world/entity resets clear the map and live ghosts, while the engine world event clears packet IDs. Optional local chat uses exact upstream English keys/templates (including floored coordinates), actual game language and complete simplified/traditional Chinese messages. Private Remote maps are accessed only by owned helpers inserted into `remote.rs`, matching the existing Blink bridge.

The shared packet pipeline service runs at ordinary PacketEvent priority and cancels Attack, Interact and SpectatorAction targets matching active fake IDs; unrelated packets are untouched. Atomic running/settings state and a synchronized ID set avoid actor/registry access from packet threads. Host calls occur after actor borrows are released.

Root serial integration:

```powershell
python native/liquidbounce-horizon/integrate-logoff-spot.py
python native/liquidbounce-horizon/integrate-logoff-spot.py --apply
python native/liquidbounce-horizon/update-logoff-spot-locales.py --directory native/liquidbounce-interop/resources/localization
```

The integrator validates 20 edits across seven shared files before writes. Native `logoff-spot-patch.py` composes five files in memory; hook `client_logoff_spot(&mut self,input:&str,output:&mut String)` and helper `client_logoff_spot_clear(&mut self)` use the ordinary native API generator. Seven nested description/message entries are present in each copied Chinese display dictionary.

Focused evidence: `build/logoff-spot-smoke-04/report.json` has passed=true, compileExit=0, patchError=null and one passing usability test. Production config serializer/parser, engine runtime, packet listener, native player capture/lifecycle/Host functions and the actual locale resolver compile under declaration-only world/entity/actor/packet providers. The test exercises the real snapshot/health/UUID/equipment logic, duplicate/exclusion handling, unload priority, reappearance, settings, cleanup and messages. Smoke01/02 retain initial test-fixture scope/compiler failures; only the same failing test was retried in Smoke03. A static private-field review then moved Remote access into its owning module and the same single test passed again in Smoke04. No broader test matrix or official build was run. Fixture success does not establish actual game parity; root owns aggregate native/engine build and real-game verification.
