"""Real input/network sprint phases and translated ordinary mining warnings."""
import math


def verify(*, api, control, check, settings, put_settings, leaf, enable, key,
           wait_status, wait_state, wait_control, probe, fresh_probe,
           choose_language, exercised, report, position):
    control("release")
    for name in ("AutoBreak", "AutoTool", "HighJump", "Timer", "TerrainSpeed"):
        enable(name, False)
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    x, y, z = math.floor(position[0]), math.floor(position[1]) + 10, math.floor(position[2])
    control("chat", f"/fill {x-18} {y} {z-18} {x+18} {y+3} {z+18} minecraft:air")
    control("chat", f"/fill {x-18} {y-1} {z-18} {x+18} {y-1} {z+18} minecraft:stone")
    control("tp", str(x+0.5), str(y), str(z+0.5))
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival" and s["player"]["on_ground"])
    control("look", "0", "0")
    tree = settings("AntiHunger")
    no_sprint = leaf(tree, "NoSprint")
    leaf(no_sprint, "Enabled")["value"] = True
    leaf(no_sprint, "WhileSwimming")["value"] = False
    leaf(tree, "KeepFloating")["value"] = True
    put_settings("AntiHunger", tree)
    enable("Sprint", True)
    enable("AntiHunger", False)

    def sprint_phase(name, expected, running, baseline):
        status = wait_status(lambda s: fresh_probe(s, name, baseline)
                             and probe(s, name).get("input", {}).get("running") is running
                             and probe(s, name).get("input", {}).get("sprint") is True
                             and probe(s, name).get("output") is expected)
        return status

    key("w", "down")
    try:
        baseline = api("/client/integration-status")
        status = sprint_phase("clientAntiHungerNetwork", True, False, baseline)
        check("antiHungerDisabledActualSprint", True, probe(status, "clientAntiHungerNetwork"))
        enable("AntiHunger", True)
        status = sprint_phase("clientAntiHungerInput", False, True, status)
        check("antiHungerAfterSprintActualInputPriority", True, probe(status, "clientAntiHungerInput"))
        status = sprint_phase("clientAntiHungerNetwork", False, True, status)
        check("antiHungerActualNetworkSprintSuppression", True, probe(status, "clientAntiHungerNetwork"))
        leaf(no_sprint, "Enabled")["value"] = False
        put_settings("AntiHunger", tree)
        status = sprint_phase("clientAntiHungerNetwork", True, True, status)
        check("antiHungerActualNoSprintSettingGuard", True, probe(status, "clientAntiHungerNetwork"))
        enable("AntiHunger", False)
        status = sprint_phase("clientAntiHungerNetwork", True, False, status)
        check("antiHungerActualDisableRestoresSprint", True, probe(status, "clientAntiHungerNetwork"))
    finally:
        key("w", "up")
        enable("Sprint", False)

    leaf(no_sprint, "Enabled")["value"] = True
    put_settings("AntiHunger", tree)
    enable("AntiHunger", True)
    control("chat", "/clear @s")
    control("tp", str(x+0.5), str(y), str(z+0.5))
    initial_language = api("/client/localization")["gameLanguage"]
    warnings = []
    for code in ("zh_cn", "zh_tw", "en_us"):
        localization = choose_language(code)
        target = [str(x), str(y+1), str(z+2)]
        control("chat", "/setblock " + " ".join(target) + " minecraft:stone")
        wait_control("block", lambda b: b.get("block") == "minecraft:stone", *target)
        control("look-at", "block", *target, "north")
        control("wait", "14")
        baseline = api("/client/integration-status")
        expected = localization["translations"]["liquidbounce.module.antiHunger.messages.warnBreak"]
        control("mouse", "left", "down")
        try:
            status = wait_status(lambda s: fresh_probe(s, "clientAntiHungerNotification", baseline)
                                 and probe(s, "clientAntiHungerNotification").get("output", {}).get("message") == expected)
        finally:
            control("mouse", "left", "up")
        value = probe(status, "clientAntiHungerNotification")
        check("antiHungerActualMiningWarning" + code,
              value["input"] == "notification" and value["output"] == {
                  "title": "AntiHunger", "message": expected, "severity": "INFO"}, value)
        warnings.append({"gameLanguage": code, "notification": value["output"]})
    if initial_language != "en_us":
        choose_language(initial_language)
    enable("AntiHunger", False)
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    control("tp", *(str(v) for v in position))
    exercised("AntiHunger")
    report["antiHungerRuntimeWarnings"] = warnings
    report["antiHungerRuntimeCoverage"] = "actual_INPUT_NETWORK_priority_after_Sprint;NoSprint_live_guard;disable;ordinary_real_mining_packet_scope;original_simplified_traditional_English_notifications;movement_flags_and_synchronous_sprint_jump_have_owned_native_fixtures"
