#!/usr/bin/env python3
"""Stage a module batch once, check it once, then run only its selected fixtures."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import subprocess
import sys

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

PROJECT = Path(__file__).resolve().parent.parent


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def run_step(command: list[str], cwd: Path, log: Path) -> dict:
    print(f"Running {' '.join(command)}; log: {log.relative_to(PROJECT)}", flush=True)
    step = {"command": command, "cwd": str(cwd), "log": str(log), "startedUtc": utc_now()}
    with log.open("w", encoding="utf-8") as output:
        try:
            env = os.environ.copy()
            if command[0] == "cargo":
                path_key = next((key for key in env if key.lower() == "path"), "PATH")
                tools = PROJECT / "native/liquidbounce-webview/build-tools"
                env[path_key] = os.pathsep.join([str(tools / "cmake/cmake/data/bin"), str(tools / "ninja"), env.get(path_key, "")])
                env["CEF_PATH"] = str(PROJECT / "build/cef-runtime")
            result = subprocess.run(command, cwd=cwd, env=env, stdout=output, stderr=subprocess.STDOUT, check=False)
            step["exitCode"] = result.returncode
        except OSError as error:
            output.write(str(error) + "\n")
            step["exitCode"] = 127
    step["finishedUtc"] = utc_now()
    step["passed"] = step["exitCode"] == 0
    if not step["passed"]:
        print("\n".join(log.read_text(encoding="utf-8", errors="replace").splitlines()[-35:]), flush=True)
    return step


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", required=True, help="Unique batch name; keeps earlier logs intact")
    parser.add_argument("--stage", default="horizon3", help="Reuse this build/staging workspace and its Cargo cache")
    parser.add_argument("--modules", nargs="+", required=True, help="Registry names or Kotlin module symbols in this batch")
    parser.add_argument("--fixture", action="append", default=[], help="Owned Python fixture path; repeat for this batch only")
    parser.add_argument("--webview", action="store_true", help="Also type-check the CEF and interop integration")
    parser.add_argument("--build", action="store_true", help="Build an executable when an in-game smoke test needs it")
    args = parser.parse_args()
    for value in (args.name, args.stage):
        if not re.fullmatch(r"[a-zA-Z][a-zA-Z0-9_-]{0,63}", value):
            parser.error("Batch and staging names must be simple folder names")
    ledger = json.loads((PROJECT / "research/module-parity.json").read_text(encoding="utf-8-sig"))
    known = {m["symbol"].removeprefix("Module").lower(): m["symbol"] for m in ledger["modules"]}
    modules = []
    for name in args.modules:
        key = name.removeprefix("Module").lower()
        if key not in known:
            parser.error(f"Unknown registered module: {name}")
        if known[key] not in modules:
            modules.append(known[key])
    fixtures = []
    for name in args.fixture:
        path = (PROJECT / name).resolve()
        try:
            relative = path.relative_to(PROJECT)
        except ValueError:
            parser.error(f"Fixture must be in this project: {name}")
        if relative.parts[0] not in ("scripts", "native") or path.suffix != ".py" or not path.is_file():
            parser.error(f"Fixture must be an existing owned Python script: {name}")
        if path not in fixtures:
            fixtures.append(path)
    destination = PROJECT / "build/batches" / args.name
    if destination.exists():
        parser.error(f"Batch evidence already exists: {destination}; use a new --name")
    destination.mkdir(parents=True)
    report = {
        "schemaVersion": 1,
        "batch": args.name,
        "modules": modules,
        "startedUtc": utc_now(),
        "steps": [],
        "passed": False,
        "gameValidation": "not_run",
        "upstreamDifferentialValidation": "not_run",
        "fullRegression": "deferred_until_all_modules_are_implemented",
    }
    stage = PROJECT / "build/staging" / args.stage
    commands = [([sys.executable, str(PROJECT / "scripts/stage-horizon-build.py"), "--name", args.stage,
                  *( ["--webview"] if args.webview else ["--behavior-hooks", "--gpu-ui"] )], PROJECT, "stage.log"),
                (["cargo", "+stable", "build" if args.build else "check", "-p", "liquidbounce-native-game", "--offline"], stage, "cargo.log")]
    commands.extend(([sys.executable, str(path)], PROJECT, f"fixture-{i + 1}-{path.stem}.log") for i, path in enumerate(fixtures))
    try:
        for command, cwd, name in commands:
            step = run_step(command, cwd, destination / name)
            report["steps"].append(step)
            if not step["passed"]:
                return step["exitCode"] or 1
            if name == "stage.log":
                staged = json.loads((stage / ".liquidbounce-staging.json").read_text(encoding="utf-8"))
                report["source"] = {key: staged[key] for key in ("horizon", "behaviorHookPatchSha256", "gpuUiPatchSha256", "nativeSourceSha256")}
        report["passed"] = True
        return 0
    finally:
        report["finishedUtc"] = utc_now()
        (destination / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"Batch {'passed' if report['passed'] else 'failed'}: {destination / 'report.json'}", flush=True)


if __name__ == "__main__":
    raise SystemExit(main())
