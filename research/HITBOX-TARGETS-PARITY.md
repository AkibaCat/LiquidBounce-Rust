# Hitbox and global Targets parity

Baseline: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`, the project's pinned Horizon source, and the actual Minecraft 26.3 merged JAR.

## Original behavior

`ModuleHitbox.kt` declares Size (default 0.1, UI range 0..1), ApplyToDebugHitbox (true), and ApplyToComponent (true). The port retains the complete declaration and raw imported float values. `Entity.getPickRadius` replaces the original margin with Size only for `shouldBeAttacked` entities. Debug rendering expands eligible bounding boxes by Size. The AttackRange and ProjectileUtil mixins add Size to the existing component margin when the attacker is the local player; this branch deliberately has no target filter.

The patch follows Reach and preserves its client-window, LAN-acting and server-copy/replay boundaries. It changes the actual mob/armor-stand picks, hand component pick, and client component containment consumer. Server `within_attack_range` and the server-simulation `KineticWeapon.damageEntities` branch represented by `spear.rs::stab_targets` remain authoritative server behavior.

Actual `EntityHitboxDebugRenderer` bytecode determines the debug geometry: the white bounding box expands, the red eye plane starts at the expanded minimum Y plus the original eye height, and the blue view ray retains the original feet plus eye position. EnderDragon parts use parent eligibility. The resident dragon's synthetic lookup ID is distinct from real entity IDs.

## Target selection

`targeting.rs` follows the original global Combat/Visual sets and EntityTaggingManager, rather than inferring classes from vanilla MobCategory. Both sets default to Players, Hostile, Angerable, WaterCreature and Invisible. Combat excludes Self. Its eligibility requires Attackable, world-border containment, a living entity, enabled alive/invisible/sleeping gates, class membership, and a TARGET tag. The local entity and an entity directly carrying it are excluded. Friends require the Friends selection. Interesting and Ignored tags are excluded from Combat.

Visual selection permits Interesting and rejects Ignored, has no world-border/Attackable gate, and handles local-player/direct-passenger Self selection before ordinary living and tag checks. The Self gate uses raw perspective or active FreeCam/FreeLook. The tag cache is cleared first each client tick and then filled lazily. Only the registered FriendManager currently contributes default tags; unimplemented AntiBot/Teams/TargetLock/MurderMystery dependencies are not simulated. Friend profile names are case-sensitive; CancelAttack defaults to false.

The Java oracle loads all 161 actual entity types and records the relevant class/interface facts. The resulting table preserves first-match precedence: Player, WaterAnimal, AgeableMob/Bat/Allay, ArmorStand, Monster/Enemy, then NeutralMob. This means Dolphin is Passive, iron golem is Angerable, and piglin/zombified piglin are Hostile. `targeting_facts.rs` is generated from the recorded JAR evidence, with source hashes in the oracle report.

## Configuration and integration

The global tree uses Settings/Targets/Combat and Visual, with original Enemies alias support. Target list imports follow the source's incremental mutation semantics: clear the live set, add entries in order, and retain the successfully applied prefix when a later entry fails. Null/object inputs clear; malformed/missing properties report issues. Gson-compatible scalar strings include single-element nested arrays, and Java-compatible case folding accepts dotted/dotless I and long s.

The engine owns the global REST tree at `/api/v1/client/global`, actor ordering, real FriendManager tags, and `global.json` persistence. Unknown imported global properties are preserved on disk while the live API exposes actual implemented settings. Missing entity snapshots are skipped; they do not become an UNKNOWN entity with permissive defaults.

Owned files are `native/liquidbounce-behaviors/src/{targeting,targeting_facts,hitbox}.rs`, engine `config/{targets,hitbox}.rs`, `hitbox-host-methods.rs`, `hitbox-patch.py`, and their oracle/fixture generators. Apply `hitbox-patch.py` after Reach. Its three hooks are `client_entity_margin`, `client_debug_hitbox_margin`, and `client_hitbox_component_margin`; the Host helper is `client_targeting_entity`.

## Evidence

`build/hitbox-hook-tests/result.json` records a successful 21-test run and Clippy with `-D warnings`. The fixture compiles complete generated Host/events/dispatch, actual transformed production pick and containment bodies, and the full gizmos module. It checks 161 JAR class rows and five floating-point margin/geometry oracle rows. The fixture anchors complete debug-render transformations; the parent build separately checks the full game module and actual GPU output.

The parent reports 39 engine tests, 28 config tests, and behavior/config Clippy with `-D warnings` passing after global integration. This document records source and fixture evidence; full packaged-game and in-game checks belong to the parent integration report.
