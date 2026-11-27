"""Prepare focused production Radar model/config checks; run only in a combined batch."""
from pathlib import Path
import argparse, ast, base64, hashlib, json, re, runpy, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def prepare(out):
    out.mkdir(parents=True, exist_ok=True)
    body = runpy.run_path(str(HERE / 'test-reset-vl.py'))['body']
    shared = (HERE / 'engine/src/config.rs').read_text(encoding='utf8')
    config = '''use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}
#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]Radar}
impl ModuleKind{fn key(self)->&'static str{"liquidbounce.module.radar"}}
'''
    for anchor in ['pub struct CommonSettings', 'pub struct ConfigError', 'impl ConfigError',
                   "struct Tree<'a>", "impl Tree<'_>", 'fn scalar_string(', "fn nodes<'a>(",
                   'fn float_json(', "fn value_of<'a>(", 'fn parse_float(', 'fn parse_int(',
                   'fn decimal_int(', 'fn parse_bool(']:
        if anchor in ['pub struct ConfigError', 'pub struct CommonSettings']:
            config += '#[derive(Debug)]\n'
        config += body(shared, anchor) + '\n'
    config += "type Nodes<'a>=BTreeMap<String,&'a Value>;\n#[derive(Debug)]pub enum SettingChange{Radar(RadarSetting)}\n#[derive(Default)]" + body(shared, 'pub struct SettingsPatch') + '\n'
    config += f'#[path="{(HERE / "engine/src/config/radar.rs").as_posix()}"]mod settings;pub use settings::*;\n'
    config += "pub fn parse_radar(p:&mut SettingsPatch,r:&Nodes<'_>){settings::parse_radar(p,r);}\n"
    config = config.replace('fn nodes<', 'pub fn nodes<')
    (out / 'config.rs').write_text(config, encoding='utf8')
    lib = '#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;\npub use lb::{breadcrumbs,tracers};\n'
    lib += f'#[path="{(ROOT / "native/liquidbounce-behaviors/src/radar.rs").as_posix()}"]pub mod radar;\nmod config;\n'
    lib += (HERE / 'radar-smoke.rs').read_text(encoding='utf8')
    (out / 'lib.rs').write_text(lib, encoding='utf8')
    manifest = '[package]\nname="radar-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n'
    manifest += f'lb={{package="liquidbounce-behaviors",path="{(ROOT / "native/liquidbounce-behaviors").as_posix()}"}}\n'
    (out / 'Cargo.toml').write_text(manifest, encoding='utf8')
    for name in ['integrate-radar.py', 'radar-patch.py', 'update-radar-locales.py']:
        ast.parse((HERE / name).read_text(encoding='utf8'))
    evidence = json.loads((HERE / 'radar-images-evidence.json').read_text(encoding='utf8'))
    images = re.findall(r'crate::skins::base64_decode\("([^"]+)"\)', (HERE / 'radar-images.rs').read_text(encoding='utf8'))
    assert len(images) == len(evidence) == 2
    for encoded, item in zip(images, evidence):
        image = base64.b64decode(encoded)
        assert image == (ROOT / item['source']).read_bytes()
        assert hashlib.sha256(image).hexdigest() == item['sha256']
    (out / 'preparation.json').write_text(json.dumps({'status': 'prepared-no-compiler-invoked', 'tests': 4, 'images': evidence, 'scope': 'Actual Radar model/config and shared Tree/parser helpers; real reused colors/curves. Native names, targeting and rendering are covered by the official combined check and later game testing.'}, indent=2) + '\n', encoding='utf8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True)
    parser.add_argument('--run', action='store_true')
    args = parser.parse_args()
    out = ROOT / args.out
    prepare(out)
    if not args.run:
        print('Radar fixture prepared; no compiler invoked')
        return
    result = subprocess.run(['cargo', '+stable', 'test', '--offline', '--manifest-path', str(out / 'Cargo.toml'), '--', '--test-threads=1'], capture_output=True, text=True, encoding='utf8', errors='replace')
    (out / 'test.log').write_text(result.stdout + result.stderr, encoding='utf8')
    (out / 'report.json').write_text(json.dumps({'passed': result.returncode == 0, 'exitCode': result.returncode, 'scope': 'Focused production Radar model/config; no game/full-suite claim.'}, indent=2) + '\n', encoding='utf8')
    print(result.stdout + result.stderr)
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
