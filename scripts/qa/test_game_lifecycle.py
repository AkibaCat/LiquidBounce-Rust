# SPDX-License-Identifier: GPL-3.0-or-later
"""Isolated, offscreen game lifecycle smoke checks tied to one executable hash.

Only starts its own new game/home directories. Control uses the game's native
localhost API; no desktop input, launcher accounts, or existing saves are used.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import time


SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT = SCRIPT_DIR.parents[1]
QA = PROJECT / "build/launcher-qa"
EXIT_LABELS = {
    "Save and Quit to Title", "Save and quit to title", "Save & Quit to Title",
    "保存并退出到标题屏幕", "保存并退回到标题画面", "保存并退出到标题画面",
}
FOLDERS = {
    "HORIZON_HOME": "home", "HORIZON_LAUNCHER_HOME": "launcher",
    "XDG_DATA_HOME": "data", "XDG_CONFIG_HOME": "config",
    "HORIZON_LAUNCHER_TRASH": "trash", "HORIZON_LAUNCHER_CLIPBOARD": "clipboard",
    "HORIZON_LAUNCHER_DESKTOP": "desktop", "LIQUIDBOUNCE_CONFIG_DIR": "liquidbounce",
    "LIQUIDBOUNCE_CACHE_DIR": "browser-cache",
}


def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def title_ready(state):
    return state.get("screen") == "title" and state.get("world") is None and state.get("player") is None


def world_ready(state, world):
    return state.get("screen") == "playing" and state.get("world") == world and isinstance(state.get("player"), dict)


def exit_label(screen, requested=None):
    candidates = [b["label"] for b in screen.get("buttons", [])
                  if b.get("enabled") and isinstance(b.get("label"), str)
                  and b["label"] in ({requested} if requested else EXIT_LABELS)]
    if len(candidates) != 1:
        raise RuntimeError("Expected one enabled save-and-quit button; observed " + json.dumps(screen, ensure_ascii=False))
    return candidates[0]


class Case:
    def __init__(self, args, root, mode):
        self.args, self.mode = args, mode
        self.path = root / mode
        self.path.mkdir()
        self.child = None
        self.started = time.monotonic()
        self.trace = (self.path / "control-trace.jsonl").open("w", encoding="utf-8")
        self.result = {"mode": mode, "passed": False, "offscreen": True,
                       "executableSha256": args.expected_sha256, "checks": {}}

    def event(self, **data):
        data["elapsedSeconds"] = round(time.monotonic() - self.started, 3)
        self.trace.write(json.dumps(data, ensure_ascii=False) + "\n")
        self.trace.flush()

    def alive(self):
        if self.child.poll() is not None:
            raise RuntimeError(f"Game exited before requested lifecycle step: {self.child.returncode}")

    @staticmethod
    def read_line(stream):
        line = stream.readline(2 * 1024 * 1024 + 1)
        if not line or not line.endswith(b"\n") or len(line) > 2 * 1024 * 1024:
            raise RuntimeError("Missing/oversized control reply")
        result = json.loads(line)
        if not isinstance(result, dict):
            raise RuntimeError("Control reply is not an object")
        return result

    def command(self, line, deadline=None):
        self.alive()
        endpoint = json.loads((self.path / "control.json").read_text(encoding="utf-8"))
        port, token = endpoint.get("port"), endpoint.get("token")
        if endpoint.get("transport") != "tcp" or not isinstance(port, int) or not 0 < port < 65536 or not isinstance(token, str) or not re.fullmatch(r"[0-9a-fA-F]{32}", token):
            raise RuntimeError("Invalid private TCP endpoint")
        timeout = self.args.command_timeout
        if deadline is not None:
            timeout = min(timeout, max(0.05, deadline - time.monotonic()))
        with socket.create_connection(("127.0.0.1", port), timeout=timeout) as sock:
            sock.settimeout(timeout)
            with sock.makefile("rb") as stream:
                sock.sendall(("auth " + token + "\n").encode("ascii"))
                if self.read_line(stream).get("ok") is not True:
                    raise RuntimeError("Control authentication rejected")
                sock.sendall((line + "\n").encode("utf-8"))
                result = self.read_line(stream)
        # The private endpoint/token never enters the evidence stream.
        self.event(command=line, reply=result)
        return result

    def must_command(self, line):
        result = self.command(line)
        if "error" in result:
            raise RuntimeError(f"{line}: {result['error']}")
        return result

    def wait_reply(self, command, predicate, label, seconds=None):
        deadline = time.monotonic() + (seconds or self.args.timeout)
        last = None
        while time.monotonic() < deadline:
            self.alive()
            try:
                last = self.command(command, deadline)
                if predicate(last):
                    return last
            except (OSError, ValueError) as error:
                last = {"connectionError": str(error)}
                self.event(command=command, retry=last)
            time.sleep(0.2)
        raise TimeoutError(f"Timed out waiting for {label}: {json.dumps(last, ensure_ascii=False)}")

    def advance(self, label, require_ticks):
        baseline = self.wait_reply("status", lambda s: isinstance(s.get("frames"), int) and isinstance(s.get("ticks"), int), label + " status")
        after = self.wait_reply("status", lambda s: isinstance(s.get("frames"), int)
                                and isinstance(s.get("ticks"), int)
                                and s["frames"] >= baseline["frames"] + 12
                                and (not require_ticks or s["ticks"] >= baseline["ticks"] + 2), label + " frames/ticks")
        self.result["checks"][label] = {"passed": True, "framesBefore": baseline["frames"],
                                       "framesAfter": after["frames"], "frameDelta": after["frames"] - baseline["frames"],
                                       "ticksBefore": baseline["ticks"], "ticksAfter": after["ticks"],
                                       "tickDelta": after["ticks"] - baseline["ticks"]}

    def capture_failure(self):
        if self.child is None or self.child.poll() is not None:
            return
        diagnostic = SCRIPT_DIR / "capture_native_stacks.py"
        if diagnostic.is_file() and sys.maxsize > 2 ** 32 and os.name == "nt":
            try:
                result = subprocess.run([sys.executable, str(diagnostic), "--pid", str(self.child.pid),
                                         "--expected-image", str(self.args.binary),
                                         "--output", str(self.path / "failure-stacks.json"),
                                         "--max-frames", "96"], capture_output=True, timeout=45)
                self.result["failureDiagnostic"] = {"exitCode": result.returncode,
                    "stdout": result.stdout.decode("utf-8", errors="replace"),
                    "stderr": result.stderr.decode("utf-8", errors="replace")}
            except Exception as error:
                self.result["failureDiagnostic"] = {"error": str(error)}

    def cleanup(self):
        if self.child is None or self.child.poll() is not None:
            return
        self.result["forcedCleanup"] = True
        # Only the exact Popen created for this case is eligible for tree cleanup.
        if os.name == "nt":
            cleanup = subprocess.run(["taskkill", "/PID", str(self.child.pid), "/T", "/F"], capture_output=True, timeout=15)
            self.result["cleanupExitCode"] = cleanup.returncode
        else:
            self.child.kill()
        try:
            self.child.wait(timeout=15)
        except subprocess.TimeoutExpired:
            self.result["cleanupError"] = "Owned game process did not terminate"

    def run(self):
        stdout = stderr = None
        try:
            current_hash = sha256(self.args.binary)
            if current_hash != self.args.expected_sha256:
                raise RuntimeError("Executable SHA-256 changed before case launch")
            for folder in set(FOLDERS.values()) | {"game", "player"}:
                (self.path / folder).mkdir()
            env = os.environ.copy()
            env.update({key: str(self.path / folder) for key, folder in FOLDERS.items()})
            env.update(HORIZON_LAUNCHER_NO_SELF_UPDATE="1", HORIZON_NO_AUDIO="1",
                       LIQUIDBOUNCE_THEME_ROOT=str(self.args.package / "theme"),
                       LIQUIDBOUNCE_CEF_RESOURCES=str(self.args.package / "runtime"))
            command = [str(self.args.binary), "--offscreen", "--resources", str(self.args.resources),
                       "--home", str(self.path / "home"), "--game-dir", str(self.path / "game"),
                       "--player-dir", str(self.path / "player"), "--control-socket", str(self.path / "control.json"),
                       "--name", "LauncherQA", "--width", "960", "--height", "540", "--render-distance", "2"]
            world = "LauncherQA"
            if self.mode != "menu-quit":
                command += ["--world", world, "--creative", "--test-terrain"]
            stdout = (self.path / "stdout.log").open("wb")
            stderr = (self.path / "stderr.log").open("wb")
            self.child = subprocess.Popen(command, cwd=self.path / "game", env=env,
                                          stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
                                          creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            self.result["pid"] = self.child.pid
            write_json(self.path / "process.json", {"pid": self.child.pid, "command": command,
                       "executableSha256": current_hash, "createdUtc": datetime.now(timezone.utc).isoformat()})
            print(json.dumps({"case": self.mode, "pid": self.child.pid, "path": str(self.path)}), flush=True)
            if self.mode == "menu-quit":
                state = self.wait_reply("state", title_ready, "main title")
                self.result["checks"]["title"] = {"passed": True, "state": state}
                self.advance("titleFrames", False)
            else:
                state = self.wait_reply("state", lambda s: world_ready(s, world), "playable isolated world")
                self.result["checks"]["world"] = {"passed": True, "state": state}
                self.advance("worldFramesAndTicks", True)
                if self.mode == "world-menu-quit":
                    leave_started = time.monotonic()
                    if self.args.leave_via == "disconnect":
                        self.must_command("disconnect")
                    else:
                        self.must_command("pause")
                        paused = self.wait_reply("screen", lambda s: s.get("screen") == "paused" and
                                                 any(b.get("label") in ({self.args.exit_label} if self.args.exit_label else EXIT_LABELS)
                                                     for b in s.get("buttons", [])), "pause menu buttons")
                        label = exit_label(paused, self.args.exit_label)
                        self.result["checks"]["pauseButton"] = {"passed": True, "label": label, "screen": paused}
                        self.must_command("click-button " + label)
                    state = self.wait_reply("state", title_ready, "saved world returned to title", self.args.shutdown_timeout)
                    self.result["checks"]["returnedToTitle"] = {"passed": True, "state": state,
                        "elapsedSeconds": round(time.monotonic() - leave_started, 3)}
                    self.advance("returnedTitleFrames", False)
            quit_started = time.monotonic()
            reply = self.must_command("quit")
            if reply.get("ok") is not True:
                raise RuntimeError("quit did not acknowledge ok=true")
            try:
                exit_code = self.child.wait(timeout=self.args.shutdown_timeout)
            except subprocess.TimeoutExpired as error:
                raise TimeoutError("quit acknowledged but game process did not exit") from error
            self.result["checks"]["quit"] = {"passed": exit_code == 0, "exitCode": exit_code,
                "elapsedSeconds": round(time.monotonic() - quit_started, 3)}
            self.result["exitCode"] = exit_code
            if exit_code != 0:
                raise RuntimeError(f"Game exited with code {exit_code}")
            if sha256(self.args.binary) != current_hash:
                raise RuntimeError("Executable SHA-256 changed during case")
            self.result["passed"] = True
        except Exception as error:
            self.result["error"] = f"{type(error).__name__}: {error}"
            self.result["timedOut"] = isinstance(error, TimeoutError)
            self.capture_failure()
        finally:
            self.cleanup()
            if self.child is not None:
                self.result["finalProcessExitCode"] = self.child.poll()
            self.result["elapsedSeconds"] = round(time.monotonic() - self.started, 3)
            self.trace.close()
            if stdout is not None:
                stdout.close()
            if stderr is not None:
                stderr.close()
            write_json(self.path / "result.json", self.result)
            print(json.dumps(self.result, ensure_ascii=True), flush=True)
        return self.result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--expected-sha256", required=True)
    parser.add_argument("--package", required=True, type=Path, help="Package containing theme/ and runtime/; never read user-data/")
    parser.add_argument("--output-root", type=Path, default=QA, help="New isolated case directories; defaults to build/launcher-qa")
    parser.add_argument("--resources", type=Path)
    parser.add_argument("--name", required=True, help="New directory name within this QA folder")
    parser.add_argument("--mode", choices=["menu-quit", "world-menu-quit", "world-quit", "all"], default="all")
    parser.add_argument("--leave-via", choices=["pause-button", "disconnect"], default="pause-button")
    parser.add_argument("--exit-label")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--shutdown-timeout", type=float, default=60)
    parser.add_argument("--command-timeout", type=float, default=4)
    args = parser.parse_args()
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,79}", args.name):
        parser.error("--name must be a simple new QA directory name")
    args.expected_sha256 = args.expected_sha256.lower()
    if not re.fullmatch(r"[0-9a-f]{64}", args.expected_sha256):
        parser.error("--expected-sha256 must contain 64 hex characters")
    args.binary, args.package = args.binary.resolve(strict=True), args.package.resolve(strict=True)
    args.resources = (args.resources or args.package / "resources/26.3").resolve(strict=True)
    if not args.binary.is_file() or not args.resources.is_dir():
        parser.error("--binary must be a file and --resources must be a directory")
    if any(not 0.1 <= value <= 600 for value in (args.timeout, args.shutdown_timeout, args.command_timeout)):
        parser.error("Timeouts must be between 0.1 and 600 seconds")
    if args.exit_label and any(c in args.exit_label for c in "\r\n"):
        parser.error("--exit-label must be a single line")
    if sha256(args.binary) != args.expected_sha256:
        parser.error("Executable SHA-256 does not match the expected final build")
    output_root = args.output_root.resolve()
    output_root.mkdir(parents=True, exist_ok=True)
    root = output_root / args.name
    root.mkdir(exist_ok=False)
    modes = ["menu-quit", "world-menu-quit", "world-quit"] if args.mode == "all" else [args.mode]
    results = []
    for mode in modes:
        result = Case(args, root, mode).run()
        results.append(result)
        if not result["passed"]:
            break
    summary = {"validationScope": "game_lifecycle_smoke", "createdUtc": datetime.now(timezone.utc).isoformat(),
               "executable": str(args.binary), "executableSha256": args.expected_sha256,
               "package": str(args.package), "resources": str(args.resources), "offscreen": True,
               "requestedCases": modes, "cases": results,
               "passed": len(results) == len(modes) and all(case["passed"] for case in results),
               "elapsedSeconds": round(sum(case["elapsedSeconds"] for case in results), 3)}
    write_json(root / "summary.json", summary)
    print(json.dumps({"summary": str(root / "summary.json"), "passed": summary["passed"]}), flush=True)
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
