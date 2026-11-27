#!/usr/bin/env python3
"""Merge SuperKnockback display help; --check validates without writing."""
from pathlib import Path
import argparse
import json

ROOT = Path(__file__).resolve().parents[2]
HELP = {
    "en_us": {
        "hurtTime": "Maximum target hurt time at which knockback modification may run.",
        "chance": "Chance of modifying knockback for each eligible attack.",
        "conditions": "Require the target to face you, your player to stand on ground, or your player to stay outside water and bubble columns.",
        "onlyOnMove": "Apply knockback modification only while your player has movement input.",
        "onlyOnMove.enabled": "Require movement input before modifying knockback.",
        "onlyOnMove.onlyForward": "Reject sideways input. Forward and backward input both remain eligible.",
        "mode.sprintTap.reSprint": "Ticks to wait after both actual sprinting and the last sent sprint state stop, before allowing sprint again.",
        "mode.wTap.untilMovementBlock": "Ticks to wait after an eligible attack before blocking directional movement.",
        "mode.wTap.untilAllowedMovement": "Ticks to wait after forward impulse stops before allowing directional movement again.",
    },
    "zh_cn": {
        "hurtTime": "目标受伤时间不超过此值时，才会调整击退。",
        "chance": "每次符合条件的攻击调整击退的概率。",
        "conditions": "可要求目标面向你、自身位于地面，或自身不在水与气泡柱内。",
        "onlyOnMove": "仅在自身有移动输入时调整击退。",
        "onlyOnMove.enabled": "调整击退前要求自身有移动输入。",
        "onlyOnMove.onlyForward": "有侧向输入时不触发；向前与向后输入均符合条件。",
        "mode.sprintTap.reSprint": "实际疾跑状态与上次发送的疾跑状态均停止后，再等待此刻数允许重新疾跑。",
        "mode.wTap.untilMovementBlock": "符合条件的攻击后，等待此刻数再阻止方向移动。",
        "mode.wTap.untilAllowedMovement": "向前移动输入停止后，再等待此刻数恢复方向移动。",
    },
    "zh_tw": {
        "hurtTime": "目標受傷時間不超過此值時，才會調整擊退。",
        "chance": "每次符合條件的攻擊調整擊退的機率。",
        "conditions": "可要求目標面向你、自身位於地面，或自身不在水與氣泡柱內。",
        "onlyOnMove": "僅在自身有移動輸入時調整擊退。",
        "onlyOnMove.enabled": "調整擊退前要求自身有移動輸入。",
        "onlyOnMove.onlyForward": "有側向輸入時不觸發；向前與向後輸入均符合條件。",
        "mode.sprintTap.reSprint": "實際疾跑狀態與上次傳送的疾跑狀態均停止後，再等待此刻數允許重新疾跑。",
        "mode.wTap.untilMovementBlock": "符合條件的攻擊後，等待此刻數再阻止方向移動。",
        "mode.wTap.untilAllowedMovement": "向前移動輸入停止後，再等待此刻數恢復方向移動。",
    },
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    pending = {}
    prefix = "liquidbounce.module.superKnockback"
    for language in ("zh_cn", "zh_tw"):
        target = ROOT / f"native/liquidbounce-interop/resources/localization/display_{language}.json"
        data = json.loads(target.read_text(encoding="utf8"))
        original = json.loads((ROOT / f"upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json").read_text(encoding="utf8"))
        key = f"{prefix}.description"
        data["descriptions"][key] = original[key]
        data["descriptions"].update({f"{prefix}.{k}.description": v for k, v in HELP[language].items()})
        if language == "zh_cn":
            english = json.loads((ROOT / "upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json").read_text(encoding="utf8"))
            data["englishDescriptions"][key] = english[key]
            data["englishDescriptions"].update({f"{prefix}.{k}.description": v for k, v in HELP["en_us"].items()})
        pending[target] = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    if args.check:
        print("Validated SuperKnockback simplified/traditional Chinese and English fallback help; no files written")
    else:
        for target, text in pending.items():
            target.write_text(text, encoding="utf8")
        print("Merged SuperKnockback simplified/traditional Chinese and English fallback help")


if __name__ == "__main__":
    main()
