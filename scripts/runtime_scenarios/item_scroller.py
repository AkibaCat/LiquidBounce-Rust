"""Move real server inventory stacks while holding semantic window input."""


def verify(*, control, check, settings, put_settings, leaf, enable, key,
           wait_state, wait_control, exercised, report):
    control("release")
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival")
    control("chat", "/clear @s")
    wait_control("inventory", lambda i: not any(s.get("item") for s in i["hotbar"]))
    control("chat", "/item replace entity @s inventory.0 with minecraft:diamond 7")

    def count(inventory, section, item):
        return sum(s.get("count", 0) for s in inventory[section] if s.get("item") == item)

    initial = wait_control("inventory", lambda i: count(i, "main", "minecraft:diamond") == 7)
    enable("ItemScroller", False)
    tree = settings("ItemScroller")
    leaf(tree, "Delay")["value"] = {"from": 0, "to": 0}
    put_settings("ItemScroller", tree)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "inventory")
    key("key.keyboard.left.shift", "down")
    try:
        # Press an empty slot first, then move while held. No click-slot or
        # shift-click is issued for the item being tested.
        held = control("pointer", "slot-8", "down", "left")
        control("pointer", "slot-9", "move")
        control("wait", "8")
        disabled = control("inventory")
        check("itemScrollerDisabledHeldPointerKeepsStack", count(disabled, "main", "minecraft:diamond") == 7
              and count(disabled, "hotbar", "minecraft:diamond") == 0, {"input":held,"inventory":disabled})
        enable("ItemScroller", True)
        moved = wait_control("inventory", lambda i: count(i, "main", "minecraft:diamond") == 0
                             and count(i, "hotbar", "minecraft:diamond") == 7)
        check("itemScrollerRealHeldPointerQuickMove", True, {"before":initial,"after":moved})
        enable("ItemScroller", False)
        control("chat", "/item replace entity @s inventory.0 with minecraft:apple 3")
        wait_control("inventory", lambda i: count(i, "main", "minecraft:apple") == 3)
        control("wait", "8")
        stopped = control("inventory")
        check("itemScrollerDisableStopsHeldPointerTransfers", count(stopped, "main", "minecraft:apple") == 3
              and count(stopped, "hotbar", "minecraft:apple") == 0, stopped)
    finally:
        control("pointer", "release")
        key("key.keyboard.left.shift", "up")
        enable("ItemScroller", False)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "playing")
    exercised("ItemScroller")
    report["itemScrollerRuntimeCoverage"] = "actual_server_inventory;registered_slot_pointer_hold_move;physical_Shift_and_left;disabled_baseline;QuickMove_transfer;disable_restore;no_target_click_slot;creative_and_random_timing_have_owned_fixtures"
