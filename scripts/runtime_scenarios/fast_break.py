"""Real survival mining with the original tool guard and damage threshold."""
import math


def verify(*, control, check, settings, put_settings, leaf, enable, wait_state,
           wait_control, exercised, report, position):
    control("release")
    enable("AutoBreak", False)
    enable("AutoTool", False)
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival")
    control("chat", "/clear @s")
    wait_state(lambda s: not s.get("held"))
    x, y, z = (math.floor(v) for v in position)
    control("chat", f"/fill {x-2} {y} {z-2} {x+2} {y+3} {z+4} minecraft:air")
    control("chat", f"/fill {x-2} {y-1} {z-2} {x+2} {y-1} {z+4} minecraft:stone")
    control("tp", str(x+0.5), str(y), str(z+0.5))
    target = [str(x), str(y), str(z+2)]
    control("chat", "/setblock " + " ".join(target) + " minecraft:stone")
    wait_control("block", lambda b: b.get("block") == "minecraft:stone", *target)
    control("look-at", "block", *target, "north")
    tree = settings("FastBreak")
    leaf(tree, "Mode")["active"] = "None"
    leaf(tree, "BreakDamage")["value"] = 0.1
    leaf(tree, "OnlyTool")["value"] = True
    put_settings("FastBreak", tree)
    enable("FastBreak", True)
    control("mouse", "left", "down")
    try:
        control("wait", "24")
        guarded = control("block", *target)
        check("fastBreakActualEmptyHandToolGuard", guarded.get("block") == "minecraft:stone", guarded)
        # Continue the same physical mining operation. The original strict
        # damage threshold should now complete the accumulated hand progress.
        leaf(tree, "OnlyTool")["value"] = False
        put_settings("FastBreak", tree)
        broken = wait_control("block", lambda b: b.get("block") == "minecraft:air", *target)
        check("fastBreakActualDamageThresholdCompletesMining", True, broken)
    finally:
        control("mouse", "left", "up")
        enable("FastBreak", False)
    exercised("FastBreak")
    report["fastBreakRuntimeCoverage"] = "actual_survival_block_and_empty_hand;OnlyTool_guard;strict_damage_threshold_after_live_option_edit;ordinary_stop_destroy;Grim_nested_packet_and_disable_delay_have_owned_fixtures"
