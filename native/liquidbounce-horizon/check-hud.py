#!/usr/bin/env python3
"""Validate the HUD source package and every upstream theme component factory."""
from pathlib import Path
import argparse
import ast
import json

HERE = Path(__file__).resolve().parent
if (HERE / "engine").is_dir():
    ROOT = HERE.parents[1]
else:
    ROOT = HERE.parents[3]
THEME = ROOT / "upstream/LiquidBounce/src-theme/public"


def check_theme() -> int:
    metadata_path = THEME / "metadata.json"
    metadata = json.loads(metadata_path.read_text(encoding="utf-8-sig"))
    components = metadata.get("components", [])
    if len(components) != 20:
        raise ValueError(f"expected 20 upstream HUD components, found {len(components)}")
    for name in components:
        path = THEME / "components" / f"{name.lower()}.json"
        if not path.is_file():
            raise FileNotFoundError(path)
        value = json.loads(path.read_text(encoding="utf-8-sig"))
        if value.get("name") != name:
            raise ValueError(f"component name mismatch in {path}")
        if not isinstance(value.get("values", []), list):
            raise ValueError(f"component values is not an array in {path}")
    return len(components)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-root", type=Path)
    args = parser.parse_args()
    rust_sources = {
        "hud.rs": ROOT / "native/liquidbounce-behaviors/src/hud.rs",
        "hud_components.rs": ROOT / "native/liquidbounce-horizon/engine/src/hud_components.rs",
        "hud_config.rs": ROOT / "native/liquidbounce-horizon/engine/src/config/hud.rs",
        "hud_runtime.rs": ROOT / "native/liquidbounce-horizon/engine/src/hud_runtime.rs",
    }
    for name, ast_path in rust_sources.items():
        if not ast_path.is_file():
            raise FileNotFoundError(ast_path)
        # Rust sources are checked for the key contract anchors below. Python
        # AST parsing still covers all companion scripts in this package.
        source = ast_path.read_text(encoding="utf-8")
        if name == "hud_components.rs":
            for needle in ("TWEAK_HOTBAR", "metadata.json", "componentsUpdate"):
                if needle not in source:
                    raise ValueError(f"missing HUD component contract {needle} in {name}")
        if name == "hud_config.rs" and "parse_float_pair" not in source:
            raise ValueError("HUD alpha range must use parse_float_pair")
    for name in ("check-hud.py", "integrate-hud.py", "publish-hud.py"):
        ast.parse((HERE / name).read_text(encoding="utf-8"), filename=name)
    count = check_theme()
    if args.source_root:
        for relative in (
            "crates/horizon-client/src/game/native_client_services.rs",
            "crates/horizon-client-render/src/lib.rs",
        ):
            if not (args.source_root / relative).is_file():
                raise FileNotFoundError(args.source_root / relative)
    print(f"HUD source validation passed: {count} upstream components")


if __name__ == "__main__":
    main()
