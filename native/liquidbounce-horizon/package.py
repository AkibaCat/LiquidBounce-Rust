#!/usr/bin/env python3
"""Build a source-only .hnmod for the recovered 9f42064 Horizon launcher."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import tempfile
import tomllib
import zipfile

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parent.parent
HORIZON_REVISION = "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0"
HORIZON_ARCHIVE_SHA256 = "4a9798eaf7b5dce1f711307efe638f4f9eed8eafc3d58cb0415ef63a38f1c95e"
MIXINS_REVISION = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
LIQUIDBOUNCE_REVISION = "ae0eb396c07f22383789b28d5912d714c6ca0a86"
MOD_ID = "liquidbounce_horizon"
VERSION = "0.1.0"
PREFIX = f"mixins/{MOD_ID}/"
MAX_BYTES = 64 * 1024 * 1024


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_baseline() -> dict:
    archive = PROJECT / "research/Horizon-source.zip"
    digest = sha256(archive)
    require(digest == HORIZON_ARCHIVE_SHA256, "Horizon source archive hash differs from the verified baseline")
    with zipfile.ZipFile(archive) as source:
        revision = source.comment.decode("ascii").strip()
        require(re.fullmatch(r"[0-9a-f]{40}", revision) is not None, "Source archive ZIP comment is not a full Git SHA")
        require(revision == HORIZON_REVISION, "Source archive revision differs from the target")
        roots = {PurePosixPath(item.filename).parts[0] for item in source.infolist()}
        require(len(roots) == 1, "Unexpected source archive root layout")
        root = next(iter(roots))
        expected = {
            "crates/horizon-client/src/game/native_hooks.rs": "NATIVE_HOOKS_API: u32 = 415;",
            "crates/horizon-core/src/version.rs": 'VANILLA_VERSION: &str = "26.3";',
            "launcher/src/native_mods.rs": f'MIXINS_REV: &str = "{MIXINS_REVISION}";',
        }
        for relative, marker in expected.items():
            archived = source.read(f"{root}/{relative}")
            require(marker in archived.decode("utf-8"), f"Missing baseline marker in {relative}")
            local = (PROJECT / "upstream/Horizon" / relative).read_bytes()
            require(local == archived, f"Local baseline file changed: {relative}")
    upstream = json.loads((PROJECT / "research/source-integrity.json").read_text(encoding="utf-8-sig"))
    require(upstream["commit"] == LIQUIDBOUNCE_REVISION and upstream["trackedSourceMatchesUpstream"], "LiquidBounce baseline is not verified")
    return {
        "horizon": HORIZON_REVISION,
        "horizonArchiveSha256": digest,
        "horizonHooks": 415,
        "mixins": MIXINS_REVISION,
        "liquidbounce": LIQUIDBOUNCE_REVISION,
        "game": "26.3",
        "packageKind": "source-only native trait adapter",
        "implementedGameplayModules": [],
        "originalClickGuiConnected": False,
    }


def read_package_source(folder: Path) -> dict[str, bytes]:
    files = {"Cargo.toml": (folder / "Cargo.toml").read_bytes()}
    # Source-side JSON files are compile-time inputs too (for example the
    # behavior schema consumed by include_str!/include_bytes!). Keep them in
    # the staged crate alongside Rust sources so batch builds see the same
    # package layout as the working tree.
    for source in sorted((folder / "src").rglob("*")):
        if source.suffix not in {".rs", ".json"}:
            continue
        require(not source.is_symlink(), f"Source symlink is not allowed: {source}")
        files[source.relative_to(folder).as_posix()] = source.read_bytes()
    # include_str! localization tables are real compilation inputs. Keep them
    # beside the crate when staging or recovering the source package.
    for source in sorted((folder / "resources").rglob("*.json")):
        require(not source.is_symlink(), f"Resource symlink is not allowed: {source}")
        files[source.relative_to(folder).as_posix()] = source.read_bytes()
    # Native source assets belong in the bundle's supported data directory.
    # NameProtect includes the byte-identical original username word lists.
    for source in sorted((folder / "data").rglob("*")):
        require(not source.is_symlink(), f"Data symlink is not allowed: {source}")
        if source.is_file():
            files[source.relative_to(folder).as_posix()] = source.read_bytes()
    # Path dependencies and include_bytes! observation tables are compilation
    # inputs, including their licenses and per-file provenance.
    for source in sorted((folder / "vendor").rglob("*")):
        require(not source.is_symlink(), f"Vendor symlink is not allowed: {source}")
        if source.is_file():
            files[source.relative_to(folder).as_posix()] = source.read_bytes()
    for name in ("README.md", "LICENSE"):
        if (folder / name).is_file():
            files[name] = (folder / name).read_bytes()
    require("src/lib.rs" in files, f"Missing library entry in {folder}")
    return files


def package_files(provenance: dict) -> dict[str, bytes]:
    files = {"horizon-native.json": (HERE / "horizon-native.json").read_bytes()}
    for name in ("horizon-mixin.toml", "README.md", "NOTICE.md"):
        files[PREFIX + name] = (HERE / name).read_bytes()
    files[PREFIX + "LICENSE"] = (PROJECT / "LICENSE").read_bytes()
    for label, directory in (("engine", HERE / "engine"), ("core", HERE.parent / "liquidbounce-core"), ("behaviors", HERE.parent / "liquidbounce-behaviors")):
        for relative, data in read_package_source(directory).items():
            if label == "engine" and relative == "Cargo.toml":
                # The old pure-trait launcher does not apply the game's GPU/phase
                # source patches or deploy CEF. Its bundle must remain resolvable
                # without the separate project-only browser dependencies.
                manifest = data.decode()
                manifest = re.sub(r'^webview\s*=.*\n', '', manifest, flags=re.MULTILINE)
                manifest = re.sub(r'^liquidbounce-(?:interop|webview)\s*=.*\n', '', manifest, flags=re.MULTILINE)
                manifest += '\n[lints.rust]\nunexpected_cfgs = { level = "warn", check-cfg = [\'cfg(feature, values("webview"))\'] }\n'
                data = manifest.encode()
            if label == "behaviors":
                if relative == "Cargo.toml":
                    # The pinned native bundle format accepts arbitrary crate
                    # data only directly under <crate>/data/. Keep the vendor
                    # tree intact there and resolve its path at that location.
                    data = data.replace(b'path = "vendor/java_regex"', b'path = "data/java_regex"')
                elif relative.startswith("vendor/java_regex/"):
                    relative = "data/java_regex/" + relative[len("vendor/java_regex/"):]
            files[PREFIX + label + "/" + relative] = data
    files[PREFIX + "data/source-provenance.json"] = (json.dumps(provenance, ensure_ascii=False, indent=2) + "\n").encode()
    return files


def allowed_path(name: str) -> bool:
    if name == "horizon-native.json":
        return True
    if not name.startswith(PREFIX):
        return False
    rest = name[len(PREFIX):]
    parts = rest.split("/")
    text = len(parts) <= 2 and parts[-1].startswith(("README", "LICENSE", "LICENCE", "COPYING", "NOTICE"))
    data = len(parts) >= 2 and (parts[0] == "data" or (len(parts) >= 3 and parts[1] == "data"))
    return rest.endswith((".rs", ".toml")) or text or data


def validate_package(path: Path) -> dict:
    with zipfile.ZipFile(path) as package:
        require(package.testzip() is None, "ZIP CRC verification failed")
        items = package.infolist()
        require(len(items) <= 512, "Native bundle has more than 512 entries")
        require(sum(item.file_size for item in items) <= MAX_BYTES, "Native bundle exceeds 64 MiB")
        names = [item.filename for item in items]
        require(len(names) == len(set(names)), "Duplicate ZIP path")
        for name in names:
            require(not any(part in ("", ".", "..") for part in name.split("/")) and "\\" not in name and ":" not in name, f"Invalid ZIP path: {name}")
            require(allowed_path(name), f"File is not permitted by the old native package format: {name}")
        manifest = json.loads(package.read("horizon-native.json"))
        require(manifest == {"schema": 1, "id": MOD_ID, "version": VERSION, "game": "26.3", "horizon": HORIZON_REVISION, "mixins": MIXINS_REVISION}, "Unexpected native manifest")
        mixin = tomllib.loads(package.read(PREFIX + "horizon-mixin.toml").decode())
        require(mixin["mod"] == {"id": MOD_ID, "version": VERSION}, "Mismatched mixin identity")
        require(mixin["compatibility"]["horizon_revision"] == HORIZON_REVISION, "Mismatched mixin revision")
        require(mixin["api"] == {"path": "engine"}, "Incorrect trait package path")
        require(mixin["crate"] == [{"id": "core", "path": "core"}, {"id": "behaviors", "path": "behaviors"}], "Incorrect sibling crate staging")
        engine = tomllib.loads(package.read(PREFIX + "engine/Cargo.toml").decode())
        core = tomllib.loads(package.read(PREFIX + "core/Cargo.toml").decode())
        require(engine["dependencies"]["liquidbounce-core"] == {"path": "../core"}, "Core dependency must use the launcher-supported sibling path")
        require(engine["dependencies"]["horizon-client"]["version"] == "0.1.0", "Missing game dependency for launcher rewrite")
        require("webview" not in engine["features"] and "liquidbounce-webview" not in engine["dependencies"], "CEF requires the separately patched game build; it cannot be claimed by the unpatched trait bundle")
        require(core["package"]["name"] == "liquidbounce-core" and not core.get("dependencies"), "Unexpected core package dependencies")
        require("workspace" not in core and "build" not in core["package"], "Core may not declare a separate workspace or build script")
        return {"package": str(path), "sha256": sha256(path), "entries": len(items), "uncompressedBytes": sum(item.file_size for item in items), "nativeManifest": manifest, "sourceOnly": True, "gameBuildVerified": False, "runtimeParityVerified": False}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--validate", type=Path, help="Only validate an existing .hnmod package")
    args = parser.parse_args()
    if args.validate:
        print(json.dumps(validate_package(args.validate.resolve()), indent=2))
        return
    provenance = verify_baseline()
    files = package_files(provenance)
    dist = HERE / "dist"
    dist.mkdir(exist_ok=True)
    output = dist / f"{MOD_ID}-{VERSION}.hnmod"
    # Write atomically, preserving an earlier good package if validation fails.
    with tempfile.TemporaryDirectory(prefix="pack-", dir=dist) as temporary:
        candidate = Path(temporary) / output.name
        with zipfile.ZipFile(candidate, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as package:
            for name, data in sorted(files.items()):
                item = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
                item.compress_type = zipfile.ZIP_DEFLATED
                item.create_system = 3
                item.external_attr = 0o100644 << 16
                package.writestr(item, data)
        validate_package(candidate)
        candidate.replace(output)
    report = validate_package(output)
    (dist / "package-validation.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    # This folder is directly usable by the old native-build command. Its files
    # are exactly those validated in the package; no unrelated file is removed.
    unpacked = dist / "unpacked"
    for name, data in files.items():
        destination = unpacked / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
