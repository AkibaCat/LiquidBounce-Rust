#!/usr/bin/env python3
"""Original ItemScroller settings with locale-specific display help."""
import json
from pathlib import Path

FOLDER = Path(__file__).resolve().parents[1] / "native/liquidbounce-interop/resources/localization"
TEXTS = {
    "clickMode": (
        "QuickMove transfers the hovered item while physical Shift and the left mouse button are held.",
        "按住物理 Shift 键和鼠标左键时，QuickMove 快速转移鼠标所指的物品。",
        "按住實體 Shift 鍵和滑鼠左鍵時，QuickMove 快速轉移滑鼠所指的物品。",
    ),
    "delay": (
        "Random delay sampled for each eligible transfer check. Default 2–3 ticks; the original slider range is 0–20.",
        "每次满足条件的转移检查都重新抽取随机延迟。默认 2–3 刻；原始滑块范围为 0–20。",
        "每次滿足條件的轉移檢查都重新抽取隨機延遲。預設 2–3 刻；原始滑桿範圍為 0–20。",
    ),
}

for locale, index in (("zh_cn", 1), ("zh_tw", 2)):
    path = FOLDER / f"display_{locale}.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    for name, texts in TEXTS.items():
        key = f"liquidbounce.module.itemScroller.{name}.description"
        data["descriptions"][key] = texts[index]
        if locale == "zh_cn":
            data["englishDescriptions"][key] = texts[0]
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(locale, len(data["descriptions"]))
