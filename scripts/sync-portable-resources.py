#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Review/apply a verified resource staging tree to an existing release.

Writes use per-file atomic replacement, an external backup, and rollback.
The existing release user-data directory is never enumerated or opened.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import tempfile

PROJECT = Path(__file__).resolve().parent.parent
CHECKED_DIRECTORIES = set()


def digest(path, algorithm="sha256"):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, algorithm).hexdigest()


def data_json(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")


def no_links(path):
    for position, parent in enumerate((path, *path.parents)):
        if position and parent in CHECKED_DIRECTORIES:
            break
        try:
            info = parent.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError(f"Reparse point is not accepted: {parent}")
        if stat.S_ISDIR(info.st_mode):
            CHECKED_DIRECTORIES.add(parent)


def local(root, name):
    if (not isinstance(name, str) or not name or "\\" in name or ":" in name
            or name != PurePosixPath(name).as_posix() or PurePosixPath(name).is_absolute()
            or name.split("/")[0].casefold() == "user-data"
            or any(p in ("", ".", "..") or p.endswith((" ", ".")) for p in name.split("/"))
            or any(ord(c) < 32 or c in '*?<>|"' for c in name)):
        raise ValueError(f"Unsafe relative path: {name!r}")
    path = root.joinpath(*name.split("/"))
    no_links(path)
    path.resolve().relative_to(root.resolve())
    return path


def tree_files(root):
    for directory, dirs, files in os.walk(root, followlinks=False):
        base = Path(directory)
        # Never inspect a user's mutable data or its reparse attributes.
        dirs[:] = [n for n in dirs if not (base == root and n.casefold() == "user-data")]
        for name in dirs:
            no_links(base / name)
        for name in files:
            path = base / name
            no_links(path)
            yield path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--plan", type=Path, default=PROJECT / "build/launcher-qa-20261127/portable-resources-sync-plan.json")
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    root, resources, plan = args.package.absolute(), args.resources.absolute(), args.plan.absolute()
    for path in (root, resources, plan):
        no_links(path)
    release_root = (PROJECT.parent / "ReleasesPack").resolve()
    root.resolve().relative_to(release_root)
    if root.resolve() == release_root:
        parser.error("Select a release subfolder")
    plan.resolve().relative_to((PROJECT / "build").resolve())
    if resources.resolve() == (root / "resources/26.3").resolve():
        parser.error("Resource staging must be outside the release")
    manifest_path = local(root, "package-manifest.json")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8-sig"))
    portable = json.loads(local(resources, "portable-assets.json").read_text(encoding="utf-8"))
    if portable.get("minecraft") != "26.3" or portable.get("allReferencesVerified") is not True:
        parser.error("Resources must contain verified portable-assets.json for Minecraft 26.3")
    index_bytes = local(resources, "asset-index.json").read_bytes()
    if hashlib.sha1(index_bytes).hexdigest() != portable.get("assetIndexSha1"):
        parser.error("Asset index differs from portable resource metadata")
    if hashlib.sha256(index_bytes).hexdigest() != portable.get("assetIndexSha256"):
        parser.error("Asset index SHA-256 differs from metadata")
    index = json.loads(index_bytes)["objects"]
    if len(index) != portable.get("indexEntries"):
        parser.error("Asset index entry count differs from metadata")
    alias = local(resources, f"assets/indexes/{portable['assetIndexId']}.json")
    if alias.read_bytes() != index_bytes:
        parser.error("Portable index alias differs")
    for name, info in index.items():
        key = info.get("hash", "")
        if not re.fullmatch(r"[0-9a-f]{40}", key):
            parser.error(f"Invalid object hash: {name}")
        for candidate in (local(resources, "assets/" + name), local(resources, f"assets/objects/{key[:2]}/{key}")):
            if candidate.stat().st_size != info["size"] or digest(candidate, "sha1") != key:
                parser.error(f"Resource reference fails SHA-1/size validation: {candidate}")
    inputs = {}

    def add_source(name, source):
        local(root, name)
        inputs[name] = {"source": str(source), "sha256": digest(source), "bytes": source.stat().st_size}

    def add_data(name, value):
        local(root, name)
        payload = data_json(value)
        inputs[name] = {"data": payload, "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)}

    resource_records = []
    for source in tree_files(resources):
        name = source.relative_to(resources).as_posix()
        if source.name in {".horizon-prepare.lock", "horizon-data.bin", "horizon-sprites.cache"}:
            continue
        add_source("resources/26.3/" + name, source)
        item = inputs["resources/26.3/" + name]
        resource_records.append({"path": name, "bytes": item["bytes"], "sha256": item["sha256"]})
    resource_records.sort(key=lambda item: item["path"])
    timestamp = datetime.now(timezone.utc).isoformat()
    resource_report = {"formatVersion": 2, "createdUtc": timestamp, "minecraft": "26.3", "relativeRoot": "resources/26.3",
                       "fileCount": len(resource_records), "bytes": sum(i["bytes"] for i in resource_records),
                       "files": resource_records, "portableAssets": portable,
                       "excludedPreparationFiles": [".horizon-prepare.lock", "horizon-data.bin", "horizon-sprites.cache"],
                       "preparation": json.loads(local(resources, "horizon-resources.json").read_text(encoding="utf-8"))}
    add_data("metadata/resources-manifest.json", resource_report)
    add_data("metadata/portable-assets.json", portable)
    for filename in ("prepare-portable-assets.py", "sync-portable-resources.py"):
        add_source("source/scripts/" + filename, Path(__file__).parent / filename)
    changed = [name for name, info in inputs.items() if not local(root, name).is_file()
               or local(root, name).stat().st_size != info["bytes"] or digest(local(root, name)) != info["sha256"]]
    removals = ["resources/26.3/" + name for name in ("horizon-data.bin", "horizon-sprites.cache")
                if local(root, "resources/26.3/" + name).is_file()]
    plan_data = {"createdUtc": timestamp, "package": str(root), "resources": str(resources), "applyRequested": args.apply,
                 "changedFiles": [{"path": n, "bytes": inputs[n]["bytes"], "sha256": inputs[n]["sha256"]} for n in changed],
                 "removedGeneratedCaches": removals, "fileCount": len(resource_records), "bytes": resource_report["bytes"],
                 "portableAssets": portable, "userDataPolicy": "Never enumerated, read, copied, renamed, overwritten, or removed."}
    plan.parent.mkdir(parents=True, exist_ok=True)
    plan.write_bytes(data_json(plan_data))
    print(json.dumps({"plan": str(plan), "changedFiles": len(changed), "resourceBytes": resource_report["bytes"],
                      "allReferencesVerified": True, "applyRequested": args.apply}), flush=True)
    if not args.apply:
        return
    backup = Path(tempfile.mkdtemp(prefix="portable-resources-backup-", dir=plan.parent))
    written = []

    def replace(name, entry):
        target = local(root, name)
        target.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(prefix=".resources-sync-", suffix=".tmp", dir=target.parent, delete=False) as stream:
            temporary = Path(stream.name)
            if "data" in entry:
                stream.write(entry["data"])
            else:
                with Path(entry["source"]).open("rb") as source:
                    shutil.copyfileobj(source, stream)
        try:
            if digest(temporary) != entry["sha256"]:
                raise ValueError(f"Copy verification failed: {name}")
            os.replace(temporary, target)
        finally:
            temporary.unlink(missing_ok=True)

    try:
        for name in changed + removals + ["package-manifest.json"]:
            source = local(root, name)
            if source.exists():
                saved = local(backup, name)
                saved.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, saved)
        for name in changed:
            written.append(name)
            replace(name, inputs[name])
        for name in removals:
            written.append(name)
            local(root, name).unlink()
        manifest["updatedUtc"] = timestamp
        manifest["preparedResources"] = {"minecraft": "26.3", "path": "resources/26.3",
                                         "manifest": "metadata/resources-manifest.json", "fileCount": len(resource_records),
                                         "bytes": resource_report["bytes"], "portableAssets": "metadata/portable-assets.json",
                                         "assetIndexSha1": portable["assetIndexSha1"], "allReferencesVerified": True}
        manifest["files"] = sorted(({"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size, "sha256": digest(p)}
                                    for p in tree_files(root) if p != manifest_path), key=lambda item: item["path"])
        payload = data_json(manifest)
        written.append("package-manifest.json")
        replace("package-manifest.json", {"data": payload, "sha256": hashlib.sha256(payload).hexdigest()})
        result = {"applied": True, "package": str(root), "changedFiles": len(changed), "removedGeneratedCaches": removals,
                  "resourceFiles": len(resource_records), "resourceBytes": resource_report["bytes"], "portableAssets": portable,
                  "backup": str(backup), "userDataTouched": False}
        (backup / "sync-result.json").write_bytes(data_json(result))
        plan.with_name(plan.stem + "-result.json").write_bytes(data_json(result))
        print(json.dumps(result), flush=True)
    except BaseException:
        for name in reversed(written):
            saved, target = local(backup, name), local(root, name)
            if saved.exists():
                shutil.copy2(saved, target)
            else:
                target.unlink(missing_ok=True)
        raise


if __name__ == "__main__":
    main()
