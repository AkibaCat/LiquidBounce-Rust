#!/usr/bin/env python3
"""Build an explicitly derived theme without modifying the upstream baseline."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import zipfile

PROJECT = Path(__file__).resolve().parent.parent
BASELINE = PROJECT / "upstream/LiquidBounce/src-theme"
OVERLAY = PROJECT / "native/liquidbounce-theme/src"
PATCH = PROJECT / "patches/liquidbounce-theme-chinese.patch"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def map_digest(files: dict[str, str]) -> str:
    return digest(json.dumps(files, sort_keys=True, separators=(",", ":")).encode())


def source_files(root: Path) -> dict[str, bytes]:
    return {path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*")) if path.is_file()
        and not any(part in {"node_modules", "dist", ".git", ".svelte-kit", ".vite"} for part in path.relative_to(root).parts)
        and not path.name.endswith((".log", ".tsbuildinfo"))}


def write_changed(path: Path, data: bytes) -> None:
    if path.is_file() and path.read_bytes() == data:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def verify_pin(files: dict[str, bytes]) -> dict:
    pin = json.loads((PROJECT / "research/source-integrity.json").read_text(encoding="utf-8-sig"))
    archive_path = PROJECT / "research/LiquidBounce-source.zip"
    if not archive_path.is_file():
        # The distributable source restoration records byte hashes for the
        # pinned tree even when the original zip is not distributed.
        integrity = PROJECT / "research/upstream-tree.json"
        tree = json.loads(integrity.read_text(encoding="utf-8-sig"))["tree"]
        checked = 0
        for entry in tree:
            if entry.get("type") != "blob" or not entry["path"].startswith("src-theme/"):
                continue
            relative = entry["path"][len("src-theme/"):]
            data = files[relative]
            blob = hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()
            if blob != entry["sha"]:
                raise ValueError(f"Theme baseline differs from pinned Git blob: {relative}")
            checked += 1
        return {"commit": pin["commit"], "verifiedTrackedFiles": checked, "verification": "pinned Git blob SHA-1"}
    if digest(archive_path.read_bytes()) != pin["archiveSha256"]:
        raise ValueError("Pinned upstream archive SHA-256 mismatch")
    checked = 0
    with zipfile.ZipFile(archive_path) as archive:
        for item in archive.infolist():
            match = re.match(r"[^/]+/src-theme/(.+)", item.filename)
            if item.is_dir() or not match:
                continue
            relative = match[1]
            if files.get(relative) != archive.read(item):
                raise ValueError(f"Theme baseline differs from pinned source: {relative}")
            checked += 1
    return {"commit": pin["commit"], "archiveSha256": pin["archiveSha256"], "verifiedTrackedFiles": checked}


def command(arguments: list[str], cwd: Path, log: Path) -> dict:
    result = subprocess.run(arguments, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    log.write_bytes(result.stdout)
    return {"command": arguments, "exitCode": result.returncode, "log": str(log.relative_to(PROJECT)),
        "logSha256": digest(result.stdout)}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", default="chinese")
    parser.add_argument("--build", action="store_true")
    args = parser.parse_args()
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]*", args.name):
        raise ValueError("Theme staging name must contain lowercase letters, numbers and hyphens")
    folder = PROJECT / "build/themes" / args.name
    source_root = folder / "src"
    output_root = folder / "dist"
    folder.mkdir(parents=True, exist_ok=True)
    baseline = source_files(BASELINE)
    pin = verify_pin(baseline)
    overlay = {"src/" + relative: data for relative, data in source_files(OVERLAY).items()}
    allowed = set(baseline) | set(overlay)
    unexpected = set(source_files(source_root)) - allowed if source_root.is_dir() else set()
    if unexpected:
        raise ValueError(f"Keep unexpected theme source files intact; select a new --name: {sorted(unexpected)}")
    for relative, data in baseline.items():
        write_changed(source_root / relative, data)
    subprocess.run(["git", "apply", "--check", "--whitespace=nowarn", str(PATCH)], cwd=source_root, check=True)
    subprocess.run(["git", "apply", "--whitespace=nowarn", str(PATCH)], cwd=source_root, check=True)
    for relative, data in overlay.items():
        write_changed(source_root / relative, data)
    modified = source_files(source_root)
    baseline_hashes = {name: digest(data) for name, data in baseline.items()}
    source_hashes = {name: digest(data) for name, data in modified.items()}
    report = {"schemaVersion": 1, "themeKind": "derived_chinese_localization",
        "originalThemeByteIdentical": False,
        "baseline": {**pin, "inputFiles": baseline_hashes, "sha256": map_digest(baseline_hashes)},
        "patch": {"path": str(PATCH.relative_to(PROJECT)), "sha256": digest(PATCH.read_bytes())},
        "overlay": {"path": str(OVERLAY.relative_to(PROJECT)),
            "files": {relative: digest(data) for relative, data in overlay.items()}},
        "generator": {"path": str(Path(__file__).relative_to(PROJECT)), "sha256": digest(Path(__file__).read_bytes())},
        "sourceRoot": str(source_root.relative_to(PROJECT)), "outputRoot": str(output_root.relative_to(PROJECT)),
        "sourceFiles": source_hashes, "sourceSha256": map_digest(source_hashes),
        "buildCompleted": False}
    record = folder / "localization-build.json"
    record.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    if args.build:
        npm = shutil.which("npm.cmd") or shutil.which("npm")
        node = shutil.which("node")
        if not npm or not node:
            raise RuntimeError("Node.js and npm are required to compile the derived theme")
        report["tools"] = {"node": subprocess.check_output([node, "--version"], text=True).strip(),
            "npm": subprocess.check_output([npm, "--version"], text=True).strip()}
        if not (source_root / "node_modules/.package-lock.json").is_file():
            report["install"] = command([npm, "ci", "--no-audit", "--no-fund"], source_root, folder / "install.log")
            if report["install"]["exitCode"]:
                record.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
                raise RuntimeError("Derived theme dependency installation failed; see install.log")
        for package in ("vite", "svelte", "typescript", "svelte-check"):
            report["tools"][package] = json.loads((source_root / "node_modules" / package / "package.json").read_text(encoding="utf-8"))["version"]
        report["typeCheck"] = command([npm, "run", "check"], source_root, folder / "typecheck.log")
        report["build"] = command([npm, "run", "build", "--", "--outDir", str(output_root), "--emptyOutDir"], source_root, folder / "build.log")
        outputs = {path.relative_to(output_root).as_posix(): digest(path.read_bytes()) for path in sorted(output_root.rglob("*")) if path.is_file()}
        report["outputFiles"] = outputs
        report["outputSha256"] = map_digest(outputs)
        report["builtAtUtc"] = datetime.now(timezone.utc).isoformat()
        report["buildCompleted"] = report["build"]["exitCode"] == 0 and (output_root / "index.html").is_file()
        # An inherited upstream type-check failure is kept visible instead of
        # being reported as a clean check. The caller can compare baseline logs.
        record.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        if not report["buildCompleted"]:
            raise RuntimeError("Derived theme build failed; see build.log")
    print(json.dumps({"metadata": str(record), "themeKind": report["themeKind"],
        "sourceSha256": report["sourceSha256"], "outputSha256": report.get("outputSha256"),
        "buildCompleted": report["buildCompleted"], "typeCheckExitCode": report.get("typeCheck", {}).get("exitCode")}, indent=2))


if __name__ == "__main__":
    main()
