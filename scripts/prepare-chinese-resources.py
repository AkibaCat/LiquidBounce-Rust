#!/usr/bin/env python3
"""Import or verify the pinned game's Chinese languages and Unifont assets."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import urllib.request

PROJECT = Path(__file__).resolve().parent.parent
RAW_CACHE = PROJECT / "build/official-asset-cache"
WANTED = (
    "pack.mcmeta",
    "minecraft/lang/zh_cn.json",
    "minecraft/lang/zh_tw.json",
    "minecraft/font/include/unifont.json",
    "minecraft/font/unifont.zip",
    "minecraft/font/unifont_jp.zip",
    "minecraft/font/unifont_pua.zip",
)


def verified(data: bytes, checksum: str, size: int) -> bool:
    return len(data) == size and hashlib.sha1(data).hexdigest() == checksum


def read_verified(paths: list[Path], checksum: str, size: int) -> bytes | None:
    for path in paths:
        if path.is_file():
            data = path.read_bytes()
            if verified(data, checksum, size):
                return data
    return None


def write_if_changed(path: Path, data: bytes) -> None:
    if path.is_file() and path.read_bytes() == data:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_bytes(data)
    temporary.replace(path)


def fetch_verified(url: str, checksum: str, size: int, label: str) -> bytes:
    with urllib.request.urlopen(url, timeout=30) as response:
        data = response.read()
    if not verified(data, checksum, size):
        raise ValueError(f"Official asset checksum or size mismatch: {label}")
    return data


def prepare_chinese_resources(
    cache: Path,
    output: Path,
    version_json: Path,
    *,
    offline: bool = False,
    check_only: bool = False,
) -> dict:
    version_bytes = version_json.read_bytes()
    version_sha1 = hashlib.sha1(version_bytes).hexdigest()
    pin = json.loads((PROJECT / "upstream/Horizon/research-lock.json").read_text(encoding="utf-8"))["minecraft"]
    accepted = [pin["manifest_sha1"], *pin.get("accepted_manifest_sha1", [])]
    if version_sha1 not in accepted:
        raise ValueError("Version metadata is not a pinned 26.3 manifest")
    version = json.loads(version_bytes)
    if version["id"] != pin["version"] or version["id"] != "26.3":
        raise ValueError("Version metadata is not Minecraft 26.3")
    asset = version["assetIndex"]
    index_checksum, index_size = asset["sha1"], asset["size"]
    raw_index = RAW_CACHE / "indexes" / (index_checksum + ".json")
    gradle_assets = cache / "fabric-loom/assets"
    index_bytes = read_verified(
        [raw_index, gradle_assets / "indexes" / f"26.3-{asset['id']}.json", gradle_assets / "indexes" / f"{asset['id']}.json"],
        index_checksum, index_size,
    )
    downloads = []
    if index_bytes is None:
        if offline or check_only:
            raise ValueError(f"Verified raw official asset index missing from local cache: {index_checksum}")
        index_bytes = fetch_verified(asset["url"], index_checksum, index_size, "asset index")
        downloads.append("assetIndex")
    if not check_only:
        # Gradle may serialize an index in a different order. Keep the exact
        # official bytes by content hash without modifying its shared cache.
        write_if_changed(raw_index, index_bytes)
    index = json.loads(index_bytes)["objects"]
    assets_output = output.resolve() / "assets"
    files = {}
    for name in WANTED:
        record = index[name]
        checksum, size = record["hash"], record["size"]
        target = assets_output / name
        raw_object = RAW_CACHE / "objects" / checksum[:2] / checksum
        if check_only:
            data = read_verified([target], checksum, size)
            if data is None:
                raise ValueError(f"Prepared Chinese resource missing or checksum/size mismatch: {target}")
        else:
            data = read_verified([raw_object, gradle_assets / "objects" / checksum[:2] / checksum, target], checksum, size)
            if data is None:
                if offline:
                    raise ValueError(f"Chinese resource missing from local cache: {name}")
                data = fetch_verified(f"https://resources.download.minecraft.net/{checksum[:2]}/{checksum}", checksum, size, name)
                downloads.append(name)
            write_if_changed(raw_object, data)
            write_if_changed(target, data)
        files[name] = {"sha1": checksum, "size": len(data), "sha256": hashlib.sha256(data).hexdigest()}
    return {
        "passed": True, "checkOnly": check_only, "offline": offline, "gameVersion": version["id"],
        "manifestSha1": version_sha1, "manifestSize": len(version_bytes),
        "assetIndexSha1": index_checksum, "assetIndexSize": index_size,
        "rawAssetIndex": str(raw_index), "resources": str(assets_output), "files": files,
        "downloaded": downloads, "redistribution": "official game data prepared locally; raw cache stays in ignored build/",
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache", type=Path, default=Path.home() / ".gradle/caches", help="Gradle cache root")
    parser.add_argument("--output", type=Path, default=PROJECT / "build/resources/26.3", help="Prepared resources root")
    parser.add_argument("--version-json", type=Path, default=PROJECT / "build/metadata/26.3.json")
    parser.add_argument("--offline", action="store_true", help="Use only verified local files")
    parser.add_argument("--check-only", action="store_true", help="Verify all seven prepared files without downloading or repairing")
    args = parser.parse_args()
    report = prepare_chinese_resources(args.cache, args.output, args.version_json, offline=args.offline, check_only=args.check_only)
    report_path = PROJECT / "build/chinese-game-resources.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
