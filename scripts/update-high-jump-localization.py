#!/usr/bin/env python3
"""Add owned HighJump help while retaining original setting identifiers."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "native/liquidbounce-interop/resources/localization"
TEXTS = {
    "highJump.mode": (
        "Vanilla replaces jump motion. Vulcan-2.7.5 also clears vertical velocity after 100 game ticks and can enable gliding.",
        "Vanilla 替换跳跃速度。Vulcan-2.7.5 还会在 100 个游戏刻后清零垂直速度，并可启用滑翔。",
        "Vanilla 替換跳躍速度。Vulcan-2.7.5 還會在 100 個遊戲刻後清零垂直速度，並可啟用滑翔。",
    ),
    "highJump.motion": (
        "Vertical motion applied at the start of each jump. Default 0.8; the original slider range is 0.2–10.",
        "每次跳跃开始时使用的垂直速度。默认 0.8；原始滑块范围为 0.2–10。",
        "每次跳躍開始時使用的垂直速度。預設 0.8；原始滑桿範圍為 0.2–10。",
    ),
    "highJump.mode.vulcan-2.7.5.glide": (
        "Apply the original gliding motion after the 100-tick delay, until landing. Disabled by default.",
        "100 刻延迟后应用原版滑翔速度，直到落地。默认关闭。",
        "100 刻延遲後套用原版滑翔速度，直到落地。預設關閉。",
    ),
}

for locale, index in (("zh_cn", 1), ("zh_tw", 2)):
    path = FOLDER / f"display_{locale}.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    for name, texts in TEXTS.items():
        key = f"liquidbounce.module.{name}.description"
        data["descriptions"][key] = texts[index]
        if locale == "zh_cn":
            data["englishDescriptions"][key] = texts[0]
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(locale, len(data["descriptions"]))
