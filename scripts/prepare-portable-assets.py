#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Build portable resources from prepared game data and a verified asset index.

By default this reads only local caches. --download permits missing official
metadata/objects to be fetched by pinned hash. No launcher account, save, or
Minecraft installation directory is copied. The output must be a new directory.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import tempfile
import urllib.request

PROJECT = Path(__file__).resolve().parent.parent
VERSION = "26.3"
MANIFEST_SHA1 = "11f1aefe473eefba0af62e3286c33b692807b2eb"
SHA1 = re.compile(r"[0-9a-f]{40}")
CHECKED_DIRECTORIES = set()


def checksum(path, algorithm="sha1"):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, algorithm).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def safe_name(name):
    if (not isinstance(name, str) or not name or "\\" in name or ":" in name
            or name != PurePosixPath(name).as_posix() or PurePosixPath(name).is_absolute()
            or any(p in ("", ".", "..") or p.endswith((" ", ".")) for p in name.split("/"))
            or any(ord(c) < 32 or c in '*?<>|"' for c in name)):
        raise ValueError(f"Unsafe asset name: {name!r}")
    return Path(*name.split("/"))


def no_links(path):
    for position, parent in enumerate((path, *path.parents)):
        if position and parent in CHECKED_DIRECTORIES:
            break
        try:
            info = parent.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError(f"Resource paths cannot traverse reparse points: {parent}")
        if stat.S_ISDIR(info.st_mode):
            CHECKED_DIRECTORIES.add(parent)


def download_verified(url, expected, size, destination):
    if not SHA1.fullmatch(expected):
        raise ValueError("Invalid official checksum")
    if not url.startswith(("https://piston-meta.mojang.com/", "https://resources.download.minecraft.net/")):
        raise ValueError("Only official metadata and asset URLs are accepted")
    with urllib.request.urlopen(url, timeout=45) as response:
        data = response.read()
    if hashlib.sha1(data).hexdigest() != expected or (size is not None and len(data) != size):
        raise ValueError(f"Official download failed checksum/size verification: {url}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_name(destination.name + ".partial")
    try:
        temporary.write_bytes(data)
        os.replace(temporary, destination)
    finally:
        temporary.unlink(missing_ok=True)
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepared", type=Path, default=PROJECT / "build/resources/26.3")
    parser.add_argument("--output", type=Path, default=PROJECT / "build/portable-resources/26.3")
    parser.add_argument("--version-json", type=Path, default=PROJECT / "build/metadata/26.3.json")
    parser.add_argument("--asset-index", type=Path, help="Existing same-version index; may contain republished translations")
    parser.add_argument("--source-assets", action="append", type=Path, help="Asset cache root containing objects/<prefix>/<hash>")
    parser.add_argument("--cache", type=Path, default=PROJECT / "build/official-asset-cache")
    parser.add_argument("--download", action="store_true", help="Fetch only missing official metadata/objects after SHA-1 validation")
    parser.add_argument("--audit-only", action="store_true")
    parser.add_argument("--report", type=Path, default=PROJECT / "build/launcher-qa-20261127/portable-assets-audit.json")
    args = parser.parse_args()
    args.prepared, args.output, args.cache = args.prepared.resolve(), args.output.absolute(), args.cache.absolute()
    for path in (args.prepared, args.output, args.cache, args.version_json.absolute(), args.report.absolute()):
        no_links(path)
    if not (args.prepared / "horizon-resources.json").is_file():
        parser.error("--prepared must contain complete Horizon 26.3 prepared resources")
    preparation = json.loads((args.prepared / "horizon-resources.json").read_text(encoding="utf-8-sig"))
    if preparation.get("minecraft") != VERSION:
        parser.error("Prepared resources do not identify Minecraft 26.3")
    if not args.version_json.is_file():
        if not args.download:
            parser.error("Pinned version metadata is missing; supply --version-json or --download")
        download_verified(f"https://piston-meta.mojang.com/v1/packages/{MANIFEST_SHA1}/{VERSION}.json",
                          MANIFEST_SHA1, None, args.version_json)
    version = json.loads(args.version_json.read_text(encoding="utf-8-sig"))
    if version.get("id") != VERSION:
        parser.error("Version metadata does not identify Minecraft 26.3")
    official = version["assetIndex"]
    if not re.fullmatch(r"[A-Za-z0-9_-]+", official["id"]) or not SHA1.fullmatch(official["sha1"]):
        parser.error("Unsafe official index metadata")
    index_path = args.asset_index or args.cache / "indexes" / (official["sha1"] + ".json")
    no_links(index_path.absolute())
    if not index_path.is_file():
        if not args.download:
            parser.error("Asset index is missing; supply --asset-index or --download")
        download_verified(official["url"], official["sha1"], official["size"], index_path)
    raw_index = index_path.read_bytes()
    index_sha1 = hashlib.sha1(raw_index).hexdigest()
    if args.asset_index is None and (index_sha1 != official["sha1"] or len(raw_index) != official["size"]):
        parser.error("Cached official index differs from the pinned version metadata")
    index = json.loads(raw_index).get("objects")
    if not isinstance(index, dict) or not index:
        parser.error("Asset index has no objects")
    sources = args.source_assets or [Path.home() / ".gradle/caches/fabric-loom/assets", args.cache]
    sources = [p.resolve() for p in sources]
    for path in sources:
        no_links(path)
    objects = {}
    for name, info in index.items():
        safe_name(name)
        if (not isinstance(info, dict) or not SHA1.fullmatch(info.get("hash", ""))
                or not isinstance(info.get("size"), int) or info["size"] < 0):
            parser.error(f"Invalid object record: {name}")
        key = info["hash"]
        previous = objects.setdefault(key, {"size": info["size"], "names": []})
        if previous["size"] != info["size"]:
            parser.error(f"Conflicting object sizes: {key}")
        previous["names"].append(name)
    print(json.dumps({"phase": "verifying", "indexEntries": len(index), "uniqueObjects": len(objects),
                      "objectBytes": sum(i["size"] for i in objects.values())}), flush=True)

    def verify_object(item):
        key, info = item
        candidates = [p / "objects" / key[:2] / key for p in sources]
        candidates += [args.prepared / "assets" / safe_name(n) for n in info["names"]]
        for source in candidates:
            no_links(source)
            if source.is_file() and source.stat().st_size == info["size"] and checksum(source) == key:
                return key, source, None
        if args.download:
            target = args.cache / "objects" / key[:2] / key
            try:
                return key, download_verified(f"https://resources.download.minecraft.net/{key[:2]}/{key}", key, info["size"], target), None
            except Exception as error:
                return key, None, str(error)
        return key, None, "Missing verified local object"

    found, missing = {}, []
    with ThreadPoolExecutor(max_workers=8) as pool:
        for key, source, error in pool.map(verify_object, objects.items()):
            if source is None:
                missing.append({"hash": key, "size": objects[key]["size"], "names": objects[key]["names"], "error": error})
            else:
                found[key] = source
    portable = {"formatVersion": 1, "minecraft": VERSION, "assetIndexId": official["id"],
                "assetIndexSha1": index_sha1, "assetIndexSha256": hashlib.sha256(raw_index).hexdigest(),
                "versionMetadataAssetIndexSha1": official["sha1"],
                "indexMatchesVersionMetadata": index_sha1 == official["sha1"],
                "indexProvenance": "Explicit local same-version asset index" if args.asset_index else "Pinned official asset index",
                "republishedIndexNote": "The local launcher may retain newer translations under the same index ID; every referenced object is checked by its own SHA-1 and size.",
                "indexEntries": len(index), "uniqueObjects": len(objects),
                "objectBytes": sum(info["size"] for info in objects.values()),
                "expandedBytes": sum(info["size"] for info in index.values()),
                "runtimeAssetRoot": "assets", "objectStore": "assets/objects", "indexPath": f"assets/indexes/{official['id']}.json",
                "allReferencesVerified": not missing, "missingCount": len(missing)}
    report = {**portable, "createdUtc": datetime.now(timezone.utc).isoformat(), "assetIndexSource": str(index_path.resolve()),
              "sourceAssets": [str(p) for p in sources], "preparedSource": str(args.prepared), "output": str(args.output),
              "downloadEnabled": args.download, "auditOnly": args.audit_only, "missing": missing}
    write_json(args.report, report)
    if missing:
        print(json.dumps({"passed": False, "report": str(args.report), "missingCount": len(missing)}), flush=True)
        return 1
    if args.audit_only:
        print(json.dumps({"passed": True, "report": str(args.report), "auditOnly": True, **portable}), flush=True)
        return 0
    if args.output.exists():
        parser.error("Output already exists; select a new staging directory")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix=args.output.name + ".preparing-", dir=args.output.parent))
    # Leave an incomplete staging folder for diagnosis on failure; never replace
    # an existing release or input folder with an incomplete tree.
    for directory, dirs, files in os.walk(args.prepared, followlinks=False):
        base = Path(directory)
        for entry in dirs:
            no_links(base / entry)
        relative = base.relative_to(args.prepared)
        for filename in files:
            source = base / filename
            no_links(source)
            if filename in {".horizon-prepare.lock", "horizon-data.bin", "horizon-sprites.cache"} or (relative == Path(".") and source.suffix == ".log"):
                continue
            target = temporary / relative / filename
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
    for key, source in found.items():
        target = temporary / "assets/objects" / key[:2] / key
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        if target.stat().st_size != objects[key]["size"] or checksum(target) != key:
            raise ValueError(f"Staged object differs from its index: {key}")
    for name, info in index.items():
        target = temporary / "assets" / safe_name(name)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(temporary / "assets/objects" / info["hash"][:2] / info["hash"], target)
        if target.stat().st_size != info["size"] or checksum(target) != info["hash"]:
            raise ValueError(f"Expanded runtime asset differs from its index: {name}")
    (temporary / "asset-index.json").write_bytes(raw_index)
    index_target = temporary / "assets/indexes" / (official["id"] + ".json")
    index_target.parent.mkdir(parents=True, exist_ok=True)
    index_target.write_bytes(raw_index)
    write_json(temporary / "portable-assets.json", portable)
    os.replace(temporary, args.output)
    report.update(passed=True, staged=True)
    write_json(args.report, report)
    print(json.dumps({"passed": True, "output": str(args.output), "report": str(args.report), **portable}), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
