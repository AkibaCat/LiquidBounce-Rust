#!/usr/bin/env python3
"""Verify Horizon's semantic control protocol without reading or clicking a screen."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import subprocess

PROJECT = Path(__file__).resolve().parent.parent


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--controller", default="D:/Develop/MCL/.references/horizon/nightly/horizon-windows-x64/horizon-ctl.exe")
    parser.add_argument("--socket", default=str(PROJECT / "build/game-test/horizon-control.json"))
    parser.add_argument("--quit", action="store_true", help="Save and quit this isolated test instance")
    args = parser.parse_args()
    records = []

    def control(*command: str):
        result = subprocess.run([args.controller, "--socket", args.socket, "--timeout", "5", *command],
                                capture_output=True, text=True, encoding="utf-8", timeout=10, check=True)
        value = json.loads(result.stdout)
        records.append({"command": list(command), "response": value})
        return value

    hello = control("hello")
    before = control("state")
    if before["world"] != "LiquidBouncePortTest":
        raise ValueError("Refusing to mutate a game outside the isolated port test world")
    control("look", "45", "10")
    after = control("state")
    assert after["player"]["yaw"] == 45.0 and after["player"]["pitch"] == 10.0
    control("key", "e", "tap")
    screen = control("screen")
    control("key", "esc", "tap")
    control("release")
    final = control("state")
    assert final["screen"] == "playing"
    events = control("input-log", "--origin", "agent", "--limit", "20")
    if args.quit:
        control("quit")
    report = {"protocol": hello["protocol"], "release": hello["release"],
              "world": before["world"], "structuredControl": True, "screenRecognitionUsed": False,
              "rotationVerified": True, "screenResponse": screen, "inputLog": events, "records": records}
    path = PROJECT / "build/direct-control-validation.json"
    path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"report": str(path), "rotationVerified": True, "structuredControl": True}, indent=2))


if __name__ == "__main__":
    main()
