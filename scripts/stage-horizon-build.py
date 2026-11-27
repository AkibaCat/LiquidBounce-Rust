#!/usr/bin/env python3
"""Create a separate exact-Horizon workspace for native adapter type checking."""

from __future__ import annotations

import argparse
import difflib
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
import zipfile

PROJECT = Path(__file__).resolve().parent.parent
OWNER = PROJECT / "native/liquidbounce-horizon"
STAGING_ROOT = PROJECT / "build/staging"
RENDER_API = "crates/horizon-client-render/src/gfx/native_render/generated.rs"
GAME_HOOKS = "crates/horizon-client/src/game/native_hooks.rs"


def build_source_input(relative: str) -> bool:
    """Exclude downloaded research caches and discarded drafts from build inputs."""
    path = PurePosixPath(relative)
    return ("cef-source" not in path.parts and path.suffix != ".crate"
            and ".draft." not in path.name
            and not path.name.startswith(("tmp-", "tmp_")))


def digest_bytes(content: bytes) -> str:
    return hashlib.sha256(content.replace(b"\r\n", b"\n")).hexdigest()


def generate_api(stage: Path) -> str:
    """Record the generator's two outputs so later batches can restore them cheaply."""
    inputs = {name: digest_bytes((stage / name).read_bytes()) for name in (RENDER_API, GAME_HOOKS)}
    layers = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in stage.glob(".liquidbounce-*-patch.patch")}
    previous_path = stage / ".liquidbounce-generated-api.json"
    if previous_path.is_file():
        previous = json.loads(previous_path.read_text(encoding="utf-8"))
        if previous.get("layers") == layers:
            for name, digest in inputs.items():
                old = previous.get("files", {}).get(name, {})
                if old.get("outputSha256") == digest:
                    inputs[name] = old["inputSha256"]
    subprocess.run(["cargo", "+stable", "run", "-p", "xtask", "--offline", "--", "mod-api"], cwd=stage, check=True)
    record = {
        "layers": layers,
        "files": {name: {"inputSha256": digest, "outputSha256": digest_bytes((stage / name).read_bytes())} for name, digest in inputs.items()},
    }
    write_changed(stage / ".liquidbounce-generated-api.json", (json.dumps(record, indent=2) + "\n").encode())
    return record["files"][RENDER_API]["outputSha256"]


def write_changed(path: Path, content: bytes) -> None:
    if path.is_file() and path.read_bytes() == content:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(content)


def workspace_members(path: Path, added: list[str]) -> None:
    text = path.read_text(encoding="utf-8")
    parsed = tomllib.loads(text)
    missing = [member for member in added if member not in parsed["workspace"]["members"]]
    if not missing:
        return
    match = re.search(r"(?m)^members\s*=\s*\[", text)
    if not match:
        raise ValueError("Cannot find original workspace members")
    close = text.index("]", match.end())
    text = text[:close] + "".join(f'    "{member}",\n' for member in missing) + text[close:]
    tomllib.loads(text)
    write_changed(path, text.encode())


def restore_formatted_patch(stage: Path, applied: Path) -> None:
    """Authenticate formatting and generated API output before replacing an overlay."""
    paths = re.findall(r"(?m)^diff --git a/(.+) b/\1$", applied.read_text(encoding="utf-8"))
    # xtask also rewrites this committed file, even when the overlay only adds
    # a hook declaration elsewhere. Restore it with the overlay it came from.
    if RENDER_API not in paths:
        paths.append(RENDER_API)
    # Both overlays can edit the same file. Authenticate and reconstruct every
    # active layer before checking formatting; a GPU edit in gfx.rs must not be
    # mistaken for an unrecognized edit when replacing the behavior layer.
    layers = []
    for stem in (".liquidbounce-gpu-ui-patch", ".liquidbounce-behavior-patch"):
        recorded = stage / (stem + ".patch")
        marker = stage / (stem + ".json")
        if recorded.is_file() and marker.is_file():
            expected_digest = json.loads(marker.read_text(encoding="utf-8"))["sha256"]
            if hashlib.sha256(recorded.read_bytes()).hexdigest() != expected_digest:
                raise ValueError(f"Stored applied patch digest differs from its record: {stem}")
            layers.append(recorded)
    if applied not in layers:
        layers.append(applied)
    generation_record = stage / ".liquidbounce-generated-api.json"
    authenticated_outputs = {}
    if generation_record.is_file():
        record = json.loads(generation_record.read_text(encoding="utf-8"))
        active_layers = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in layers}
        if record.get("layers") == active_layers:
            authenticated_outputs = record.get("files", {})
    all_paths = set(paths)
    for layer in layers:
        all_paths.update(re.findall(r"(?m)^diff --git a/(.+) b/\1$", layer.read_text(encoding="utf-8")))
    replacements = []
    with tempfile.TemporaryDirectory(prefix="patch-check-", dir=STAGING_ROOT) as temporary:
        expected_root = Path(temporary)
        for relative in sorted(all_paths):
            baseline = PROJECT / "upstream/Horizon" / relative
            if baseline.is_file():
                write_changed(expected_root / relative, baseline.read_bytes())
        for layer in layers:
            subprocess.run(["git", "apply", "--whitespace=nowarn", str(layer)], cwd=expected_root, check=True)
        expected_files = {relative: (expected_root / relative).read_bytes() for relative in paths}
        derived_files = None
        for relative in paths:
            expected = expected_files[relative]
            actual_path = stage / relative
            actual = actual_path.read_bytes()
            if expected.replace(b"\r\n", b"\n") == actual.replace(b"\r\n", b"\n"):
                continue
            generated = authenticated_outputs.get(relative, {})
            if generated.get("inputSha256") == digest_bytes(expected) and generated.get("outputSha256") == digest_bytes(actual):
                replacements.append((actual_path, expected))
                continue
            if actual_path.suffix != ".rs":
                raise ValueError(f"Patched staging file has other changes: {relative}")
            formatted = subprocess.run(["rustfmt", "+stable", "--edition", "2024", "--config", "skip_children=true", "--emit", "stdout"],
                input=expected, capture_output=True, check=True).stdout
            if formatted.replace(b"\r\n", b"\n") != actual.replace(b"\r\n", b"\n"):
                if derived_files is None:
                    # Reproduce the generator from pristine sources plus the
                    # authenticated active overlays. Never accept arbitrary
                    # changes merely because their path says "generated".
                    baseline_root = PROJECT / "upstream/Horizon"
                    inputs = [baseline_root / "native-hooks.json"]
                    inputs.extend((baseline_root / "crates").rglob("*.rs"))
                    inputs.extend((baseline_root / "tools/native-hook-fragments").rglob("*.json"))
                    for source in inputs:
                        destination = expected_root / source.relative_to(baseline_root)
                        if not destination.exists():
                            destination.parent.mkdir(parents=True, exist_ok=True)
                            shutil.copyfile(source, destination)
                    generator = stage / "target/debug" / ("xtask.exe" if sys.platform == "win32" else "xtask")
                    if not generator.is_file():
                        raise ValueError("Cached xtask is required to authenticate generated API changes; build xtask in this staging first")
                    subprocess.run([str(generator), "mod-api"], cwd=expected_root, check=True)
                    derived_files = {name: (expected_root / name).read_bytes() for name in paths}
                if derived_files[relative].replace(b"\r\n", b"\n") != actual.replace(b"\r\n", b"\n"):
                    raise ValueError(f"Patched staging file differs beyond rustfmt and reproducible API generation: {relative}")
            replacements.append((actual_path, expected))
    # Validate every affected file before changing any of them.
    for path, expected in replacements:
        write_changed(path, expected)


def apply_patch(stage: Path, patch: Path, marker_name: str) -> str:
    digest = hashlib.sha256(patch.read_bytes()).hexdigest()
    marker = stage / marker_name
    applied = stage / (marker_name.removesuffix(".json") + ".patch")
    if marker.is_file():
        previous = json.loads(marker.read_text())
        if previous["sha256"] != digest:
            # Older staging records kept only a digest. Reconstruct their exact
            # diff from the pristine source and staged files, then authenticate
            # it with that digest before reversing anything.
            if not applied.is_file():
                output = []
                paths = re.findall(r"(?m)^diff --git a/(.+) b/\1$", patch.read_text(encoding="utf-8"))
                for relative in paths:
                    baseline = PROJECT / "upstream/Horizon" / relative
                    before = baseline.read_text(encoding="utf-8") if baseline.is_file() else ""
                    staged_file = stage / relative
                    if not staged_file.is_file():
                        raise ValueError(f"Missing previously patched file: {relative}")
                    after = staged_file.read_text(encoding="utf-8")
                    if before == after:
                        continue
                    output.append(f"diff --git a/{relative} b/{relative}\n")
                    if not before:
                        output.append("new file mode 100644\n")
                    output.extend(difflib.unified_diff(before.splitlines(keepends=True), after.splitlines(keepends=True),
                        fromfile=f"a/{relative}" if before else "/dev/null", tofile=f"b/{relative}"))
                recovered = "".join(output).encode()
                if hashlib.sha256(recovered).hexdigest() != previous["sha256"]:
                    raise ValueError("Previous patch cannot be authenticated; retain this workspace and use a new --name")
                write_changed(applied, recovered)
            if hashlib.sha256(applied.read_bytes()).hexdigest() != previous["sha256"]:
                raise ValueError("Stored applied patch digest differs from the staging record")
            restore_formatted_patch(stage, applied)
            subprocess.run(["git", "apply", "--reverse", "--check", str(applied)], cwd=stage, check=True)
            subprocess.run(["git", "apply", "--reverse", str(applied)], cwd=stage, check=True)
            try:
                subprocess.run(["git", "apply", "--check", "--whitespace=nowarn", str(patch)], cwd=stage, check=True)
                subprocess.run(["git", "apply", "--whitespace=nowarn", str(patch)], cwd=stage, check=True)
            except subprocess.CalledProcessError:
                subprocess.run(["git", "apply", "--whitespace=nowarn", str(applied)], cwd=stage, check=True)
                raise
        write_changed(applied, patch.read_bytes())
        write_changed(marker, (json.dumps({"sha256": digest}) + "\n").encode())
        return digest
    subprocess.run(["git", "apply", "--check", "--whitespace=nowarn", str(patch)], cwd=stage, check=True)
    subprocess.run(["git", "apply", "--whitespace=nowarn", str(patch)], cwd=stage, check=True)
    write_changed(applied, patch.read_bytes())
    write_changed(marker, (json.dumps({"sha256": digest}) + "\n").encode())
    return digest


def native_runner(stage: Path, behavior_hooks: bool, webview: bool) -> str:
    """Equivalent to the old launcher's linked trait-mod executable entry."""
    relative = "crates/liquidbounce-native-game"
    runner = stage / relative
    manifest = '''[package]
name = "liquidbounce-native-game"
version = "0.1.0"
edition = "2024"
publish = false
build = "build.rs"

[[bin]]
name = "liquidbounce-native-game"
path = "src/main.rs"

[dependencies]
horizon-client = { path = "../horizon-client" }
horizon-mods = { path = "../horizon-mods" }
horizon-threads = { path = "../horizon-threads" }
liquidbounce-horizon = { path = "../../native-api/liquidbounce_horizon" }

[build-dependencies]
winresource = "0.1"
'''
    main = '''//! Local native-mod runner following Horizon 9f42064's launcher entry.
#![forbid(unsafe_code)]

#[global_allocator]
static ALLOCATOR: horizon_threads::GameAllocator = horizon_threads::GAME_ALLOCATOR;

static MOD_API_LEVEL: [u8; horizon_mods::api_level::MARK.len()] = horizon_mods::api_level::MARK;

fn main() {
    std::hint::black_box(&MOD_API_LEVEL);
    let mods: horizon_client::mod_api::Mods = vec![liquidbounce_horizon::hooks()].into();
    horizon_client::mod_api::name_mods(&["liquidbounce_horizon"]);
    horizon_client::entry::main_with(mods);
}
'''
    enabled_features = ["webview"] if webview else ["behavior-hooks"] if behavior_hooks else []
    if enabled_features:
        manifest = manifest.replace('liquidbounce-horizon = { path = "../../native-api/liquidbounce_horizon" }',
            'liquidbounce-horizon = { path = "../../native-api/liquidbounce_horizon", features = ' + json.dumps(enabled_features) + ' }')
    if webview:
        main = main.replace('    let mods: horizon_client::mod_api::Mods = vec![liquidbounce_horizon::hooks()].into();', '''    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let metadata_only = arguments.iter().any(|value| matches!(value.as_str(), "--launch-api" | "--help" | "-h" | "--dedicated"));
    let hooks = if metadata_only {
        liquidbounce_horizon::hooks()
    } else {
        let path = |name: &str| std::env::var_os(name).map(std::path::PathBuf::from)
            .unwrap_or_else(|| { eprintln!("Missing {name}; use scripts/Start-LiquidBounce-Test.ps1"); std::process::exit(1) });
        let resources = path("LIQUIDBOUNCE_CEF_RESOURCES");
        let config = liquidbounce_horizon::BrowserConfig {
            theme_root: path("LIQUIDBOUNCE_THEME_ROOT"),
            helper_executable: resources.join("liquidbounce-cef-helper.exe"),
            resources_directory: resources,
            cache_directory: path("LIQUIDBOUNCE_CACHE_DIR"),
            config_directory: path("LIQUIDBOUNCE_CONFIG_DIR"),
            initial_size: [960, 540],
        };
        liquidbounce_horizon::browser_hooks(config).unwrap_or_else(|error| {
            eprintln!("LiquidBounce initialization: {error}"); std::process::exit(1)
        })
    };
    let mods: horizon_client::mod_api::Mods = vec![hooks].into();''')
        main = main.replace('    horizon_client::entry::main_with(mods);', '''    horizon_client::entry::main_with(mods);
    if !metadata_only {
        match liquidbounce_horizon::shutdown_browser_current_thread() {
            Ok(true) => {},
            result => { eprintln!("LiquidBounce browser shutdown: {result:?}"); std::process::exit(1) },
        }
    }''')
    write_changed(runner / "Cargo.toml", manifest.encode())
    write_changed(runner / "src/main.rs", main.encode())
    write_changed(runner / "build.rs", b'''//! The runner has its own binary name for Windows link arguments.
#[path = "../../packaging/windows/exe.rs"]
mod windows_exe;

fn main() {
    windows_exe::resources("liquidbounce-native-game");
}
''')
    return relative


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", default="horizon", help="Named folder under build/staging; existing staging is reused")
    parser.add_argument("--gpu-ui", action="store_true", help="Apply the optional GPU hook patch to the staging copy")
    parser.add_argument("--behavior-hooks", action="store_true", help="Apply the exact game-phase patch and enable implemented modules")
    parser.add_argument("--webview", action="store_true", help="Link the real interop server and CEF original-theme browser (enables both patches)")
    args = parser.parse_args()
    if args.webview:
        args.gpu_ui = True
        args.behavior_hooks = True
    if args.behavior_hooks:
        # Hook declarations and executable adapter sources must describe the
        # same module batch; never consume a stale generated patch silently.
        subprocess.run([sys.executable, str(OWNER / "make-behavior-patch.py")], check=True)
    if not re.fullmatch(r"[a-zA-Z][a-zA-Z0-9_-]{0,63}", args.name):
        raise ValueError("--name must be a simple staging folder name")
    stage = (STAGING_ROOT / args.name).resolve()
    stage.relative_to(STAGING_ROOT.resolve())
    spec = importlib.util.spec_from_file_location("native_package", OWNER / "package.py")
    packaging = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(packaging)
    baseline = packaging.verify_baseline()
    marker = stage / ".liquidbounce-staging.json"
    if stage.exists():
        if not marker.is_file():
            raise ValueError(f"Existing unrecognized staging folder: {stage}; choose a new --name")
        previous = json.loads(marker.read_text(encoding="utf-8"))
        if previous["horizonArchiveSha256"] != baseline["horizonArchiveSha256"]:
            raise ValueError("Existing staging source differs from the verified archive")
    else:
        stage.mkdir(parents=True)
        with zipfile.ZipFile(PROJECT / "research/Horizon-source.zip") as archive:
            for item in archive.infolist():
                if item.is_dir():
                    continue
                parts = PurePosixPath(item.filename).parts[1:]
                if not parts or any(part in (".", "..") for part in parts) or "\\" in item.filename or ":" in item.filename:
                    raise ValueError(f"Invalid archive member: {item.filename}")
                destination = stage.joinpath(*parts)
                destination.resolve().relative_to(stage)
                write_changed(destination, archive.read(item))
        write_changed(marker, (json.dumps(baseline, indent=2) + "\n").encode())

    gpu_digest = apply_patch(stage, PROJECT / "patches/horizon-ui-render-hook.patch", ".liquidbounce-gpu-ui-patch.json") if args.gpu_ui else None
    behavior_digest = apply_patch(stage, OWNER / "patches/horizon-behavior-hooks.patch", ".liquidbounce-behavior-patch.json") if args.behavior_hooks else None
    navigation_sine_digest = None
    if args.behavior_hooks:
        navigation_sine = (PROJECT / "native/liquidbounce-behaviors/src/navigation/simulation/sine-table.bin").read_bytes()
        write_changed(stage / "crates/horizon-client/src/navigation/simulation/sine-table.bin", navigation_sine)
        navigation_sine_digest = hashlib.sha256(navigation_sine).hexdigest()
    engine_relative = "native-api/liquidbounce_horizon"
    core_relative = "native-crates/liquidbounce_horizon/core"
    behavior_relative = "native-crates/liquidbounce_horizon/behaviors"
    interop_relative = "native-crates/liquidbounce_horizon/interop"
    webview_relative = "native-crates/liquidbounce_horizon/webview"
    sources = [("engine", OWNER / "engine", engine_relative), ("core", OWNER.parent / "liquidbounce-core", core_relative), ("behaviors", OWNER.parent / "liquidbounce-behaviors", behavior_relative)]
    # Cargo resolves optional path dependencies even with their features off.
    for label in ("interop", "webview"):
        source = OWNER.parent / f"liquidbounce-{label}"
        if f"liquidbounce-{label}" in (OWNER / "engine/Cargo.toml").read_text(encoding="utf-8"):
            sources.append((label, source, interop_relative if label == "interop" else webview_relative))
    native_source_files = {}
    for label, source, relative in sources:
        for name, data in packaging.read_package_source(source).items():
            if not build_source_input(name):
                continue
            if name == "Cargo.toml" or name.endswith((".rs", ".json")) or name.startswith(("resources/", "vendor/", "data/")):
                native_source_files[(source / name).relative_to(PROJECT).as_posix()] = hashlib.sha256(data).hexdigest()
            if label == "engine" and name == "Cargo.toml":
                text = data.decode()
                text = re.sub(r'^horizon-client\s*=.*$', 'horizon-client = { path = "../../crates/horizon-client", default-features = true }', text, flags=re.MULTILINE)
                text = re.sub(r'^liquidbounce-core\s*=.*$', 'liquidbounce-core = { path = "../../native-crates/liquidbounce_horizon/core" }', text, flags=re.MULTILINE)
                text = re.sub(r'^liquidbounce-behaviors\s*=.*$', 'liquidbounce-behaviors = { path = "../../native-crates/liquidbounce_horizon/behaviors", optional = true }', text, flags=re.MULTILINE)
                text = re.sub(r'^liquidbounce-interop\s*=.*$', 'liquidbounce-interop = { path = "../../native-crates/liquidbounce_horizon/interop", optional = true }', text, flags=re.MULTILINE)
                text = re.sub(r'^liquidbounce-webview\s*=.*$', 'liquidbounce-webview = { path = "../../native-crates/liquidbounce_horizon/webview", optional = true, features = ["cef-runtime", "horizon-gpu"] }', text, flags=re.MULTILINE)
                data = text.encode()
            if label == "webview" and name == "Cargo.toml":
                text = data.decode()
                text = re.sub(r'(?ms)^\[workspace\]\s*(?=\[)', '', text)
                text = re.sub(r'(?ms)^\[patch\.crates-io\]\n.*?(?=^\[|\Z)', '', text)
                text = re.sub(r'^horizon-hooks\s*=.*$', 'horizon-hooks = { path = "../../../crates/horizon-hooks", features = ["gpu-ui"], optional = true }', text, flags=re.MULTILINE)
                data = text.encode()
            write_changed(stage / relative / name, data)
        # CEF smoke's include_str! fixture is needed when checking all targets.
        if label == "webview":
            for shader in sorted((source / "src").rglob("*.wgsl")):
                native_source_files[shader.relative_to(PROJECT).as_posix()] = hashlib.sha256(shader.read_bytes()).hexdigest()
                write_changed(stage / relative / shader.relative_to(source), shader.read_bytes())
            for fixture in sorted((source / "fixtures").rglob("*")):
                if fixture.is_file():
                    write_changed(stage / relative / fixture.relative_to(source), fixture.read_bytes())
    runner_relative = native_runner(stage, args.behavior_hooks, args.webview)
    members = [relative for _, _, relative in sources] + [runner_relative]
    workspace_members(stage / "Cargo.toml", members)
    generated_api_digest = None
    if args.behavior_hooks or args.gpu_ui:
        # Generate once for the complete module batch. Cargo reuses this named
        # workspace's xtask and game dependency cache across subsequent batches.
        generated_api_digest = generate_api(stage)
    for source in (Path(__file__), PROJECT / "patches/horizon-ui-render-hook.patch", OWNER / "patches/horizon-behavior-hooks.patch"):
        native_source_files[source.relative_to(PROJECT).as_posix()] = hashlib.sha256(source.read_bytes()).hexdigest()
    # Keep every patch generator and its source templates in the rebuild map.
    # They produce the game-side behavior code outside the native crates.
    for source in sorted([*OWNER.glob("*.py"), *OWNER.glob("*.rs")]):
        if not build_source_input(source.relative_to(PROJECT).as_posix()):
            continue
        native_source_files[source.relative_to(PROJECT).as_posix()] = hashlib.sha256(source.read_bytes()).hexdigest()
    native_source_digest = hashlib.sha256(json.dumps(native_source_files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    # All source copies are independent files. The upstream checkout, its lock
    # file and all prepared game resources are untouched by this script.
    report = {
        "workspace": str(stage),
        "horizon": baseline["horizon"],
        "clientDefaultFeatures": ["audio", "controllers", "horizon-extras"],
        "gpuUiPatchSha256": gpu_digest,
        "behaviorHookPatchSha256": behavior_digest,
        "navigationSineTableSha256": navigation_sine_digest,
        "generatedApiSha256": generated_api_digest,
        "nativeMembers": members,
        "nativeSourceSha256": native_source_digest,
        "nativeSourceFiles": native_source_files,
        "originalThemeLinked": args.webview,
        "typeCheckCommand": "cargo +stable check -p liquidbounce-horizon",
        "integratedTypeCheckCommand": "cargo +stable check -p liquidbounce-native-game",
        "integratedBuildCommand": "cargo +stable build -p liquidbounce-native-game",
        "buildStarted": False,
    }
    write_changed(stage / ".liquidbounce-staging.json", (json.dumps({**baseline, **report}, indent=2) + "\n").encode())
    write_changed(PROJECT / "build/staging-report.json", (json.dumps(report, indent=2) + "\n").encode())
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
