# MurderMystery implementation handoff

Pinned source: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, all eight Kotlin files in `features/module/modules/render/murdermystery`, plus the unchanged twenty fingerprints from `resources/liquidbounce/data/hypixel_mm_letters.json`. Target source remains Horizon `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, MC 26.3. Category Render, disabled by default.

The integration owner applies these after Macros/CommandManager, Teams, AutoDodge, Blink, ReportHelper, AutoWeapon and the common native hooks are present:

```powershell
python native/liquidbounce-horizon/integrate-murder-mystery.py --apply
python scripts/update-murder-mystery-localization.py
```

The transactional shared plan validates 22 changes across 8 files before writing. Native composition extends five files. This owner has not applied the shared plan, written staging, run Cargo, or performed the parent's official compilation.

Implemented source behavior:

- `SetTeamPrefix=true`, `Mode=Classic`, and empty Classic/Infection/Assassination settings groups serialize to real Interop/File trees. Mode changes reset all mode state, without clearing pending sound flags. Both Chinese game locales include the module, both settings, all three modes and their descriptions; status messages follow the current game language. The role prefix text remains literal `[BOW] ` / `[MURD] `, matching the source.
- Equipment processing observes main/offhand nonempty entries of genuine incoming SetEquipment packets. It resolves real client-player UUIDs, excludes nonplayers, tests sword before bow, and reports each role only once. The native reader posts a dedicated actor observation for equipment/sound/login/respawn even when another ordinary PacketEvent listener cancelled the packet, preserving actor packet order. These observations do not enter incoming replay again.
- Sword detection retains the 46 explicit known items, 4 known blocks, real `swords/axes/pickaxes/shovels/hoes` tags and the actual BoatItem facts. Wooden/golden shovel exclusions win over those tags. BowItem/MapItem correspond to the actual vanilla bow/filled_map registry entries; no item-suffix guesses are used.
- Classic records every newly discovered murderer and bow holder. Infection records all murderers, announces only the first, and discovers bow holders from actual using-item flags or arrows in either hand. Local role scans mainhand then offhand and chooses the first recognized item, testing bow/arrow before sword for each stack. Murderer role wins over bow role for remote UUIDs.
- `shouldAttack` feeds the shared lazy TargetInfo cache: local murderers target every nonmurderer; other local roles target murderers; Assassination targets only the contract target or assassin. Role colors use priority 35 (higher than Teams 20): detective `0xff0090ff`, murderer `0xffcb0909`. The tag pass queues an actual existing scoreboard team's prefix replacement; the next actor frame consumes it without inventing teams or restoring prefixes on disable.
- Assassination reads real loaded map colors from hotbar index 3. Only a changed MAP_ID is read; removing the map resets `lastMap`. Contract OCR uses row 105, height 7, MapColor.WOOD id 13 with brightness discarded, the upstream scanline spacing and exact fingerprints. Target matching uses lowercase trimmed player-info names and UTF16 Levenshtein distance, retaining the first minimum in Horizon's actual player-info map iteration.
- The basedrum sound packet resolves its real sound registry identifier. The fitted distance expression retains the f32 reciprocal, and player distance retains the f32 `distanceTo` result before comparing the double error. Every world player, including the local player, remains a candidate. Only a changed assassin produces chat.
- Render consumes the hurt/bow flags once, in hurt-then-bow order, and plays genuine relative UI villager-hurt/crossbow-shoot sounds. MC 26.3 javap confirmed `SimpleSoundInstance.forUI(sound,pitch)` uses UI source, no attenuation, relative position and **0.25** volume; this is preserved. Invisible armor stands with an actual mainhand bow render the interpolated `[-0.6,0,-0.6]..[0.6,2.5,0.6]` through-wall box with source fill/outline colors.
- Disable, world change, login and respawn reset roles and sounds. Source details intentionally retained: UUID mode resets do not reset `currentPlayerType`; Assassination reset does not reset `lastMap`. Mode changes do not clear pending sound flags.

AutoDodge's real `murder_mystery_disallows` dependency is updated on native mode/tick/reset events and on enable/disable/config changes. The exact condition is running MurderMystery + Infection mode + that mode's retained local detective-like role. Classic, Assassination, and a disabled module do not forbid dodge. The Timer provider lifecycle also reads the same live condition.

Validation on 2026-11-26: `python native/liquidbounce-horizon/test-murder-mystery.py` passed **one** combined focused model/OCR test and five-file in-memory native patch verification, with 22 shared changes/8 files validated. No additional full suite was run. Official generated-API/native compilation and full-client differential verification remain `not_run` by this owner and belong to the parent's final batch.

Practical boundaries: scoreboard prefix writes are consumed at the next world-render actor callback because TargetInfo is queried through read-only Host APIs. The remote packet observer runs on the actor in transport order; it does not acquire live game objects from the reader thread. These timing boundaries are explicit and should be checked in the final full-client differential pass. Rendering, packet handling, map storage, and profile identity are real native state, not fabricated probes.

Native patch ordering: `murder-mystery-patch.py` must run after `blink-patch.py`, which introduces `Event::ReplayPacket` and the incoming-with-replayer transport seam. Its owned integrator now inserts at that dependency.
