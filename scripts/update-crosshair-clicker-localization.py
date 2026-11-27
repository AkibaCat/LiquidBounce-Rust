#!/usr/bin/env python3
"""TriggerBot / AutoClicker Simplified and Traditional Chinese vocabulary."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "native/liquidbounce-interop/resources/localization"
LABELS = {
    "OnItemUse": ("使用物品时", "使用物品時"),
    "Criticals": ("暴击选择", "暴擊選擇"),
    "DelayPostStopUse": ("停止使用后延迟", "停止使用後延遲"),
    "Clicker": ("点击计划", "點擊計畫"),
    "Human": ("人类节奏", "人類節奏"),
    "Constant": ("固定节奏", "固定節奏"),
    "CPS": ("每秒点击数", "每秒點擊數"),
    "MaxPerTick": ("每刻最多点击数", "每刻最多點擊數"),
    "ItemCooldown": ("物品冷却", "物品冷卻"),
    "MissCooldown": ("未命中冷却", "未命中冷卻"),
    "DelayOnBroken": ("破坏方块后延迟", "破壞方塊後延遲"),
    "Objective": ("目标类型", "目標類型"),
    "HoldingItemsForIgnore": ("忽略的手持物品", "忽略的手持物品"),
    "BlocksForIgnore": ("忽略的目标方块", "忽略的目標方塊"),
    "DelayStart": ("开始点击前延迟", "開始點擊前延遲"),
    "OnlyBlock": ("仅放置方块", "僅放置方塊"),
    "RequiresNoInput": ("无需按住使用键", "無需按住使用鍵"),
}
TEXTS = {
    "triggerBot": ("Attacks the crosshair target when it passes the combat target filters.", "准星对准符合战斗筛选的目标时自动攻击。", "準星對準符合戰鬥篩選的目標時自動攻擊。"),
    "autoClicker": ("Plans repeated attack and use clicks while their physical key is held.", "按住对应按键时，按计划连续攻击或使用物品。", "按住對應按鍵時，按計畫連續攻擊或使用物品。"),
    "autoClicker.attack.delayOnBroken": ("Pause attack clicks for 300 milliseconds after finishing block destruction.", "完成方块破坏后暂停攻击点击 300 毫秒。", "完成方塊破壞後暫停攻擊點擊 300 毫秒。"),
    "autoClicker.attack.objective": ("Choose enemy, any entity, block, or any crosshair objective.", "选择敌人、任意实体、方块或任意目标。", "選擇敵人、任意實體、方塊或任意目標。"),
    "autoClicker.use.holdingItemsForIgnore": ("Skip use clicks while either hand holds a listed item.", "任一手持有列表内物品时跳过使用点击。", "任一手持有清單內物品時跳過使用點擊。"),
    "autoClicker.use.blocksForIgnore": ("Skip use clicks when looking at a listed block.", "准星对准列表内方块时跳过使用点击。", "準星對準清單內方塊時跳過使用點擊。"),
    "autoClicker.use.delayStart": ("Wait two game ticks when starting a new held-use sequence.", "每次开始按住使用键时先等待 2 刻。", "每次開始按住使用鍵時先等待 2 刻。"),
    "autoClicker.use.onlyBlock": ("Only click use while either hand holds a block item.", "仅在任一手持有方块物品时点击使用。", "僅在任一手持有方塊物品時點擊使用。"),
    "autoClicker.use.requiresNoInput": ("Allow use clicks without holding the physical use key.", "无需按住实际使用键也可自动点击。", "無需按住實際使用鍵也可自動點擊。"),
}
for prefix in ("triggerBot", "autoClicker.attack"):
    TEXTS.update({
        f"{prefix}.onItemUse": ("Wait for, stop, or ignore current item use before attacking.", "攻击前等待当前物品使用结束、停止使用，或忽略使用状态。", "攻擊前等待目前物品使用結束、停止使用，或忽略使用狀態。"),
        f"{prefix}.weapon": ("Only attack while the main hand matches a selected weapon type.", "仅在主手物品符合所选武器类型时攻击。", "僅在主手物品符合所選武器類型時攻擊。"),
        f"{prefix}.criticals": ("Smart waits for a viable critical hit; Ignore allows any hit; Always requires a critical hit.", "智能模式等待适合的暴击；忽略模式允许普通攻击；始终模式只允许暴击。", "智慧模式等待適合的暴擊；忽略模式允許普通攻擊；始終模式只允許暴擊。"),
        f"{prefix}.delayPostStopUse": ("Game ticks to wait after item use ends or is stopped.", "物品使用结束或停止后等待的游戏刻数。", "物品使用結束或停止後等待的遊戲刻數。"),
    })
for prefix in ("triggerBot.clicker", "autoClicker.attack.clicker", "autoClicker.use.clicker"):
    TEXTS.update({
        f"{prefix}.technique": ("Choose human timing or a constant interval.", "选择人类点击节奏或固定间隔。", "選擇人類點擊節奏或固定間隔。"),
        f"{prefix}.cps": ("Requested clicks per second.", "每秒计划的点击次数。", "每秒計畫的點擊次數。"),
        f"{prefix}.maxPerTick": ("Maximum planned clicks in one game tick.", "每个游戏刻内计划的最大点击次数。", "每個遊戲刻內計畫的最大點擊次數。"),
    })
    if prefix != "autoClicker.use.clicker":
        TEXTS[f"{prefix}.missCooldown"] = ("Respect the game's cooldown after a missed attack.", "遵守游戏中攻击未命中后的冷却。", "遵守遊戲中攻擊未命中後的冷卻。")
        TEXTS[f"{prefix}.itemCooldown.minimum"] = ("Minimum attack cooldown progress before a click is accepted.", "接受攻击点击所需的最低攻击冷却进度。", "接受攻擊點擊所需的最低攻擊冷卻進度。")


def main():
    for locale, index in (("zh_cn", 1), ("zh_tw", 2)):
        path = FOLDER / f"display_{locale}.json"
        data = json.loads(path.read_text(encoding="utf8"))
        data["moduleNames"].update({"TriggerBot": ("准星自动攻击", "準星自動攻擊")[index - 1], "AutoClicker": ("自动连点", "自動連點")[index - 1]})
        data["labels"].update({key: pair[index - 1] for key, pair in LABELS.items()})
        for name, texts in TEXTS.items():
            key = f"liquidbounce.module.{name}.description"
            data["descriptions"][key] = texts[index]
            if locale == "zh_cn":
                data["englishDescriptions"][key] = texts[0]
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf8")
        print(locale, len(data["descriptions"]))


if __name__ == "__main__":
    main()
