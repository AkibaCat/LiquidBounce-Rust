"""Actual Timer mode consumers exercised through the official game controller."""
import time

def verify(*, api, control, check, settings, put_settings, leaf, enable, module,
           key, wait_status, wait_state, wait_control, probe, fresh_probe,
           close_number, exercised, report, position):
    speed_sample = 0

    def speed(expected):
        nonlocal speed_sample
        speed_sample += 1
        baseline = api("/client/integration-status")
        status = wait_status(lambda s: fresh_probe(s, "clientTimerScale", baseline)
                             and close_number(probe(s, "clientTimerScale").get("output", {}).get("speed"), expected))
        sample = probe(status, "clientTimerScale")
        check(f"timerActualFrameSpeed{speed_sample}_{expected}",
              close_number(sample["output"]["delta"], sample["input"] * expected), sample)
        return status

    control("release")
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    control("tp", *(str(v) for v in position))
    tree = settings("Timer")
    mode = leaf(tree, "Mode")
    mode["active"] = "Classic"
    leaf(mode["choices"]["Classic"], "Speed")["value"] = 3.25
    put_settings("Timer", tree)
    enable("Timer", True)
    speed(3.25)
    check("timerClassicOriginalTag", module("Timer")["tag"] == "Classic", module("Timer"))

    mode["active"] = "Pulse"
    pulse = mode["choices"]["Pulse"]
    # Keep phases visible across actual frame sampling on an unoptimized game.
    # The owned original-coroutine oracle covers short phase boundaries.
    for name, value in (("NormalSpeed", 0.6), ("NormalSpeedTicks", 60),
                        ("BoostSpeed", 2.5), ("BoostSpeedTicks", 60), ("OnMove", False)):
        leaf(pulse, name)["value"] = value
    put_settings("Timer", tree)
    speed(0.6)
    speed(2.5)
    speed(0.6)
    check("timerPulseOriginalTag", module("Timer")["tag"] == "Pulse", module("Timer"))

    # A mode change must cancel the pulse coroutine immediately; enough real
    # ticks elapse for both old phases to have resumed if cancellation failed.
    mode["active"] = "Classic"
    leaf(mode["choices"]["Classic"], "Speed")["value"] = 1.75
    put_settings("Timer", tree)
    speed(1.75)
    control("wait", "15")
    speed(1.75)
    check("timerModeChangeCancelledPulse", module("Timer")["tag"] == "Classic")

    enable("Timer", False)
    speed(1.0)
    mode["active"] = "Pulse"
    leaf(pulse, "OnMove")["value"] = True
    put_settings("Timer", tree)
    enable("Timer", True)
    speed(1.0)
    baseline = api("/client/integration-status")
    status = wait_status(lambda s: fresh_probe(s, "clientTimerContext", baseline)
                         and probe(s, "clientTimerContext").get("input", {}).get("moving") is False)
    check("timerPulseActualStationaryGuard", probe(status, "clientTimerContext")["input"]["running"],
          probe(status, "clientTimerContext"))
    key("w", "down")
    try:
        speed(0.6)
    finally:
        key("w", "up")
    control("wait", "15")
    speed(1.0)
    enable("Timer", False)

    mode["active"] = "Boost"
    boost = mode["choices"]["Boost"]
    for name, value in (("BoostSpeed", 1.5), ("SlowSpeed", 0.5),
                        ("NormalizeDuringCombat", False), ("Balance", {"from": -100, "to": 200})):
        leaf(boost, name)["value"] = value
    put_settings("Timer", tree)
    enable("Timer", True)
    speed(0.5)
    check("timerBoostOriginalTag", module("Timer")["tag"] == "Boost", module("Timer"))
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival")
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "inventory")
    speed(1.0)
    baseline = api("/client/integration-status")
    status = wait_status(lambda s: fresh_probe(s, "clientTimerContext", baseline)
                         and probe(s, "clientTimerContext").get("input", {}).get("inventoryOrContainer") is True)
    check("timerBoostActualInventoryScreenGuard", True, probe(status, "clientTimerContext"))
    key("esc", "tap")
    wait_state(lambda s: s.get("screen") == "playing")
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    speed(0.5)
    key("w", "down")
    try:
        speed(1.5)
    finally:
        key("w", "up")
    speed(0.5)

    # Combat normalization consumes the real AttackEntityEvent, using a fresh
    # NoAI target so server AI cannot perturb this timing fixture.
    leaf(boost, "NormalizeDuringCombat")["value"] = True
    put_settings("Timer", tree)
    control("tp", *(str(v) for v in position))
    px, py, pz = position
    control("chat", f'/summon minecraft:zombie {px + 2} {py} {pz} {{NoAI:1b,Silent:1b,PersistenceRequired:1b,Tags:["lb_timer_test"]}}')
    entities = wait_control("entities", lambda s: any(
        e.get("type") == "minecraft:zombie" and abs(e["pos"][0] - (px + 2)) < 0.5
        for e in s.get("entities", [])), "8", "minecraft:zombie")
    target = next(e for e in entities["entities"] if abs(e["pos"][0] - (px + 2)) < 0.5)
    control("look-at", "entity", str(target["id"]))
    hit = control("attack", str(target["id"]))
    check("timerCombatActualAttack", hit.get("target") == target["id"] and hit.get("hit") is True, hit)
    baseline = api("/client/integration-status")
    status = wait_status(lambda s: fresh_probe(s, "clientTimerContext", baseline)
                         and probe(s, "clientTimerContext").get("input", {}).get("combat") is True)
    check("timerBoostActualCombatNormalization", True, probe(status, "clientTimerContext"))
    speed(1.0)
    control("chat", "/kill @e[tag=lb_timer_test]")
    enable("Timer", False)
    speed(1.0)
    baseline = api("/client/integration-status")
    status = wait_status(lambda s: fresh_probe(s, "clientTimerScale", baseline)
                         and probe(s, "clientTimerScale").get("output", {}).get("running") is False)
    check("timerDisableRestoresActualClock", True, probe(status, "clientTimerScale"))
    control("tp", *(str(v) for v in position))

    # Client world clocks extrapolate on client ticks, so they cannot prove
    # server pacing. Observe real SetTime packets from the independent server.
    mode["active"] = "Classic"
    samples = []
    for requested in (0.5, 3.0):
        leaf(mode["choices"]["Classic"], "Speed")["value"] = requested
        put_settings("Timer", tree)
        enable("Timer", True)
        speed(requested)
        baseline = api("/client/integration-status")
        previous = probe(baseline, "clientTimerContext")["output"]["serverTime"]["count"]
        status = wait_status(lambda s: probe(s, "clientTimerContext").get("output", {}).get("serverTime", {}).get("count", 0) > previous)
        server_before = probe(status, "clientTimerContext")["output"]["serverTime"]
        client_before = control("state")["ticks"]
        started = time.monotonic()
        status = wait_status(lambda s: probe(s, "clientTimerContext").get("output", {}).get("serverTime", {}).get("count", 0) >= server_before["count"] + 2)
        elapsed = time.monotonic() - started
        client_after = control("state")["ticks"]
        server_after = probe(status, "clientTimerContext")["output"]["serverTime"]
        server_seconds = (server_after["receivedAtMs"] - server_before["receivedAtMs"]) / 1000.0
        server_ticks = server_after["gameTime"] - server_before["gameTime"]
        sample = {"requestedSpeed": requested, "serverBefore": server_before, "serverAfter": server_after,
                  "serverTicksPerSecond": server_ticks / server_seconds if server_seconds > 0 else 0,
                  "clientTicksPerSecond": (client_after - client_before) / elapsed,
                  "clientTicks": [client_before, client_after], "elapsedSeconds": elapsed}
        check("timerIndependentServerPacing" + str(requested), server_before["session"] == server_after["session"]
              and server_ticks >= 20 and 12 <= sample["serverTicksPerSecond"] <= 30, sample)
        samples.append(sample)
    check("timerActualClientClockRateChanges", samples[1]["clientTicksPerSecond"] > samples[0]["clientTicksPerSecond"] * 3,
          samples)
    report["timerIndependentServerSamples"] = samples

    # A command teleports through the normal server and ClientLevel replacement,
    # which the old world name/session comparison cannot recognize.
    baseline = api("/client/integration-status")
    status = wait_status(lambda s: abs(probe(s, "clientTimerContext").get("output", {}).get("balance", 0)) > 2)
    check("timerHasBalanceBeforeRealDimensionChange", True, probe(status, "clientTimerContext"))
    control("chat", "/execute in minecraft:the_nether run tp @s 0 100 0")
    wait_state(lambda s: s.get("dimension") == "minecraft:the_nether")
    status = wait_status(lambda s: fresh_probe(s, "clientWorldChange", baseline)
                         and probe(s, "clientWorldChange").get("input") == "minecraft:the_nether")
    sample = probe(status, "clientWorldChange")
    check("timerRealDimensionReplacementResetsAccounting", sample["output"] == {"balance": 0.0, "burstTicks": 0, "heldTicks": 0}, sample)
    check("timerDimensionPreservesModuleActivation", module("Timer")["enabled"] is True)
    speed(3.0)
    baseline = status
    control("chat", "/execute in minecraft:overworld run tp @s " + " ".join(str(v) for v in position))
    wait_state(lambda s: s.get("dimension") == "minecraft:overworld")
    status = wait_status(lambda s: fresh_probe(s, "clientWorldChange", baseline)
                         and probe(s, "clientWorldChange").get("input") == "minecraft:overworld")
    check("timerActualReturnLevelReset", probe(status, "clientWorldChange")["output"] == {"balance": 0.0, "burstTicks": 0, "heldTicks": 0},
          probe(status, "clientWorldChange"))
    enable("Timer", False)
    speed(1.0)
    exercised("Timer")
    report["timerRuntimeCoverage"] = "real_Float_clock;Classic;Pulse_phases_mode_change_OnMove;Boost_stationary_moving_inventory_combat;disable_restore;real_dimension_world_reset;independent_server_packet_pacing"
