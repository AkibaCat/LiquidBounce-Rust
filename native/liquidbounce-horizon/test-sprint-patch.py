#!/usr/bin/env python3
"""Compile actual transformed SprintState, jump and render extraction code."""
from pathlib import Path
import json
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = ROOT / "build/sprint-hook-tests"


def main():
    ns = runpy.run_path(str(HERE / "sprint-patch.py"))
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
    remote = changed["crates/horizon-client/src/game/remote.rs"]
    assert remote.index("self.native_client_directional_input(") < remote.index("let input = u8::from(keys.forward)")
    assert remote.index("self.client_sprint_queries_for_input(") < remote.index("self.remote_ride(keys, sprint_window)?")
    assert "if actual_sprinting {\n                shared |= 0x08;" in remote
    assert "self.native_client_sprint_network(&mut sprinting);" in remote
    assert "self.native_client_rotation_network(&mut yaw, &mut pitch);" in remote
    assert "keys.analog = None;\n        }\n        let input =" in remote
    motion = changed["crates/horizon-client/src/motion.rs"]
    assert "self.client_sprint_queries.for_body(self.client_side, self.server_copy, self.replaying)" in motion
    assert "state.tick_with_queries(keys, conditions, sprint_window.clamp(0, 10), queries," in motion
    perspective = changed["crates/horizon-client/src/game/perspective.rs"]
    extraction = perspective.split("        let partial = alpha as f32;", 1)[1].split("        let eye = self", 1)[0]
    extraction = "        let partial = alpha as f32;" + extraction
    extraction += "        m.clone()\n"
    input_source = (ROOT / "upstream/Horizon/crates/horizon-client-input/src/input.rs").read_text(encoding="utf-8")
    anim = (ROOT / "upstream/Horizon/crates/horizon-client/src/mobs/anim.rs").read_text(encoding="utf-8")
    riders = (ROOT / "upstream/Horizon/crates/horizon-client/src/mobs/riders.rs").read_text(encoding="utf-8")
    living = (ROOT / "upstream/Horizon/crates/horizon-client/src/mobs/living.rs").read_text(encoding="utf-8")
    source = (HERE / "sprint-fixture.rs").read_text(encoding="utf-8")
    replacements = {
        "INPUT_LENGTH": method(input_source, "fn length("),
        "INPUT_DIRECTION": method(input_source, "pub fn keyboard_direction("),
        "SPRINT": changed["crates/horizon-client/src/sprint.rs"],
        "LIVING_MOTION": changed["crates/horizon-world/src/living_motion.rs"],
        "QUERIES": method(motion, "pub(crate) fn client_sprint_queries("),
        "JUMP_BOOST": method(motion, "fn sprint_jump_boost("),
        "BODY_ROT": method(anim, "pub fn body_rot("),
        "SOLVE_BODY_ROT": method(riders, "pub fn solve_body_rot("),
        "WRAP": method(living, "pub fn wrap("),
        "RENDER_EXTRACTION": extraction,
        "HAND_SWAY_RESET": method(changed["crates/horizon-client/src/first_person.rs"], "pub fn reset_yaw("),
    }
    for name, code in replacements.items():
        source = source.replace(f"__{name}__", code)
    source = source.replace("__LIVING_AIR_PATH__", (ROOT / "upstream/Horizon/crates/horizon-world/src/living_air.rs").as_posix())
    host_source = (HERE / "sprint-host-fixture.rs").read_text(encoding="utf-8").replace("__SPRINT_HOST_HOOKS__", ns["HOOKS"])
    source += "\nmod game { pub mod shield { pub(super) const OFFHAND_SLOT: usize = 40; }\nmod host_fixture {\n" + host_source + "\n} }\n"
    OUT.mkdir(parents=True, exist_ok=True)
    # Exercise the engine's real signature parser instead of a hand-written
    # dispatch substitute, then compile all actual host hook bodies against it.
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
name = "liquidbounce-sprint-hook-tests"
version = "0.1.0"
edition = "2024"

[dependencies]
horizon-core = { path = "__CORE__" }

[build-dependencies]
horizon-mod-api-gen = { path = "__GENERATOR__" }
'''.replace("__CORE__", (ROOT / "upstream/Horizon/crates/horizon-core").as_posix()).replace("__GENERATOR__", (ROOT / "upstream/Horizon/crates/horizon-mod-api-gen").as_posix())
    (OUT / "Cargo.toml").write_text(manifest, encoding="utf-8")
    (OUT / "src").mkdir(exist_ok=True)
    (OUT / "src/lib.rs").write_text(source, encoding="utf-8")
    result = subprocess.run(["cargo", "+stable", "test", "--manifest-path", str(OUT / "Cargo.toml")], cwd=ROOT)
    evidence = {"pristinePaths": list(changed), "hookFragment": fragment, "exitCode": result.returncode,
                "productionMethods": list(replacements) + ["ALL_HOST_HOOKS"], "apiGenerator": "horizon_mod_api_gen::generate_test_hooks",
                "scope": "complete_transformed_SprintState_and_living_jump_with_real_Horizon_trig,_renderer_rotation_extraction,_and_all_host_hooks_compiled_against_actual_generated_dispatch;network_and_phase_wiring_source_assertions"}
    (OUT / "result.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
