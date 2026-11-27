"""Actual inventory render consumers, driven by the semantic controller."""
import struct


def f32(bits):
    return struct.unpack("<f", struct.pack("<I", bits & 0xffffffff))[0]


def verify(*, api, control, check, settings, put_settings, leaf, enable, key,
           wait_status, wait_state, wait_control, probe, fresh_probe,
           close_number, exercised, report, position, run):
    control("release")
    control("tp", *(str(v) for v in position))
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival")
    control("chat", "/clear @s")
    tree = settings("BetterInventory")
    highlight = leaf(tree, "HighlightClicked")
    leaf(highlight, "Enabled")["value"] = True
    leaf(highlight, "Mode")["active"] = "Border"
    cooldown = leaf(tree, "TextCooldownProgress")
    leaf(cooldown, "Enabled")["value"] = True
    container = leaf(tree, "ContainerItemView")
    leaf(container, "Enabled")["value"] = True
    leaf(container, "SkipEmptyStack")["value"] = False
    leaf(container, "Scale")["value"] = 1.0
    leaf(container, "RelativeToMouse")["value"] = True
    leaf(container, "RenderOffsetX")["value"] = 0.0
    leaf(container, "RenderOffsetY")["value"] = -60.0
    leaf(tree, "MoveCursor")["value"] = True
    put_settings("BetterInventory", tree)
    enable("BetterInventory", True)

    # Vomit executes a real InventoryPlan. Manual clicks intentionally do not
    # change InventoryManager.lastClickedSlot in the original client.
    vomit = settings("Vomit")
    constraints = leaf(vomit, "Constraints")
    for name in ("StartDelay", "ClickDelay", "CloseDelay", "MissChance"):
        leaf(constraints, name)["value"] = {"from": 0, "to": 0}
    # Preserve the original imported delay (imports may exceed the slider), so
    # the real manager's lastClickedSlot survives long enough for frame reads.
    leaf(constraints, "ClickDelay")["value"] = {"from": 10000, "to": 10000}
    leaf(constraints, "SlotMultiplier")["value"] = 0.0
    leaf(constraints, "Requires")["value"] = ["InventoryOpen"]
    put_settings("Vomit", vomit)
    control("chat", "/item replace entity @s hotbar.0 with minecraft:cobblestone 7")
    wait_control("inventory", lambda i: i["hotbar"][0].get("item") == "minecraft:cobblestone")
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "inventory")
    # The official semantic middle-click moves through real window events.
    # Clone is unavailable in survival, leaving the carried stack empty.
    hovered = control("click", "slot-8", "middle")
    check("betterInventorySemanticPointer", hovered.get("clicked") == "slot-8", hovered)
    baseline = api("/client/integration-status")
    enable("Vomit", True)
    wait_control("inventory", lambda i: not i["hotbar"][0].get("item"))
    status = wait_status(lambda s: fresh_probe(s, "clientBetterInventory:slot:active", baseline)
                         and probe(s, "clientBetterInventory:slot:active").get("output", {}).get("kind") == "border")
    sample = probe(status, "clientBetterInventory:slot:active")
    check("betterInventoryManagedClickBorder", sample["input"]["index"] == 36
          and len(sample["output"]["rects"]) == 4, sample)
    moved = probe(status, "clientInventoryMoveCursor")
    check("betterInventoryActualVisibleCursor", fresh_probe(status, "clientInventoryMoveCursor", baseline)
          and isinstance(moved.get("output"), list) and len(moved["output"]) == 2
          and all(close_number(a, b) for a, b in zip(moved["input"], moved["output"])), moved)
    control("screenshot", str(run / "better-inventory-border.png"))

    leaf(highlight, "Mode")["active"] = "Texture"
    put_settings("BetterInventory", tree)
    baseline = status
    status = wait_status(lambda s: fresh_probe(s, "clientBetterInventory:slot:active", baseline)
                         and probe(s, "clientBetterInventory:slot:active").get("output", {}).get("kind") == "texture")
    check("betterInventoryManagedClickTexture", probe(status, "clientBetterInventory:slot:active")["input"]["index"] == 36,
          probe(status, "clientBetterInventory:slot:active"))
    control("screenshot", str(run / "better-inventory-texture.png"))
    enable("Vomit", False)
    enable("BetterInventory", False)
    baseline = status
    status = wait_status(lambda s: fresh_probe(s, "clientBetterInventory:slot:36", baseline)
                         and probe(s, "clientBetterInventory:slot:36").get("output") is None)
    check("betterInventoryDisableRemovesActualHighlight", True, probe(status, "clientBetterInventory:slot:36"))
    enable("BetterInventory", True)

    # Keep the same manager and screen, but turn off its visible cursor gate.
    leaf(tree, "MoveCursor")["value"] = False
    put_settings("BetterInventory", tree)
    control("click", "slot-8", "middle")
    control("chat", "/item replace entity @s hotbar.1 with minecraft:dirt 3")
    wait_control("inventory", lambda i: i["hotbar"][1].get("item") == "minecraft:dirt")
    baseline = api("/client/integration-status")
    enable("Vomit", True)
    wait_control("inventory", lambda i: not i["hotbar"][1].get("item"))
    status = wait_status(lambda s: fresh_probe(s, "clientBetterInventory:slot:active", baseline)
                         and probe(s, "clientBetterInventory:slot:active").get("input", {}).get("index") == 37)
    check("betterInventoryCursorOptionGuardsRealPlan", not fresh_probe(status, "clientInventoryMoveCursor", baseline),
          {"before": probe(baseline, "clientInventoryMoveCursor"), "after": probe(status, "clientInventoryMoveCursor")})
    enable("Vomit", False)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "playing")

    # Real component decoding preserves sparse holes through the last slot.
    control("chat", '/item replace entity @s hotbar.2 with minecraft:shulker_box[minecraft:container=[{slot:0,item:{id:"minecraft:diamond",count:3}},{slot:10,item:{id:"minecraft:apple",count:2}}]]')
    wait_control("inventory", lambda i: i["hotbar"][2].get("item") == "minecraft:shulker_box")
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "inventory")
    baseline = api("/client/integration-status")
    control("click", "slot-2", "middle")
    status = wait_status(lambda s: fresh_probe(s, "clientBetterInventory:container:active", baseline))
    sample = probe(status, "clientBetterInventory:container:active")
    check("betterInventoryActualSparseContainer", sample["input"]["stacks"] == [True] + [False] * 9 + [True]
          and sample["output"]["indices"] == list(range(11))
          and sample["output"]["dimensions"] == [162, 36], sample)
    control("screenshot", str(run / "better-inventory-container.png"))
    leaf(container, "SkipEmptyStack")["value"] = True
    leaf(container, "Scale")["value"] = 1.25
    put_settings("BetterInventory", tree)
    baseline = status
    status = wait_status(lambda s: fresh_probe(s, "clientBetterInventory:container:active", baseline)
                         and probe(s, "clientBetterInventory:container:active").get("output", {}).get("indices") == [0, 10])
    sample = probe(status, "clientBetterInventory:container:active")
    check("betterInventoryActualContainerFilterAndScale", sample["output"]["dimensions"] == [36, 18]
          and close_number(f32(sample["output"]["scale"]), 1.25), sample)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "playing")

    control("chat", '/item replace entity @s hotbar.3 with minecraft:snowball[minecraft:use_cooldown={seconds:30.0,cooldown_group:"liquidbounce:integration"}] 16')
    wait_control("inventory", lambda i: i["hotbar"][3].get("item") == "minecraft:snowball")
    control("select", "4")
    control("look", "0", "-50")
    control("mouse", "right", "down")
    try:
        used = wait_control("inventory", lambda i: i["hotbar"][3].get("count", 0) < 16)
        check("betterInventoryActualCooldownItemUse", True, used["hotbar"][3])
    finally:
        control("mouse", "right", "up")
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "inventory")
    for mode in ("Percentage", "DurationTicks", "DurationSeconds"):
        leaf(cooldown, "Mode")["value"] = mode
        put_settings("BetterInventory", tree)
        baseline = api("/client/integration-status")

        def expected(s):
            if not fresh_probe(s, "clientBetterInventory:cooldown:active", baseline):
                return False
            value = probe(s, "clientBetterInventory:cooldown:active")
            source = value["input"]
            if source.get("group") != "liquidbounce:integration":
                return False
            if mode == "Percentage":
                progress = f32(source["progress"])
                text = str(int(f32(struct.unpack("<I", struct.pack("<f", progress * 100.0))[0]))) + "%"
            elif mode == "DurationTicks":
                text = str(source["end"] - source["tick"])
            else:
                ticks = source["end"] - source["tick"]
                if ticks <= 20:
                    return False
                text = str(int(f32(struct.unpack("<I", struct.pack("<f", ticks * 0.05))[0]))) + "s"
            return value["output"].get("text") == text

        status = wait_status(expected)
        check("betterInventoryActualCooldown" + mode, True, probe(status, "clientBetterInventory:cooldown:active"))
    control("screenshot", str(run / "better-inventory-cooldown.png"))
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "playing")
    enable("BetterInventory", False)
    control("chat", "/clear @s")
    control("chat", "/kill @e[type=minecraft:item,distance=..12]")
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    control("tp", *(str(v) for v in position))
    exercised("BetterInventory")
    report["betterInventoryRuntimeCoverage"] = "actual_manager_click_Border_Texture;visible_cursor_gate;component_sparse_preview_filter_scale;three_real_cooldown_modes;visual_review_pending"
    report["betterInventoryScreenshotPendingVisualReview"] = [
        str(run / ("better-inventory-" + name + ".png"))
        for name in ("border", "texture", "container", "cooldown")]
