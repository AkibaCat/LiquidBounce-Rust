#!/usr/bin/env python3
"""Read-only integration audit for HUD-owned sources.

Shared registry/actor wiring is intentionally left to the batch integrator;
this command proves that the owned package is ready and that the WebView HUD
surface has its independent page/compositor hooks.
"""
from pathlib import Path
import argparse
import ast

HERE = Path(__file__).resolve().parent
if (HERE / "engine").is_dir():
    ROOT = HERE.parents[1]
else:
    ROOT = HERE.parents[3]


def require(path: Path, *needles: str) -> None:
    if not path.is_file():
        raise FileNotFoundError(path)
    source = path.read_text(encoding="utf-8")
    for needle in needles:
        if needle not in source:
            raise ValueError(f"missing integration anchor {needle!r} in {path}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if not args.check:
        raise SystemExit("HUD integrator is audit-only; pass --check")
    check = HERE / "check-hud.py"
    ast.parse(check.read_text(encoding="utf-8"), filename=str(check))
    require(
        ROOT / "native/liquidbounce-webview/src/input.rs",
        "HudOverlay",
    )
    require(
        ROOT / "native/liquidbounce-webview/src/runtime.rs",
        "hud_mailbox",
        "hud_snapshot_current_thread",
        "clear_hud_pixels",
    )
    require(
        ROOT / "native/liquidbounce-webview/src/horizon.rs",
        "hud_compositor",
        "hud_snapshot_current_thread",
    )
    require(
        ROOT / "native/liquidbounce-horizon/engine/src/hud_browser.rs",
        "hud_request",
        "hud_frame",
        "HudOverlay",
    )
    require(
        ROOT / "native/liquidbounce-horizon/engine/src/browser.rs",
        "mod hud_browser",
        "components",
        ".load(",
        "browser.hud_frame",
    )
    print("HUD owned integration audit passed")


if __name__ == "__main__":
    main()
