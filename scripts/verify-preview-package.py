#!/usr/bin/env python3
"""Verify a fresh Windows preview package, then smoke-test its own launcher."""
from __future__ import annotations

import argparse
import base64
from datetime import datetime, timezone
import hashlib
import http.cookiejar
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import time
import urllib.parse
import urllib.request

PROJECT = Path(__file__).resolve().parent.parent
MODULES = {"Zoom", "NoFOV", "FullBright", "NoBob", "NoHurtCam"}
SHA = re.compile(r"[0-9a-f]{64}")
HORIZON_COMMIT = "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0"
CEF_VERSION = "152.0.6+g708dc14+chromium-152.0.7977.83"


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def no_links(path):
    for item in (path, *path.parents):
        if item.is_symlink() or (item.exists() and getattr(item.lstat(), "st_file_attributes", 0) & 0x400):
            raise ValueError(f"Reparse points are not accepted: {item}")


def local_path(root, name):
    if not isinstance(name, str) or not name:
        raise ValueError(f"Unsafe package path: {name!r}")
    parts = name.split("/")
    if (name != PurePosixPath(name).as_posix() or "\\" in name
            or any(ord(character) < 32 or character in ':*?<>|"' for character in name)
            or any(part in ("", ".", "..") or part.endswith((" ", ".")) for part in parts)):
        raise ValueError(f"Unsafe package path: {name}")
    path = root.joinpath(*parts)
    no_links(path)
    path.resolve().relative_to(root.resolve())
    return path


def read_json(path):
    def unique_object(pairs):
        value = {}
        for key, item in pairs:
            if key in value:
                raise ValueError(f"Duplicate JSON key in {path}: {key}")
            value[key] = item
        return value

    return json.loads(path.read_text(encoding="utf-8-sig"), object_pairs_hook=unique_object)


def redact(value, secrets):
    if isinstance(value, str):
        for secret in secrets:
            if secret:
                value = value.replace(secret, "<redacted>")
        value = re.sub(r"(?i)(lb_code|lb_auth|token)=([^\s&\"']+)", r"\1=<redacted>", value)
        return re.sub(r'(?i)("(?:token|auth_code|lb_code)"\s*:\s*")[^"]*', r'\1<redacted>', value)
    if isinstance(value, list):
        return [redact(item, secrets) for item in value]
    if isinstance(value, dict):
        return {key: "<redacted>" if key.lower() in ("token", "auth_code", "lb_code", "themeurl")
                else redact(item, secrets) for key, item in value.items()}
    return value


def verify_files(package):
    manifest = read_json(local_path(package, "package-manifest.json"))
    if (type(manifest.get("formatVersion")) is not int or manifest["formatVersion"] != 1
            or manifest.get("completePort") is not False or manifest.get("cefVersion") != CEF_VERSION
            or not isinstance(manifest.get("files"), list) or not manifest["files"]):
        raise ValueError("Invalid preview manifest format, scope, CEF version, or file list")
    expected, folded = {}, set()
    for item in manifest["files"]:
        name, sha = item["path"], item["sha256"]
        path = local_path(package, name)
        if (name.casefold() in folded or name.casefold() == "package-manifest.json"
                or not isinstance(sha, str) or not SHA.fullmatch(sha)
                or type(item.get("bytes")) is not int or item["bytes"] < 0):
            raise ValueError(f"Duplicate file or invalid SHA-256: {name}")
        if not path.is_file() or path.stat().st_size != item["bytes"] or digest(path) != sha:
            raise ValueError(f"Package file size or SHA-256 mismatch: {name}")
        expected[name] = sha
        folded.add(name.casefold())
    actual = set()
    for path in package.rglob("*"):
        no_links(path)
        if path.is_file() and path != package / "package-manifest.json":
            actual.add(path.relative_to(package).as_posix())
    if actual != set(expected):
        raise ValueError(f"Manifest file set mismatch: missing={sorted(set(expected) - actual)}, extra={sorted(actual - set(expected))}")
    source = read_json(local_path(package, "source/native-source-manifest.json"))
    files = source["nativeSourceFiles"]
    if not isinstance(files, dict) or not files:
        raise ValueError("Missing compiled native source manifest")
    for required_source in ("patches/horizon-ui-render-hook.patch", "native/liquidbounce-horizon/patches/horizon-behavior-hooks.patch"):
        if required_source not in files:
            raise ValueError(f"Compiled native source manifest omits required patch: {required_source}")
    aggregate = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    if aggregate != source["nativeSourceSha256"] or aggregate != manifest["nativeSourceSha256"]:
        raise ValueError("Native source manifest aggregate mismatch")
    folded = set()
    for name, sha in files.items():
        path = local_path(package / "source", name)
        if (name.casefold() in folded or not isinstance(sha, str) or not SHA.fullmatch(sha)
                or expected.get("source/" + name) != sha):
            raise ValueError(f"Compiled native source differs from package manifest: {name}")
        folded.add(name.casefold())
        if digest(path) != sha:
            raise ValueError(f"Compiled native source differs on disk: {name}")
    build = read_json(local_path(package, "metadata/build-summary.json"))
    def five_modules(value):
        return isinstance(value, list) and len(value) == len(MODULES) and all(isinstance(item, str) for item in value) and set(value) == MODULES

    if (build.get("operation") != "build" or build.get("passed") is not True
            or type(build.get("exitCode")) is not int or build["exitCode"] != 0
            or build.get("horizon") != HORIZON_COMMIT or build.get("originalThemeLinked") is not True
            or build.get("nativeSourceSha256") != aggregate or build.get("nativeSourceFiles") != files
            or build.get("executableSha256") != expected.get("runtime/liquidbounce-native-game.exe")
            or build.get("lockSha256") != expected.get("source/rebuild-locks/horizon-Cargo.lock")
            or build.get("gpuUiPatchSha256") != files.get("patches/horizon-ui-render-hook.patch")
            or build.get("behaviorHookPatchSha256") != files.get("native/liquidbounce-horizon/patches/horizon-behavior-hooks.patch")
            or not five_modules(build.get("implementedModules")) or not five_modules(manifest.get("implementedModules"))):
        raise ValueError("Build summary, compiled sources, executable, or five-module declaration disagree")
    for required in ("Start-LiquidBounce.ps1", "runtime/liquidbounce-native-game.exe",
                     "runtime/liquidbounce-cef-helper.exe", "theme/index.html",
                     "source/rebuild-locks/horizon-Cargo.lock",
                     "source/patches/horizon-ui-render-hook.patch",
                     "source/native/liquidbounce-horizon/patches/horizon-behavior-hooks.patch"):
        if required not in expected:
            raise ValueError(f"Required package entry is not covered: {required}")
    if "metadata/validation-summary.json" in expected:
        validation = read_json(local_path(package, "metadata/validation-summary.json"))
        checks = validation.get("checks")
        if (validation.get("passed") is not True or validation.get("executableSha256") != build["executableSha256"]
                or validation.get("nativeSourceSha256") != aggregate or not five_modules(validation.get("implementedModules"))
                or not isinstance(checks, dict) or not checks or any(value is not True for value in checks.values())):
            raise ValueError("Recorded native integration validation did not pass for this package")
    return {"files": len(expected), "nativeSourceFiles": len(files), "nativeSourceSha256": aggregate,
            "gameSha256": expected["runtime/liquidbounce-native-game.exe"], "manifestSha256": digest(package / "package-manifest.json")}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--controller", type=Path, required=True)
    parser.add_argument("--name", default="preview-" + datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S-%f"))
    parser.add_argument("--timeout", type=int, default=300, help="Startup deadline in seconds")
    args = parser.parse_args()
    if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_-]{0,63}", args.name) or not 30 <= args.timeout <= 900:
        parser.error("Use a simple --name and --timeout between 30 and 900 seconds")
    package = args.package.absolute()
    output = PROJECT / "build/preview-smoke" / args.name
    no_links(package)
    no_links(output)
    package = package.resolve()
    output = output.resolve()
    if output.is_relative_to(package) or package.is_relative_to(output):
        parser.error("The package and smoke report directory must not overlap")
    if output.exists():
        parser.error("Choose a fresh --name; existing reports/logs are preserved")
    output.mkdir(parents=True)
    report_path, log_dir = output / "report.json", output / "logs"
    log_dir.mkdir()
    report = {"passed": False, "structuredControl": True, "screenRecognitionUsed": False,
              "offscreen": True, "checks": {}, "records": [], "logs": [],
              "startedUtc": datetime.now(timezone.utc).isoformat(), "reportDirectory": str(output)}
    secrets, failures = [], []
    data = package / "user-data"
    socket = data / "horizon-control.json"
    process, streams, created_data, opened_gui = None, [], False, False
    known_pids = set()
    shell = shutil.which("pwsh")
    quiet = getattr(subprocess, "CREATE_NO_WINDOW", 0)

    def check(name, condition, observed=None):
        report["checks"][name] = {"passed": bool(condition), "observed": observed}
        if not condition:
            raise AssertionError(f"{name}: {observed!r}")

    def control(*command, record=True):
        completed = subprocess.run([str(args.controller.resolve()), "--socket", str(socket), "--timeout", "5", *command],
                                   capture_output=True, text=True, encoding="utf-8", errors="replace",
                                   timeout=10, creationflags=quiet)
        if completed.returncode:
            raise RuntimeError(f"Official control {command!r}: {completed.returncode}: {completed.stderr or completed.stdout}")
        value = json.loads(completed.stdout)
        if isinstance(value, dict) and value.get("error"):
            raise RuntimeError(f"Official control {command!r}: {value['error']}")
        if record:
            report["records"].append({"control": list(command), "response": value})
        return value

    def processes():
        code = ("$ErrorActionPreference = 'Stop'; "
                "@(Get-CimInstance Win32_Process -Filter \"Name = 'liquidbounce-native-game.exe' OR Name = 'liquidbounce-cef-helper.exe'\" | "
                "Select-Object Name,ProcessId,ParentProcessId,ExecutablePath,CommandLine) | ConvertTo-Json -Depth 3 -Compress")
        result = subprocess.run([shell, "-NoProfile", "-NonInteractive", "-EncodedCommand",
                                 base64.b64encode(code.encode("utf-16-le")).decode()],
                                capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=20, creationflags=quiet)
        if result.returncode:
            raise RuntimeError("Could not inspect preview processes: " + result.stderr)
        values = json.loads(result.stdout.strip() or "[]")
        values = values if isinstance(values, list) else [values]
        root = os.path.normcase(str(package.resolve())) + os.sep
        selected = {}
        # Keep descendants once observed, even if CIM cannot later read a
        # shutting-down process's executable path. No other instance is touched.
        while True:
            before = len(selected)
            for item in values:
                if not item:
                    continue
                executable = item.get("ExecutablePath")
                if ((executable and os.path.normcase(os.path.abspath(executable)).startswith(root))
                        or item["ProcessId"] in known_pids or item["ParentProcessId"] in known_pids):
                    selected[item["ProcessId"]] = item
                    known_pids.add(item["ProcessId"])
            if len(selected) == before:
                return list(selected.values())

    def remember_secrets():
        # Startup and teardown can race an atomic metadata update. An
        # incomplete diagnostic file must never prevent the official quit.
        for path in (socket, data / "liquidbounce/interop.json"):
            try:
                value = read_json(path)
                token, url = value.get("token"), value.get("themeUrl")
                if isinstance(token, str) and token:
                    secrets.append(token)
                if isinstance(url, str) and url:
                    secrets.append(url)
                    secrets.extend(urllib.parse.parse_qs(urllib.parse.urlsplit(url).query).get("lb_code", []))
            except (OSError, ValueError, TypeError, AttributeError):
                continue

    try:
        if os.name != "nt" or not shell:
            raise RuntimeError("Windows and PowerShell 7 (pwsh) are required")
        check("packageDirectoryExists", package.is_dir(), str(package))
        no_links(data)
        check("freshIndependentUserData", not data.exists(), str(data))
        check("explicitPreparedResources", args.resources.is_dir() and (args.resources / "horizon-resources.json").is_file(), str(args.resources.resolve()))
        check("officialControllerExists", args.controller.is_file(), str(args.controller.resolve()))
        report["package"] = str(package)
        report["controller"] = {"path": str(args.controller.resolve()), "sha256": digest(args.controller.resolve())}
        report["integrity"] = verify_files(package)
        check("packageAndNativeSourceHashes", True, report["integrity"])
        check("noPackageProcessesBeforeLaunch", not processes())
        # Launcher paths are package-owned. Remove inherited development or
        # another instance's runtime/config/socket environment before it runs.
        environment = {name: value for name, value in os.environ.items()
                       if not name.upper().startswith(("HORIZON_", "LIQUIDBOUNCE_"))
                       and name.upper() not in ("XDG_DATA_HOME", "XDG_CONFIG_HOME")}
        report["clearedEnvironmentNames"] = sorted(set(os.environ) - set(environment))
        world = "PreviewSmoke" + datetime.now(timezone.utc).strftime("%Y%m%d%H%M%S")
        command = [shell, "-NoProfile", "-NonInteractive", "-File", str(package / "Start-LiquidBounce.ps1"),
                   "-Resources", str(args.resources.resolve()), "-World", world, "-NoAudio", "-Offscreen",
                   "-Width", "960", "-Height", "540"]
        report["launcherCommand"] = command
        for name in ("launcher.stdout.log", "launcher.stderr.log"):
            stream = (log_dir / name).open("wb")
            streams.append(stream)
        # Reserve the originally absent data directory exclusively, so it is
        # unambiguously this run's data when it is archived after shutdown.
        no_links(data)
        data.mkdir()
        created_data = True
        process = subprocess.Popen(command, cwd=package, env=environment, stdin=subprocess.DEVNULL,
                                   stdout=streams[0], stderr=streams[1], creationflags=quiet)
        known_pids.add(process.pid)
        report["launcherPid"] = process.pid
        deadline, last_error = time.monotonic() + args.timeout, None
        while True:
            if process.poll() is not None:
                raise RuntimeError(f"Package launcher exited before ready: {process.returncode}")
            try:
                state = control("state", record=False)
                if state.get("world") == world and state.get("screen") == "playing" and state.get("player"):
                    break
            except Exception as error:
                last_error = str(error)
            if time.monotonic() >= deadline:
                raise TimeoutError("Package startup timed out: " + str(last_error))
            time.sleep(0.25)
        remember_secrets()
        report["state"] = state
        check("ownWorldPlaying", state["world"] == world and state["screen"] == "playing", state)
        running = processes()
        expected_game = os.path.normcase(str(package / "runtime/liquidbounce-native-game.exe"))
        games = [item for item in running if item.get("ExecutablePath") and os.path.normcase(os.path.abspath(item["ExecutablePath"])) == expected_game]
        check("runningGameIsPackageRuntime", len(games) == 1 and games[0]["ParentProcessId"] == process.pid
              and str(socket).lower() in (games[0].get("CommandLine") or "").lower(), running)
        info = read_json(data / "liquidbounce/interop.json")
        parsed = urllib.parse.urlsplit(info["themeUrl"])
        if parsed.scheme != "http" or parsed.hostname != "127.0.0.1":
            raise ValueError("Preview must expose only its own localhost interop")
        origin = f"{parsed.scheme}://{parsed.netloc}"
        browser = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
        with browser.open(info["themeUrl"], timeout=10) as response:
            check("packageThemeServed", response.status == 200, response.status)

        def api(path, record=True):
            with browser.open(origin + "/api/v1" + path, timeout=10) as response:
                value = json.load(response)
            if record:
                report["records"].append({"api": path, "response": value})
            return value

        modules = api("/client/modules")
        check("actualFiveModuleRegistry", len(modules) == 5 and {item["name"] for item in modules} == MODULES, modules)
        control("key", "key.keyboard.right.shift", "tap")
        opened_gui = True
        deadline = time.monotonic() + 30
        last_error = None
        while True:
            try:
                status = api("/client/integration-status", record=False)
                visible = status.get("browser", {})
                page = visible.get("page") or {}
                if (visible.get("visible") and visible.get("paintCount", 0) > 0 and page.get("completedLoads", 0) > 0
                        and not page.get("loading", True)):
                    break
            except Exception as error:
                last_error = str(error)
            if time.monotonic() >= deadline:
                raise TimeoutError("Packaged ClickGUI did not paint a completed document: " + str(last_error))
            time.sleep(0.1)
        control("wait", "4")
        status = api("/client/integration-status")
        page = status["browser"]["page"]
        errors = [line for line in page.get("console", [])
                  if re.search(r"\b(error|fatal|typeerror|referenceerror|syntaxerror|uncaught)\b", line, re.IGNORECASE)]
        check("packagedClickGuiRendered", control("state").get("screen") == "mod"
              and status["browser"]["visible"] and status["browser"]["paintCount"] > 0
              and page["completedLoads"] > 0 and page["httpStatus"] == 200
              and page.get("title") == "LiquidBounce UI" and not page.get("loading")
              and not page.get("loadErrors") and not errors, status["browser"])
        screenshot = log_dir / "packaged-clickgui.png"
        control("screenshot", str(screenshot))
        check("packageScreenshotSaved", screenshot.is_file() and screenshot.stat().st_size > 0, str(screenshot))
        report["smokePassed"] = True
    except Exception as error:
        report["error"] = str(error)
    finally:
        if process is not None:
            remember_secrets()
            commands = [("release",)]
            if opened_gui:
                commands.append(("key", "esc", "tap"))
            commands.append(("quit",))
            for command in commands:
                try:
                    control(*command)
                    if command == ("quit",):
                        check("officialQuitSent", True)
                except Exception as error:
                    failures.append(f"Official {command[0]}: {error}")
                    if command == ("quit",):
                        report["checks"]["officialQuitSent"] = {"passed": False, "observed": str(error)}
            try:
                code = process.wait(timeout=45)
                report["launcherExitCode"] = code
                check("launcherExitedCleanly", code == 0, code)
            except Exception as error:
                failures.append("Launcher exit: " + str(error))
            try:
                deadline = time.monotonic() + 10
                remaining = processes()
                while remaining and time.monotonic() < deadline:
                    time.sleep(0.25)
                    remaining = processes()
                check("noPackageGameOrHelperRemains", not remaining, remaining)
            except Exception as error:
                failures.append("Package process cleanup: " + str(error))
        for stream in streams:
            stream.close()
        try:
            remember_secrets()
            if created_data and data.exists():
                for index, path in enumerate(sorted(data.rglob("*.log"))):
                    no_links(path)
                    target = log_dir / f"game-{index:02d}-{path.name}"
                    target.write_text(redact(path.read_text(encoding="utf-8", errors="replace"), secrets), encoding="utf-8")
                # Move only the directory reserved by this run, after a clean
                # launcher exit and absence of all package game/helper PIDs.
                can_archive = (process is None and not processes()) or (
                    report["checks"].get("launcherExitedCleanly", {}).get("passed")
                    and report["checks"].get("noPackageGameOrHelperRemains", {}).get("passed"))
                if can_archive:
                    destination = output / "user-data"
                    no_links(data)
                    no_links(destination)
                    expected_data = package.resolve() / "user-data"
                    if (data.absolute() != expected_data or data.resolve() != expected_data
                            or data.resolve().parent != package.resolve() or data.name != "user-data"
                            or destination.exists() or destination.resolve().parent != output.resolve()):
                        raise ValueError("Unexpected smoke user-data archive path or existing destination")
                    for path in data.rglob("*"):
                        no_links(path)
                    # Expired smoke credentials have no diagnostic value.
                    for path in [*data.rglob("*.log"), socket, data / "liquidbounce/interop.json"]:
                        if path.is_file():
                            path.write_text(redact(path.read_text(encoding="utf-8", errors="replace"), secrets), encoding="utf-8")
                    shutil.move(str(data), str(destination))
                    report["archivedUserData"] = str(destination)
                    check("testUserDataArchived", not data.exists() and destination.is_dir(), str(destination))
                else:
                    raise RuntimeError("Generated user-data is preserved in the package because clean shutdown was not confirmed")
        except Exception as error:
            failures.append("Log preservation/user-data archive: " + str(error))
        try:
            for path in log_dir.glob("*.log"):
                path.write_text(redact(path.read_text(encoding="utf-8", errors="replace"), secrets), encoding="utf-8")
            report["logs"] = [str(path) for path in sorted(log_dir.iterdir())]
        except Exception as error:
            failures.append("Log redaction: " + str(error))
        if report.get("integrity"):
            try:
                after = verify_files(package)
                check("packageUnchangedAfterSmoke", after == report["integrity"], after)
                report["finalIntegrity"] = after
            except Exception as error:
                report["checks"]["packageUnchangedAfterSmoke"] = {"passed": False, "observed": str(error)}
                failures.append("Final package integrity: " + str(error))
        if failures:
            report["cleanupErrors"] = failures
        report["finishedUtc"] = datetime.now(timezone.utc).isoformat()
        report["passed"] = (bool(report.get("smokePassed")) and not failures
                            and all(item["passed"] for item in report["checks"].values()))
        report_path.write_text(json.dumps(redact(report, secrets), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"report": str(report_path), "passed": report["passed"]}, ensure_ascii=False))
    if not report["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
