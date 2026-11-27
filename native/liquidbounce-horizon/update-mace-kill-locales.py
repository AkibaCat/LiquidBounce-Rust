#!/usr/bin/env python3
"""Merge source MaceKill descriptions and FallHeight help; --check only validates."""
from pathlib import Path
import argparse
import json

ROOT = Path(__file__).resolve().parents[2]
HELP = {
    "en_us": "Maximum fall height to imitate. The highest offset with no block collision is used; intermediate heights are skipped.",
    "zh_cn": "模拟下落的最大高度。使用没有方块碰撞的最高偏移位置，不检查中间高度。",
    "zh_tw": "模擬下落的最大高度。使用沒有方塊碰撞的最高偏移位置，不檢查中間高度。",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    pending = {}
    for language in ("zh_cn", "zh_tw"):
        path = ROOT / f"native/liquidbounce-interop/resources/localization/display_{language}.json"
        data = json.loads(path.read_text(encoding="utf8"))
        original = json.loads((ROOT / f"upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json").read_text(encoding="utf8"))
        data["descriptions"]["liquidbounce.module.maceKill.description"] = original["liquidbounce.module.maceKill.description"]
        data["descriptions"]["liquidbounce.module.maceKill.fallHeight.description"] = HELP[language]
        if language == "zh_cn":
            english = json.loads((ROOT / "upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json").read_text(encoding="utf8"))
            data["englishDescriptions"]["liquidbounce.module.maceKill.description"] = english["liquidbounce.module.maceKill.description"]
            data["englishDescriptions"]["liquidbounce.module.maceKill.fallHeight.description"] = HELP["en_us"]
        pending[path] = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    if args.check:
        print("Validated MaceKill Chinese help and English fallback; no files written")
    else:
        for path, text in pending.items():
            path.write_text(text, encoding="utf8")
        print("Merged MaceKill Chinese help and English fallback")


if __name__ == "__main__":
    main()
