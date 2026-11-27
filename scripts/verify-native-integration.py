#!/usr/bin/env python3
"""Exercise the real port through REST and Horizon's semantic control socket."""
from __future__ import annotations

import argparse
import hashlib
import http.cookiejar
import json
import math
from pathlib import Path
import re
import struct
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
import zlib
from runtime_scenarios.timer import verify as verify_timer
from runtime_scenarios.better_inventory import verify as verify_better_inventory
from runtime_scenarios.terrain_speed import verify as verify_terrain_speed
from runtime_scenarios.auto_break import verify as verify_auto_break
from runtime_scenarios.high_jump import verify as verify_high_jump
from runtime_scenarios.item_scroller import verify as verify_item_scroller
from runtime_scenarios.fast_break import verify as verify_fast_break
from runtime_scenarios.anti_hunger import verify as verify_anti_hunger
from runtime_scenarios.extended_firework import verify as verify_extended_firework
from runtime_scenarios.elytra_recast import verify as verify_elytra_recast
from runtime_scenarios.elytra_swap import verify as verify_elytra_swap
from runtime_scenarios.gui_closer import verify as verify_gui_closer
from runtime_scenarios.text_field_protect import verify as verify_text_field_protect

PROJECT = Path(__file__).resolve().parent.parent
MODULES = ('AbortBreaking', 'Aimbot', 'AirJump', 'AirPlace', 'Anchor', 'AntiAFK', 'AntiBlind', 'AntiBot', 'AntiBounce', 'AntiCheatDetect', 'AntiExploit', 'AntiHunger', 'AntiLevitation', 'AntiReducedDebugInfo', 'AntiStaff', 'AntiVoid', 'Aspect', 'AutoAccount', 'AutoArmor', 'AutoBow', 'AutoBreak', 'AutoBuff', 'AutoChatGame', 'AutoClicker', 'AutoCrafter', 'AutoDeposit', 'AutoDisable', 'AutoDodge', 'AutoF5', 'AutoFish', 'AutoLeave', 'AutoMobHeal', 'AutoPearl', 'AutoQueue', 'AutoRespawn', 'AutoRod', 'AutoShoot', 'AutoTool', 'AutoWalk', 'AutoWeapon', 'AutoWindCharge', 'AvoidHazards', 'BedDefender', 'BedPlates', 'BetterChat', 'BetterInventory', 'BetterTab', 'Blink', 'BlockBounce', 'BlockESP', 'BlockIn', 'BlockOutline', 'BlockWalk', 'BookBot', 'Breadcrumbs', 'CameraClip', 'Chams', 'ChestCleaner', 'ChestStealer', 'ClickGUI', 'Clip', 'Clutch', 'CombineMobs', 'Criticals', 'Crosshair', 'CrystalView', 'CustomAmbience', 'Damage', 'DamageParticles', 'DankBobbing', 'Debug', 'Derp', 'Eagle', 'EasyPearl', 'ElytraRecast', 'ElytraSwap', 'ElytraTarget', 'EntityControl', 'ESP', 'ExtendedFirework', 'Extinguish', 'FakeLag', 'FastBreak', 'FastExp', 'FastPlace', 'FastUse', 'FlagCheck', 'FreeCam', 'FreeLook', 'Freeze', 'FullBright', 'GhostHand', 'GUICloser', 'HandDerp', 'Hats', 'HighJump', 'Hitbox', 'HitFX', 'HoleESP', 'HUD', 'InventoryCleaner', 'InventoryMove', 'InventoryTracker', 'ItemChams', 'ItemESP', 'ItemScroller', 'ItemTags', 'JumpEffect', 'KeepSprint', 'Kick', 'KillAura', 'LiquidFiller', 'LiquidPlace', 'LiquidWalk', 'LogoffSpot', 'MaceKill', 'Macros', 'MiddleClickAction', 'MobOwners', 'MoreCarry', 'MultiActions', 'MurderMystery', 'NameCollector', 'NameProtect', 'Nametags', 'NewChunks', 'NoBlockInteract', 'NoBob', 'NoClip', 'NoEntityInteract', 'NoFOV', 'NoHurtCam', 'NoInterpolation', 'NoJumpDelay', 'NoMissCooldown', 'NoPitchLimit', 'NoPose', 'NoPush', 'NoRotateSet', 'NoSlotSet', 'NoSlow', 'NoSlowBreak', 'NoSwing', 'Notifier', 'NoWeb', 'Offhand', 'OldRegen', 'PacketLogger', 'PacketMine', 'Parkour', 'Particles', 'PingSpoof', 'Plugins', 'PortalMenu', 'PotionFX', 'PotionSpoof', 'ProjectilePuncher', 'ProphuntESP', 'ProtectionZones', 'QuickPerspectiveSwap', 'Radar', 'Reach', 'Replenish', 'ReportHelper', 'ResetVL', 'ReverseStep', 'SafeWalk', 'Scaffold', 'SilentHotbar', 'SkinDerp', 'SleepWalker', 'SmartEat', 'SmoothCamera', 'SnapTap', 'Sneak', 'Spammer', 'SpearKill', 'Speed', 'Spider', 'Sprint', 'Step', 'StorageESP', 'Strafe', 'StrongholdFinder', 'SuperKnockback', 'Surround', 'SwordBlock', 'TargetLock', 'TargetStrafe', 'Teams', 'TerrainSpeed', 'TextFieldProtect', 'Timer', 'TNTTimer', 'TotemEffect', 'Tracers', 'Trajectories', 'TridentBoost', 'TriggerBot', 'TrueSight', 'Twerk', 'VehicleBoost', 'VehicleControl', 'VehicleOneHit', 'VoidESP', 'Vomit', 'Wings', 'XRay', 'YggdrasilSignatureFix', 'Zoom')
RENDER_MODULES = ('AntiBlind', 'Aspect', 'AutoF5', 'BedPlates', 'BetterChat', 'BetterInventory', 'BetterTab', 'BlockESP', 'BlockOutline', 'Breadcrumbs', 'CameraClip', 'Chams', 'ClickGUI', 'CombineMobs', 'Crosshair', 'CrystalView', 'CustomAmbience', 'DamageParticles', 'Debug', 'ESP', 'FreeCam', 'FreeLook', 'FullBright', 'Hats', 'HitFX', 'HoleESP', 'HUD', 'ItemChams', 'ItemESP', 'ItemTags', 'JumpEffect', 'LogoffSpot', 'MobOwners', 'MurderMystery', 'Nametags', 'NewChunks', 'NoBob', 'NoFOV', 'NoHurtCam', 'NoSwing', 'Particles', 'PotionFX', 'ProphuntESP', 'ProtectionZones', 'QuickPerspectiveSwap', 'Radar', 'SilentHotbar', 'SmoothCamera', 'StorageESP', 'TNTTimer', 'TotemEffect', 'Tracers', 'Trajectories', 'TrueSight', 'VoidESP', 'Wings', 'XRay', 'Zoom')
CAMERA_MODULES = ("FullBright", "NoBob", "NoFOV", "NoHurtCam", "Zoom")
# ModuleManager's VALUE_NAME_ORDER is case-insensitive, independent of registration order.
ORIGINAL_MODULE_ORDER = tuple(sorted(MODULES, key=str.lower))
HOOKS = ("cameraFovOption", "cameraFovReturn", "playerFovReturn", "lightmapGammaOption", "clientPlayerTickPost",
         "cameraBobEffects", "cameraBobResult", "cameraSpinningResult")
LIQUIDBOUNCE_COMMIT = "ae0eb396c07f22383789b28d5912d714c6ca0a86"
HORIZON_COMMIT = "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0"


def build_provenance():
    result = {"liquidBounceCommit": LIQUIDBOUNCE_COMMIT, "horizonCommit": HORIZON_COMMIT}
    theme = PROJECT / "build/themes/chinese/localization-build.json"
    if theme.is_file():
        metadata = json.loads(theme.read_text(encoding="utf-8-sig"))
        result["theme"] = {key: metadata[key] for key in
                           ("themeKind", "sourceSha256", "outputSha256", "buildCompleted") if key in metadata}
    stage_keys = ("horizon", "gpuUiPatchSha256", "behaviorHookPatchSha256", "nativeSourceSha256", "originalThemeLinked", "nativeMembers")
    for filename, key, fields in (
        ("staging-report.json", "staging", stage_keys),
        ("integrated-build.json", "integratedBuild", ("operation", "startedUtc", "finishedUtc", "exitCode",
                                                       "passed", "lockSha256", "executable", "executableSha256")),
    ):
        path = PROJECT / "build" / filename
        if path.is_file():
            source = json.loads(path.read_text(encoding="utf-8-sig"))
            result[key] = {field: source[field] for field in fields if field in source}
            if key == "integratedBuild" and isinstance(source.get("staging"), dict):
                result[key]["staging"] = {field: source["staging"][field] for field in stage_keys
                                           if field in source["staging"]}
    executable = PROJECT / "build/staging/horizon/target/debug/liquidbounce-native-game.exe"
    if executable.is_file():
        digest = hashlib.sha256()
        with executable.open("rb") as binary:
            for chunk in iter(lambda: binary.read(1024 * 1024), b""):
                digest.update(chunk)
        result["executableOnDisk"] = {"path": str(executable), "sha256": digest.hexdigest()}
    return result


def redact(value, secrets):
    if isinstance(value, str):
        for secret in secrets:
            if secret:
                value = value.replace(secret, "<redacted>")
        return re.sub(r"(?i)(lb_code|lb_auth|token)=([^\s&\"']+)", r"\1=<redacted>", value)
    if isinstance(value, list):
        return [redact(item, secrets) for item in value]
    if isinstance(value, dict):
        return {key: "<redacted>" if key.lower() in ("token", "auth_code", "lb_code", "themeurl")
                else redact(item, secrets) for key, item in value.items()}
    return value


def probe(status, name):
    return status.get("probes", {}).get(name, {})


def probe_count(status, name):
    return probe(status, name).get("count", 0)


def fresh_probe(status, name, before):
    return probe_count(status, name) > probe_count(before, name)


def fresh_console(status, before):
    """Return new actual CEF messages, accounting for its bounded log buffer."""
    previous = before["browser"]["page"]["console"]
    current = status["browser"]["page"]["console"]
    for overlap in range(min(len(previous), len(current)), 0, -1):
        if previous[-overlap:] == current[:overlap]:
            return current[overlap:]
    return current


def fresh_route(status, before, name, *, reloaded=False):
    messages = fresh_console(status, before)
    if reloaded:
        # App.svelte logs its initial route only after await insertPersistentData.
        # Require that message from the new document, not the previous route.
        starts = [index for index, line in enumerate(messages) if "Connecting to server at:" in line]
        if not starts:
            return False
        messages = messages[starts[-1] + 1:]
    return any("[Router] Redirecting to " + name in line for line in messages)


def close_number(actual, expected):
    return (isinstance(actual, (int, float)) and not isinstance(actual, bool)
            and math.isfinite(actual) and math.isclose(actual, expected, rel_tol=1e-6, abs_tol=1e-6))


IDENTITY = [1.0 if row == column else 0.0 for column in range(4) for row in range(4)]


def finite_matrix(value):
    return (isinstance(value, list) and len(value) == 16
            and all(isinstance(item, (int, float)) and not isinstance(item, bool)
                    and math.isfinite(item) for item in value))


def matrix_equal(left, right, tolerance=1e-5):
    return (finite_matrix(left) and finite_matrix(right)
            and all(abs(a - b) <= tolerance for a, b in zip(left, right)))


def matrix_product(left, right):
    # Horizon/glam diagnostics expose column-major matrices.
    return [sum(left[k * 4 + row] * right[column * 4 + k] for k in range(4))
            for column in range(4) for row in range(4)]


def nonidentity(value):
    return finite_matrix(value) and not matrix_equal(value, IDENTITY)


def bob_result(status):
    result = probe(status, "cameraBobResult")
    return result.get("input", {}), result.get("output", {})


def camera_branches(status, hurt, view):
    effect = probe(status, "cameraBobEffects")
    return (effect.get("input") == {"hurt": True, "view": True}
            and effect.get("output") == {"hurt": hurt, "view": view})


def camera_composition(status):
    source, result = bob_result(status)
    return (finite_matrix(result.get("bob")) and finite_matrix(source.get("shake"))
            and matrix_equal(result.get("combined"), matrix_product(result["bob"], source["shake"])))


def walking_sample(status):
    source, _ = bob_result(status)
    return (source.get("deathTime") is None and source.get("hurtTime", 0) < 0
            and abs(source.get("bob", 0)) > 1e-4 and nonidentity(source.get("original")))


def hurt_sample(status):
    source, _ = bob_result(status)
    return (source.get("deathTime") is None and source.get("hurtDuration") == 10
            and 0 < source.get("hurtTime", 0) < 10 and source.get("damageTiltStrength", 0) > 0
            and nonidentity(source.get("original")))


def active_effects(response):
    # The pinned controller returns effects.active, not a flat array. Do not
    # mistake a thrown potion, diagnostic string or error for an active effect.
    active = response["effects"]["active"]
    if not isinstance(active, list) or not all(isinstance(item, dict) for item in active):
        raise ValueError(f"Unexpected effects response: {response!r}")
    return active


def night_vision(response):
    return [item for item in active_effects(response)
            if item.get("id", "").removeprefix("minecraft:") == "night_vision"]


def stable_settings(value):
    """Compare persistent values and API identifiers across game languages."""
    if isinstance(value, dict):
        return {key: stable_settings(item) for key, item in value.items() if key != "description"}
    if isinstance(value, list):
        return [stable_settings(item) for item in value]
    return value


def described_settings(value):
    if isinstance(value, dict):
        if "description" in value:
            yield value
        for item in value.values():
            yield from described_settings(item)
    elif isinstance(value, list):
        for item in value:
            yield from described_settings(item)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True, help="Isolated build/game-runs/<name> folder")
    parser.add_argument("--controller", default="D:/Develop/MCL/.references/horizon/nightly/horizon-windows-x64/horizon-ctl.exe")
    parser.add_argument("--quit", action="store_true")
    parser.add_argument("--ui-only", action="store_true", help="Diagnose the real ClickGUI and restore state; does not validate module effects")
    parser.add_argument("--languages", action="store_true", help="Switch actual game options zh_cn, zh_tw, en_us and capture the live GUI")
    parser.add_argument("--scenarios", nargs="+", choices=("Timer", "BetterInventory", "TerrainSpeed", "AutoBreak", "HighJump", "ItemScroller", "FastBreak", "AntiHunger", "ExtendedFirework", "ElytraRecast", "ElytraSwap", "GUICloser", "TextFieldProtect"),
                        help="Exercise selected real gameplay consumers and restore state; skips the camera/UI regression")
    args = parser.parse_args()
    if args.scenarios and (args.ui_only or args.languages):
        parser.error("--scenarios runs gameplay consumers; use a separate --languages/UI run")
    run = args.run.resolve()
    run.relative_to((PROJECT / "build/game-runs").resolve())
    run.mkdir(parents=True, exist_ok=True)
    report = {"structuredControl": True, "screenRecognitionUsed": False, "records": [], "checks": {}, "passed": False,
              "validationScope": "selected_runtime_scenarios" if args.scenarios else "ui_diagnostic" if args.ui_only else "camera_regression_and_registry", "fullIntegrationPassed": False,
              "behaviorModulesExercised": [] if args.ui_only or args.scenarios else list(CAMERA_MODULES),
              "behaviorModulesNotExercised": list(MODULES) if args.ui_only or args.scenarios else [name for name in MODULES if name not in CAMERA_MODULES]}
    socket = run / "horizon-control.json"
    records = report["records"]
    secrets = []

    def control(*command: str):
        # A long tick wait can exceed wall-clock time under parallel compilation.
        # Keep command completion tied to the game's reply, with a bounded budget.
        budget = 45 if command and command[0] == "wait" else 10
        completed = subprocess.run([args.controller, "--socket", str(socket), "--timeout", str(budget), *command],
                                   capture_output=True, text=True, encoding="utf-8", timeout=budget + 5, check=False)
        if completed.returncode:
            records.append({"control": list(command), "returncode": completed.returncode,
                            "stdout": completed.stdout, "stderr": completed.stderr})
            raise RuntimeError(f"Control {command!r} failed: {completed.stderr or completed.stdout}")
        try:
            value = json.loads(completed.stdout)
        except json.JSONDecodeError as error:
            records.append({"control": list(command), "stdout": completed.stdout, "stderr": completed.stderr})
            raise ValueError(f"Control {command!r} did not return JSON") from error
        records.append({"control": list(command), "response": json.loads(json.dumps(value))})
        if isinstance(value, dict) and value.get("error"):
            raise RuntimeError(f"Control {command!r}: {value['error']}")
        return value

    held_keys = set()

    def key(name: str, action: str):
        # Record presses before dispatch: a timed-out command may already have
        # reached the game, so cleanup must still send an explicit key release.
        if action in ("down", "tap"):
            held_keys.add(name)
        value = control("key", name, action)
        if action in ("up", "tap"):
            held_keys.discard(name)
        return value

    def check(name: str, condition: bool, observed=None):
        report["checks"][name] = {"passed": bool(condition), "observed": json.loads(json.dumps(observed))}
        print(json.dumps({"check": name, "passed": bool(condition)}), flush=True)
        if not condition:
            raise AssertionError(f"{name}: {observed!r}")

    def api(path: str, method="GET", body=None):
        payload = None if body is None else json.dumps(body).encode()
        request = urllib.request.Request(origin + "/api/v1" + path, data=payload, method=method,
                                         headers={"Content-Type": "application/json"})
        try:
            with browser.open(request, timeout=10) as response:
                data = response.read()
                value = json.loads(data) if data else None
                records.append({"api": path, "method": method, "status": response.status,
                                "response": json.loads(json.dumps(value))})
                return value
        except urllib.error.HTTPError as error:
            records.append({"api": path, "method": method, "status": error.code,
                            "error": error.read().decode("utf-8", errors="replace")})
            raise

    def wait_status(predicate, seconds=12):
        deadline = time.monotonic() + seconds
        while True:
            status = api("/client/integration-status")
            if predicate(status):
                return status
            if time.monotonic() >= deadline:
                raise AssertionError(f"Observed game state did not reach expected result: {status!r}")
            # A bounded harness samples actual callbacks; it never guesses completion.
            time.sleep(0.1)

    def wait_state(predicate, seconds=12):
        deadline = time.monotonic() + seconds
        while True:
            current = control("state")
            if predicate(current):
                return current
            if time.monotonic() >= deadline:
                raise AssertionError(f"Official game state did not reach expected result: {current!r}")
            time.sleep(0.1)

    def wait_control(command, predicate, *arguments, seconds=12):
        deadline = time.monotonic() + seconds
        while True:
            current = control(command, *arguments)
            if predicate(current):
                return current
            if time.monotonic() >= deadline:
                raise AssertionError(f"Official {command} did not reach expected result: {current!r}")
            time.sleep(0.1)

    def fresh_camera(status, before):
        return (fresh_probe(status, "cameraBobEffects", before)
                and fresh_probe(status, "cameraBobResult", before) and camera_composition(status))

    def reload_theme():
        before = api("/client/integration-status")
        key("ctrl", "down")
        try:
            key("r", "tap")
        finally:
            key("ctrl", "up")
        return wait_status(lambda s: s["browser"]["page"]["completedLoads"]
                           > before["browser"]["page"]["completedLoads"]
                           and s["browser"]["paintCount"] > before["browser"]["paintCount"]
                           and not s["browser"]["page"]["loading"]
                           and s["observed"]["keyboardEvents"] > before["observed"]["keyboardEvents"]
                           and fresh_route(s, before, "clickgui", reloaded=True))

    def stable_theme_storage():
        # The original tab restore schedules a whole-map PUT after 200 ms.
        # Drain that writer before taking the baseline and injecting layout.
        deadline = time.monotonic() + 12
        stable_since = time.monotonic()
        previous = None
        while True:
            payload = api("/client/localStorage/all")
            current = {item["key"]: item["value"] for item in payload["items"]}
            now = time.monotonic()
            if previous != current:
                previous, stable_since = current, now
            elif now - stable_since >= 0.5:
                return payload
            if now >= deadline:
                raise AssertionError("Theme storage did not remain stable for 500 ms")
            time.sleep(0.1)

    def check_render_layout(name):
        payload = api("/client/localStorage/all")
        value = {item["key"]: item["value"] for item in payload["items"]}.get("clickgui.panel.Render", "null")
        try:
            actual = json.loads(value)
        except (TypeError, json.JSONDecodeError):
            actual = None
        check(name, isinstance(actual, dict) and actual.get("left") == 20 and actual.get("top") == 140
              and actual.get("expanded") is True and actual.get("scrollTop") == 0, payload)
        return payload

    def module(name):
        return api("/client/module/" + urllib.parse.quote(name))

    def enable(name, enabled):
        if module(name)["enabled"] != enabled:
            api("/client/modules/toggle", "PUT" if enabled else "DELETE", {"name": name})

    def settings(name):
        return api("/client/modules/settings?" + urllib.parse.urlencode({"name": name}))

    def put_settings(name, tree):
        api("/client/modules/settings?" + urllib.parse.urlencode({"name": name}), "PUT", tree)

    def leaf(tree, name):
        for item in tree.get("value", []):
            if item["name"] == name:
                return item
        raise KeyError(name)

    def exercised(name):
        if name not in report["behaviorModulesExercised"]:
            report["behaviorModulesExercised"].append(name)
        if name in report["behaviorModulesNotExercised"]:
            report["behaviorModulesNotExercised"].remove(name)

    def set_perspective(perspective):
        snapshot = api("/client/integration-status")
        for _ in range(3):
            if probe(snapshot, "clientCameraPerspective").get("input", {}).get("rawPerspective") == perspective:
                return snapshot
            before = snapshot
            key("f5", "tap")
            snapshot = wait_status(lambda s: fresh_probe(s, "clientCameraPerspective", before)
                                   and probe(s, "clientCameraPerspective").get("input", {}).get("rawPerspective")
                                   != probe(before, "clientCameraPerspective").get("input", {}).get("rawPerspective"))
        raise AssertionError("F5 did not reach the requested stored perspective")

    def choose_language(code):
        before = api("/client/localization")
        if api("/client/integration-status")["browser"]["visible"]:
            key("esc", "tap")
            wait_status(lambda s: not s["browser"]["visible"])
        control("options", "language")
        widgets = control("ui")["widgets"]
        fields = [widget for widget in widgets if widget["kind"] == "field"]
        check("languageSearchField" + code, len(fields) == 1, fields)
        control("click", fields[0]["id"])
        key("ctrl", "down")
        try:
            key("a", "tap")
        finally:
            key("ctrl", "up")
        control("type", code)
        widgets = control("ui")["widgets"]
        rows = [widget for widget in widgets if widget["kind"] == "row"]
        check("officialLanguageAvailable" + code, len(rows) == 1, rows)
        control("click", rows[0]["id"])
        widgets = control("ui")["widgets"]
        done = [widget for widget in widgets if widget["kind"] == "button"
                and widget.get("label") in ("Done", "完成")]
        check("languageDoneButton" + code, len(done) == 1, done)
        control("click", done[0]["id"])
        for _ in range(4):
            if control("state")["screen"] == "playing":
                break
            key("esc", "tap")
        check("languageOptionsClosed" + code, control("state")["screen"] == "playing")
        deadline = time.monotonic() + 20
        while True:
            snapshot = api("/client/localization")
            if snapshot["gameLanguage"] == code and (before["gameLanguage"] == code or snapshot["generation"] > before["generation"]):
                break
            if time.monotonic() >= deadline:
                raise AssertionError(f"Loaded game language did not become {code}")
            time.sleep(0.1)
        check("actualGameLanguage" + code, snapshot["locale"] == code,
              {key:snapshot[key] for key in ("gameLanguage", "locale", "generation")})
        return snapshot

    saved = {}
    saved_global = None
    global_mutated = False
    isolated = False
    mutated = False
    game_mutated = False
    storage_saved = None
    storage_mutated = False
    initial_state = None
    original_perspective = None
    failure = None
    try:
        report["runtimeProvenance"] = build_provenance()
        built = report["runtimeProvenance"].get("integratedBuild", {})
        on_disk = report["runtimeProvenance"].get("executableOnDisk", {})
        if built.get("executableSha256") and on_disk.get("sha256"):
            check("buildExecutableUnchanged", built["executableSha256"].lower() == on_disk["sha256"], on_disk)
        if socket.is_file():
            endpoint = json.loads(socket.read_text(encoding="utf-8-sig"))
            secrets.append(endpoint.get("token", ""))
        state = control("state")
        initial_state = state
        check("isolatedWorld", state.get("world") == "LiquidBouncePortTest", state.get("world"))
        isolated = True
        check("initialPlayingScreen", state.get("screen") == "playing", state.get("screen"))
        info = json.loads((run / "liquidbounce/interop.json").read_text(encoding="utf-8"))
        theme_url = info["themeUrl"]
        parsed = urllib.parse.urlsplit(theme_url)
        secrets.append(theme_url)
        secrets.extend(urllib.parse.parse_qs(parsed.query).get("lb_code", []))
        if parsed.scheme != "http" or parsed.hostname != "127.0.0.1":
            raise ValueError("Only the isolated localhost interop server may be tested")
        origin = f"{parsed.scheme}://{parsed.netloc}"
        cookies = http.cookiejar.CookieJar()
        browser = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cookies))
        with browser.open(theme_url, timeout=10) as response:
            check("localizedThemeServed", response.status == 200, response.status)
        report["hello"] = control("hello")
        available = api("/client/modules")
        check("actualModuleRegistry", len(available) == len(MODULES)
              and {entry["name"] for entry in available} == set(MODULES), available)
        check("originalModuleOrder", [entry["name"] for entry in available] == list(ORIGINAL_MODULE_ORDER),
              [entry["name"] for entry in available])
        for name in MODULES:
            saved[name] = {"enabled": module(name)["enabled"], "settings": settings(name)}
        saved_global = api("/client/global")
        targets = leaf(saved_global, "Targets")
        check("globalTargetsOriginalTree", saved_global.get("name") == "Settings"
              and len(leaf(targets, "Combat")["choices"]) == 10
              and leaf(targets, "Visual")["choices"][0] == "Self", saved_global)
        inventory_global = leaf(saved_global, "Inventory")
        check("globalInventoryOriginalDefaults", leaf(inventory_global, "ResponseTime")["value"] == 400
              and leaf(inventory_global, "LockAfterRejections")["value"] == 3, inventory_global)
        # Disabling NightVision intentionally removes an existing effect in the
        # original module. A fixture with one cannot promise effect restoration.
        effects = control("effects")
        report["initialEffects"] = effects
        check("noPreexistingNightVision", not night_vision(effects), effects)
        check("noPreexistingEffects", not active_effects(effects), effects)
        mutated = True
        control("release")
        for name in MODULES:
            enable(name, False)
            tree = settings(name)
            leaf(tree, "Bind")["value"] = {"boundKey": "key.keyboard.unknown", "action": "Toggle"}
            put_settings(name, tree)
        status = wait_status(lambda s: s["world"]["worldPresent"] and s["world"]["playerPresent"]
                             and all(probe_count(s, name) > 0 for name in HOOKS)
                             and s["browser"]["paintCount"] > 0
                             and s["browser"].get("page") is not None
                             and s["browser"]["page"]["completedLoads"] > 0
                             and not s["browser"]["page"]["loading"])
        check("actualGameHooksExecuted", all(probe_count(status, name) > 0 for name in HOOKS), status["probes"])
        check("realCefOnPaint", status["browser"]["paintCount"] > 0, status["browser"])
        check("cefDocumentLoaded", status["browser"]["page"]["httpStatus"] == 200 and not status["browser"]["page"]["loadErrors"], status["browser"]["page"])
        check("originalThemeTitle", status["browser"]["page"]["title"] == "LiquidBounce UI", status["browser"]["page"]["title"])
        if args.scenarios:
            game_mutated = True
            control("chat", "/gamerule spawn_mobs false")
            control("chat", "/kill @e[type=!minecraft:player]")
            selected_position = initial_state["player"]["position"]
            for scenario in args.scenarios:
                if scenario == "TextFieldProtect":
                    verify_text_field_protect(api=api, control=control, check=check, settings=settings,
                                              put_settings=put_settings, leaf=leaf, enable=enable, key=key,
                                              wait_status=wait_status, wait_state=wait_state, probe=probe,
                                              fresh_probe=fresh_probe, exercised=exercised, report=report, run=run)
                elif scenario == "GUICloser":
                    verify_gui_closer(api=api, control=control, check=check, settings=settings,
                                      put_settings=put_settings, leaf=leaf, enable=enable, key=key,
                                      wait_status=wait_status, wait_state=wait_state, probe=probe,
                                      fresh_probe=fresh_probe, choose_language=choose_language,
                                      exercised=exercised, report=report, run=run)
                elif scenario == "ElytraSwap":
                    verify_elytra_swap(control=control, check=check, settings=settings,
                                      put_settings=put_settings, leaf=leaf, enable=enable,
                                      module=module, key=key, wait_state=wait_state,
                                      wait_control=wait_control, exercised=exercised, report=report)
                elif scenario == "ExtendedFirework":
                    verify_extended_firework(api=api, control=control, check=check, settings=settings,
                                             put_settings=put_settings, leaf=leaf, enable=enable, module=module,
                                             key=key, wait_status=wait_status, wait_state=wait_state,
                                             wait_control=wait_control, probe=probe, fresh_probe=fresh_probe,
                                             exercised=exercised, report=report, position=selected_position)
                elif scenario == "ElytraRecast":
                    verify_elytra_recast(api=api, control=control, check=check, enable=enable,
                                        key=key, wait_state=wait_state, wait_control=wait_control,
                                        exercised=exercised, report=report, position=selected_position)
                elif scenario == "Timer":
                    verify_timer(api=api, control=control, check=check, settings=settings,
                                 put_settings=put_settings, leaf=leaf, enable=enable, module=module,
                                 key=key, wait_status=wait_status, wait_state=wait_state,
                                 wait_control=wait_control, probe=probe, fresh_probe=fresh_probe,
                                 close_number=close_number, exercised=exercised, report=report,
                                 position=selected_position)
                elif scenario == "BetterInventory":
                    verify_better_inventory(api=api, control=control, check=check, settings=settings,
                                            put_settings=put_settings, leaf=leaf, enable=enable,
                                            key=key, wait_status=wait_status, wait_state=wait_state,
                                            wait_control=wait_control, probe=probe, fresh_probe=fresh_probe,
                                            close_number=close_number, exercised=exercised, report=report,
                                            position=selected_position, run=run)
                elif scenario == "TerrainSpeed":
                    verify_terrain_speed(api=api, control=control, check=check, settings=settings,
                                         put_settings=put_settings, leaf=leaf, enable=enable, module=module,
                                         key=key, wait_status=wait_status, wait_state=wait_state,
                                         probe=probe, fresh_probe=fresh_probe, exercised=exercised,
                                         report=report, position=selected_position)
                elif scenario == "HighJump":
                    verify_high_jump(api=api, control=control, check=check, settings=settings,
                                     put_settings=put_settings, leaf=leaf, enable=enable, module=module,
                                     key=key, wait_status=wait_status, wait_state=wait_state,
                                     probe=probe, fresh_probe=fresh_probe, close_number=close_number,
                                     exercised=exercised, report=report, position=selected_position)
                elif scenario == "ItemScroller":
                    verify_item_scroller(control=control, check=check, settings=settings,
                                         put_settings=put_settings, leaf=leaf, enable=enable, key=key,
                                         wait_state=wait_state, wait_control=wait_control,
                                         exercised=exercised, report=report)
                elif scenario == "FastBreak":
                    verify_fast_break(control=control, check=check, settings=settings,
                                      put_settings=put_settings, leaf=leaf, enable=enable,
                                      wait_state=wait_state, wait_control=wait_control,
                                      exercised=exercised, report=report, position=selected_position)
                elif scenario == "AntiHunger":
                    verify_anti_hunger(api=api, control=control, check=check, settings=settings,
                                       put_settings=put_settings, leaf=leaf, enable=enable, key=key,
                                       wait_status=wait_status, wait_state=wait_state, wait_control=wait_control,
                                       probe=probe, fresh_probe=fresh_probe, choose_language=choose_language,
                                       exercised=exercised, report=report, position=selected_position)
                else:
                    verify_auto_break(control=control, check=check, enable=enable,
                                      wait_state=wait_state, wait_control=wait_control,
                                      exercised=exercised, report=report, position=selected_position)
        if not args.ui_only and not args.scenarios:
            game_mutated = True
            baseline = status
            raw_fov = probe(status, "cameraFovOption")["input"]

            tree = settings("Aspect")
            leaf(tree, "Ratio")["value"] = 175
            put_settings("Aspect", tree)
            enable("Aspect", True)
            status = wait_status(lambda s: fresh_probe(s, "clientCameraAspect", baseline)
                                 and probe(s, "clientCameraAspect").get("output") == 175)
            check("aspectActualFramePolicy", True, probe(status, "clientCameraAspect"))
            baseline = status
            enable("Aspect", False)
            status = wait_status(lambda s: fresh_probe(s, "clientCameraAspect", baseline)
                                 and probe(s, "clientCameraAspect").get("output") == 100)
            check("aspectDisabledRestores", True, probe(status, "clientCameraAspect"))
            exercised("Aspect")

            original_perspective = probe(status, "clientCameraPerspective")["input"]["rawPerspective"]
            tree = settings("QuickPerspectiveSwap")
            leaf(tree, "RearView")["value"] = False
            leaf(tree, "Bind")["value"] = {"boundKey":"key.keyboard.v", "action":"Hold"}
            put_settings("QuickPerspectiveSwap", tree)
            set_perspective(0)
            baseline = api("/client/integration-status")
            key("v", "down")
            try:
                status = wait_status(lambda s: fresh_probe(s, "clientCameraPerspective", baseline)
                                     and probe(s, "clientCameraPerspective").get("output") == 2)
                check("quickPerspectiveActualHold", probe(status, "clientCameraPerspective")["input"]["rawPerspective"] == 0,
                      probe(status, "clientCameraPerspective"))
                status = set_perspective(2)
                check("quickPerspectiveF5UsesFreshStoredOption", probe(status, "clientCameraPerspective")["output"] == 1,
                      probe(status, "clientCameraPerspective"))
            finally:
                key("v", "up")
            baseline = status
            status = wait_status(lambda s: fresh_probe(s, "clientCameraPerspective", baseline)
                                 and probe(s, "clientCameraPerspective").get("output") == 2)
            check("quickPerspectiveReleaseRestoresStoredOption", not module("QuickPerspectiveSwap")["enabled"],
                  probe(status, "clientCameraPerspective"))
            exercised("QuickPerspectiveSwap")

            tree = settings("FreeLook")
            leaf(tree, "Bind")["value"] = {"boundKey":"key.keyboard.h", "action":"Hold"}
            leaf(tree, "Perspective")["value"] = "Back"
            leaf(tree, "SenseBoost")["value"] = 1.0
            leaf(tree, "NoPitchLimit")["value"] = False
            put_settings("FreeLook", tree)
            set_perspective(0)
            control("look", "30", "10")
            baseline = api("/client/integration-status")
            key("h", "down")
            try:
                status = wait_status(lambda s: fresh_probe(s, "clientFreeLookCamera", baseline)
                                     and probe(s, "clientFreeLookCamera").get("output") == {"apply":True,"rotation":[30.0,10.0]})
                check("freeLookCapturedActualEnableAngles", module("FreeLook")["enabled"], probe(status, "clientFreeLookCamera"))
                baseline = status
                control("mouse-move", "120", "40")
                status = wait_status(lambda s: fresh_probe(s, "clientFreeLookCamera", baseline)
                                     and probe(s, "clientFreeLookCamera").get("output", {}).get("rotation", [30])[0] != 30)
                camera_rotation = probe(status, "clientFreeLookCamera")["output"]["rotation"]
                player = control("state")["player"]
                check("freeLookMouseMovesCameraAndCancelsPlayerTurn", player["yaw"] == 30 and player["pitch"] == 10,
                      {"player":player,"cameraRotation":camera_rotation})
                leaf(tree, "Perspective")["value"] = "Front"
                put_settings("FreeLook", tree)
                baseline = status
                status = wait_status(lambda s: fresh_probe(s, "clientFreeLookCamera", baseline)
                                     and all(close_number(a,b) for a,b in zip(probe(s,"clientFreeLookCamera").get("output",{}).get("rotation",[]),
                                                                                 [camera_rotation[0]+180,-camera_rotation[1]])))
                check("freeLookFrontUsesAccumulatedCameraAngles", probe(status,"clientCameraPerspective").get("output") == 2,
                      probe(status,"clientFreeLookCamera"))
                control("mouse-move", "0", "5000")
                baseline = status
                status = wait_status(lambda s: fresh_probe(s,"clientFreeLookCamera",baseline)
                                     and abs(probe(s,"clientFreeLookCamera").get("output",{}).get("rotation",[0,0])[1]) == 90)
                check("freeLookActualPitchLimit", True, probe(status,"clientFreeLookCamera"))
            finally:
                key("h", "up")
            baseline = status
            status = wait_status(lambda s: fresh_probe(s,"clientFreeLookCamera",baseline)
                                 and probe(s,"clientFreeLookCamera").get("output",{}).get("apply") is False)
            control("mouse-move", "30", "0")
            player = wait_state(lambda s: s["player"]["yaw"] != 30)["player"]
            check("freeLookReleaseRestoresRealPlayerTurn", not module("FreeLook")["enabled"], player)
            control("look", str(initial_state["player"]["yaw"]), str(initial_state["player"]["pitch"]))
            set_perspective(original_perspective)
            exercised("FreeLook")
            report["freeLookRuntimeCoverage"] = "hold_lifecycle;real_mouse_cancel;camera_back_front;pitch_limit;release"

            # Unknown registry strings deserialize to the original null key.
            # A real inventory menu has that key; AutoF5 then precedes Quick.
            tree = settings("AutoF5")
            by_type = leaf(tree, "CheckScreenHandlerType")
            leaf(by_type, "Enabled")["value"] = True
            leaf(by_type, "Types")["value"] = ["minecraft:port_test_unknown_menu"]
            leaf(by_type, "Filter")["value"] = "Whitelist"
            leaf(leaf(tree, "CheckScreenTitle"), "Enabled")["value"] = False
            put_settings("AutoF5", tree)
            enable("AutoF5", True)
            set_perspective(0)
            baseline = api("/client/integration-status")
            key("e", "tap")
            status = wait_status(lambda s: fresh_probe(s, "clientCameraPerspective", baseline)
                                 and probe(s, "clientCameraPerspective").get("input", {}).get("screen") is not None
                                 and probe(s, "clientCameraPerspective").get("output") == 1)
            check("autoF5ActualInventoryMenu", probe(status, "clientCameraPerspective")["input"]["rawPerspective"] == 0,
                  probe(status, "clientCameraPerspective"))
            baseline = status
            enable("QuickPerspectiveSwap", True)
            status = wait_status(lambda s: fresh_probe(s, "clientCameraPerspective", baseline)
                                 and probe(s, "clientCameraPerspective").get("output") == 2)
            check("autoF5RunsBeforeQuickPerspective", True, probe(status, "clientCameraPerspective"))
            enable("QuickPerspectiveSwap", False)
            key("esc", "tap")
            baseline = status
            status = wait_status(lambda s: fresh_probe(s, "clientCameraPerspective", baseline)
                                 and probe(s, "clientCameraPerspective").get("output") == 0
                                 and probe(s, "clientCameraPerspective").get("input", {}).get("screen") is None)
            check("autoF5ClosedMenuRestoresStoredOption", True, probe(status, "clientCameraPerspective"))
            enable("AutoF5", False)
            set_perspective(original_perspective)
            exercised("AutoF5")

            tree = settings("CameraClip")
            leaf(tree, "CameraDistance")["value"] = 12.0
            leaf(leaf(tree, "ScrollAdjust"), "Enabled")["value"] = False
            animation = leaf(tree, "Animation")
            leaf(animation, "Enabled")["value"] = True
            leaf(animation, "Speed")["value"] = 1.0
            leaf(animation, "Easing")["value"] = "Linear"
            put_settings("CameraClip", tree)
            set_perspective(1)
            baseline = api("/client/integration-status")
            enable("CameraClip", True)
            status = wait_status(lambda s: fresh_probe(s, "clientCameraTick", baseline)
                                 and probe(s, "clientCameraTick").get("output", {}).get("noClip") is True
                                 and close_number(probe(s, "clientCameraTick").get("output", {}).get("distance"), 12.0))
            check("cameraClipActualTickDistance", True, probe(status, "clientCameraTick"))
            leaf(animation, "Enabled")["value"] = False
            baseline = status
            put_settings("CameraClip", tree)
            status = wait_status(lambda s: fresh_probe(s, "clientCameraTick", baseline)
                                 and probe(s, "clientCameraTick").get("output", {}).get("noClip") is True
                                 and close_number(probe(s, "clientCameraTick").get("output", {}).get("distance"), 4.0))
            check("cameraClipAnimationDisabledUsesNativeDistance", True, probe(status, "clientCameraTick"))
            baseline = status
            enable("CameraClip", False)
            status = wait_status(lambda s: fresh_probe(s, "clientCameraTick", baseline)
                                 and probe(s, "clientCameraTick").get("output", {}).get("noClip") is False)
            check("cameraClipDisabledRestoresCollisionPolicy", True, probe(status, "clientCameraTick"))
            set_perspective(original_perspective)
            report["cameraClipRuntimeCoverage"] = "actual_tick_distance_and_collision_policy;wheel_and_physical_obstacle_scenarios_not_run"
            exercised("CameraClip")

            tree = settings("Twerk")
            leaf(tree, "Delay")["value"] = 3
            put_settings("Twerk", tree)
            enable("Twerk", True)
            for expected in (True, False):
                baseline = api("/client/integration-status")
                status = wait_status(lambda s: fresh_probe(s, "clientTwerkInput", baseline)
                                     and probe(s, "clientTwerkInput").get("input", {}).get("running") is True
                                     and isinstance(probe(s, "clientTwerkInput").get("input", {}).get("tickCount"), int)
                                     and (probe(s, "clientTwerkInput")["input"]["tickCount"] % 6 < 3) is expected)
                sample = probe(status, "clientTwerkInput")
                check("twerkActualSneakPhase" + str(expected), sample["output"]["shift"] is expected
                      and sample["output"]["finalShift"] is expected, sample)
            baseline = status
            enable("Twerk", False)
            status = wait_status(lambda s: fresh_probe(s, "clientTwerkInput", baseline)
                                 and probe(s, "clientTwerkInput").get("input", {}).get("running") is False)
            sample = probe(status, "clientTwerkInput")
            check("twerkDisabledPreservesPhysicalSneak", sample["output"]["shift"] == sample["input"]["shift"], sample)
            exercised("Twerk")

            tree = settings("Reach")
            leaf(leaf(tree, "Entity"), "RangeIncrease")["value"] = 2.0
            leaf(tree, "BlockRangeIncrease")["value"] = 1.25
            put_settings("Reach", tree)
            baseline = api("/client/integration-status")
            enable("Reach", True)
            status = wait_status(lambda s: fresh_probe(s, "clientEntityReach", baseline)
                                 and fresh_probe(s, "clientBlockReach", baseline)
                                 and close_number(probe(s, "clientEntityReach").get("output"),
                                                  probe(s, "clientEntityReach").get("input", {}).get("original", 0) + 2.0)
                                 and close_number(probe(s, "clientBlockReach").get("output"),
                                                  probe(s, "clientBlockReach").get("input", {}).get("original", 0) + 1.25))
            check("reachActualInteractionGetters", True,
                  {name:probe(status, name) for name in ("clientEntityReach", "clientBlockReach")})
            baseline = status
            enable("Reach", False)
            status = wait_status(lambda s: fresh_probe(s, "clientEntityReach", baseline)
                                 and close_number(probe(s, "clientEntityReach").get("output"),
                                                  probe(s, "clientEntityReach").get("input", {}).get("original", 0)))
            check("reachDisabledRestoresAttributeGetter", True, probe(status, "clientEntityReach"))
            # Through-wall selection and all item attack-range consumers still
            # need their own game scenarios; getter coverage is recorded apart.
            report["reachRuntimeCoverage"] = "interaction_getters_only"

            tree = settings("NoFOV")
            mode = leaf(tree, "Mode")
            mode["active"] = "Constant"
            constant_fov = 100 if raw_fov != 100 else 110
            leaf(mode["choices"]["Constant"], "FOV")["value"] = constant_fov
            put_settings("NoFOV", tree)
            enable("NoFOV", True)
            status = wait_status(lambda s: fresh_probe(s, "cameraFovOption", baseline)
                                 and probe(s, "cameraFovOption").get("output") == constant_fov
                                 and fresh_probe(s, "playerFovReturn", baseline)
                                 and close_number(probe(s, "playerFovReturn").get("output"), 1.0))
            check("noFovActualCamera", module("NoFOV")["enabled"], status["probes"])

            baseline = status
            tree = settings("NoFOV")
            mode = leaf(tree, "Mode")
            mode["active"] = "Custom"
            custom = mode["choices"]["Custom"]
            leaf(custom, "BaseFOV")["value"] = 0.5
            leaf(custom, "Multiplier")["value"] = 0.5
            leaf(custom, "Limit")["value"] = {"from": 0.0, "to": 1.5}
            put_settings("NoFOV", tree)
            status = wait_status(lambda s: fresh_probe(s, "playerFovReturn", baseline)
                                 and close_number(probe(s, "playerFovReturn").get("output"),
                                                  min(1.5, max(0.0, probe(s, "playerFovReturn").get("input", 0) * 0.5)))
                                 and not close_number(probe(s, "playerFovReturn").get("output"),
                                                      probe(s, "playerFovReturn").get("input", 0)))
            check("noFovActualPlayerModifier", True, status["probes"])
            baseline = status
            enable("NoFOV", False)
            status = wait_status(lambda s: fresh_probe(s, "cameraFovOption", baseline)
                                 and probe(s, "cameraFovOption").get("output") == raw_fov
                                 and fresh_probe(s, "playerFovReturn", baseline)
                                 and close_number(probe(s, "playerFovReturn").get("output"),
                                                  probe(s, "playerFovReturn").get("input", 0)))
            check("noFovDisabledRestoresHooks", True, status["probes"])

            tree = settings("Zoom")
            zoom_fov = 30 if raw_fov != 30 else 31
            leaf(tree, "Zoom")["value"] = zoom_fov
            leaf(tree, "DurationFactor")["value"] = 0
            leaf(tree, "Bind")["value"] = {"boundKey": "key.keyboard.c", "action": "Hold"}
            put_settings("Zoom", tree)
            baseline = api("/client/integration-status")
            key("c", "down")
            try:
                status = wait_status(lambda s: fresh_probe(s, "cameraFovOption", baseline)
                                     and probe(s, "cameraFovOption").get("output") == zoom_fov
                                     and fresh_probe(s, "cameraFovReturn", baseline)
                                     and s["observed"]["keyboardEvents"] > baseline["observed"]["keyboardEvents"]
                                     and module("Zoom")["enabled"])
                check("zoomRealKeyAndCamera", True, status["probes"])
                key("c", "repeat")
                check("zoomHoldRepeatKeepsEnabled", module("Zoom")["enabled"])
            finally:
                key("c", "up")
            baseline = status
            status = wait_status(lambda s: not module("Zoom")["enabled"]
                                 and fresh_probe(s, "cameraFovOption", baseline)
                                 and probe(s, "cameraFovOption").get("output") == raw_fov
                                 and fresh_probe(s, "cameraFovReturn", baseline))
            check("zoomReleaseRestoresCamera", True, status["probes"])

            tree = settings("FullBright")
            mode = leaf(tree, "Mode")
            mode["active"] = "Gamma"
            leaf(mode["choices"]["Gamma"], "Brightness")["value"] = 10
            put_settings("FullBright", tree)
            baseline = api("/client/integration-status")
            enable("FullBright", True)
            status = wait_status(lambda s: fresh_probe(s, "lightmapGammaOption", baseline)
                                 and fresh_probe(s, "clientPlayerTickPost", baseline)
                                 and probe(s, "lightmapGammaOption").get("output", 0)
                                 > probe(s, "lightmapGammaOption").get("input", 0))
            gamma = probe(status, "lightmapGammaOption")
            elapsed_ticks = probe_count(status, "clientPlayerTickPost") - probe_count(baseline, "clientPlayerTickPost")
            check("fullBrightTicksProgressively", gamma["output"] <= min(10, gamma["input"] + 0.10001 * (elapsed_ticks + 1)),
                  {"gamma": gamma, "playerTicksSinceEnable": elapsed_ticks})
            check("fullBrightActualLightmap", True, status["probes"])
            tree = settings("FullBright")
            leaf(tree, "Mode")["active"] = "NightVision"
            put_settings("FullBright", tree)
            baseline = status
            status = wait_status(lambda s: fresh_probe(s, "clientPlayerTickPost", baseline)
                                 and fresh_probe(s, "lightmapGammaOption", baseline)
                                 and close_number(probe(s, "lightmapGammaOption").get("output"),
                                                  probe(s, "lightmapGammaOption").get("input", 0)))
            effects = control("effects")
            nv = night_vision(effects)
            check("fullBrightRealNightVision", len(nv) == 1 and nv[0].get("amplifier") == 0
                  and 1335 <= nv[0].get("duration", 0) <= 1337, effects)
            control("wait", "20")
            effects = control("effects")
            nv = night_vision(effects)
            check("nightVisionRefreshedByPlayerTicks", len(nv) == 1 and nv[0].get("amplifier") == 0
                  and 1335 <= nv[0].get("duration", 0) <= 1337, effects)
            enable("FullBright", False)
            effects = control("effects")
            check("nightVisionRemoved", not night_vision(effects), effects)

            # Exercise actual walk interpolation, not just module configuration.
            check("cameraFixtureOnGround", initial_state["player"]["on_ground"]
                  and not initial_state["player"].get("flying", False), initial_state["player"])
            game_mutated = True
            px, py, pz = (math.floor(value) for value in initial_state["player"]["position"])
            # This disposable world has only deliberately summoned actors.
            # Natural night spawns must not kill a survival inventory fixture.
            control("chat", "/gamerule minecraft:spawn_mobs false")
            control("chat", "/kill @e[type=!minecraft:player]")
            # --test-terrain rolls uphill. A flat lane in this disposable test
            # world keeps the source walk active across the REST toggles.
            control("chat", f"/fill {px - 16} {py - 1} {pz - 16} {px + 16} {py - 1} {pz + 16} minecraft:stone")
            control("chat", f"/fill {px - 16} {py} {pz - 16} {px + 16} {py + 3} {pz + 16} minecraft:air")
            control("wait", "5")
            report["cameraFixture"] = {"kind": "official_command_flat_platform", "disposableWorld": True,
                                       "floor": [[px - 16, py - 1, pz - 16], [px + 16, py - 1, pz + 16]],
                                       "air": [[px - 16, py, pz - 16], [px + 16, py + 3, pz + 16]]}
            fixture_state = control("state")
            report["cameraFixture"]["state"] = fixture_state
            check("flatCameraFixtureReady", fixture_state["player"]["on_ground"]
                  and abs(fixture_state["player"]["position"][1] - py) < 0.05, fixture_state["player"])

            enable("SnapTap", True)
            for first, second, expected in (("a", "d", "right"), ("d", "a", "left"),
                                            ("w", "s", "backward"), ("s", "w", "forward")):
                baseline = api("/client/integration-status")
                key(first, "down")
                key(second, "down")
                try:
                    status = wait_status(lambda s: fresh_probe(s, "clientSnapTapInput", baseline)
                                         and probe(s, "clientSnapTapInput").get("output", {}).get(expected) is True)
                    direction = probe(status, "clientSnapTapInput")["output"]
                    opposite = {"left":"right", "right":"left", "forward":"backward", "backward":"forward"}[expected]
                    check("snapTapRealLastPressed" + expected.title(), direction[opposite] is False, direction)
                finally:
                    key(first, "up")
                    key(second, "up")
            enable("SnapTap", False)
            control("tp", *(str(value) for value in fixture_state["player"]["position"]))
            exercised("SnapTap")
            report["snapTapRuntimeCoverage"] = "real_keyboard_both_axis_press_orders;rebinding_menu_and_stale_lifecycle_covered_by_owned_fixture"

            control("chat", "/time set midnight")
            control("chat", f'/summon minecraft:zombie {px + 3} {py} {pz} {{NoAI:1b,Silent:1b,PersistenceRequired:1b}}')
            zombie = None
            deadline = time.monotonic() + 12
            while time.monotonic() < deadline:
                candidates = control("entities", "12", "minecraft:zombie")["entities"]
                if candidates:
                    zombie = candidates[0]
                    break
                control("wait", "2")
            check("hitboxActualTargetSpawned", zombie is not None, zombie)
            entity_probe = "clientEntityMargin:" + str(zombie["id"])
            hitbox_tree = settings("Hitbox")
            leaf(hitbox_tree, "Size")["value"] = 0.65
            put_settings("Hitbox", hitbox_tree)
            global_tree = json.loads(json.dumps(saved_global))
            leaf(leaf(global_tree, "Targets"), "Combat")["value"] = ["Hostile"]
            global_mutated = True
            api("/client/global", "PUT", global_tree)
            enable("Hitbox", True)
            baseline = api("/client/integration-status")
            status = wait_status(lambda s: fresh_probe(s, entity_probe, baseline)
                                 and probe(s, entity_probe).get("input", {}).get("target") is True
                                 and close_number(probe(s, entity_probe).get("output"), 0.65))
            check("hitboxActualHostilePickMargin", True, probe(status, entity_probe))
            baseline = status
            leaf(leaf(global_tree, "Targets"), "Combat")["value"] = []
            api("/client/global", "PUT", global_tree)
            status = wait_status(lambda s: fresh_probe(s, entity_probe, baseline)
                                 and probe(s, entity_probe).get("input", {}).get("target") is False
                                 and close_number(probe(s, entity_probe).get("output"), 0.0))
            check("hitboxGlobalEmptyTargetsRestoresActualPickMargin", True, probe(status, entity_probe))
            persisted_global = json.loads((run / "liquidbounce/global.json").read_text(encoding="utf-8"))
            check("globalTargetsActualFilePersistence", leaf(leaf(persisted_global, "Targets"), "Combat")["value"] == [], persisted_global)
            enable("Hitbox", False)
            api("/client/global", "PUT", saved_global)
            report["hitboxRuntimeCoverage"] = "actual_hostile_pick_radius_and_global_targets;component_and_debug_render_game_scenarios_pending"

            # These commands pass through the official game command registry;
            # no test-only inventory or packet mutations are used.
            control("chat", "/clear @s")
            control("chat", "/item replace entity @s hotbar.0 with minecraft:stick")
            control("chat", "/item replace entity @s hotbar.1 with minecraft:diamond_sword")
            control("wait", "3")
            control("select", "1")
            weapon_tree = settings("AutoWeapon")
            leaf(weapon_tree, "Preferred")["value"] = ["Sword"]
            leaf(weapon_tree, "ChangeOn")["value"] = ["OnAttack"]
            leaf(weapon_tree, "SwitchBack")["value"] = 300
            leaf(weapon_tree, "AutoMace")["value"] = False
            leaf(weapon_tree, "AutoShieldBreak")["value"] = False
            put_settings("AutoWeapon", weapon_tree)
            enable("AutoWeapon", True)
            baseline = api("/client/integration-status")
            attacked = control("attack", str(zombie["id"]))
            check("autoWeaponActualAttackTargeted", attacked.get("hit") is True, attacked)
            # The controller reports after four client ticks, independently of
            # the integrated server thread. Observe its authoritative metadata
            # before deciding whether the ordinary attack damaged the target.
            damaged = wait_control("entities", lambda r: any(
                e.get("id") == zombie["id"]
                and isinstance(e.get("health"), (int, float))
                and e["health"] < attacked["health_before"]
                for e in r["entities"]), "12", "minecraft:zombie")
            check("autoWeaponActualAttackDamagedTarget", True,
                  {"attack": attacked, "authoritativeEntities": damaged})
            status = wait_status(lambda s: fresh_probe(s, "clientAttackEntity", baseline)
                                 and probe(s, "clientAttackEntity").get("output", {}).get("slot") == 1
                                 and (s.get("silentHotbar") or {}).get("enforced") == 1)
            check("autoWeaponActualSilentSwordSelection", status["silentHotbar"]["client"] == 0, status["silentHotbar"])
            enable("SilentHotbar", False)
            baseline = status
            status = wait_status(lambda s: fresh_probe(s, "clientSilentHotbarRender", baseline)
                                 and probe(s, "clientSilentHotbarRender").get("output", {}).get("visibleSlot") == 1)
            check("silentHotbarDisabledShowsServerItem", True, probe(status, "clientSilentHotbarRender"))
            enable("SilentHotbar", True)
            baseline = status
            status = wait_status(lambda s: fresh_probe(s, "clientSilentHotbarRender", baseline)
                                 and probe(s, "clientSilentHotbarRender").get("output", {}).get("visibleSlot") == 0)
            check("silentHotbarActualClientItemAndCooldown", close_number(
                probe(status, "clientSilentHotbarRender")["output"]["swapScale"], 1.0),
                probe(status, "clientSilentHotbarRender"))
            enable("AutoWeapon", False)
            status = wait_status(lambda s: s.get("silentHotbar") is None)
            check("autoWeaponDisableRestoresOwnedSlot", control("state")["selected"] == 1, status.get("silentHotbar"))
            enable("SilentHotbar", False)
            exercised("AutoWeapon")
            exercised("SilentHotbar")
            report["autoWeaponRuntimeCoverage"] = "actual_OnAttack_sword_choice_damage_and_owner_disable;OnTarget_and_other_combat_module_dependencies_pending"

            control("chat", "/item replace entity @s hotbar.1 with minecraft:diamond_pickaxe")
            control("wait", "3")
            tool_tree = settings("AutoTool")
            leaf(tool_tree, "NotDuringCombat")["value"] = False
            leaf(tool_tree, "RequireSneaking")["value"] = False
            leaf(leaf(tool_tree, "RequireNearBed"), "Enabled")["value"] = False
            leaf(tool_tree, "Filter")["value"] = "Blacklist"
            leaf(tool_tree, "Blocks")["value"] = []
            leaf(tool_tree, "SwitchDelay")["value"] = 0
            leaf(tool_tree, "SwapBackDelay")["value"] = 100
            selector = leaf(tool_tree, "ToolSelector")
            selector["active"] = "Dynamic"
            leaf(leaf(selector["choices"]["Dynamic"], "ConsiderInventory"), "Enabled")["value"] = False
            put_settings("AutoTool", tool_tree)
            control("chat", "/gamemode survival")
            wait_state(lambda s: s.get("gamemode") == "survival")
            mining_pos = [px + 1, py, pz + 1]
            control("chat", "/setblock " + " ".join(map(str, mining_pos)) + " minecraft:stone")
            wait_control("block", lambda b: b.get("block") == "minecraft:stone", *map(str, mining_pos))
            control("select", "1")
            baseline = api("/client/integration-status")
            enable("AutoTool", True)
            mined = control("break", *map(str, mining_pos), "--timeout", "8")
            status = wait_status(lambda s: fresh_probe(s, "clientBlockBreakingProgress", baseline)
                                 and probe(s, "clientBlockBreakingProgress").get("output", {}).get("slot") == 1)
            check("autoToolActualDynamicPickaxeMining", mined.get("ok") is True,
                  {"mining":mined,"progress":probe(status, "clientBlockBreakingProgress")})
            enable("AutoTool", False)
            wait_status(lambda s: s.get("silentHotbar") is None)
            selector["active"] = "Static"
            leaf(selector["choices"]["Static"], "Slot")["value"] = 1
            put_settings("AutoTool", tool_tree)
            control("chat", "/setblock " + " ".join(map(str, mining_pos)) + " minecraft:stone")
            wait_control("block", lambda b: b.get("block") == "minecraft:stone", *map(str, mining_pos))
            baseline = api("/client/integration-status")
            enable("AutoTool", True)
            mined = control("break", *map(str, mining_pos), "--timeout", "8")
            status = wait_status(lambda s: fresh_probe(s, "clientBlockBreakingProgress", baseline)
                                 and probe(s, "clientBlockBreakingProgress").get("output", {}).get("slot") == 1)
            check("autoToolActualStaticPickaxeMining", mined.get("ok") is True,
                  {"mining":mined,"progress":probe(status, "clientBlockBreakingProgress")})
            enable("AutoTool", False)
            wait_status(lambda s: s.get("silentHotbar") is None)
            control("chat", "/gamemode creative")
            wait_state(lambda s: s.get("gamemode") == "creative")
            exercised("AutoTool")
            report["autoToolRuntimeCoverage"] = "actual_Dynamic_and_Static_hotbar_mining;inventory_chains_and_special_block_scenarios_pending"

            # Exercise the shared manager with real creative fill/throw and
            # survival container clicks. Armor and offhand are not storage.
            control("chat", "/kill @e[type=minecraft:item,distance=..12]")
            control("chat", "/clear @s")
            control("chat", "/item replace entity @s hotbar.0 with minecraft:cobblestone 7")
            control("chat", "/item replace entity @s inventory.0 with minecraft:dirt 11")
            control("chat", "/item replace entity @s weapon.offhand with minecraft:torch 3")
            control("chat", "/item replace entity @s armor.head with minecraft:carved_pumpkin")
            control("chat", "/gamemode survival")
            wait_state(lambda s: s.get("gamemode") == "survival")
            control("look", "0", "-25")
            vomit_tree = settings("Vomit")
            constraints = leaf(vomit_tree, "Constraints")
            for delay in ("StartDelay", "ClickDelay", "CloseDelay", "MissChance"):
                leaf(constraints, delay)["value"] = {"from": 0, "to": 0}
            leaf(constraints, "SlotMultiplier")["value"] = 0.0
            leaf(constraints, "Requires")["value"] = ["InventoryOpen"]
            put_settings("Vomit", vomit_tree)
            enable("Vomit", True)
            control("wait", "5")
            inventory = control("inventory")
            check("vomitRealInventoryOpenConstraint", inventory["hotbar"][0].get("item") == "minecraft:cobblestone"
                  and inventory["main"][0].get("item") == "minecraft:dirt", inventory)
            key("e", "tap")
            wait_state(lambda s: s.get("screen") == "inventory")
            inventory = wait_control("inventory", lambda i: all(not slot.get("item") for slot in i["hotbar"] + i["main"]))
            enable("Vomit", False)
            check("vomitRealSurvivalStorageDropped", inventory["offhand"].get("item") == "minecraft:torch"
                  and inventory["offhand"].get("count") == 3
                  and inventory["armor"][0].get("item") == "minecraft:carved_pumpkin", inventory)
            drops = wait_control("entities", lambda r: {e.get("item", {}).get("item") for e in r["entities"]}
                                 >= {"minecraft:cobblestone", "minecraft:dirt"}, "12", "minecraft:item")
            check("vomitRealServerItemEntities", True, drops)
            key("e", "tap")
            wait_state(lambda s: s.get("screen") == "playing")
            control("chat", "/clear @s")
            control("chat", "/kill @e[type=minecraft:item,distance=..12]")
            control("chat", "/gamemode creative")
            wait_state(lambda s: s.get("gamemode") == "creative")
            leaf(constraints, "Requires")["value"] = []
            leaf(constraints, "ClickDelay")["value"] = {"from": 20, "to": 20}
            put_settings("Vomit", vomit_tree)
            enable("Vomit", True)
            drops = wait_control("entities", lambda r: any(e.get("item", {}).get("count") == 64 for e in r["entities"]),
                                 "12", "minecraft:item")
            enable("Vomit", False)
            check("vomitRealCreativeRandomBlockFillAndThrow", True, drops)
            exercised("Vomit")
            report["vomitRuntimeCoverage"] = "real_inventory_open_constraint;survival_hotbar_and_main_throw;armor_offhand_preserved;creative_random_block_fill_and_throw"
            control("chat", "/clear @s")
            control("chat", "/kill @e[type=minecraft:item,distance=..12]")

            # Single-player joins its independent integrated server through a
            # real memory connection. The module's ordinary attack packets
            # must destroy a real vehicle on that server, not just be counted.
            control("chat", "/gamemode survival")
            wait_state(lambda s: s.get("gamemode") == "survival")
            control("tp", str(px + 0.5), str(py), str(pz + 0.5))
            vehicle_samples = []
            for vehicle in ("minecraft:oak_boat", "minecraft:minecart"):
                for running in (False, True):
                    control("chat", "/kill @e[tag=lb_vehicle_test]")
                    control("chat", "/kill @e[type=minecraft:item,distance=..12]")
                    control("chat", f'/summon {vehicle} {px + 0.5} {py} {pz + 2.5} {{NoGravity:1b,Tags:["lb_vehicle_test"]}}')
                    control("wait", "5")
                    control("look-at", str(px + 0.5), str(py + 0.25), str(pz + 2.5))
                    enable("VehicleOneHit", running)
                    baseline = api("/client/integration-status")
                    control("attack")
                    status = wait_status(lambda s: fresh_probe(s, "clientAttackEntity", baseline))
                    attack_probe = probe(status, "clientAttackEntity")
                    check("vehicleOneHitNativeActions" + vehicle.split(":")[1] + str(running),
                          attack_probe.get("output", {}).get("vehicleActions") == (38 if running else 0), attack_probe)
                    if running:
                        drops = wait_control("entities", lambda r: any(e.get("item", {}).get("item") == vehicle for e in r["entities"]),
                                             "12", "minecraft:item")
                    else:
                        control("wait", "5")
                        drops = control("entities", "12", "minecraft:item")
                        check("vehicleOneHitDisabledVehicleSurvives" + vehicle.split(":")[1],
                              not any(e.get("item", {}).get("item") == vehicle for e in drops["entities"]), drops)
                    vehicle_samples.append({"type": vehicle, "enabled": running, "attack": attack_probe, "drops": drops})
            enable("VehicleOneHit", False)
            control("chat", "/kill @e[tag=lb_vehicle_test]")
            control("chat", "/kill @e[type=minecraft:item,distance=..12]")
            control("chat", "/gamemode creative")
            wait_state(lambda s: s.get("gamemode") == "creative")
            control("tp", *(str(value) for value in fixture_state["player"]["position"]))
            exercised("VehicleOneHit")
            report["vehicleOneHitRuntimeCoverage"] = {"scope": "actual_integrated_memory_connection;oak_boat_and_minecart_server_destruction;disabled_control", "samples": vehicle_samples}

            chams_tree = settings("Chams")
            chams_mode = leaf(chams_tree, "Mode")
            chams_mode["active"] = "Normal"
            put_settings("Chams", chams_tree)
            leaf(leaf(global_tree, "Targets"), "Visual")["value"] = ["Hostile"]
            api("/client/global", "PUT", global_tree)
            enable("Chams", True)
            chams_probe = "clientChamsEntity:" + str(zombie["id"])
            baseline = api("/client/integration-status")
            control("look-at", "entity", str(zombie["id"]))
            status = wait_status(lambda s: fresh_probe(s, chams_probe, baseline) and probe(s, chams_probe).get("output") is True)
            check("chamsActualHostileVisualTarget", True, probe(status, chams_probe))
            baseline = status
            leaf(leaf(global_tree, "Targets"), "Visual")["value"] = []
            api("/client/global", "PUT", global_tree)
            status = wait_status(lambda s: fresh_probe(s, chams_probe, baseline) and probe(s, chams_probe).get("output") is False)
            check("chamsActualVisualTargetExclusion", True, probe(status, chams_probe))
            leaf(leaf(global_tree, "Targets"), "Visual")["value"] = ["Hostile"]
            api("/client/global", "PUT", global_tree)
            picture = run / "chams-normal.png"
            for _ in range(2):
                control("wait", "110")
            control("screenshot", str(picture))
            # Image only replaces occluded pixels. Establish a real wall and a
            # disabled control before assessing its material through that wall.
            wall_start = [px + 1, py, pz - 1]
            wall_end = [px + 1, py + 3, pz + 1]
            control("chat", f"/fill {' '.join(map(str, wall_start))} {' '.join(map(str, wall_end))} minecraft:stone")
            wall_block = wait_control("block", lambda s: s.get("block") == "minecraft:stone",
                                     *(str(v) for v in [px + 1, py + 1, pz]))
            check("chamsActualOccludingWall", True, wall_block)
            enable("Chams", False)
            # client_chams_entity deliberately exits before dispatch whenever
            # capture is inactive. Its old target sample is then historical.
            check("chamsDisabledForOccludedControl", module("Chams")["enabled"] is False)
            control("wait", "5")
            disabled_wall_picture = run / "chams-disabled-occluded.png"
            control("screenshot", str(disabled_wall_picture))
            enable("Chams", True)
            baseline = api("/client/integration-status")
            wait_status(lambda s: fresh_probe(s, chams_probe, baseline)
                        and probe(s, chams_probe).get("output") is True)
            control("wait", "5")
            normal_wall_picture = run / "chams-normal-occluded.png"
            control("screenshot", str(normal_wall_picture))
            fixture = run / "liquidbounce/chams-fixture.png"
            fixture.parent.mkdir(exist_ok=True)
            def png_chunk(kind, data):
                return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)
            pixels = b"".join(b"\x00" + b"".join(bytes((255, 40, 180, 255) if (x // 4 + y // 4) % 2 else (20, 220, 255, 255))
                                                     for x in range(32)) for y in range(32))
            fixture.write_bytes(b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", struct.pack(">IIBBBBB",32,32,8,6,0,0,0))
                                + png_chunk(b"IDAT", zlib.compress(pixels)) + png_chunk(b"IEND", b""))
            chams_mode["active"] = "Image"
            leaf(chams_mode["choices"]["Image"], "File")["value"] = str(fixture)
            put_settings("Chams", chams_tree)
            stored_file = leaf(leaf(settings("Chams"), "Mode")["choices"]["Image"], "File")["value"]
            check("chamsActualFilePathNormalization", stored_file == "chams-fixture.png", stored_file)
            control("wait", "3")
            image_picture = run / "chams-image.png"
            control("screenshot", str(image_picture))
            report["chamsScreenshotsPendingVisualReview"] = [str(picture), str(disabled_wall_picture),
                                                           str(normal_wall_picture), str(image_picture)]
            enable("Chams", False)
            control("chat", f"/fill {' '.join(map(str, wall_start))} {' '.join(map(str, wall_end))} minecraft:air")
            wait_control("block", lambda s: s.get("block") == "minecraft:air",
                         *(str(v) for v in [px + 1, py + 1, pz]))
            api("/client/global", "PUT", saved_global)
            exercised("Chams")
            report["chamsRuntimeCoverage"] = "actual_visual_target_filter;visible_Normal;occluded_disabled_Normal_Image_frames;visual_review_pending"

            anti_tree = settings("AntiAFK")
            leaf(anti_tree, "Mode")["active"] = "Old"
            put_settings("AntiAFK", anti_tree)
            control("look", "0", "0")
            enable("AntiAFK", True)
            control("wait", "12")
            anti_state = control("state")
            enable("AntiAFK", False)
            # The controller round trip can span another coroutine wake. Old
            # adds 180 degrees each cycle and intentionally leaves yaw unwrapped.
            anti_yaw = anti_state["player"]["yaw"]
            check("antiAFKActualOldYawTurn", anti_yaw >= 180
                  and close_number(anti_yaw / 180, round(anti_yaw / 180)), anti_state["player"])
            exercised("AntiAFK")
            report["antiAFKRuntimeCoverage"] = "actual_Old_wait_and_native_yaw;RandomInteraction_Custom_live_scenarios_pending"

            # Exercise real TridentItem use/release paths on dry land. The
            # callback exposes the original push arguments before velocity adds.
            control("chat", "/weather clear")
            control("chat", f"/tp @s {px + 0.5} {py} {pz + 0.5}")
            control("chat", '/item replace entity @s hotbar.0 with minecraft:trident[minecraft:enchantments={"minecraft:riptide":3}]')
            control("select", "1")
            control("look", "0", "-30")
            control("wait", "3")
            baseline = api("/client/integration-status")
            control("use-item", "--hold", "0.65")
            status = wait_status(lambda s: fresh_probe(s, "clientTridentWater", baseline))
            check("tridentDisabledDryUseRejected", probe(status, "clientTridentWater").get("input") is False
                  and probe(status, "clientTridentWater").get("output") is False
                  and not fresh_probe(status, "clientTridentImpulse", baseline), probe(status, "clientTridentWater"))
            trident_tree = settings("TridentBoost")
            leaf(trident_tree, "HorizontalMultiplier")["value"] = 1.75
            leaf(trident_tree, "VerticalMultiplier")["value"] = 3.25
            leaf(trident_tree, "OnLand")["value"] = True
            put_settings("TridentBoost", trident_tree)
            enable("TridentBoost", True)
            baseline = status
            control("use-item", "--hold", "0.65")
            status = wait_status(lambda s: fresh_probe(s, "clientTridentImpulse", baseline))
            impulse = probe(status, "clientTridentImpulse")
            original, boosted = impulse.get("input", []), impulse.get("output", [])
            check("tridentActualIndependentImpulseMultipliers", len(original) == len(boosted) == 3
                  and all(close_number(boosted[i], original[i] * (3.25 if i == 1 else 1.75)) for i in range(3))
                  and abs(original[1]) > 0.1 and abs(original[2]) > 0.1, impulse)
            check("tridentActualDryWaterOverride", probe(status, "clientTridentWater").get("input") is False
                  and probe(status, "clientTridentWater").get("output") is True, probe(status, "clientTridentWater"))
            control("wait", "25")
            control("chat", f"/tp @s {px + 0.5} {py} {pz + 0.5}")
            leaf(trident_tree, "OnLand")["value"] = False
            put_settings("TridentBoost", trident_tree)
            baseline = api("/client/integration-status")
            control("use-item", "--hold", "0.65")
            status = wait_status(lambda s: fresh_probe(s, "clientTridentWater", baseline))
            check("tridentOnLandFalseRestoresDryGate", probe(status, "clientTridentWater").get("output") is False
                  and not fresh_probe(status, "clientTridentImpulse", baseline), probe(status, "clientTridentWater"))
            enable("TridentBoost", False)
            exercised("TridentBoost")
            report["tridentBoostRuntimeCoverage"] = "actual_dry_use_gate_release_push_and_independent_multipliers;rain_and_water_fixture_pending"

            derp_tree = settings("Derp")
            yaw_mode, pitch_mode = leaf(derp_tree, "Yaw"), leaf(derp_tree, "Pitch")
            yaw_mode["active"], pitch_mode["active"] = "Static", "Static"
            leaf(yaw_mode["choices"]["Static"], "Yaw")["value"] = 40
            leaf(pitch_mode["choices"]["Static"], "Pitch")["value"] = -35
            leaf(derp_tree, "NotDuringSprint")["value"] = False
            put_settings("Derp", derp_tree)
            baseline = api("/client/integration-status")
            enable("Derp", True)
            status = wait_status(lambda s: fresh_probe(s, "clientDerpTick", baseline)
                                 and probe(s, "clientDerpTick").get("output") == [40.0, -35.0])
            check("derpActualStaticRotationRequest", True, probe(status, "clientDerpTick"))
            leaf(derp_tree, "NotDuringSprint")["value"] = True
            put_settings("Derp", derp_tree)
            key("ctrl", "down")
            try:
                baseline = status
                status = wait_status(lambda s: fresh_probe(s, "clientDerpTick", baseline)
                                     and probe(s, "clientDerpTick").get("input", {}).get("sprintKey") is True
                                     and probe(s, "clientDerpTick").get("output") is None)
                check("derpActualSprintKeyGuard", True, probe(status, "clientDerpTick"))
            finally:
                key("ctrl", "up")
            enable("Derp", False)
            exercised("Derp")
            report["derpRuntimeCoverage"] = "actual_Static_yaw_pitch_and_logical_sprint_guard;all_other_modes_have_source_oracle_only"


            tree = settings("SmoothCamera")
            leaf(tree, "EnableFirstPOV")["value"] = True
            leaf(tree, "HorizontalFactor")["value"] = 1.0
            leaf(tree, "VerticalFactor")["value"] = 1.0
            put_settings("SmoothCamera", tree)
            perspective_before_smooth = probe(api("/client/integration-status"), "clientCameraPerspective")["input"]["rawPerspective"]
            set_perspective(0)
            baseline = api("/client/integration-status")
            enable("SmoothCamera", True)
            status = wait_status(lambda s: fresh_probe(s, "clientCameraPosition", baseline))
            frozen_camera = probe(status, "clientCameraPosition")["output"]
            control("tp", str(frozen_camera[0] + 2), str(py), str(frozen_camera[2]))
            baseline = status
            status = wait_status(lambda s: fresh_probe(s, "clientCameraPosition", baseline)
                                 and abs(probe(s, "clientCameraPosition")["input"]["position"][0] - frozen_camera[0]) > 1
                                 and probe(s, "clientCameraPosition")["output"] == frozen_camera)
            check("smoothCameraActualSetPosition", True, probe(status, "clientCameraPosition"))
            baseline = status
            enable("SmoothCamera", False)
            status = wait_status(lambda s: fresh_probe(s, "clientCameraPosition", baseline)
                                 and probe(s, "clientCameraPosition")["input"]["position"] == probe(s, "clientCameraPosition")["output"])
            check("smoothCameraDisableRestoresActualPosition", True, probe(status, "clientCameraPosition"))
            control("tp", *(str(value) for value in fixture_state["player"]["position"]))
            set_perspective(perspective_before_smooth)
            exercised("SmoothCamera")

            # The rod is equipped through the actual command/inventory path;
            # no callback or artificial splash is injected into the module.
            before_fishing = control("state")
            control("chat", "/item replace entity @s weapon.mainhand with minecraft:fishing_rod")
            wait_state(lambda s: (s.get("held") or {}).get("item") == "minecraft:fishing_rod")
            tree = settings("AutoFish")
            leaf(leaf(tree, "AutoCastRod"), "Enabled")["value"] = True
            leaf(leaf(tree, "AutoCastRod"), "Delay")["value"] = {"from":0,"to":0}
            leaf(leaf(tree, "RecastRod"), "Enabled")["value"] = False
            put_settings("AutoFish", tree)
            baseline = api("/client/integration-status")
            control("look", "0", "0")
            enable("AutoFish", True)
            status = wait_status(lambda s: fresh_probe(s, "autoFishUseItem", baseline)
                                 and probe(s, "autoFishObserved").get("output", {}).get("hook") is not None)
            hook = probe(status, "autoFishObserved").get("output", {}).get("hook")
            check("autoFishActualUseCreatedHook", isinstance(hook, list) and len(hook) == 3
                  and all(isinstance(value, (int, float)) and math.isfinite(value) for value in hook),
                  {"use":probe(status, "autoFishUseItem"),"actualHook":probe(status, "autoFishObserved")})
            enable("AutoFish", False)
            baseline = status
            control("mouse", "right", "click")
            status = wait_status(lambda s: fresh_probe(s, "autoFishObserved", baseline)
                                 and probe(s, "autoFishObserved").get("output", {}).get("hook") is None)
            check("manualReelRemovedActualHook", True, probe(status, "autoFishObserved"))
            old_held = before_fishing.get("held")
            restore_item = "minecraft:air"
            if old_held:
                components = json.loads(old_held.get("components", "{}"))
                component_text = ",".join(name + "=" + json.dumps(value, ensure_ascii=False, separators=(",", ":"))
                                          for name, value in components.items())
                restore_item = old_held["item"] + ("[" + component_text + "]" if component_text else "")
                restore_item += " " + str(old_held["count"])
            control("chat", "/item replace entity @s weapon.mainhand with " + restore_item)
            wait_state(lambda s: s.get("held") == old_held)
            report["autoFishRuntimeCoverage"] = "automatic_cast_and_real_local_hook_only"

            control("look", str(initial_state["player"]["yaw"]), "0")
            walk_start = control("state")["player"]["position"]
            baseline = api("/client/integration-status")
            key("w", "down")
            try:
                control("wait", "8")
                status = wait_status(lambda s: fresh_camera(s, baseline) and walking_sample(s)
                                     and camera_branches(s, True, True)
                                     and matrix_equal(bob_result(s)[1].get("bob"), bob_result(s)[0].get("original")))
                moved = control("state")
                distance = math.hypot(moved["player"]["position"][0] - walk_start[0],
                                      moved["player"]["position"][2] - walk_start[2])
                check("walkingCameraSource", distance > 0.05 and moved["player"]["on_ground"],
                      {"distance": distance, "camera": probe(status, "cameraBobResult")})
                walking_before = probe(status, "cameraBobResult")
                baseline = status
                enable("NoBob", True)
                status = wait_status(lambda s: fresh_camera(s, baseline) and walking_sample(s)
                                     and camera_branches(s, True, False)
                                     and matrix_equal(bob_result(s)[1].get("bob"), IDENTITY))
                walking_cancelled = probe(status, "cameraBobResult")
                baseline = status
                enable("NoBob", False)
                status = wait_status(lambda s: fresh_camera(s, baseline) and walking_sample(s)
                                     and camera_branches(s, True, True)
                                     and matrix_equal(bob_result(s)[1].get("bob"), bob_result(s)[0].get("original")))
                check("noBobActualCamera", True,
                      {"before": walking_before, "cancelled": walking_cancelled,
                       "restored": probe(status, "cameraBobResult")})
                baseline = status
                enable("NoHurtCam", True)
                status = wait_status(lambda s: fresh_camera(s, baseline) and walking_sample(s)
                                     and camera_branches(s, False, True)
                                     and matrix_equal(bob_result(s)[1].get("bob"), bob_result(s)[0].get("original")))
                check("noHurtCamPreservesWalking", True, probe(status, "cameraBobResult"))
            finally:
                key("w", "up")
                control("release")

            # NoBob isolates hurt/death from residual walk bob. The normal server
            # command must change real health as well as animate the renderer.
            enable("NoBob", True)
            enable("NoHurtCam", False)

            # Exercise the original Entity.turn path, including the disable
            # callback's clamp, through official structured input.
            enable("NoPitchLimit", False)
            control("look", "0", "80")
            control("turn", "0", "50")
            check("vanillaPitchClamped", close_number(control("state")["player"]["pitch"], 90.0))
            enable("NoPitchLimit", True)
            control("turn", "0", "50")
            check("noPitchLimitActualEntityTurn", close_number(control("state")["player"]["pitch"], 140.0))
            enable("NoPitchLimit", False)
            check("noPitchLimitDisableClamps", close_number(control("state")["player"]["pitch"], 90.0))
            report["behaviorModulesExercised"].append("NoPitchLimit")
            report["behaviorModulesNotExercised"].remove("NoPitchLimit")

            control("look", "0", "-90")
            tree = settings("NoSwing")
            leaf(tree, "HideFor")["value"] = ["Client"]
            put_settings("NoSwing", tree)
            enable("NoSwing", True)
            control("wait", "12")
            before_swings = control("state")["swings"]
            control("mouse", "left", "click")
            control("wait", "2")
            check("noSwingClientCancelsActualSwing", control("state")["swings"] == before_swings)
            leaf(tree, "HideFor")["value"] = ["Server"]
            put_settings("NoSwing", tree)
            control("wait", "12")
            control("mouse", "left", "click")
            control("wait", "2")
            check("noSwingServerOnlyPreservesClientSwing", control("state")["swings"] > before_swings)
            enable("NoSwing", False)
            report["noSwingLiveScope"] = "client swing only; packet cancellation is independently covered by the real-codec fixture"
            control("tp", *(str(value) for value in initial_state["player"]["position"]))
            control("wait", "21")
            control("chat", "/gamemode survival")
            wait_state(lambda s: s.get("gamemode") == "survival" and s["player"]["on_ground"])

            def damage_camera(cancelled):
                enable("NoHurtCam", cancelled)
                control("wait", "21")
                health = control("state")["player"]["health"]
                if health <= 2:
                    raise AssertionError("Damage fixture needs more than two health points")
                before = api("/client/integration-status")
                control("chat", "/damage @s 1 minecraft:generic")
                current = wait_status(lambda s: fresh_camera(s, before) and hurt_sample(s)
                                      and camera_branches(s, not cancelled, False)
                                      and (matrix_equal(bob_result(s)[1].get("bob"), IDENTITY) if cancelled
                                           else nonidentity(bob_result(s)[1].get("bob"))), seconds=3)
                damaged = wait_state(lambda s: 0 < s["player"]["health"] < health, seconds=3)
                return {"healthBefore": health, "healthAfter": damaged["player"]["health"],
                        "branches": probe(current, "cameraBobEffects"), "camera": probe(current, "cameraBobResult")}

            hurt_before = damage_camera(False)
            check("noBobPreservesActualHurt", True, hurt_before)
            hurt_cancelled = damage_camera(True)
            hurt_restored = damage_camera(False)
            check("noHurtCamActualCamera", True,
                  {"before": hurt_before, "cancelled": hurt_cancelled, "restored": hurt_restored})

            baseline = api("/client/integration-status")
            control("chat", "/kill @s")
            wait_state(lambda s: s.get("screen") == "dead" and s["player"]["health"] <= 0)
            control("wait", "21")
            status = wait_status(lambda s: fresh_camera(s, baseline)
                                 and bob_result(s)[0].get("deathTime") is not None
                                 and bob_result(s)[0]["deathTime"] >= 20
                                 and nonidentity(bob_result(s)[0].get("original"))
                                 and nonidentity(bob_result(s)[1].get("bob")))
            death_before = probe(status, "cameraBobResult")
            death_fov = probe(status, "cameraFovReturn")
            baseline = status
            enable("NoHurtCam", True)
            status = wait_status(lambda s: fresh_camera(s, baseline)
                                 and bob_result(s)[0].get("deathTime") is not None
                                 and bob_result(s)[0]["deathTime"] >= 20
                                 and nonidentity(bob_result(s)[0].get("original"))
                                 and camera_branches(s, False, False)
                                 and matrix_equal(bob_result(s)[1].get("bob"), IDENTITY)
                                 and fresh_probe(s, "cameraFovReturn", baseline))
            death_cancelled = probe(status, "cameraBobResult")
            check("noHurtCamPreservesDeathFov",
                  close_number(probe(status, "cameraFovReturn").get("input"), death_fov.get("input"))
                  and close_number(probe(status, "cameraFovReturn").get("output"), death_fov.get("output"))
                  and close_number(death_fov.get("input"), death_fov.get("output")),
                  {"before": death_fov, "after": probe(status, "cameraFovReturn")})
            baseline = status
            enable("NoHurtCam", False)
            status = wait_status(lambda s: fresh_camera(s, baseline)
                                 and bob_result(s)[0].get("deathTime") is not None
                                 and bob_result(s)[0]["deathTime"] >= 20
                                 and camera_branches(s, True, False)
                                 and nonidentity(bob_result(s)[1].get("bob")))
            check("noHurtCamActualDeathRoll", True,
                  {"before": death_before, "cancelled": death_cancelled,
                   "restored": probe(status, "cameraBobResult")})
            control("respawn")
            wait_state(lambda s: s.get("screen") == "playing" and s["player"]["health"] > 0)
            control("chat", "/gamemode " + initial_state["gamemode"])
            wait_state(lambda s: s.get("gamemode") == initial_state["gamemode"])
            control("tp", *(str(value) for value in initial_state["player"]["position"]))
            control("look", str(initial_state["player"]["yaw"]), str(initial_state["player"]["pitch"]))

            control("effect", "nausea", "30")
            control("wait", "160")
            effects = control("effects")
            check("actualNauseaEffect", any(item.get("id", "").removeprefix("minecraft:") == "nausea"
                  and item.get("duration", 0) > 60 for item in active_effects(effects)), effects)
            nausea_samples = []
            for no_bob, no_hurt in ((False, False), (True, False), (False, True), (True, True)):
                enable("NoBob", no_bob)
                enable("NoHurtCam", no_hurt)
                baseline = api("/client/integration-status")
                status = wait_status(lambda s: fresh_probe(s, "cameraSpinningResult", baseline)
                                     and fresh_camera(s, baseline)
                                     and camera_branches(s, not no_hurt, not no_bob)
                                     and probe(s, "cameraSpinningResult").get("input", {}).get("portal") == 0
                                     and probe(s, "cameraSpinningResult").get("input", {}).get("nausea", 0) > 0.9
                                     and close_number(probe(s, "cameraSpinningResult").get("input", {}).get("scale"), 1)
                                     and probe(s, "cameraSpinningResult").get("input", {}).get("intensity", 0) > 0.9
                                     and nonidentity(probe(s, "cameraSpinningResult").get("output", {}).get("matrix")))
                nausea_samples.append({"noBob": no_bob, "noHurtCam": no_hurt,
                                       "spin": probe(status, "cameraSpinningResult"),
                                       "camera": probe(status, "cameraBobResult")})
            check("cameraNauseaPreserved", True, nausea_samples)
            report["nativeShakeValidation"] = {"status": "not_exercised",
                "reason": "The fixed runtime has no official ctl producer of nonzero native camera shake; real bob*shake composition was checked."}
            control("effect", "clear")
            check("testNauseaRemoved", not active_effects(control("effects")))
            enable("NoBob", False)
            enable("NoHurtCam", False)

            # Drain real hunger through the official effect command, then let
            # SmartEat select and consume food without a physical use press.
            control("chat", "/gamemode survival")
            wait_state(lambda s: s.get("gamemode") == "survival")
            control("chat", "/clear @s")
            control("chat", "/item replace entity @s hotbar.0 with minecraft:stick")
            control("chat", "/item replace entity @s hotbar.5 with minecraft:cooked_beef 4")
            control("select", "1")
            control("look", "0", "-90")
            # Sample the food threshold as it happens. Amplifier 255 can drain
            # the whole bar and kill the player before slow controller round
            # trips reach the module setup on a loaded development machine.
            control("chat", "/effect give @s minecraft:hunger 30 63 true")
            hungry = wait_state(lambda s: s["player"]["food"] < 18)
            control("chat", "/effect clear @s minecraft:hunger")
            smart_tree = settings("SmartEat")
            leaf(smart_tree,"NotDuringCombat")["value"] = False
            leaf(leaf(smart_tree,"SilentOffhand"),"Enabled")["value"] = False
            leaf(leaf(smart_tree,"AutoEat"),"Enabled")["value"] = True
            leaf(leaf(smart_tree,"AutoEat"),"MinHunger")["value"] = 18
            put_settings("SmartEat",smart_tree)
            baseline = api("/client/integration-status")
            enable("SmartEat",True)
            try:
                status = wait_status(lambda s: fresh_probe(s,"smartEatSelection",baseline)
                                     and probe(s,"smartEatSelection").get("input",{}).get("slot") == 5)
                fed = wait_state(lambda s: s["player"]["food"] > 18, seconds=20)
                check("smartEatActualFoodConsumption", fed["player"]["food"] > hungry["player"]["food"],
                      {"before":hungry["player"],"after":fed["player"],"selection":probe(status,"smartEatSelection")})
            finally:
                enable("SmartEat",False)
            exercised("SmartEat")
            report["smartEatRuntimeCoverage"] = "actual_hunger_auto_eat_silent_selection_and_consumption;pass_offhand_and_overlay_game_scenarios_pending"

            # Read the actual stored options that SkinDerp changes. A disabled
            # module restores its enable-time set, including unselected parts.
            part_keys = ["modelPart_" + name for name in
                         ("cape", "jacket", "left_sleeve", "right_sleeve", "left_pants_leg", "right_pants_leg", "hat")]
            old_parts = {name: entry["value"] for name, entry in control("options-get", *part_keys)["options"].items()}
            skin_tree = settings("SkinDerp")
            leaf(skin_tree, "Sync")["value"] = True
            leaf(skin_tree, "Delay")["value"] = 20
            leaf(skin_tree, "Parts")["value"] = ["Hat"]
            put_settings("SkinDerp", skin_tree)
            enable("SkinDerp", True)
            deadline = time.monotonic() + 12
            while True:
                actual_parts = {name: entry["value"] for name, entry in control("options-get", *part_keys)["options"].items()}
                if actual_parts["modelPart_hat"] != old_parts["modelPart_hat"]:
                    break
                if time.monotonic() >= deadline:
                    raise AssertionError("SkinDerp did not change the actual stored hat option")
                control("wait", "1")
            check("skinDerpChangesOnlySelectedActualOption", all(actual_parts[name] == old_parts[name]
                  for name in part_keys if name != "modelPart_hat"), actual_parts)
            enable("SkinDerp", False)
            restored_parts = {name: entry["value"] for name, entry in control("options-get", *part_keys)["options"].items()}
            check("skinDerpDisabledRestoresCapturedActualOptions", restored_parts == old_parts, restored_parts)
            exercised("SkinDerp")
            report["skinDerpRuntimeCoverage"] = "sync_hat_actual_options_and_enable_time_restore;normal_remote_info_broadcast_pending"

            boost_tree = settings("VehicleBoost")
            leaf(boost_tree, "HorizontalSpeed")["value"] = 1.75
            leaf(boost_tree, "VerticalSpeed")["value"] = 0.8
            put_settings("VehicleBoost", boost_tree)
            control("chat", f'/summon minecraft:oak_boat {px + 0.5} {py} {pz + 0.5} {{Tags:["port_vehicle_boost"]}}')
            control("wait", "3")
            control("chat", '/ride @s mount @e[type=minecraft:oak_boat,tag=port_vehicle_boost,limit=1]')
            enable("VehicleBoost", True)
            status = wait_status(lambda s: probe(s,"clientVehicleBoostState").get("output",{}).get("passenger") is True)
            baseline = status
            control("chat", "/ride @s dismount")
            status = wait_status(lambda s: fresh_probe(s,"clientVehicleBoost",baseline)
                                 and probe(s,"clientVehicleBoost").get("output",{}).get("applied") is True)
            boost = probe(status,"clientVehicleBoost")
            velocity = boost["output"]["velocity"]
            check("vehicleBoostActualDismountVelocity", close_number(velocity[1], 0.8)
                  and close_number(math.hypot(velocity[0],velocity[2]),1.75), boost)
            enable("VehicleBoost", False)
            control("chat", '/kill @e[tag=port_vehicle_boost]')
            control("tp", *(str(value) for value in fixture_state["player"]["position"]))
            wait_state(lambda s: s["player"]["on_ground"])
            exercised("VehicleBoost")
            report["vehicleBoostRuntimeCoverage"] = "real_boat_mount_dismount_and_assigned_velocity;cross_disable_state_covered_by_owned_fixture"

            bounce_tree = settings("BlockBounce")
            leaf(bounce_tree,"Motion")["value"] = 1.0
            put_settings("BlockBounce",bounce_tree)
            control("chat", f"/fill {px - 1} {py - 1} {pz - 1} {px + 1} {py - 1} {pz + 1} minecraft:slime_block")
            control("chat", f"/fill {px - 1} {py} {pz - 1} {px + 1} {py + 8} {pz + 1} minecraft:air")
            control("tp",str(px+0.5),str(py),str(pz+0.5))
            wait_state(lambda s: s["player"]["on_ground"])
            enable("BlockBounce",True)
            key("space","down")
            try:
                control("wait","4")
                bounced = control("state")
                check("blockBounceActualSlimeJump", bounced["player"]["position"][1] > py + 2.0,
                      bounced["player"])
            finally:
                key("space","up")
                enable("BlockBounce",False)
            control("chat", f"/fill {px - 1} {py - 1} {pz - 1} {px + 1} {py - 1} {pz + 1} minecraft:stone")
            control("tp", *(str(value) for value in fixture_state["player"]["position"]))
            wait_state(lambda s: s["player"]["on_ground"])
            exercised("BlockBounce")
            report["blockBounceRuntimeCoverage"] = "real_slime_jump_height;bed_honey_and_boundary_identity_covered_by_owned_fixture"

            # A two-layer snow block has a 1/8 collision shape. BlockWalk
            # replaces that actual scanner result with a full cube.
            bx, by, bz = px + 9, py + 8, pz + 9
            control("chat", f"/fill {bx-2} {by-1} {bz-2} {bx+2} {by-1} {bz+2} minecraft:stone")
            control("chat", f"/fill {bx-2} {by} {bz-2} {bx+2} {by+5} {bz+2} minecraft:air")
            control("chat", f"/setblock {bx} {by} {bz} minecraft:snow[layers=2]")
            walk_tree = settings("BlockWalk")
            leaf(walk_tree, "Blocks")["value"] = ["minecraft:snow"]
            put_settings("BlockWalk", walk_tree)
            snow_samples = []
            for running, height in [(False, by + 0.125), (True, by + 1.0), (False, by + 0.125)]:
                enable("BlockWalk", running)
                control("tp", str(bx+0.5), str(by+3), str(bz+0.5))
                landed = wait_state(lambda s: s["player"]["on_ground"]
                                    and abs(s["player"]["position"][1]-height) < 0.001)
                snow_samples.append({"enabled":running,"position":landed["player"]["position"]})
            check("blockWalkActualSnowCollisionHeight", True, snow_samples)
            exercised("BlockWalk")
            report["blockWalkRuntimeCoverage"] = "actual_snow_collision_full_cube_and_disable_restore;all_shape_scanner_combinations_have_owned_fixture"

            # Official crosshair picking must prefer a selected chest behind
            # stone and return to the occluding stone when disabled.
            control("chat", f"/setblock {bx} {by} {bz-2} minecraft:stone")
            control("tp", str(bx+0.5), str(by+1), str(bz-1.5))
            control("look", "0", "0")
            control("chat", f"/setblock {bx} {by+2} {bz+1} minecraft:stone")
            control("chat", f"/setblock {bx} {by+2} {bz+2} minecraft:chest")
            ghost_tree = settings("GhostHand")
            leaf(ghost_tree, "TargetedBlocks")["value"] = ["minecraft:chest"]
            put_settings("GhostHand", ghost_tree)
            ghost_samples = []
            for running, wanted in [(False,"minecraft:stone"),(True,"minecraft:chest"),(False,"minecraft:stone")]:
                enable("GhostHand",running)
                picked = wait_state(lambda s: s.get("looking_at",{}).get("block",{}).get("block")==wanted)
                ghost_samples.append({"enabled":running,"lookingAt":picked["looking_at"]})
            check("ghostHandActualTargetBehindOccluder",True,ghost_samples)
            exercised("GhostHand")
            report["ghostHandRuntimeCoverage"] = "actual_outline_crosshair_target_priority_and_disable_restore;fluid_camera_and_other_clip_contexts_have_owned_fixture"

            control("tp", *(str(value) for value in fixture_state["player"]["position"]))
            control("look","0","0")
            # Air drag preserves a measurable stored horizontal velocity across
            # controller round trips; ground friction reaches vanilla's small
            # motion cutoff before the next external state request.
            control("chat", "/gamemode creative")
            wait_state(lambda s:s.get("gamemode") == "creative")
            control("tp", str(px+0.5), str(py+160), str(pz+0.5))
            strafe_tree = settings("Strafe")
            leaf(strafe_tree,"StrictMovement")["value"] = True
            put_settings("Strafe",strafe_tree)
            enable("Strafe",True)
            key("w","down")
            control("wait","12")
            key("w","up")
            control("wait","1")
            stopped = control("state")
            control("wait","2")
            held_position = control("state")
            check("strafeStrictMovementStopsActualDisplacement",all(abs(held_position["player"]["position"][i]-stopped["player"]["position"][i])<1e-8 for i in [0,2]),
                  {"before":stopped["player"],"after":held_position["player"]})
            check("strafeStrictMovementRetainsStoredVelocity",math.hypot(*[stopped["player"]["velocity"][i] for i in [0,2]])>0,
                  stopped["player"]["velocity"])
            enable("Strafe",False)
            exercised("Strafe")
            report["strafeRuntimeCoverage"] = "actual_strict_no_input_displacement_and_velocity_retention;full_strength_yaw_nonSELF_rawbits_have_owned_fixture"
            control("tp", *(str(value) for value in fixture_state["player"]["position"]))
            wait_state(lambda s:s["player"]["on_ground"])
            control("chat", "/gamemode survival")
            wait_state(lambda s:s.get("gamemode") == "survival")

            # A real bed updates synchronized sleeping position. SleepWalker
            # clears it locally and closes the actual bed screen without wake.
            control("chat", "/kill @e[type=minecraft:zombie]")
            control("chat", "/time set midnight")
            control("chat", f"/tp @s {px + 0.5} {py} {pz + 0.5}")
            control("chat", f"/setblock {px + 1} {py} {pz + 1} minecraft:red_bed[facing=north,part=foot]")
            control("chat", f"/setblock {px + 1} {py} {pz} minecraft:red_bed[facing=north,part=head]")
            control("wait", "4")
            baseline = api("/client/integration-status")
            enable("SleepWalker", True)
            control("interact", str(px + 1), str(py), str(pz + 1))
            status = wait_status(lambda s: fresh_probe(s, "clientSleepWalker", baseline))
            awake = wait_state(lambda s: s.get("sleeping") is None and s.get("screen") == "playing")
            check("sleepWalkerActualSleepingPositionCleared", probe(status, "clientSleepWalker").get("output") is True, awake)
            enable("SleepWalker", False)
            exercised("SleepWalker")
            report["sleepWalkerRuntimeCoverage"] = "actual_bed_sleep_position_clear_and_screen_close;network_packet_differential_pending"
            # The server deliberately remains asleep after SleepWalker clears
            # only the client state. Finish this fixture through the real
            # world clock before camera/teleport tests reuse the player.
            control("chat", "/time set day")
            control("wait", "25")
            control("tp", *(str(value) for value in fixture_state["player"]["position"]))

            verify_terrain_speed(api=api, control=control, check=check, settings=settings,
                                 put_settings=put_settings, leaf=leaf, enable=enable, module=module,
                                 key=key, wait_status=wait_status, wait_state=wait_state,
                                 probe=probe, fresh_probe=fresh_probe, exercised=exercised,
                                 report=report, position=fixture_state["player"]["position"])

            verify_timer(api=api, control=control, check=check, settings=settings,
                         put_settings=put_settings, leaf=leaf, enable=enable, module=module,
                         key=key, wait_status=wait_status, wait_state=wait_state,
                         wait_control=wait_control, probe=probe, fresh_probe=fresh_probe,
                         close_number=close_number, exercised=exercised, report=report,
                         position=fixture_state["player"]["position"])

            verify_better_inventory(api=api, control=control, check=check, settings=settings,
                                    put_settings=put_settings, leaf=leaf, enable=enable,
                                    key=key, wait_status=wait_status, wait_state=wait_state,
                                    wait_control=wait_control, probe=probe, fresh_probe=fresh_probe,
                                    close_number=close_number, exercised=exercised, report=report,
                                    position=fixture_state["player"]["position"], run=run)

            verify_auto_break(control=control, check=check, enable=enable,
                              wait_state=wait_state, wait_control=wait_control,
                              exercised=exercised, report=report,
                              position=fixture_state["player"]["position"])

            verify_high_jump(api=api, control=control, check=check, settings=settings,
                             put_settings=put_settings, leaf=leaf, enable=enable, module=module,
                             key=key, wait_status=wait_status, wait_state=wait_state,
                             probe=probe, fresh_probe=fresh_probe, close_number=close_number,
                             exercised=exercised, report=report,
                             position=fixture_state["player"]["position"])

            verify_item_scroller(control=control, check=check, settings=settings,
                                 put_settings=put_settings, leaf=leaf, enable=enable, key=key,
                                 wait_state=wait_state, wait_control=wait_control,
                                 exercised=exercised, report=report)

            verify_fast_break(control=control, check=check, settings=settings,
                              put_settings=put_settings, leaf=leaf, enable=enable,
                              wait_state=wait_state, wait_control=wait_control,
                              exercised=exercised, report=report,
                              position=fixture_state["player"]["position"])

            verify_anti_hunger(api=api, control=control, check=check, settings=settings,
                               put_settings=put_settings, leaf=leaf, enable=enable, key=key,
                               wait_status=wait_status, wait_state=wait_state,
                               wait_control=wait_control, probe=probe, fresh_probe=fresh_probe,
                               choose_language=choose_language, exercised=exercised, report=report,
                               position=fixture_state["player"]["position"])

        if not args.scenarios:
            baseline = api("/client/integration-status")
            key("key.keyboard.right.shift", "tap")
            status = wait_status(lambda s: s["browser"]["visible"] and fresh_route(s, baseline, "clickgui"))
            screen = control("state")
            check("originalClickGuiOpened", status["browser"]["visible"] and screen["screen"] == "mod", screen)
            if not args.ui_only:
                key("c", "down")
                try:
                    control("wait", "2")
                    check("zoomBindRespectsOpenScreen", not module("Zoom")["enabled"])
                finally:
                    key("c", "up")
            # The original panel defaults to collapsed. Its existing persistent
            # storage protocol sets test presentation state without a mouse click.
            # Let existing 200-ms whole-map writes finish before the REST mutation.
            control("wait", "10")
            storage_saved = stable_theme_storage()
            report["themeStorageSettled"] = {"officialTicks": 10, "stableMilliseconds": 500, "snapshot": storage_saved}
            storage = {item["key"]: item["value"] for item in storage_saved["items"]}
            panel = json.loads(storage.get("clickgui.panel.Render", "null"))
            if panel is None:
                panel = {"top": 20, "left": 20, "expanded": False, "scrollTop": 0, "zIndex": 0}
            if not isinstance(panel, dict):
                raise AssertionError("Unexpected original Render panel configuration")
            # Keep the registered render rows visible, with other category panels beside
            # them. Expanding Render alone leaves the original stacked panels over it.
            panel["left"] = 20
            panel["top"] = 140
            panel["expanded"] = True
            panel["scrollTop"] = 0
            storage["clickgui.panel.Render"] = json.dumps(panel, separators=(",", ":"))
            for category, left, top in (("Exploit",320,140), ("Movement",620,140),
                                        ("World",320,210), ("Combat",620,210),
                                        ("Player",320,280), ("Settings",620,280),
                                        ("Fun",320,350)):
                storage["clickgui.panel." + category] = json.dumps(
                    {"left":left,"top":top,"expanded":False,"scrollTop":0,"zIndex":0}, separators=(",", ":"))
            storage_mutated = True
            api("/client/localStorage/all", "PUT",
                {"items": [{"key": name, "value": value} for name, value in storage.items()]})
            check_render_layout("renderPanelExpandedStored")
            status = reload_theme()
            check("clickGuiRealCtrlRReload", status["browser"]["visible"]
                  and control("state")["screen"] == "mod", status["browser"])
            check_render_layout("renderPanelLayoutAfterReload")
            # Ctrl+R preserves #/clickgui. Panel reads localStorage synchronously,
            # before App's asynchronous persistent-data insertion can finish. The
            # first reload's post-insertion route log establishes that the browser
            # has now loaded our fixture. A second real document reload constructs
            # Panel from that already populated localStorage. An Esc/reopen cycle
            # cannot establish this: hiding CEF can pause the original global fade
            # outro, allowing the old Panel to be reused when the page is shown.
            first_reload = status
            status = reload_theme()
            check("clickGuiSecondDocumentReload", status["browser"]["visible"]
                  and control("state")["screen"] == "mod"
                  and status["browser"]["page"]["completedLoads"]
                      > first_reload["browser"]["page"]["completedLoads"], status["browser"])
            report["panelFixtureReloads"] = {
                "method": "two_official_ctrl_r_document_reloads",
                "firstCompletedLoads": first_reload["browser"]["page"]["completedLoads"],
                "secondCompletedLoads": status["browser"]["page"]["completedLoads"],
                "reason": "The first document inserts persistent data; the second synchronously constructs Panel from that browser localStorage.",
                "visualReviewRequired": True,
            }
            # Cover both the original 150-ms fade and its 200-ms debounced storage.
            control("wait", "10")
            check_render_layout("renderPanelLayoutAfterSecondReload")
            status = api("/client/integration-status")
            errors = [line for line in status["browser"]["page"]["console"]
                      if re.search(r"\b(error|fatal|typeerror|referenceerror|syntaxerror|uncaught)\b", line, re.IGNORECASE)]
            check("clickGuiJavascript", not errors, errors)
            check("clickGuiDocumentStillHealthy", status["browser"]["page"]["httpStatus"] == 200
                  and not status["browser"]["page"]["loadErrors"], status["browser"]["page"])
            render_modules = api("/client/modules")
            check("registeredRenderModules", {item["name"] for item in render_modules
                  if item.get("category") == "Render"} == set(RENDER_MODULES), render_modules)
            screenshot = run / "original-clickgui.png"
            check_render_layout("renderPanelLayoutBeforeScreenshot")
            control("screenshot", str(screenshot))
            check("clickGuiScreenshotSaved", screenshot.is_file() and screenshot.stat().st_size > 0, str(screenshot))
            report["screenshotPendingVisualReview"] = {
                "status": "pending_visual_review", "path": str(screenshot),
                "expected": {"panel": "Render", "left": 20, "top": 140, "expanded": True,
                             "moduleRows": list(RENDER_MODULES)},
                "automatedEvidence": "Stored panel state, two actual document reloads after persistent-data insertion and a fresh CEF paint; registry checks do not establish visible module rows."
            }
            if args.languages:
                language_records = []
                settings_before_language = {name: stable_settings(settings(name)) for name in MODULES}
                for code in ("zh_cn", "zh_tw", "en_us"):
                    snapshot = choose_language(code)
                    check("stableModuleIdentifiers" + code, [item["name"] for item in api("/client/modules")]
                          == list(ORIGINAL_MODULE_ORDER))
                    expected_description = snapshot["translations"]["liquidbounce.module.noPitchLimit.serverSide.description"]
                    check("localizedSettingHelp" + code, leaf(settings("NoPitchLimit"), "ServerSide")["description"]
                          == expected_description)
                    current_settings = {name: settings(name) for name in MODULES}
                    check("stableSettingsIdentifiersAndValues" + code,
                          {name: stable_settings(tree) for name, tree in current_settings.items()}
                          == settings_before_language)
                    descriptions = [item for tree in current_settings.values() for item in described_settings(tree)]
                    missing_help = [item.get("key", item.get("name")) for item in descriptions
                                    if not isinstance(item.get("description"), str)
                                    or not item["description"]
                                    or item["description"].startswith("liquidbounce.")]
                    check("allRegisteredSettingHelpResolved" + code, not missing_help, missing_help)
                    menus = api("/client/registry/menu")
                    check("menuRegistryAvailable" + code, len(menus) == 25
                          and "minecraft:generic_9x3" in menus and "minecraft:shulker_box" in menus)
                    sounds = api("/client/registry/sound_event")
                    check("soundRegistryAvailable" + code,
                          "minecraft:entity.fishing_bobber.splash" in sounds and len(sounds) > 100)
                    items = api("/client/registry/item")
                    entities = api("/client/registry/entity_type")
                    check("interactionRegistriesAvailable" + code,
                          len(items) > 1500 and len(entities) > 150
                          and "minecraft:copper_pickaxe" in items and "minecraft:pig" in entities,
                          {"items":len(items),"entities":len(entities)})
                    baseline = api("/client/integration-status")
                    key("key.keyboard.right.shift", "tap")
                    status = wait_status(lambda s: s["browser"]["visible"]
                                         and s["browser"]["paintCount"] > baseline["browser"]["paintCount"])
                    control("wait", "10")
                    picture = run / ("clickgui-" + code + ".png")
                    control("screenshot", str(picture))
                    check("languageScreenshotSaved" + code, picture.is_file() and picture.stat().st_size > 0)
                    language_records.append({"gameLanguage":code,"generation":snapshot["generation"],
                        "expectedModuleNames":{name:snapshot["moduleNames"].get(name,name) for name in RENDER_MODULES},
                        "screenshot":str(picture),"visualReview":"pending"})
                report["languageSwitches"] = language_records
            key("esc", "tap")
            wait_status(lambda s: not s["browser"]["visible"])
            check("clickGuiClosed", control("state")["screen"] == "playing")
        report["finalStatus"] = api("/client/integration-status")
        report["verificationPassed"] = True
    except Exception as error:
        report["error"] = str(error)
        failure = error
    finally:
        cleanup_errors = []
        for name in list(held_keys):
            try:
                key(name, "up")
            except Exception as error:
                cleanup_errors.append(f"Release {name}: {error}")
        if game_mutated:
            try:
                control("release")
                cleanup_state = control("state")
                if cleanup_state["screen"] == "mod":
                    key("esc", "tap")
                    wait_status(lambda s: not s["browser"]["visible"])
                if control("state")["screen"] == "dead":
                    control("respawn")
                    wait_state(lambda s: s.get("screen") == "playing" and s["player"]["health"] > 0)
                for _ in range(4):
                    if control("state")["screen"] == "playing":
                        break
                    key("esc", "tap")
                control("effect", "clear")
                check("noTestNauseaRemains", not any(item.get("id", "").removeprefix("minecraft:") == "nausea"
                      for item in active_effects(control("effects"))))
                control("chat", "/gamemode " + initial_state["gamemode"])
                wait_state(lambda s: s.get("gamemode") == initial_state["gamemode"])
                control("tp", *(str(value) for value in initial_state["player"]["position"]))
                control("look", str(initial_state["player"]["yaw"]), str(initial_state["player"]["pitch"]))
                restored_game = control("state")
                check("restoredGameModeAndPosition", restored_game["gamemode"] == initial_state["gamemode"]
                      and all(abs(a - b) < 0.05 for a, b in zip(restored_game["player"]["position"],
                                                                             initial_state["player"]["position"])),
                      restored_game)
            except Exception as error:
                cleanup_errors.append(f"Restore game fixture: {error}")
        if original_perspective is not None:
            try:
                set_perspective(original_perspective)
            except Exception as error:
                cleanup_errors.append(f"Restore F5 perspective: {error}")
        if global_mutated and saved_global is not None:
            try:
                api("/client/global", "PUT", saved_global)
                check("restoredGlobalSettings", stable_settings(api("/client/global")) == stable_settings(saved_global))
            except Exception as error:
                cleanup_errors.append(f"Restore global settings: {error}")
        if storage_mutated:
            try:
                api("/client/localStorage/all", "PUT", storage_saved)
                restored_storage = api("/client/localStorage/all")
                check("restoredThemeStorage", {item["key"]: item["value"] for item in restored_storage["items"]}
                      == {item["key"]: item["value"] for item in storage_saved["items"]}, restored_storage)
            except Exception as error:
                cleanup_errors.append(f"Restore theme storage: {error}")
        if mutated:
            for name, old in saved.items():
                try:
                    enable(name, False)
                    put_settings(name, old["settings"])
                    enable(name, old["enabled"])
                    check("restored" + name, module(name)["enabled"] == old["enabled"]
                          and settings(name) == old["settings"])
                except Exception as error:
                    cleanup_errors.append(f"Restore {name}: {error}")
            try:
                old_bright = saved["FullBright"]
                if old_bright["enabled"] and leaf(old_bright["settings"], "Mode")["active"] == "NightVision":
                    before = api("/client/integration-status")
                    wait_status(lambda s: fresh_probe(s, "clientPlayerTickPost", before))
                    check("restoredNightVisionEffect", bool(night_vision(control("effects"))))
                else:
                    check("noTestNightVisionRemains", not night_vision(control("effects")))
            except Exception as error:
                cleanup_errors.append(f"Restore effect: {error}")
        if isolated:
            try:
                control("release")
                if mutated and control("state")["screen"] == "mod":
                    key("esc", "tap")
                    wait_status(lambda s: not s["browser"]["visible"])
                if mutated:
                    check("restoredPlayingScreen", control("state")["screen"] == "playing")
            except Exception as error:
                cleanup_errors.append(f"Restore input/screen: {error}")
            if args.quit:
                try:
                    control("quit")
                except Exception as error:
                    cleanup_errors.append(f"Quit: {error}")
        if cleanup_errors:
            report["cleanupErrors"] = cleanup_errors
        report["passed"] = bool(report.get("verificationPassed")) and not cleanup_errors
        report["cameraRegressionPassed"] = report["passed"] and not args.ui_only and not args.scenarios
        report["fullIntegrationPassed"] = (report["passed"] and report["validationScope"] == "full_integration"
                                           and not report["behaviorModulesNotExercised"] and not args.ui_only)
        output = run / ("ui-diagnostic-validation.json" if args.ui_only else "native-integration-validation.json")
        output.write_text(json.dumps(redact(report, secrets), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"report": str(output), "passed": report["passed"], "validationScope": report["validationScope"],
                          "fullIntegrationPassed": report["fullIntegrationPassed"]}, ensure_ascii=False))
    if failure is not None or report.get("cleanupErrors"):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
