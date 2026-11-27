#!/usr/bin/env python3
"""Compile actual AirJump/NoPush jump, fluid, fishing and collision methods."""
from pathlib import Path
import ast
import json
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = ROOT / "build/airjump-push-tests"


def main():
    ns = runpy.run_path(str(HERE / "airjump-push-patch.py"))
    method = runpy.run_path(str(HERE / "test-movement-patch.py"))["method"]
    changed = {}

    def replace_once(text, original, replacement):
        if text.count(original) != 1:
            raise ValueError(f"Expected one source occurrence: {original[:100]}")
        return text.replace(original, replacement, 1)

    def edit(path, transform):
        current = changed.get(path)
        if current is None:
            current = (ROOT / "upstream/Horizon" / path).read_text(encoding="utf-8")
        changed[path] = transform(current)

    fragment = ns["apply"](edit, replace_once)
    motion = changed["crates/horizon-client/src/motion.rs"]
    remote = changed["crates/horizon-client/src/game/remote.rs"]
    mounts = changed["crates/horizon-client/src/game/remote_vehicles.rs"]
    assert remote.index("player.client_airjump_push_queries =") < remote.index("self.remote_ride(keys, sprint_window)?")
    assert "ClientJump { on_ground } => self.native_client_player_jump(on_ground)" in remote
    assert "entities_allowed && order(m.id)" in remote
    assert "entities_allowed && bounds.intersects(bb)" in remote
    assert "bounds.intersects(bb.inflate(area))\n                    && horizon_client::scoreboard::pushable_by(None, player_team, true)" in remote
    assert "if p.event == 31 { self.remote_hook_pull_event(p.id as u32); }" in remote
    assert mounts.index("let client_air_jump =") < mounts.index("let m = &mut self.mobs[i];")
    assert mounts.index("mob.extra.client_air_jump = client_air_jump;") < mounts.index("mob.tick(")
    assert mounts.index("mob.tick(") < mounts.index("mob.extra.client_air_jump = false;")
    source = (HERE / "airjump-push-fixture.rs").read_text(encoding="utf-8")
    replacements = {
        "QUERIES": ns["QUERIES"],
        "PLAYER_QUERY": method(motion, "pub(crate) fn client_airjump_push_queries("),
        "JUMP_FROM_GROUND": method(motion, "pub fn jump_from_ground("),
        "JUMP_PHASE": method(changed["crates/horizon-client/src/motion/environment.rs"], "pub(super) fn jump_phase(").replace("pub(super) fn jump_phase", "pub(crate) fn jump_phase", 1),
        "FLUID_FLOW": method(changed["crates/horizon-client/src/motion/fluids.rs"], "fn flow("),
        "MOB_HANDLE_JUMP": method(changed["crates/horizon-client/src/mobs/living.rs"], "pub fn handle_jumping("),
        "FISHING_EVENT": method(changed["crates/horizon-client/src/game/fishing.rs"], "pub(super) fn remote_hook_pull_event("),
        "FISHING_PULL": method(changed["crates/horizon-client/src/game/fishing.rs"], "fn pull_hooked("),
        "COLLISION_SCAN": changed["crates/horizon-world/src/block_collision_scan.rs"],
    }
    for name, code in replacements.items():
        source = source.replace(f"__{name}__", code)
    source = source.replace("__AIR_JUMP_PATH__", (ROOT / "native/liquidbounce-behaviors/src/air_jump.rs").as_posix())
    source = source.replace("__NO_PUSH_PATH__", (ROOT / "native/liquidbounce-behaviors/src/no_push.rs").as_posix())
    host_source = (HERE / "airjump-push-host-fixture.rs").read_text(encoding="utf-8").replace("__AIRJUMP_PUSH_HOST_HOOKS__", ns["HOOKS"])
    source += "\nmod host_fixture {\n" + host_source + "\n}\n"
    # Compose every existing transform in memory, including the main helper's
    # hook declarations. No staging or source tree is regenerated here.
    tree = ast.parse((HERE / "make-behavior-patch.py").read_text(encoding="utf-8"))
    base_hooks = next(ast.literal_eval(node.value) for node in tree.body if isinstance(node, ast.Assign)
                      and any(isinstance(t, ast.Name) and t.id == "hooks" for t in node.targets))
    composed = {}

    def compose_edit(path, transform):
        current = composed.get(path)
        if current is None:
            current = (ROOT / "upstream/Horizon" / path).read_text(encoding="utf-8")
        composed[path] = transform(current)

    compose_edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: replace_once(text,
                 "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n",
                 base_hooks + "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n"))
    extensions = ["movement-patch.py", "entity-bounce-patch.py", "attack-patch.py", "interpolation-patch.py", "sprint-patch.py", "airjump-push-patch.py"]
    for extension in extensions:
        runpy.run_path(str(HERE / extension))["apply"](compose_edit, replace_once)
    OUT.mkdir(parents=True, exist_ok=True)
    api_root = OUT / "api-source"
    api_game = api_root / "crates/horizon-client/src/game"
    api_game.mkdir(parents=True, exist_ok=True)
    (api_root / "tools/native-hook-fragments").mkdir(parents=True, exist_ok=True)
    (api_root / "crates/horizon-client/src/mod_api.rs").write_text("", encoding="utf-8")
    (api_game / "native_hooks.rs").write_text("impl Game<'_> {\n" + ns["HOOKS"] + "\n}\n", encoding="utf-8")
    catalog = {"helpers": [], "hooks": [{"name": name, "target": f"horizon_client::game::native_hooks::Game<'_>::native_{name}", "mod_api": {"contract": "listen"}} for name in ns["HOOK_NAMES"]]}
    (api_root / "native-hooks.json").write_text(json.dumps(catalog, indent=2) + "\n", encoding="utf-8")
    (OUT / "build.rs").write_text('fn main() {\nlet root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");\nlet out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());\nhorizon_mod_api_gen::generate_test_hooks(&root, &out).unwrap();\n}\n', encoding="utf-8")
    manifest = '''[package]
name = "liquidbounce-airjump-push-tests"
version = "0.1.0"
edition = "2024"

[dependencies]
horizon-core = { path = "__CORE__" }
horizon-world = { path = "__WORLD__" }

[build-dependencies]
horizon-mod-api-gen = { path = "__GENERATOR__" }
'''.replace("__CORE__", (ROOT / "upstream/Horizon/crates/horizon-core").as_posix()).replace("__WORLD__", (ROOT / "upstream/Horizon/crates/horizon-world").as_posix()).replace("__GENERATOR__", (ROOT / "upstream/Horizon/crates/horizon-mod-api-gen").as_posix())
    (OUT / "Cargo.toml").write_text(manifest, encoding="utf-8")
    (OUT / "src").mkdir(exist_ok=True)
    (OUT / "src/lib.rs").write_text(source, encoding="utf-8")
    result = subprocess.run(["cargo", "+stable", "test", "--manifest-path", str(OUT / "Cargo.toml")], cwd=ROOT)
    evidence = {"pristinePaths": list(changed), "hookFragment": fragment, "exitCode": result.returncode,
                "productionMethods": list(replacements) + ["ALL_HOST_HOOKS"], "composedHelpers": extensions, "apiGenerator": "horizon_mod_api_gen::generate_test_hooks",
                "scope": "complete_transformed_player_jump_phase_and_jumpFromGround,_mob_jump_phase,_fluid_flow,_fishing_event31,_collision_iterator_with_real_Horizon_geometry,_and_all_host_hooks_compiled_against_actual_generated_dispatch;phase_wiring_source_assertions"}
    (OUT / "result.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
