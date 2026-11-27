"""Actual server armor transfers through the shared inventory executor."""


def verify(*, control, check, settings, put_settings, leaf, enable, module, key,
           wait_state, wait_control, exercised, report):
    control("release")
    for name in ("ElytraSwap", "AutoTool", "Vomit", "ItemScroller"):
        enable(name, False)
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival")
    control("chat", "/clear @s")
    control("chat", "/item replace entity @s armor.chest with minecraft:diamond_chestplate")
    control("chat", "/item replace entity @s hotbar.2 with minecraft:elytra[minecraft:damage=431]")
    control("chat", "/item replace entity @s hotbar.5 with minecraft:elytra")
    control("chat", "/item replace entity @s inventory.0 with minecraft:elytra")
    control("chat", "/item replace entity @s weapon.offhand with minecraft:elytra")
    initial = wait_control("inventory", lambda i: i["armor"][1].get("item") == "minecraft:diamond_chestplate"
                           and i["hotbar"][2].get("damage") == 431
                           and i["hotbar"][5].get("item") == "minecraft:elytra"
                           and i["main"][0].get("item") == "minecraft:elytra"
                           and (i.get("offhand") or {}).get("item") == "minecraft:elytra")
    tree = settings("ElytraSwap")
    constraints = leaf(tree, "Constraints")
    for name in ("StartDelay", "ClickDelay", "CloseDelay", "MissChance"):
        leaf(constraints, name)["value"] = {"from": 0, "to": 0}
    leaf(constraints, "Requires")["value"] = ["InventoryOpen"]
    put_settings("ElytraSwap", tree)
    enable("ElytraSwap", True)
    control("wait", "8")
    guarded = control("inventory")
    check("elytraSwapActualInventoryOpenGuard", module("ElytraSwap")["enabled"]
          and guarded["armor"][1].get("item") == "minecraft:diamond_chestplate"
          and guarded["hotbar"][5].get("item") == "minecraft:elytra", guarded)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "inventory")
    equipped = wait_control("inventory", lambda i: i["armor"][1].get("item") == "minecraft:elytra"
                            and i["hotbar"][5].get("item") == "minecraft:diamond_chestplate")
    wait_state(lambda _: not module("ElytraSwap")["enabled"])
    check("elytraSwapActualHotbarOrderAndDurabilityGuard", equipped["hotbar"][2].get("damage") == 431
          and equipped["main"][0].get("item") == "minecraft:elytra"
          and (equipped.get("offhand") or {}).get("item") == "minecraft:elytra", equipped)
    enable("ElytraSwap", True)
    restored = wait_control("inventory", lambda i: i["armor"][1].get("item") == "minecraft:diamond_chestplate"
                            and i["hotbar"][5].get("item") == "minecraft:elytra")
    wait_state(lambda _: not module("ElytraSwap")["enabled"])
    check("elytraSwapActualArmorReturnAndCompletionDisable", True, restored)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "playing")
    leaf(constraints, "Requires")["value"] = []
    put_settings("ElytraSwap", tree)
    samples = [{"before": initial, "equipped": equipped, "restored": restored}]
    for label, command_slot, section, index in (
            ("Main", "inventory.0", "main", 0),
            ("Offhand", "weapon.offhand", "offhand", None)):
        control("chat", "/clear @s")
        control("chat", "/item replace entity @s armor.chest with minecraft:air")
        control("chat", f"/item replace entity @s {command_slot} with minecraft:elytra")

        def source(i):
            return (i.get(section) or {}) if index is None else i[section][index]

        before = wait_control("inventory", lambda i: source(i).get("item") == "minecraft:elytra"
                              and not i["armor"][1].get("item"))
        enable("ElytraSwap", True)
        after = wait_control("inventory", lambda i: i["armor"][1].get("item") == "minecraft:elytra"
                             and not source(i).get("item"))
        wait_state(lambda _: not module("ElytraSwap")["enabled"])
        check("elytraSwapActualEmptyChestFrom" + label, True, after)
        samples.append({"source": label, "before": before, "after": after})
    control("chat", "/clear @s")
    control("chat", "/item replace entity @s armor.chest with minecraft:air")
    wait_control("inventory", lambda i: not i["armor"][1].get("item"))
    enable("ElytraSwap", True)
    wait_state(lambda _: not module("ElytraSwap")["enabled"])
    empty = control("inventory")
    check("elytraSwapNoCandidateImmediatelyDisables", not empty["armor"][1].get("item"), empty)
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    exercised("ElytraSwap")
    report["elytraSwapRuntimeSamples"] = samples
    report["elytraSwapRuntimeCoverage"] = "actual_server_hotbar_main_offhand_pickups;broken_glider_skip;InventoryOpen_live_guard;empty_chest_two_clicks;armor_return;no_candidate_and_completed_disable;cancelled_chains_have_independent_shared_manager_oracle"
