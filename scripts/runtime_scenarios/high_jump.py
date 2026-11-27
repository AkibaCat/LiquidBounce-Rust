"""Physical jumps and native Vulcan wait/glide consumers, using semantic control."""
import math


def verify(*, api, control, check, settings, put_settings, leaf, enable, module,
           key, wait_status, wait_state, probe, fresh_probe, close_number,
           exercised, report, position):
    control("release")
    for name in ("AirJump", "BlockBounce", "BlockWalk", "TerrainSpeed", "Timer", "Twerk"):
        enable(name, False)
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival")
    control("chat", "/effect give @s minecraft:resistance infinite 4 true")
    x, y, z = math.floor(position[0]), math.floor(position[1]) + 12, math.floor(position[2])
    control("chat", f"/fill {x-3} {y-1} {z-3} {x+3} {y-1} {z+3} minecraft:stone")
    control("chat", f"/fill {x-3} {y} {z-3} {x+3} {y+12} {z+3} minecraft:air")
    tree = settings("HighJump")
    mode = leaf(tree, "Mode")

    def settle():
        control("release")
        control("tp", str(x+0.5), str(y), str(z+0.5))
        return wait_state(lambda s: s["player"]["on_ground"] and abs(s["player"]["position"][1] - y) < 0.01)

    def jump(expected, label):
        before = api("/client/integration-status")
        key("space", "down")
        try:
            status = wait_status(lambda s: fresh_probe(s, "clientHighJump", before))
        finally:
            key("space", "up")
        sample = probe(status, "clientHighJump")
        check(label, close_number(sample["output"], expected), sample)
        return status

    enable("HighJump", False)
    settle()
    jump(0.42, "highJumpDisabledPhysicalJumpUsesVanillaPower")
    wait_state(lambda s: s["player"]["on_ground"])
    mode["active"] = "Vanilla"
    leaf(tree, "Motion")["value"] = 0.8
    put_settings("HighJump", tree)
    enable("HighJump", True)
    settle()
    jump(0.8, "highJumpVanillaReplacesActualJumpPower")
    raised = wait_state(lambda s: s["player"]["position"][1] > y + 1.5)
    check("highJumpVanillaActualPlayerRises", True, raised["player"])
    check("highJumpVanillaOriginalModeTag", module("HighJump")["tag"] == "Vanilla")
    wait_state(lambda s: s["player"]["on_ground"])

    mode["active"] = "Vulcan-2.7.5"
    leaf(mode["choices"]["Vulcan-2.7.5"], "Glide")["value"] = True
    leaf(tree, "Motion")["value"] = 6.0
    put_settings("HighJump", tree)
    settle()
    before = api("/client/integration-status")
    jump(6.0, "highJumpVulcanReplacesActualJumpPower")
    completed = wait_status(lambda s: fresh_probe(s, "clientHighJumpWait", before), seconds=45)
    check("highJumpVulcanActualDelayedVelocityClear", close_number(probe(completed, "clientHighJumpWait")["output"], 0),
          probe(completed, "clientHighJumpWait"))
    gliding = wait_status(lambda s: fresh_probe(s, "clientHighJumpGlide", before)
                          and close_number(probe(s, "clientHighJumpGlide")["output"], -0.155))
    sample = probe(gliding, "clientHighJumpGlide")
    check("highJumpVulcanActualEvenTickGlide", sample["input"]["tickCount"] % 2 == 0
          and sample["input"]["fallDistance"] > 0 and not sample["input"]["onGround"], sample)
    actual = control("state")
    check("highJumpVulcanActualAirborneVelocity", not actual["player"]["on_ground"]
          and -0.5 < actual["player"]["velocity"][1] < 0, actual["player"])
    check("highJumpVulcanOriginalModeTag", module("HighJump")["tag"] == "Vulcan-2.7.5")
    enable("HighJump", False)
    settle()
    jump(0.42, "highJumpDisableRestoresPhysicalJumpPower")
    control("chat", "/effect clear @s minecraft:resistance")
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    control("tp", *(str(v) for v in position))
    exercised("HighJump")
    report["highJumpRuntimeCoverage"] = "physical_space_jump;Vanilla_actual_rise;Vulcan_actual_jump_and_100_tick_wait_consumer;even_tick_glide_and_native_velocity;disable_restore;exact_wait_lifecycle_has_original_coroutine_oracle"
