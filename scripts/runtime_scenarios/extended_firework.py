"""Actual equipped elytra, physical glide input and a server-tracked rocket."""
import math


def verify(*, api, control, check, settings, put_settings, leaf, enable, module,
           key, wait_status, wait_state, wait_control, probe, fresh_probe,
           exercised, report, position):
    control("release")
    for name in ("AirJump", "HighJump", "NoJumpDelay", "TerrainSpeed", "Timer",
                 "Strafe", "ElytraRecast", "Sprint"):
        enable(name, False)
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    control("chat", "/clear @s")
    control("chat", "/item replace entity @s armor.chest with minecraft:elytra")
    wait_control("inventory", lambda i: i["armor"][1].get("item") == "minecraft:elytra")
    control("chat", '/item replace entity @s hotbar.0 with minecraft:firework_rocket[minecraft:fireworks={flight_duration:127,explosions:[]}] 64')
    wait_control("inventory", lambda i: i["hotbar"][0].get("item") == "minecraft:firework_rocket")
    control("select", "1")
    control("tp", str(position[0]), str(position[1] + 120), str(position[2]))
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival" and not s["player"]["flying"])
    control("look", "0", "-20")
    key("space", "down")
    try:
        flying = wait_state(lambda s: s["player"]["fall_flying"])
    finally:
        key("space", "up")
    check("extendedFireworkActualEquippedGlide", True, flying["player"])
    enable("ExtendedFirework", False)
    baseline = api("/client/integration-status")
    control("mouse", "right", "down")
    try:
        used = wait_control("inventory", lambda i: i["hotbar"][0].get("count", 0) < 64)
    finally:
        control("mouse", "right", "up")
    check("extendedFireworkActualRocketUse", True, used["hotbar"][0])
    status = wait_status(lambda s: fresh_probe(s, "clientFireworkBoost", baseline)
                         and probe(s, "clientFireworkBoost").get("output", {}).get("apply") is False)
    check("extendedFireworkDisabledActualRocketConsumer", True, probe(status, "clientFireworkBoost"))
    tree = settings("ExtendedFirework")
    mode = leaf(tree, "Mode")
    mode["active"] = "Normal"
    put_settings("ExtendedFirework", tree)
    enable("ExtendedFirework", True)
    status = wait_status(lambda s: fresh_probe(s, "clientFireworkBoost", status)
                         and probe(s, "clientFireworkBoost").get("output", {}).get("apply") is True)
    sample = probe(status, "clientFireworkBoost")
    check("extendedFireworkNormalActualMultiplier", sample["output"]["multiplier"] == [0.1, 1.65, 0.5]
          and module("ExtendedFirework")["tag"] is None, sample)
    mode["active"] = "Custom"
    leaf(mode["choices"]["Custom"], "VelocityMultiplier")["value"] = {"x": 0.2, "y": 3.0, "z": 0.9}
    put_settings("ExtendedFirework", tree)
    status = wait_status(lambda s: fresh_probe(s, "clientFireworkBoost", status)
                         and probe(s, "clientFireworkBoost").get("output", {}).get("multiplier") == [0.2, 3.0, 0.9])
    custom = wait_state(lambda s: s["player"]["fall_flying"]
                        and math.sqrt(sum(v * v for v in s["player"]["velocity"])) > 2.5)
    check("extendedFireworkCustomChangesActualVelocity", module("ExtendedFirework")["tag"] is None,
          {"consumer": probe(status, "clientFireworkBoost"), "player": custom["player"]})
    enable("ExtendedFirework", False)
    status = wait_status(lambda s: fresh_probe(s, "clientFireworkBoost", status)
                         and probe(s, "clientFireworkBoost").get("output", {}).get("apply") is False)
    restored = wait_state(lambda s: s["player"]["fall_flying"]
                          and math.sqrt(sum(v * v for v in s["player"]["velocity"])) < 2.1)
    check("extendedFireworkDisableRestoresActualRocketVelocity", True,
          {"consumer": probe(status, "clientFireworkBoost"), "player": restored["player"]})
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    control("chat", "/kill @e[type=minecraft:firework_rocket]")
    control("chat", "/item replace entity @s armor.chest with minecraft:air")
    control("chat", "/clear @s")
    control("tp", *(str(v) for v in position))
    exercised("ExtendedFirework")
    report["extendedFireworkRuntimeCoverage"] = "real_chest_elytra_and_physical_glide;real_long_lived_server_rocket;Normal_and_Custom_live_multipliers;actual_velocity_changes_and_disable_restore;managed_rotation_has_owned_fixture"
