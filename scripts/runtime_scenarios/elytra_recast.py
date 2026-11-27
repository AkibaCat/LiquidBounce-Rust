"""Equipped elytra, physical held jump, actual server-driven glide endings."""
import math


def verify(*, api, control, check, enable, key, wait_state, wait_control,
           exercised, report, position):
    control("release")
    for name in ("AirJump", "BlockBounce", "BlockWalk", "HighJump", "NoJumpDelay",
                 "TerrainSpeed", "Timer", "Strafe", "ExtendedFirework", "Sprint"):
        enable(name, False)
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    control("chat", "/clear @s")
    control("chat", "/item replace entity @s armor.chest with minecraft:elytra")
    wait_control("inventory", lambda i: i["armor"][1].get("item") == "minecraft:elytra")
    control("chat", "/effect give @s minecraft:resistance infinite 4 true")
    x, y, z = math.floor(position[0]), math.floor(position[1]) + 16, math.floor(position[2])
    control("chat", f"/fill {x-8} {y} {z-8} {x+8} {y+14} {z+8} minecraft:air")
    control("chat", f"/fill {x-8} {y-1} {z-8} {x+8} {y-1} {z+8} minecraft:stone")
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival" and not s["player"]["flying"])
    samples = []

    def dive(running):
        control("release")
        enable("ElytraRecast", running)
        control("tp", str(x + 0.5), str(y), str(z + 0.5))
        wait_state(lambda s: s["player"]["on_ground"] and not s["player"]["fall_flying"])
        control("tp", str(x + 0.5), str(y + 8), str(z + 0.5))
        control("look", "0", "90")
        wait_state(lambda s: not s["player"]["on_ground"] and not s["player"]["fall_flying"])
        key("space", "down")
        try:
            started = wait_state(lambda s: s["player"]["fall_flying"])
            # A vertical dive across eight blocks reaches the solid platform.
            # Keep the same press held through landing and repeated ground jumps;
            # vanilla requires a new press to start a subsequent glide.
            control("wait", "60")
            after = control("state")
            check("elytraRecastActualDiveReachesPlatform" + str(running),
                  after["ticks"] - started["ticks"] >= 60
                  and y - 0.1 <= after["player"]["position"][1] < y + 2,
                  {"started": started["player"], "after": after["player"]})
            samples.append({"running": running, "started": started, "after": after})
            return after
        finally:
            key("space", "up")

    off = dive(False)
    check("elytraRecastDisabledHeldJumpDoesNotRestartGlide", not off["player"]["fall_flying"], off["player"])
    on = dive(True)
    check("elytraRecastActualLandingRestartsGlideWithSameHeldKey", on["player"]["fall_flying"], on["player"])
    restored = dive(False)
    check("elytraRecastDisableRestoresGlidePressRequirement", not restored["player"]["fall_flying"], restored["player"])
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    control("chat", "/effect clear @s minecraft:resistance")
    control("chat", "/item replace entity @s armor.chest with minecraft:air")
    control("tp", *(str(v) for v in position))
    exercised("ElytraRecast")
    report["elytraRecastRuntimeSamples"] = samples
    report["elytraRecastRuntimeCoverage"] = "real_equipped_elytra_and_physical_dive;server_driven_landing;held_jump_without_new_press;disabled_enabled_disabled_comparison;all_guards_and_tail_order_have_separate_Kotlin_oracle"
