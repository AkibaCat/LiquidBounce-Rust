#!/usr/bin/env python3
"""Copy the editable source tree into a separate Git checkout, with an audit."""
from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat

ROOT = Path(__file__).resolve().parents[1]
TOP_FILES = {"README.md", "LICENSE", "THIRD_PARTY.md", ".gitignore",
             "Restore-Source-Baselines.py", "source-baselines.json"}
SOURCE_DIRS = ("native", "patches", "scripts", "licenses")
SKIP_DIRS = {"target", "build", "build-tools", "cef-source", "dist", "node_modules",
             "__pycache__", ".git", ".gradle", ".horizon", ".local", "drafts", "user-data", "logs"}
SKIP_SUFFIXES = {".exe", ".dll", ".pdb", ".jar", ".class", ".pyc", ".zip", ".crate",
                 ".whl", ".log", ".dupbackup", ".bak", ".partial"}
RESEARCH_FILES = {"source-integrity.json", "horizon-source-integrity.json", "upstream-tree.json",
                  "upstream-commit.json", "module-parity.json", "target-entity-facts.json",
                  "rotation-source-vectors.json", "minecraft-agent-baseline.json",
                  "minecraft-agent-baseline-commit.json", "localization-coverage.json",
                  "localization-coverage.md", "CHINESE-LOCALIZATION.md", "DISPLAY-MODULE-DEPENDENCIES.md",
                  "interop-bootstrap.md", "PORTING.md", "port-progress.md"}
REQUIRED = ["native/liquidbounce-horizon/engine/src/lib.rs",
            "native/liquidbounce-horizon/menu-backend-patch.py",
            "native/liquidbounce-launcher/src/auth.rs",
            "native/liquidbounce-launcher/Cargo.lock",
            "native/liquidbounce-launcher/assets/OFL-Inter.txt",
            "native/liquidbounce-behaviors/src/navigation/simulation/sine-table.bin",
            "native/liquidbounce-behaviors/vendor/minecraft-math/sine-table.bin",
            "native/liquidbounce-behaviors/vendor/java_regex/jdk25/names.bin",
            "native/liquidbounce-behaviors/vendor/java_regex/LICENSE-MIT",
            "native/liquidbounce-behaviors/vendor/java_regex/LICENSE-APACHE",
            "scripts/preview-package/source/Restore-Source-Baselines.py",
            "licenses/Horizon-MIT.txt", "licenses/Horizon-Mixins-MIT.txt",
            "licenses/minecraft-auth-LGPL-3.0.txt", "licenses/CEF-BSD.txt",
            "research/source-integrity.json", "research/upstream-tree.json",
            "Restore-Source-Baselines.py", "source-baselines.json", "LICENSE", "THIRD_PARTY.md"]


def no_links(path: Path) -> None:
    for parent in (path, *path.parents):
        try:
            info = parent.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError(f"Source export does not follow reparse points: {parent}")


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def excluded_file(path: Path) -> bool:
    name = path.name.lower()
    if path.parent.name.endswith("-focused-tests") and name in {"preparation.json", "report.json"}:
        return True
    if path.parent == ROOT / "native/liquidbounce-webview" and name in {
            "cef-osr-smoke.json", "cef-osr-smoke.png", "native-smoke-verification.json",
            "original-theme-preview-verification.json", "original-theme-preview.png",
            "unreachable-smoke-verification.json"}:
        return True
    return (path.suffix.lower() in SKIP_SUFFIXES or name.startswith(("tmp-", "tmp_"))
            or ".draft." in name or ".javap." in name or name.endswith("-java-facts.txt")
            or name.endswith("-mc26.3-facts.txt"))


def selected_sources() -> tuple[dict[str, Path], dict[str, int]]:
    selected = {name: ROOT / name for name in TOP_FILES}
    excluded: Counter[str] = Counter()
    for top in SOURCE_DIRS:
        directory = ROOT / top
        if not directory.exists():
            continue
        for base, dirs, files in os.walk(directory, followlinks=False):
            for name in list(dirs):
                if name in SKIP_DIRS:
                    excluded[f"directory:{name}"] += 1
                    dirs.remove(name)
                else:
                    no_links(Path(base) / name)
            for name in files:
                path = Path(base) / name
                if excluded_file(path):
                    excluded[f"file:{path.suffix.lower() or 'temporary'}"] += 1
                    continue
                no_links(path)
                selected[path.relative_to(ROOT).as_posix()] = path
    for path in (ROOT / "research").iterdir():
        if path.name in RESEARCH_FILES or path.name.endswith("-PARITY.md"):
            no_links(path)
            selected[path.relative_to(ROOT).as_posix()] = path
    for relative in REQUIRED:
        if relative not in selected or not selected[relative].is_file():
            raise ValueError(f"Required source was omitted: {relative}")
    for name, path in selected.items():
        no_links(path)
        if not path.is_file():
            raise ValueError(f"Source file is missing: {name}")
    return selected, dict(sorted(excluded.items()))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--audit", type=Path, default=ROOT / "build/source-export-audit.json")
    args = parser.parse_args()
    destination = args.destination.absolute()
    no_links(destination)
    if destination.resolve() == ROOT or destination.resolve() in ROOT.parents or ROOT in destination.resolve().parents:
        parser.error("Destination must be separate from the source directory and its parent/child chain")
    selected, excluded = selected_sources()
    destination.mkdir(parents=True, exist_ok=True)
    destination_id = hashlib.sha256(os.path.normcase(str(destination.resolve())).encode()).hexdigest()
    marker = ROOT / "build/source-export-state" / (destination_id + ".json")
    no_links(marker)
    legacy_marker = destination / ".export-clean-source.json"
    no_links(legacy_marker)
    previous_marker = marker if marker.is_file() else legacy_marker
    previous = json.loads(previous_marker.read_text(encoding="utf-8")) if previous_marker.is_file() else {"files": {}}
    hashes = {name: digest(path) for name, path in sorted(selected.items())}
    copied = 0
    for name, source in sorted(selected.items()):
        target = destination.joinpath(*name.split("/"))
        target.resolve().relative_to(destination.resolve())
        no_links(target)
        if not target.is_file() or digest(target) != hashes[name]:
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
            copied += 1
        if digest(target) != hashes[name]:
            raise ValueError(f"Export copy failed verification: {name}")
    removed = []
    for name, old_hash in previous.get("files", {}).items():
        if name in selected:
            continue
        parts = name.split("/")
        if (any(part in ("", ".", "..", ".git") for part in parts)
                or "\\" in name or ":" in name
                or (name not in TOP_FILES and parts[0] not in (*SOURCE_DIRS, "research"))):
            raise ValueError(f"Unrecognized formerly exported path; preserved: {name}")
        target = destination.joinpath(*name.split("/"))
        target.resolve().relative_to(destination.resolve())
        no_links(target)
        if target.exists():
            if not target.is_file() or digest(target) != old_hash:
                raise ValueError(f"Preserving edited formerly exported source: {target}")
            target.unlink()
            removed.append(name)
    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.write_text(json.dumps({"formatVersion": 1, "files": hashes}, indent=2) + "\n", encoding="utf-8")
    # Restored baselines, local build caches and .git are never rewritten.
    binaries = [name for name in hashes if Path(name).suffix.lower() in SKIP_SUFFIXES]
    suspicious = [name for name in hashes if any(part.lower() in {"accounts", "saves", "user-data", "sessions"}
                                                for part in Path(name).parts)]
    missing = [name for name in REQUIRED if name not in hashes]
    report = {"formatVersion": 1, "createdUtc": datetime.now(timezone.utc).isoformat(),
              "source": str(ROOT), "destination": str(destination), "files": len(hashes),
              "bytes": sum(path.stat().st_size for path in selected.values()),
              "copiedFiles": copied, "removedFormerlyExportedFiles": removed,
              "excluded": excluded, "missingRequiredFiles": missing,
              "compiledOrArchiveFiles": binaries, "personalDataPaths": suspicious,
              "sourceTreeSha256": hashlib.sha256(json.dumps(hashes, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
              "passed": not (binaries or suspicious or missing)}
    args.audit.parent.mkdir(parents=True, exist_ok=True)
    args.audit.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    if not report["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
