#!/usr/bin/env python3
"""Static Nametags checks; never compiles or mutates staging/official locale files."""
from pathlib import Path
import argparse, ast, json, runpy, shutil, subprocess, sys, tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def compose_native():
    source = (HERE / "make-behavior-patch.py").read_text(encoding="utf8")
    source = source[:source.index("\noutput = []")]
    anchor = "'bed-plates-patch.py',"
    assert source.count(anchor) == 1
    source = source.replace(anchor, anchor + " 'nametags-patch.py',")
    context = {"__file__": str(HERE / "make-behavior-patch.py"), "__name__": "nametags_static"}
    exec(compile(source, context["__file__"], "exec"), context)
    assert context["names"].count("client_nametags") == 1
    fragment = json.loads(context["changes"]["tools/native-hook-fragments/liquidbounce_behavior.json"][1])
    hooks = [h for h in fragment["hooks"] if h["name"] == "client_nametags"]
    assert len(hooks) == 1
    assert hooks[0]["target"] == "horizon_client::game::native_hooks::Game<'_>::native_client_nametags"
    return {p: after for p, (_, after) in context["changes"].items()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--native", action="store_true")
    args = ap.parse_args()
    for name in ["integrate-nametags.py", "nametags-patch.py", "update-nametags-locales.py"]:
        ast.parse((HERE / name).read_text(encoding="utf8"))
    for path in [ROOT / "native/liquidbounce-behaviors/src/nametags.rs", HERE / "engine/src/nametags_runtime.rs", HERE / "engine/src/config/nametags.rs", HERE / "nametags-game.rs", HERE / "nametags-snapshot.rs"]:
        assert path.is_file(), path
    assert "if stroke>0." in (HERE / "nametags-game.rs").read_text(encoding="utf8")
    assert "name_with_color" in (HERE / "engine/src/nametags_runtime.rs").read_text(encoding="utf8")
    snapshot = (HERE / "nametags-snapshot.rs").read_text(encoding="utf8")
    assert "g.eye_height()" in snapshot and "p.using()" in snapshot and "body.extra=m.extra.body_extra.clone()" in snapshot
    config = (HERE / "engine/src/config/nametags.rs").read_text(encoding="utf8")
    assert "!registry.is_empty()" in config
    native = compose_native() if args.native else {}
    if args.native:
        hooks = native["crates/horizon-client/src/game/native_hooks.rs"]
        assert "native_client_nametags(&self" in hooks
        assert "client_nametags_vanilla(entity_id,text.is_some())" in hooks
        game = native["crates/horizon-client/src/game.rs"]
        assert "mod nametags;" in game and "client_nametags_overlay(&mut batch);" in game
    with tempfile.TemporaryDirectory(prefix="liquidbounce-nametags-static-") as folder:
        files = [HERE / "engine/src/nametags_runtime.rs", HERE / "engine/src/config/nametags.rs", ROOT / "native/liquidbounce-behaviors/src/nametags.rs", HERE / "nametags-game.rs", HERE / "nametags-snapshot.rs"]
        paths = []
        for i, path in enumerate(files):
            out = Path(folder) / f"{i}.rs"
            out.write_text(path.read_text(encoding="utf8"), encoding="utf8")
            paths.append(str(out))
        result = subprocess.run(["rustfmt", "--edition", "2024", "--config", "skip_children=true", *paths], capture_output=True, text=True)
        if result.returncode:
            raise RuntimeError(result.stderr)
        locale = Path(folder) / "locales"
        locale.mkdir()
        for code in ["zh_cn", "zh_tw"]:
            shutil.copyfile(ROOT / f"native/liquidbounce-interop/resources/localization/display_{code}.json", locale / f"display_{code}.json")
        sys.path.insert(0, str(HERE))
        ctx = runpy.run_path(str(HERE / "update-nametags-locales.py"))
        ctx["update"](locale)
        first = {p.name: p.read_bytes() for p in locale.iterdir()}
        ctx["update"](locale)
        assert first == {p.name: p.read_bytes() for p in locale.iterdir()}
        for raw in first.values():
            data = json.loads(raw)
            assert data["moduleNames"]["Nametags"]
            assert data["descriptions"]["liquidbounce.module.nametags.description"]
            assert data["labels"]["liquidbounce.module.nametags.equipment.slots.Chest"] in ("胸部", "胸部")
    print("Nametags static checks passed: owned source, transactional native composition, temporary rustfmt, idempotent zh_cn/zh_tw updater; no Cargo/build/staging writes")


if __name__ == "__main__":
    main()

