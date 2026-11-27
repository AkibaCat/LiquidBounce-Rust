#!/usr/bin/env python3
"""Restore and verify the project's fixed upstream source archives (Python 3.11+)."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).absolute().parent


def no_links(path: Path) -> None:
    for parent in (path, *path.parents):
        try:
            info = parent.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError(f"Reparse points are not accepted: {parent}")


def local_path(relative: str) -> Path:
    parts = PurePosixPath(relative).parts
    if (not parts or relative != "/".join(parts) or relative.startswith("/")
            or any(character in relative for character in ("\\", ":", "\0"))
            or any(part in (".", "..") for part in parts)):
        raise ValueError(f"Unsafe source path: {relative}")
    path = ROOT.joinpath(*parts)
    path.resolve().relative_to(ROOT.resolve())
    no_links(path)
    return path


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def archive_parts(name: str, archive_root: str) -> tuple[str, ...]:
    if not name or any(character in name for character in ("\\", ":", "\0")):
        raise ValueError(f"Unsafe archive member: {name}")
    parts = name.removesuffix("/").split("/")
    if not parts or parts[0] != archive_root or any(part in ("", ".", "..") for part in parts):
        raise ValueError(f"Unexpected archive root: {name}")
    return tuple(parts)


def archive_content(source: zipfile.ZipFile, entry: zipfile.ZipInfo,
                    members: dict[tuple[str, ...], zipfile.ZipInfo], archive_root: str) -> zipfile.ZipInfo:
    """Resolve internal archive links into regular files on every host platform."""
    visited: set[str] = set()
    while stat.S_IFMT(entry.external_attr >> 16) == stat.S_IFLNK:
        if entry.filename in visited:
            raise ValueError(f"Archive symlink cycle: {entry.filename}")
        visited.add(entry.filename)
        target = source.read(entry).decode("utf-8")
        if not target or target.startswith("/") or any(c in target for c in ("\\", ":", "\0")):
            raise ValueError(f"Unsafe archive symlink target: {entry.filename}")
        parts = list(archive_parts(entry.filename, archive_root)[:-1])
        for part in target.split("/"):
            if part in ("", "."):
                continue
            if part == "..":
                if len(parts) <= 1:
                    raise ValueError(f"Archive symlink escapes its root: {entry.filename}")
                parts.pop()
            else:
                parts.append(part)
        if not parts or parts[0] != archive_root:
            raise ValueError(f"Archive symlink escapes its root: {entry.filename}")
        entry = members.get(tuple(parts))
        if entry is None:
            raise ValueError(f"Archive symlink target is missing: {target}")
    if entry.is_dir() or stat.S_IFMT(entry.external_attr >> 16) not in (0, stat.S_IFREG):
        raise ValueError(f"Archive target is not a regular file: {entry.filename}")
    return entry


def acquire_archive(baseline: dict, args: argparse.Namespace) -> Path:
    archive = local_path(baseline["archive"])
    if archive.exists():
        if not archive.is_file() or sha256(archive) != baseline["archiveSha256"]:
            raise ValueError(f"Cached source archive differs; preserved: {archive}")
        return archive
    cached = None
    if args.archive_cache:
        cache = args.archive_cache.absolute()
        for candidate in (cache / baseline["archive"], cache / archive.name):
            no_links(candidate)
            if candidate.is_file():
                if sha256(candidate) != baseline["archiveSha256"]:
                    raise ValueError(f"External source archive differs: {candidate}")
                cached = candidate
                break
    if args.verify_only:
        if cached is None:
            raise ValueError(f"Archive needed for offline verification: {archive}")
        return cached
    if cached is None and args.offline:
        raise ValueError(f"Missing cached archive in offline mode: {archive}")
    archive.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(prefix="source-", suffix=".part", dir=archive.parent, delete=False) as output:
            temporary = Path(output.name)
            if cached is not None:
                with cached.open("rb") as stream:
                    shutil.copyfileobj(stream, output)
            else:
                request = urllib.request.Request(baseline["archiveUrl"], headers={"User-Agent": "LiquidBounce-Rust-source-bootstrap/1"})
                with urllib.request.urlopen(request, timeout=120) as response:
                    shutil.copyfileobj(response, output)
        if sha256(temporary) != baseline["archiveSha256"]:
            raise ValueError(f"Downloaded source archive hash mismatch: {baseline['name']}")
        # Exclusive creation preserves a concurrently supplied cache file.
        with archive.open("xb") as output, temporary.open("rb") as stream:
            shutil.copyfileobj(stream, output)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
    return archive


def restore_archive(baseline: dict, args: argparse.Namespace) -> dict:
    archive = acquire_archive(baseline, args)
    destination = local_path(baseline["destination"])
    with zipfile.ZipFile(archive) as source:
        if source.comment.decode("ascii").strip() != baseline["commit"]:
            raise ValueError(f"Source archive commit mismatch: {baseline['name']}")
        if source.testzip() is not None:
            raise ValueError(f"Source archive CRC failure: {baseline['name']}")
        members: dict[tuple[str, ...], zipfile.ZipInfo] = {}
        names: set[str] = set()
        for entry in source.infolist():
            parts = archive_parts(entry.filename, baseline["archiveRoot"])
            key = "/".join(parts).casefold()
            if key in names:
                raise ValueError(f"Duplicate archive file: {entry.filename}")
            names.add(key)
            members[parts] = entry
        entries = [(parts[1:], archive_content(source, entry, members, baseline["archiveRoot"]))
                   for parts, entry in members.items() if not entry.is_dir()]
        if destination.exists():
            for relative, entry in entries:
                path = local_path("/".join((baseline["destination"], *relative)))
                if not path.is_file() or hashlib.sha256(source.read(entry)).hexdigest() != sha256(path):
                    raise ValueError(f"Existing baseline differs; preserved without overwriting: {path}")
        elif args.verify_only:
            raise ValueError(f"Baseline is missing; run bootstrap first: {destination}")
        else:
            destination.parent.mkdir(parents=True, exist_ok=True)
            with tempfile.TemporaryDirectory(prefix=destination.name + "-", dir=destination.parent) as temporary:
                tree = Path(temporary) / "tree"
                tree.mkdir()
                for relative, entry in entries:
                    path = tree.joinpath(*relative)
                    path.parent.mkdir(parents=True, exist_ok=True)
                    with path.open("xb") as output, source.open(entry) as stream:
                        shutil.copyfileobj(stream, output)
                os.rename(tree, destination)
    result = {"name": baseline["name"], "commit": baseline["commit"],
              "archiveSha256": baseline["archiveSha256"], "verifiedFiles": len(entries)}
    print(json.dumps(result), flush=True)
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify-only", action="store_true", help="Verify every pinned upstream file without downloads or extraction")
    parser.add_argument("--offline", action="store_true", help="Require local archives; never use the network")
    parser.add_argument("--archive-cache", type=Path, help="Existing archive folder or project root used as an offline cache")
    args = parser.parse_args()
    lock = json.loads(local_path("source-baselines.json").read_text(encoding="utf-8"))
    if lock.get("formatVersion") != 1 or len(lock.get("baselines", [])) != 3:
        raise ValueError("Unsupported or incomplete source baseline lock")
    for baseline in lock["baselines"]:
        if (not re.fullmatch(r"[0-9a-f]{40}", baseline["commit"])
                or not re.fullmatch(r"[0-9a-f]{64}", baseline["archiveSha256"])
                or not baseline["archiveUrl"].startswith("https://codeload.github.com/")):
            raise ValueError("Invalid source baseline pin")
        restore_archive(baseline, args)
    print("All three pinned source baselines verified. Local port sources remain editable.")


if __name__ == "__main__":
    main()
