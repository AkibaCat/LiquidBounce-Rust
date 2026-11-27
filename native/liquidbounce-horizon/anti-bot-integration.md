# AntiBot handoff — MC 26.3

Pinned LiquidBounce: `ae0eb396c07f22383789b28d5912d714c6ca0a86`.
Pinned Horizon: `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`.

Integration owner, apply serially after Notifier and ProjectilePuncher:

```powershell
python native/liquidbounce-horizon/integrate-anti-bot.py --apply
python scripts/update-anti-bot-localization.py
```

The integrator validates all 27 anchors across 9 shared files before writing and checks pre-write byte snapshots. This owner has not applied it, changed shared files/locales, written staging, or run Cargo. The main owner should regenerate/stage the combined native patch and include AntiBot in its single final compile/check cycle.

Implemented scope:

- Disabled-by-default Misc module, Custom / Matrix-2021-10-18 / Intave-2022-07-28 / Horizon-2021-10-18 modes, original mode aliases.
- All 23 primitive settings (19 Custom leaves plus Mode, LiteralNPC and NotInTabList Enabled/Grace), original defaults, ranges, units, masks and per-slot armor choices. Helmet excludes Elytra; the sparse enum mask preserves TurtleScute/Pumpkin/Skull.
- Custom's 11 condition predicates and all seven history groups. Only the first metadata packet counts. Radius/movement history is recorded even when its group is disabled; armor history is cleared whenever equipment becomes valid. Names use UTF-16 length and intersect every selected validator.
- Matrix's exact count==1 profile predicates, property/latency filters, one-tick armor wait, retained source iteration order, ItemStack object identity, and original entity identity across removal/reused IDs. Mode change cancels the continuation and retains mode histories; disable/world reset clears all four modes.
- Intave's lowest-ordinal packet action, producer-time 15 ms boundary, wrapping Java integer/long subtraction and source-disabled single-entry workaround.
- Horizon's raw ADD entry gameMode-null check.
- Accepted vanilla profile availability, property presence and full double scale observations. Online UUIDs and server-listed players remain distinct. PlayerInfo construction starts in DEFAULT_MODE; a raw packet entry can still have no game mode. Actual entity/tick/armor/profile state comes from the native client and resident LAN guests.
- Lazy shared TagEntityEvent-style ignore and `Notifier.bot_entities` populated before its tick. Attack history is observed before event cancellation. Relevant packets are observed before the receive-cancelled continuation, and the ground rule sees state before accepted vanilla handling.
- Shared CoroutineTicker owner/dispatch and disable/disconnect/world cleanup. No synthetic game facts or module-state-only placeholder services.
- 61 descriptions with Simplified Chinese, Traditional Chinese and English fallback, plus mode/condition/armor/validator labels. Updater mutates only when the integration owner runs it.

Verification performed once:

```powershell
python native/liquidbounce-horizon/test-anti-bot.py
```

Result: **11 passed, 0 failed**; 7 native files composed in memory; 27 shared edits/9 files validated in memory; 61 localized descriptions covered. The model/config harness uses actual owned Rust sources and extracts the current shared Tree/parser helpers, then links the existing serde_json build artifact with rustc. No Cargo or official build was invoked by this owner. Only two expected unused-helper warnings arose from the extracted shared parser fixture.

`anti-bot-java-facts.txt` records actual Minecraft 26.3 bytecode: Entity.setXRot has a finite guard and clamps modulo-360 pitch to [-90,90], matching Horizon; PlayerInfo initializes gameMode from DEFAULT_MODE. This corrected the initial absent-mode assumption before smoke.

Not claimed: Kotlin differential execution or official generated-native compilation. Those remain recorded as not run; the parent owns the latter.
