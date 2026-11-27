#!/usr/bin/env python3
"""Prepare isolated Horizon resources from the user's existing 26.3 Gradle cache."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import runpy
import shutil
import subprocess
import urllib.request

PROJECT = Path(__file__).resolve().parent.parent


def sha1(path: Path) -> str:
    digest = hashlib.sha1()
    with path.open("rb") as source:
        while chunk := source.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache", type=Path, default=Path.home() / ".gradle/caches")
    parser.add_argument("--release", type=Path, default=Path("D:/Develop/MCL/.references/horizon/nightly/horizon-windows-x64"))
    parser.add_argument("--java-home", type=Path, default=Path("D:/Develop/MCMOD/sulfur-shulker-client/.tools/java/jdk-25.0.5.1"))
    parser.add_argument("--version-json", type=Path, default=PROJECT / "build/metadata/26.3.json")
    parser.add_argument("--output", type=Path, default=PROJECT / "build/resources/26.3")
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("--offline", action="store_true", help="Do not fetch missing Java dependencies or game assets")
    args = parser.parse_args()
    runtime = PROJECT / "build/resource-runtime"
    output = args.output.resolve()
    manifest = args.version_json
    version = json.loads(manifest.read_text(encoding="utf-8"))
    if version["id"] != "26.3":
        raise ValueError("Existing game cache is not Minecraft 26.3")
    baseline = json.loads((PROJECT / "upstream/Horizon/research-lock.json").read_text())
    pinned = baseline["minecraft"]
    release_pin = json.loads((args.release / "research-lock.json").read_text())["minecraft"]
    if release_pin != pinned:
        raise ValueError("Resource preparer is not the pinned Horizon release")
    accepted = [pinned["manifest_sha1"], *pinned["accepted_manifest_sha1"]]
    if sha1(manifest) not in accepted:
        raise ValueError("Version JSON is not one of Horizon's pinned 26.3 manifests")
    client = args.cache / "fabric-loom/26.3/minecraft-client.jar"
    server = args.cache / "fabric-loom/26.3/minecraft-server.jar"
    for path, expected in ((client, pinned["client_sha1"]), (server, pinned["server_sha1"])):
        if sha1(path) != expected:
            raise ValueError(f"Official game JAR hash mismatch: {path}")
    if not (args.java_home / "bin/java.exe").is_file():
        raise ValueError("Java 25 runtime is missing")
    runtime.mkdir(parents=True, exist_ok=True)
    shutil.copy2(args.release / "research-lock.json", runtime / "research-lock.json")
    for folder in ("exporters", "tools", "mods"):
        shutil.copytree(args.release / folder, runtime / folder, dirs_exist_ok=True)
    libraries = runtime / "libraries"
    found, missing, downloaded = [], [], []
    for library in version["libraries"]:
        rules = library.get("rules")
        if rules:
            allowed = False
            for rule in rules:
                system = rule.get("os", {})
                if system.get("name", "windows") == "windows":
                    allowed = rule["action"] == "allow"
            if not allowed:
                continue
        artifact = library.get("downloads", {}).get("artifact")
        if artifact is None:
            continue
        parts = PurePosixPath(artifact["path"]).parts
        if any(part in ("..", ".") for part in parts):
            raise ValueError("Invalid library path")
        group, name, number, *_ = library["name"].split(":")
        candidates = (args.cache / "modules-2/files-2.1" / group / name / number).glob("*/*.jar")
        candidate = next((path for path in candidates if sha1(path) == artifact["sha1"]), None)
        destination = libraries.joinpath(*parts)
        if candidate is None and destination.is_file() and sha1(destination) == artifact["sha1"]:
            candidate = destination
        if candidate is None:
            if args.offline:
                missing.append(library["name"])
                continue
            data = urllib.request.urlopen(artifact["url"], timeout=30).read()
            if hashlib.sha1(data).hexdigest() != artifact["sha1"]:
                raise ValueError("Downloaded Java dependency hash mismatch")
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
            candidate = destination
            downloaded.append(library["name"])
        destination.parent.mkdir(parents=True, exist_ok=True)
        if not destination.is_file() or sha1(destination) != artifact["sha1"]:
            shutil.copy2(candidate, destination)
        found.append(library["name"])
    if missing:
        raise ValueError("Required exporter libraries missing from local cache: " + ", ".join(missing))
    command = [str(args.release / "horizon-prepare.exe"), "--check" if args.check_only else "--prepare",
               "--root", str(runtime), "--output", str(output)]
    if not args.check_only:
        command += ["--version-json", str(manifest), "--client-jar", str(client), "--server-jar", str(server),
                    "--libraries-dir", str(libraries), "--assets-dir", str(args.cache / "fabric-loom/assets"),
                    "--java-home", str(args.java_home), "--progress", "json"]
    env = os.environ.copy()
    env["HORIZON_HOME"] = str(PROJECT / "build/resource-home")
    env["HORIZON_NO_AUDIO"] = "1"
    # All Java scratch files and prepared game data stay under ignored build/.
    log = PROJECT / "build/resource-preparation.log"
    with log.open("w", encoding="utf-8") as stream:
        result = subprocess.run(command, cwd=runtime, env=env, stdout=stream, stderr=subprocess.STDOUT)
    exit_code = result.returncode
    chinese_resources = None
    chinese_error = None
    if exit_code == 0 or args.check_only:
        prepare_chinese = runpy.run_path(str(PROJECT / "scripts/prepare-chinese-resources.py"))["prepare_chinese_resources"]
        try:
            chinese_resources = prepare_chinese(args.cache, output, manifest, offline=args.offline, check_only=args.check_only)
            (PROJECT / "build/chinese-game-resources.json").write_text(json.dumps(chinese_resources, indent=2) + "\n", encoding="utf-8")
        except (OSError, ValueError, KeyError) as error:
            exit_code = 1
            chinese_error = str(error)
    report = {"exitCode": exit_code, "resources": str(output), "log": str(log),
              "gameVersion": version["id"], "verifiedLocalLibraries": len(found),
              "downloadedJavaDependencies": downloaded, "soundsImported": False, "gameStarted": False,
              "chineseResources": chinese_resources, "chineseResourceError": chinese_error}
    (PROJECT / "build/resource-preparation.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    raise SystemExit(exit_code)


if __name__ == "__main__":
    main()
