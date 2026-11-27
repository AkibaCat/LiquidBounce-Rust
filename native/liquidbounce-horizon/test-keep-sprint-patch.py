#!/usr/bin/env python3
"""Compile complete KeepSprint attack/tail methods against the real generated API."""
from pathlib import Path
import json
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = ROOT / "build/keep-sprint-tests"


def main():
    ns = runpy.run_path(str(HERE / "keep-sprint-patch.py"))
    method = runpy.run_path(str(HERE / "test-movement-patch.py"))["method"]
    changed = {}

    def replace_once(text, original, replacement):
        if text.count(original) != 1:
            raise ValueError(f"Expected one occurrence: {original[:100]}")
        return text.replace(original, replacement, 1)

    def edit(path, transform):
        current = changed.get(path)
        if current is None:
            current = (ROOT / "upstream/Horizon" / path).read_text(encoding="utf-8")
        changed[path] = transform(current)

    fragment = ns["apply"](edit, replace_once)
    source = (HERE / "keep-sprint-fixture.rs").read_text(encoding="utf-8")
    replacements = {
        "CLIENT_GUARD": method(changed["crates/horizon-client/src/motion.rs"], "pub fn native_client_combat_queries_allowed("),
        "HOOKS": ns["HOOKS"],
        "REMOTE_ATTACK": method(changed["crates/horizon-client/src/game/remote.rs"], "fn remote_player_attack("),
        "EXTRA_KNOCKBACK": method(changed["crates/horizon-client/src/game/enchant_effects.rs"], "pub(super) fn extra_knockback("),
        "KNOCKBACK_TAIL": method(changed["crates/horizon-client/src/game/enchant_effects.rs"], "pub(super) fn stop_sprint_after_extra_knockback("),
        "SPECIAL_TAIL": method(changed["crates/horizon-client/src/game/special_attacks.rs"], "pub(super) fn special_attack_slowdown("),
        "ATTACK_EFFECTS": method(changed["crates/horizon-client/src/game/melee.rs"], "pub(super) fn attack_effects("),
    }
    for name, code in replacements.items():
        source = source.replace(f"__{name}__", code.replace("fn remote_player_attack(", "pub(crate) fn remote_player_attack("))
    source = source.replace("__KEEP_SPRINT_PATH__", (ROOT / "native/liquidbounce-behaviors/src/keep_sprint.rs").as_posix())
    source = source.replace("__RANDOM_PATH__", (ROOT / "native/liquidbounce-behaviors/src/random.rs").as_posix())
    source = source.replace("__ATTACK_STRENGTH_PATH__", (ROOT / "upstream/Horizon/crates/horizon-client/src/attack_strength.rs").as_posix())
    java = ROOT.parent / "sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe"
    kotlin = ROOT.parent / "sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib/kotlin-stdlib-2.4.21.jar"
    reference = subprocess.run([str(java), "--add-opens", "java.base/java.lang=ALL-UNNAMED", "--class-path", str(kotlin), str(HERE / "KeepSprintReference.java")], capture_output=True, text=True, check=True)
    vectors = json.loads(reference.stdout)
    source = source.replace("__JAVA_SEED__", str(vectors["seed"])).replace("__JAVA_THREAD_ID__", str(vectors["threadId"])).replace("__JAVA_BITS__", str(vectors["drawBits"]))
    ranges = [f'({v["first"]},{v["last"]},{v["values"]},{v["next"]})' for v in vectors["inclusive"]]
    source = source.replace("__JAVA_INT_RANGES__", "[" + ",".join(ranges) + "]")
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "java-vectors.json").write_text(json.dumps(vectors, indent=2) + "\n", encoding="utf-8")
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
name = "liquidbounce-keep-sprint-tests"
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
                "productionMethods": list(replacements), "apiGenerator": "horizon_mod_api_gen::generate_test_hooks",
                "scope": "complete_transformed_remote_Player.attack,_extra_knockback,_shared_causeExtraKnockback_tail,_special_tail,_attack_effects_and_all_host_hooks;JDK25_ThreadLocalRandom_seeded_float_oracle;full_client_differential_not_run"}
    (OUT / "result.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
