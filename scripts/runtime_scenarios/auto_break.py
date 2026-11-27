"""Break a real server-owned block without issuing an attack command."""
import math


def verify(*, control, check, enable, wait_state, wait_control, exercised, report, position):
    control("release")
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival")
    x, y, z = (math.floor(v) for v in position)
    control("chat", f"/fill {x-2} {y-1} {z-2} {x+2} {y-1} {z+4} minecraft:stone")
    control("tp", str(x+0.5), str(y), str(z+0.5))
    control("select", "1")
    control("chat", "/item replace entity @s weapon.mainhand with minecraft:iron_pickaxe")
    wait_state(lambda s: (s.get("held") or {}).get("item") == "minecraft:iron_pickaxe")
    target = [x, y, z+2]
    coordinates = [str(v) for v in target]

    def stone():
        control("chat", "/setblock " + " ".join(coordinates) + " minecraft:stone")
        wait_control("block", lambda b: b.get("block") == "minecraft:stone", *coordinates)
        control("look-at", "block", *coordinates, "north")
        wait_state(lambda s: s.get("looking_at", {}).get("block", {}).get("block") == "minecraft:stone")

    enable("AutoBreak", False)
    stone()
    control("wait", "16")
    before = control("block", *coordinates)
    check("autoBreakDisabledRealBlockSurvives", before.get("block") == "minecraft:stone", before)
    enable("AutoBreak", True)
    try:
        removed = wait_control("block", lambda b: b.get("block") == "minecraft:air", *coordinates)
        check("autoBreakRealSurvivalBlockMinedWithoutAttackCommand", True, removed)
    finally:
        enable("AutoBreak", False)
    stone()
    control("wait", "16")
    restored = control("block", *coordinates)
    check("autoBreakDisableStopsActualMining", restored.get("block") == "minecraft:stone", restored)
    exercised("AutoBreak")
    report["autoBreakRuntimeCoverage"] = "real_survival_mining_from_late_KeyMapping_query_without_control_attack_or_break;disable_restore;queued_mapping_identity_has_owned_fixtures"
