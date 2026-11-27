# SPDX-License-Identifier: GPL-3.0-or-later
"""Offscreen native menu/API smoke check; only uses an isolated QA profile."""
import argparse
from datetime import datetime, timezone
import http.cookiejar
import io
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

from test_game_lifecycle import Case, FOLDERS, QA, sha256, title_ready, write_json


def read_servers_dat(path):
    """Read this test's small uncompressed servers.dat independently of the API."""
    raw = path.read_bytes()
    if len(raw) > 1024 * 1024:
        raise ValueError("QA servers.dat exceeds expected size")
    stream = io.BytesIO(raw)

    def read(size):
        data = stream.read(size)
        if len(data) != size:
            raise ValueError("Truncated servers.dat")
        return data

    def number(fmt):
        return struct.unpack(">" + fmt, read(struct.calcsize(fmt)))[0]

    def text():
        return read(number("H")).decode("utf-8")

    def tag(kind, depth=0):
        if depth > 16:
            raise ValueError("Excessive servers.dat depth")
        if kind in {1, 2, 3, 4, 5, 6}:
            return number({1: "b", 2: "h", 3: "i", 4: "q", 5: "f", 6: "d"}[kind])
        if kind == 8:
            return text()
        if kind == 9:
            child, count = number("B"), number("i")
            if not 0 <= count <= 1000:
                raise ValueError("Invalid QA NBT list size")
            return [tag(child, depth + 1) for _ in range(count)]
        if kind == 10:
            values = {}
            while True:
                child = number("B")
                if child == 0:
                    return values
                name = text()
                values[name] = tag(child, depth + 1)
        raise ValueError(f"Unexpected QA NBT tag {kind}")

    if number("B") != 10:
        raise ValueError("servers.dat root is not a compound")
    text()
    value = tag(10)
    if stream.read(1):
        raise ValueError("Unexpected bytes after servers.dat")
    return value["servers"]


def setting(tree, name):
    matches = [item for item in tree.get("value", []) if item.get("name") == name]
    if len(matches) != 1:
        raise ValueError(f"Expected exactly one setting {name}")
    return matches[0]


class MenuCase(Case):
    def __init__(self, args, root):
        super().__init__(args, root, "menu-api")
        self.base = None
        self.http = urllib.request.build_opener(urllib.request.ProxyHandler({}),
                                               urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))

    def api(self, method, path, payload=None, expected=(200, 204)):
        self.alive()
        body = json.dumps(payload).encode("utf-8") if payload is not None else None
        request = urllib.request.Request(self.base + "/api/v1/client/" + path,
                                         data=body, method=method,
                                         headers={"Content-Type": "application/json"})
        try:
            response = self.http.open(request, timeout=self.args.command_timeout)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            status = response.status
            raw = response.read(2 * 1024 * 1024 + 1)
        if len(raw) > 2 * 1024 * 1024:
            raise RuntimeError("Oversized localhost API reply")
        value = json.loads(raw) if raw else None
        self.event(httpMethod=method, apiPath=path, status=status,
                   responseShape=list(value) if isinstance(value, dict) else type(value).__name__)
        if status not in expected:
            raise RuntimeError(f"{method} {path}: HTTP {status}: {str(value)[:800]}")
        return value

    def wait_for(self, function, predicate, label):
        deadline = time.monotonic() + self.args.timeout
        last = None
        while time.monotonic() < deadline:
            self.alive()
            last = function()
            if predicate(last):
                return last
            time.sleep(0.2)
        raise TimeoutError(f"Timed out waiting for {label}: {str(last)[:1000]}")

    def bootstrap(self):
        endpoint = self.path / "liquidbounce/interop.json"
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.alive()
            if endpoint.is_file():
                try:
                    config = json.loads(endpoint.read_text(encoding="utf-8"))
                    break
                except ValueError:
                    pass
            time.sleep(0.2)
        else:
            raise TimeoutError("Private browser interop endpoint was not published")
        address = config["address"]
        if not re.fullmatch(r"127\.0\.0\.1:[0-9]{1,5}", address):
            raise ValueError("Interop endpoint is not the isolated loopback server")
        parsed = urllib.parse.urlsplit(config["themeUrl"])
        if parsed.scheme != "http" or parsed.netloc != address or parsed.path != "/resource/liquidbounce/":
            raise ValueError("Interop authentication URL differs from the private loopback endpoint")
        self.base = "http://" + address
        # Authenticate using the native protocol's HttpOnly cookie bootstrap.
        # The URL's lb_code and resulting cookie are never included in evidence.
        try:
            with self.http.open(config["themeUrl"], timeout=self.args.command_timeout) as response:
                if response.status != 200:
                    raise RuntimeError("Theme authentication bootstrap did not return 200")
                response.read(1024)
        except Exception:
            raise RuntimeError("Private localhost theme authentication bootstrap failed") from None
        self.result["endpointDiscovery"] = "isolated liquidbounce/interop.json"

    def route(self, name, initial=False):
        started = time.monotonic()
        if not initial:
            self.api("PUT", "screen", {"name": name})
        virtual = self.wait_for(lambda: self.api("GET", "virtualScreen"),
                                lambda value: isinstance(value, dict) and value.get("name") == name,
                                name + " virtual screen")
        def painted(status):
            browser = status.get("browser", {})
            menu = browser.get("menu", {})
            ack, count = menu.get("acknowledgedPaint"), browser.get("viewPaintCount")
            return (menu.get("native") == name and menu.get("route") == name
                    and menu.get("ready") is True and isinstance(ack, int)
                    and isinstance(count, int) and count > ack)

        status = self.wait_for(lambda: self.api("GET", "integration-status"), painted, name + " ACK and new OnPaint")
        mode = {"title": "main", "singleplayer": "select-world", "multiplayer": "multiplayer"}[name]
        state = self.wait_reply("state", lambda value: title_ready(value) and value.get("menu", {}).get("mode") == mode,
                                name + " native screen")
        browser = status["browser"]
        result = {"passed": True, "virtualScreen": virtual["name"], "nativeMenuMode": state["menu"]["mode"],
                  "ready": browser["menu"]["ready"], "acknowledgedPaint": browser["menu"]["acknowledgedPaint"],
                  "viewPaintCount": browser["viewPaintCount"], "paintCount": browser.get("paintCount"),
                  "elapsedSeconds": round(time.monotonic() - started, 3)}
        self.result["checks"].setdefault("routes", []).append(result)
        self.event(route=name, verification=result)

    def servers(self):
        if self.api("GET", "servers") != []:
            raise RuntimeError("Fresh QA server list is not empty")
        path = self.path / "player/servers.dat"
        operations = []

        def assert_rows(expected):
            api = self.api("GET", "servers")
            actual = [(r["name"], r["address"], r["resourcePackPolicy"]) for r in api]
            disk = read_servers_dat(path)
            disk_rows = [(r["name"], r["ip"], {1: "Enabled", 0: "Disabled"}.get(r.get("acceptTextures"), "Prompt"))
                         for r in disk if not r.get("hidden", 0)]
            if actual != expected or disk_rows != expected:
                raise RuntimeError(f"API/servers.dat differ: expected={expected}, api={actual}, disk={disk_rows}")
            operations.append({"rows": len(expected), "serversDatSha256": sha256(path)})

        a = {"name": "QA Loopback A", "address": "127.0.0.1:1", "resourcePackPolicy": "Prompt"}
        b = {"name": "QA Loopback B", "address": "127.0.0.1:2", "resourcePackPolicy": "Enabled"}
        self.api("PUT", "servers/add", a)
        assert_rows([(a["name"], a["address"], "Prompt")])
        self.api("PUT", "servers/add", b)
        assert_rows([(a["name"], a["address"], "Prompt"), (b["name"], b["address"], "Enabled")])
        edited = {"id": 0, "name": "QA Loopback Edited", "address": "127.0.0.1:3", "resourcePackPolicy": "Disabled"}
        self.api("PUT", "servers/edit", edited)
        assert_rows([(edited["name"], edited["address"], "Disabled"), (b["name"], b["address"], "Enabled")])
        self.api("POST", "servers/order", {"order": [1, 0]})
        assert_rows([(b["name"], b["address"], "Enabled"), (edited["name"], edited["address"], "Disabled")])
        self.api("DELETE", "servers/remove", {"id": 0})
        assert_rows([(edited["name"], edited["address"], "Disabled")])
        self.api("DELETE", "servers/remove", {"id": 0})
        assert_rows([])
        for method, endpoint, payload in [("DELETE", "servers/remove", {"id": 999}),
                                           ("PUT", "servers/edit", {**edited, "id": 999}),
                                           ("POST", "servers/order", {"order": [999]})]:
            self.api(method, endpoint, payload, expected=(400,))
        assert_rows([])
        self.result["checks"]["servers"] = {"passed": True, "addEditOrderRemove": True,
            "invalidIdsReturn400": True, "realServersDatVerified": True, "externalServerConnections": 0, "operations": operations}

    def module_save(self):
        self.api("PUT", "screen", {"name": "clickgui"})
        self.wait_for(lambda: self.api("GET", "virtualScreen"), lambda v: v.get("name") == "clickgui", "ClickGUI overlay")
        before = self.api("GET", "modules/settings?name=ClickGUI")
        original = setting(before, "SearchBarAutoFocus")["value"]
        if not isinstance(original, bool):
            raise RuntimeError("ClickGUI SearchBarAutoFocus is not boolean")
        target = not original
        payload = {"name": "ClickGUI", "value": [{"name": "SearchBarAutoFocus", "value": target}]}
        self.api("PUT", "modules/settings?name=ClickGUI", payload)
        after = self.api("GET", "modules/settings?name=ClickGUI")
        if setting(after, "SearchBarAutoFocus")["value"] is not target:
            raise RuntimeError("ClickGUI setting did not reach the actual actor")
        path = self.path / "liquidbounce/modules.json"
        saved = json.loads(path.read_text(encoding="utf-8"))
        clickgui = next(v for v in saved["value"] if v["name"] == "ClickGUI")
        if setting(clickgui, "SearchBarAutoFocus")["value"] is not target:
            raise RuntimeError("ClickGUI setting was not persisted to modules.json")
        inert = [v for v in saved["value"] if v["name"] == "Disabler"]
        if len(inert) != 1:
            raise RuntimeError("Full module persistence did not include Disabler")
        self.result["checks"]["moduleSave"] = {"passed": True, "module": "ClickGUI", "setting": "SearchBarAutoFocus",
            "before": original, "after": target, "savedModuleCount": len(saved["value"]), "disablerIncluded": True,
            "modulesFileSha256": sha256(path)}
        self.api("DELETE", "screen")
        self.route("title")

    def run(self):
        stdout = stderr = None
        try:
            if sha256(self.args.binary) != self.args.expected_sha256:
                raise RuntimeError("Executable SHA-256 changed before menu API launch")
            for folder in set(FOLDERS.values()) | {"game", "player"}:
                (self.path / folder).mkdir()
            # This belongs only to the new QA profile; the warning is tested by
            # the native game separately from the multiplayer list API route.
            (self.path / "player/options.txt").write_text("skipMultiplayerWarning:true\n", encoding="utf-8")
            env = os.environ.copy()
            env.update({key: str(self.path / folder) for key, folder in FOLDERS.items()})
            env.update(HORIZON_LAUNCHER_NO_SELF_UPDATE="1", HORIZON_NO_AUDIO="1",
                       LIQUIDBOUNCE_THEME_ROOT=str(self.args.package / "theme"),
                       LIQUIDBOUNCE_CEF_RESOURCES=str(self.args.package / "runtime"))
            command = [str(self.args.binary), "--offscreen", "--resources", str(self.args.resources),
                       "--home", str(self.path / "home"), "--game-dir", str(self.path / "game"),
                       "--player-dir", str(self.path / "player"), "--control-socket", str(self.path / "control.json"),
                       "--name", "LauncherQA", "--width", "960", "--height", "540", "--render-distance", "2"]
            stdout, stderr = (self.path / "stdout.log").open("wb"), (self.path / "stderr.log").open("wb")
            self.child = subprocess.Popen(command, cwd=self.path / "game", env=env, stdin=subprocess.DEVNULL,
                                          stdout=stdout, stderr=stderr, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            self.result["pid"] = self.child.pid
            write_json(self.path / "process.json", {"pid": self.child.pid, "command": command,
                       "executableSha256": self.args.expected_sha256, "createdUtc": datetime.now(timezone.utc).isoformat()})
            print(json.dumps({"pid": self.child.pid, "path": str(self.path)}), flush=True)
            self.wait_reply("state", title_ready, "native title")
            self.bootstrap()
            self.route("title", initial=True)
            self.route("singleplayer")
            worlds = self.api("GET", "worlds")
            if worlds != []:
                raise RuntimeError("Fresh QA world list is not empty")
            self.result["checks"]["worlds"] = {"passed": True, "count": 0}
            self.route("multiplayer")
            self.servers()
            self.route("title")
            session, accounts = self.api("GET", "session"), self.api("GET", "accounts")
            if (session.get("username") != "LauncherQA" or session.get("online") is not False
                    or not any(v.get("username") == "LauncherQA" and v.get("uuid") == session.get("uuid") for v in accounts)):
                raise RuntimeError("Session/accounts did not identify the isolated QA player")
            self.result["checks"]["sessionAccounts"] = {"passed": True, "username": session["username"], "online": False,
                                                       "accountCount": len(accounts)}
            self.module_save()
            started = time.monotonic()
            self.api("POST", "exit")
            try:
                code = self.child.wait(timeout=self.args.shutdown_timeout)
            except subprocess.TimeoutExpired as error:
                raise TimeoutError("POST exit acknowledged but the game process did not exit") from error
            self.result["checks"]["exit"] = {"passed": code == 0, "exitCode": code,
                                              "elapsedSeconds": round(time.monotonic() - started, 3)}
            if code != 0:
                raise RuntimeError(f"Menu API exit code was {code}")
            if sha256(self.args.binary) != self.args.expected_sha256:
                raise RuntimeError("Executable changed during menu API smoke")
            self.result["passed"] = True
        except Exception as error:
            self.result["error"] = f"{type(error).__name__}: {error}"
            self.result["timedOut"] = isinstance(error, TimeoutError)
            self.capture_failure()
        finally:
            self.cleanup()
            self.result["finalProcessExitCode"] = self.child.poll() if self.child else None
            self.result["elapsedSeconds"] = round(time.monotonic() - self.started, 3)
            self.trace.close()
            if stdout:
                stdout.close()
            if stderr:
                stderr.close()
            write_json(self.path / "result.json", self.result)
        return self.result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--expected-sha256", required=True)
    parser.add_argument("--name", required=True)
    parser.add_argument("--package", required=True, type=Path, help="Package containing theme/ and runtime/; never read user-data/")
    parser.add_argument("--output-root", type=Path, default=QA, help="New isolated case directories; defaults to build/launcher-qa")
    parser.add_argument("--resources", type=Path)
    parser.add_argument("--timeout", type=float, default=90)
    parser.add_argument("--command-timeout", type=float, default=10)
    parser.add_argument("--shutdown-timeout", type=float, default=60)
    args = parser.parse_args()
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,79}", args.name):
        parser.error("Use a simple new QA directory name")
    args.expected_sha256 = args.expected_sha256.lower()
    args.binary, args.package = args.binary.resolve(strict=True), args.package.resolve(strict=True)
    args.resources = (args.resources or args.package / "resources/26.3").resolve(strict=True)
    if not re.fullmatch(r"[0-9a-f]{64}", args.expected_sha256) or sha256(args.binary) != args.expected_sha256:
        parser.error("Executable SHA-256 differs from the expected final build")
    if any(not 0.1 <= n <= 600 for n in (args.timeout, args.command_timeout, args.shutdown_timeout)):
        parser.error("Timeouts must be between 0.1 and 600 seconds")
    output_root = args.output_root.resolve()
    output_root.mkdir(parents=True, exist_ok=True)
    root = output_root / args.name
    root.mkdir(exist_ok=False)
    result = MenuCase(args, root).run()
    summary = {"validationScope": "menu_api_smoke", "passed": result["passed"],
               "executableSha256": args.expected_sha256, "executable": str(args.binary),
               "offscreen": True, "checks": result["checks"], "elapsedSeconds": result["elapsedSeconds"],
               "evidence": str(root / "menu-api/result.json")}
    if "error" in result:
        summary["error"] = result["error"]
    write_json(root / "summary.json", summary)
    print(json.dumps({"summary": str(root / "summary.json"), **summary}), flush=True)
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
