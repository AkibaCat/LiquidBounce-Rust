#!/usr/bin/env python3
"""Publish HUD's owned sources; --check never modifies the workspace."""
from pathlib import Path
import argparse

HERE = Path(__file__).resolve().parent
SOURCE = HERE if HERE.name == "hud" else HERE / "drafts" / "hud"
NATIVE = SOURCE.parents[1]
ROOT = NATIVE.parents[1]
FILES = {
    "hud.rs": ROOT / "native/liquidbounce-behaviors/src/hud.rs",
    "hud_components.rs": NATIVE / "engine/src/hud_components.rs",
    "hud_config.rs": NATIVE / "engine/src/config/hud.rs",
    "hud_runtime.rs": NATIVE / "engine/src/hud_runtime.rs",
        "check-hud.py": NATIVE / "check-hud.py",
        "integrate-hud.py": NATIVE / "integrate-hud.py",
        "publish-hud.py": NATIVE / "publish-hud.py",
}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    for name, destination in FILES.items():
        source = SOURCE / name
        if not source.is_file():
            raise FileNotFoundError(source)
        if destination.exists() and destination.read_bytes() != source.read_bytes() and name.endswith(".rs"):
            raise FileExistsError(f"Owned target has different contents: {destination}")
    if not args.check:
        for name, destination in FILES.items():
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes((SOURCE / name).read_bytes())
    print(f"{'Validated' if args.check else 'Published'} {len(FILES)} HUD owned files")


if __name__ == "__main__":
    main()
