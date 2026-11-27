#!/usr/bin/env python3
"""Compile complete generated ItemEntity/TNT moves with a controlled scene."""
from pathlib import Path
import json
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = ROOT / "build/entity-bounce-tests"


def main():
    ns = runpy.run_path(str(HERE / "entity-bounce-patch.py"))
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
    item = changed["crates/horizon-client/src/entities.rs"]
    tnt = changed["crates/horizon-client/src/game/tnt.rs"]
    replacements = {
        "VERTICAL_COLLISION_VELOCITY": method(item, "pub fn vertical_collision_velocity("),
        "MOVE_APPLIES": method(item, "pub fn move_applies("),
        "ITEM_WRAPPER": method(item, "pub fn tick_move_before_block_effects("),
        "ITEM_MOVE": method(item, "pub fn tick_move_before_block_effects_with_bounce("),
        "TNT_WRAPPER": method(tnt, "pub(super) fn tick_among("),
        "TNT_MOVE": method(tnt, "pub(super) fn tick_among_with_bounce("),
        "CAPTURE": ns["CAPTURE"],
    }
    source = (HERE / "entity-bounce-fixture.rs").read_text(encoding="utf-8")
    for name, code in replacements.items():
        placeholder = f"__{name}__"
        if source.count(placeholder) != 1:
            raise ValueError(f"Expected one fixture placeholder: {placeholder}")
        source = source.replace(placeholder, code)
    # Client item tables pass the captured policy into the actual full move;
    # vanilla/server wrappers above always pass false and retain their API.
    tables = changed["crates/horizon-client/src/game/item_entities/ahead.rs"]
    assert "Some(&self.motion.blocks),\n            self.client_suppress_bounce,\n" in tables
    assert "|| game.remote.is_some()" in tables
    assert "|| !horizon_client::mod_api::mods().is_empty()" in tables
    remote = changed["crates/horizon-client/src/game/remote_projectiles.rs"]
    assert remote.index("let client_suppress_bounce =") < remote.index("self.tnts.retain_mut(")
    assert "if client_suppress_bounce {" in remote
    assert "blocks, &[], true," in remote
    assert "t.tick(world, collision, assets, facts, fluids, border, blocks);" in remote
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = '''[package]
name = "liquidbounce-entity-bounce-tests"
version = "0.1.0"
edition = "2024"

[dependencies]
horizon-core = { path = "__CORE__" }
'''.replace("__CORE__", (ROOT / "upstream/Horizon/crates/horizon-core").as_posix())
    (OUT / "Cargo.toml").write_text(manifest, encoding="utf-8")
    (OUT / "src").mkdir(exist_ok=True)
    (OUT / "src/lib.rs").write_text(source, encoding="utf-8")
    result = subprocess.run(["cargo", "+stable", "test", "--manifest-path", str(OUT / "Cargo.toml")], cwd=ROOT)
    evidence = {"pristinePaths": list(changed), "hookFragment": fragment, "exitCode": result.returncode, "productionMethods": list(replacements), "scope": "complete_transformed_item_and_tnt_move_methods_with_controlled_collision_scene_and_original_default_entry_points"}
    (OUT / "result.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
