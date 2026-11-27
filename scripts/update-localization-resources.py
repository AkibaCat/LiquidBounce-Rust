#!/usr/bin/env python3
"""Refresh byte-exact language resources and stable module keys from the pin."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re

PROJECT = Path(__file__).resolve().parent.parent
SOURCE = PROJECT / "upstream/LiquidBounce"
DESTINATION = PROJECT / "native/liquidbounce-interop/resources/localization"


def main() -> None:
    DESTINATION.mkdir(parents=True, exist_ok=True)
    provenance = {"upstreamCommit": "ae0eb396c07f22383789b28d5912d714c6ca0a86", "resources": {}}
    for locale in ("en_us", "zh_cn", "zh_tw"):
        relative = f"src/main/resources/resources/liquidbounce/lang/{locale}.json"
        data = (SOURCE / relative).read_bytes()
        (DESTINATION / f"{locale}.json").write_bytes(data)
        provenance["resources"][locale] = {"source": relative, "sha256": hashlib.sha256(data).hexdigest()}
    catalog = json.loads((PROJECT / "research/module-parity.json").read_text(encoding="utf-8-sig"))
    english = json.loads((DESTINATION / "en_us.json").read_text(encoding="utf-8"))
    keys = {}
    for module in catalog["modules"]:
        source = (SOURCE / module["source"]).read_text(encoding="utf-8")
        match = re.search(r"\bobject\s+" + re.escape(module["symbol"]) + r'\s*:\s*ClientModule\s*\(\s*(?:name\s*=\s*)?"([^"]+)"', source)
        if not match:
            raise ValueError(f"Missing literal ClientModule name: {module['symbol']}")
        name = match[1]
        key = "liquidbounce.module." + name[0].lower() + name[1:]
        if key + ".description" not in english:
            matches = [candidate.removesuffix(".description") for candidate in english
                if candidate.endswith(".description") and candidate.casefold() == (key + ".description").casefold()]
            if not matches:
                raise ValueError(f"Missing original module description: {name}")
            key = sorted(matches, key=lambda candidate: (candidate.split(".")[-1][0].isupper(), candidate))[0]
        keys[name] = key
    display = json.loads((DESTINATION / "display_zh_cn.json").read_text(encoding="utf-8"))
    if set(display["moduleNames"]) != set(keys):
        raise ValueError(f"Chinese display coverage differs from stable names: missing={sorted(set(keys)-set(display['moduleNames']))}, extra={sorted(set(display['moduleNames'])-set(keys))}")
    (DESTINATION / "module_keys.json").write_text(json.dumps(keys, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    provenance["registeredUpstreamModuleNames"] = len(keys)
    provenance["displayOnly"] = True
    traditional_display = json.loads((DESTINATION / "display_zh_tw.json").read_text(encoding="utf-8"))
    provenance["ownedDisplayResources"] = {locale: {
        "sha256": hashlib.sha256((DESTINATION / f"display_{locale}.json").read_bytes()).hexdigest(),
        "independentModuleNames": len(dictionary["moduleNames"]),
        "independentLabels": len(dictionary["labels"]),
        "ownedSettingDescriptions": len(dictionary.get("descriptions", {})),
    } for locale, dictionary in (("zh_cn", display), ("zh_tw", traditional_display))}
    provenance["settingDescriptionPolicy"] = "The pinned language files contain module descriptions but none of the implemented setting descriptions; explicitly owned English/Chinese help entries fill those missing keys. Upstream entries take priority."
    provenance["fallback"] = {"zh_cn": ["zh_cn", "en_us"], "zh_tw": ["zh_tw", "zh_cn", "en_us"], "other": ["en_us"]}
    (DESTINATION / "provenance.json").write_text(json.dumps(provenance, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(provenance, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
