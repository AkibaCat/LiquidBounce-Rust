#!/usr/bin/env python3
"""Restore hash-pinned source baselines beside the packaged local port sources."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import stat
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).absolute().parent


def local_path(relative: str) -> Path:
    parts = PurePosixPath(relative).parts
    if not parts or relative.startswith("/") or "\\" in relative or ":" in relative or any(p in (".", "..") for p in parts):
        raise ValueError(f"Unsafe source path: {relative}")
    path = ROOT.joinpath(*parts)
    path.resolve().relative_to(ROOT.resolve())
    for parent in (path, *path.parents):
        if parent.is_symlink() or (parent.exists() and getattr(parent.stat(), "st_file_attributes", 0) & 0x400):
            raise ValueError(f"Reparse points are not accepted: {parent}")
    return path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_port() -> None:
    manifest = json.loads(local_path("native-source-manifest.json").read_text(encoding="utf-8"))
    files = manifest["nativeSourceFiles"]
    if not isinstance(files, dict) or not files:
        raise ValueError("Missing compiled native source manifest")
    canonical = json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    if hashlib.sha256(canonical).hexdigest() != manifest["nativeSourceSha256"]:
        raise ValueError("Native source manifest aggregate mismatch")
    for name, expected in files.items():
        if sha256(local_path(name)) != expected:
            raise ValueError(f"Packaged native source changed: {name}")


def archive_parts(name: str, archive_root: str) -> tuple[str, ...]:
    if not name or any(character in name for character in ("\\", ":", "\0")):
        raise ValueError(f"Unsafe archive member: {name}")
    parts = name.removesuffix("/").split("/")
    if not parts or parts[0] != archive_root or any(part in ("", ".", "..") for part in parts):
        raise ValueError(f"Unexpected archive root: {name}")
    return tuple(parts)


def archive_content(source: zipfile.ZipFile, entry: zipfile.ZipInfo, members: dict[tuple[str, ...], zipfile.ZipInfo], archive_root: str) -> zipfile.ZipInfo:
    """Resolve internal archive links, copying regular bytes without OS symlinks."""
    visited: set[str] = set()
    while stat.S_IFMT(entry.external_attr >> 16) == stat.S_IFLNK:
        if entry.filename in visited:
            raise ValueError(f"Archive symlink cycle: {entry.filename}")
        visited.add(entry.filename)
        target = source.read(entry).decode("utf-8")
        if not target or target.startswith("/") or any(character in target for character in ("\\", ":", "\0")):
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
        resolved = members.get(tuple(parts))
        if resolved is None:
            raise ValueError(f"Archive symlink target is missing: {entry.filename}")
        entry = resolved
    if entry.is_dir() or stat.S_IFMT(entry.external_attr >> 16) not in (0, stat.S_IFREG):
        raise ValueError(f"Archive symlink target is not a regular file: {entry.filename}")
    return entry


def restore_archive(baseline: dict) -> None:
    archive = local_path(baseline["archive"])
    archive.parent.mkdir(parents=True, exist_ok=True)
    if not archive.exists():
        temporary = None
        try:
            with tempfile.NamedTemporaryFile(prefix="source-", suffix=".part", dir=archive.parent, delete=False) as output:
                temporary = Path(output.name)
                request = urllib.request.Request(baseline["archiveUrl"], headers={"User-Agent": "LiquidBounce-Horizon-source-restore/1"})
                with urllib.request.urlopen(request, timeout=120) as response:
                    shutil.copyfileobj(response, output)
            if sha256(temporary) != baseline["archiveSha256"]:
                raise ValueError(f"Downloaded source archive hash mismatch: {baseline['name']}")
            with archive.open("xb") as output, temporary.open("rb") as source:
                shutil.copyfileobj(source, output)
        finally:
            if temporary is not None:
                temporary.unlink(missing_ok=True)
    if sha256(archive) != baseline["archiveSha256"]:
        raise ValueError(f"Cached source archive hash mismatch; it was preserved: {archive}")

    destination = local_path(baseline["destination"])
    with zipfile.ZipFile(archive) as source:
        if source.comment.decode("ascii").strip() != baseline["commit"]:
            raise ValueError(f"Source archive commit mismatch: {baseline['name']}")
        if source.testzip() is not None:
            raise ValueError(f"Source archive CRC failure: {baseline['name']}")
        entries: list[tuple[zipfile.ZipInfo, Path]] = []
        members: dict[tuple[str, ...], zipfile.ZipInfo] = {}
        names = set()
        for entry in source.infolist():
            parts = archive_parts(entry.filename, baseline["archiveRoot"])
            key = "/".join(parts).casefold()
            if key in names:
                raise ValueError(f"Duplicate archive file: {entry.filename}")
            names.add(key)
            members[parts] = entry
        for parts, entry in members.items():
            if entry.is_dir():
                continue
            relative = "/".join((baseline["destination"], *parts[1:]))
            content = archive_content(source, entry, members, baseline["archiveRoot"])
            entries.append((content, local_path(relative)))
        if destination.exists():
            for entry, path in entries:
                if not path.is_file() or hashlib.sha256(source.read(entry)).hexdigest() != sha256(path):
                    raise ValueError(f"Existing baseline differs; preserved without overwriting: {path}")
        else:
            destination.mkdir(parents=True)
            for entry, path in entries:
                path.parent.mkdir(parents=True, exist_ok=True)
                with path.open("xb") as output, source.open(entry) as archived:
                    shutil.copyfileobj(archived, output)
    print(f"Verified {baseline['name']} {baseline['commit']}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify-only", action="store_true", help="Only verify included local port sources; do not download or extract")
    args = parser.parse_args()
    verify_port()
    if args.verify_only:
        print("Included compiled native sources verified")
        return
    lock = json.loads(local_path("source-baselines.json").read_text(encoding="utf-8"))
    if lock["formatVersion"] != 1:
        raise ValueError("Unsupported source baseline lock")
    for baseline in lock["baselines"]:
        restore_archive(baseline)


if __name__ == "__main__":
    main()
