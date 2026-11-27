"""Real terrain and physical-key consumers, through the official controller."""
import math


def verify(*, api, control, check, settings, put_settings, leaf, enable, module,
           key, wait_status, wait_state, probe, fresh_probe, exercised, report,
           position):
    control("release")
    for name in ("AirJump", "BlockBounce", "BlockWalk", "Sprint", "Strafe", "Twerk", "Timer"):
        enable(name, False)
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    x, y, z = math.floor(position[0]), math.floor(position[1]) + 24, math.floor(position[2])
    tree = settings("TerrainSpeed")
    climb, ice, water = (leaf(tree, n) for n in ("FastClimb", "IceSpeed", "WaterSpeed"))
    for group in (climb, ice, water):
        leaf(group, "Enabled")["value"] = False
    put_settings("TerrainSpeed", tree)
    enable("TerrainSpeed", True)
    baseline = api("/client/integration-status")
    status = wait_status(lambda s: fresh_probe(s, "clientTerrainSpeed", baseline)
                         and probe(s, "clientTerrainSpeed").get("output", {}).get("running") is True)
    check("terrainSpeedActualRegistryConsumer", "FastClimb" in module("TerrainSpeed")["aliases"],
          probe(status, "clientTerrainSpeed"))

    # Each sample uses actual tick counts and positions with keys held across
    # the complete interval, including asynchronous command round trips.
    def travel(start, held, ticks=10):
        control("release")
        control("tp", *(str(v) for v in start))
        control("look", "0", "0")
        control("wait", "3")
        for name in held:
            key(name, "down")
        try:
            control("wait", "3")
            before = control("state")
            control("wait", str(ticks))
            after = control("state")
        finally:
            for name in reversed(held):
                key(name, "up")
        elapsed = after["ticks"] - before["ticks"]
        check("terrainSampleAdvancesTicks" + str(len(samples)), elapsed > 0, elapsed)
        delta = [after["player"]["position"][i] - before["player"]["position"][i] for i in range(3)]
        sample = {"start": start, "held": held, "before": before["player"], "after": after["player"],
                  "ticks": elapsed, "delta": delta,
                  "horizontalPerTick": math.hypot(delta[0], delta[2]) / elapsed,
                  "verticalPerTick": delta[1] / elapsed}
        samples.append(sample)
        return sample

    samples = []
    control("chat", f"/fill {x-2} {y-1} {z-2} {x+2} {y+32} {z+2} minecraft:air")
    control("chat", f"/fill {x-2} {y-1} {z-2} {x+2} {y-1} {z+2} minecraft:stone")
    control("chat", f"/fill {x} {y} {z+1} {x} {y+30} {z+1} minecraft:stone")
    control("chat", f"/fill {x} {y} {z} {x} {y+30} {z} minecraft:ladder[facing=north]")
    leaf(climb, "Enabled")["value"] = True
    mode = leaf(climb, "Mode")
    mode["active"] = "Motion"
    climb_samples = []
    for motion in (0.1, 0.5):
        leaf(mode["choices"]["Motion"], "Motion")["value"] = motion
        put_settings("TerrainSpeed", tree)
        climb_samples.append(travel([x+0.5, y, z+0.45], ["w"], 12))
    check("terrainClimbMotionActualSpeed", climb_samples[0]["verticalPerTick"] > 0.04
          and climb_samples[1]["verticalPerTick"] > climb_samples[0]["verticalPerTick"] * 2,
          climb_samples)
    mode["active"] = "Clip"
    put_settings("TerrainSpeed", tree)
    control("tp", str(x+0.5), str(y), str(z+0.45))
    key("w", "down")
    try:
        clipped = wait_state(lambda s: s["player"]["position"][1] > y + 12)
    finally:
        key("w", "up")
    check("terrainClimbClipActualPosition", True, clipped["player"])
    leaf(climb, "Enabled")["value"] = False

    # Ice coast is measured after releasing input; the high-friction sample
    # should stop while the low-friction sample retains horizontal motion.
    control("chat", f"/fill {x-15} {y} {z-15} {x+15} {y+32} {z+15} minecraft:air")
    control("chat", f"/fill {x-15} {y-1} {z-15} {x+15} {y-1} {z+15} minecraft:ice")
    leaf(ice, "Enabled")["value"] = True
    leaf(leaf(ice, "Motion"), "Enabled")["value"] = False
    coast = []
    for friction in (0.3, 1.0):
        leaf(ice, "Slipperiness")["value"] = friction
        put_settings("TerrainSpeed", tree)
        travel([x+0.5, y, z-8.5], ["w"], 16)
        before = control("state")
        control("wait", "10")
        after = control("state")
        coast.append({"slipperiness": friction, "distance": math.dist(
            [before["player"]["position"][i] for i in (0, 2)],
            [after["player"]["position"][i] for i in (0, 2)]),
            "before": before["player"], "after": after["player"]})
    check("terrainIceActualCoastChanges", coast[1]["distance"] > coast[0]["distance"] + 0.1, coast)
    leaf(ice, "Enabled")["value"] = False

    control("chat", f"/fill {x-19} {y-1} {z-19} {x+19} {y+12} {z+19} minecraft:stone")
    control("chat", f"/fill {x-18} {y} {z-18} {x+18} {y+11} {z+18} minecraft:water")
    leaf(water, "Enabled")["value"] = True
    leaf(water, "AutoSwim")["value"] = False
    base, sprint = leaf(water, "BaseSpeed"), leaf(water, "SprintSpeed")
    for name, value in (("Horizontal", 0.4), ("Vertical", 0.3)):
        leaf(base, name)["value"] = value
    leaf(sprint, "Enabled")["value"] = True
    leaf(sprint, "Horizontal")["value"] = 0.8
    leaf(sprint, "Vertical")["value"] = 0.6
    put_settings("TerrainSpeed", tree)
    base_sample = travel([x+0.5, y+4, z-10.5], ["w"], 8)
    sprint_sample = travel([x+0.5, y+4, z-10.5], ["w", "key.keyboard.left.control"], 8)
    check("terrainWaterActualBaseAndPhysicalSprint", base_sample["horizontalPerTick"] > 0.2
          and sprint_sample["horizontalPerTick"] > base_sample["horizontalPerTick"] * 1.5,
          [base_sample, sprint_sample])
    up = travel([x+0.5, y+3, z+0.5], ["space"], 4)
    down = travel([x+0.5, y+8, z+0.5], ["key.keyboard.left.shift"], 4)
    check("terrainWaterActualPhysicalVerticalKeys", up["verticalPerTick"] > 0.1
          and down["verticalPerTick"] < -0.1, [up, down])
    enable("TerrainSpeed", False)
    disabled = travel([x+0.5, y+4, z-10.5], ["w"], 8)
    check("terrainWaterDisableRestoresMovement", disabled["horizontalPerTick"] < base_sample["horizontalPerTick"] * 0.7,
          {"enabled": base_sample, "disabled": disabled})
    control("tp", *(str(v) for v in position))
    exercised("TerrainSpeed")
    report["terrainSpeedRuntimeSamples"] = samples
    report["terrainSpeedRuntimeCoverage"] = "real_ladder_Motion_and_Clip;ice_coast;water_base_physical_sprint_vertical;disable;global_nonplayer_friction_and_passengers_have_owned_fixtures"
