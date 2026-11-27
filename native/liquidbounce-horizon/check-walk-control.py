#!/usr/bin/env python3
"""Check owned walking sources without modifying shared navigation exports."""
import os,re,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
nav=ROOT/'native/liquidbounce-behaviors/src/navigation'
out=ROOT/'build/walk-control-check';out.mkdir(exist_ok=True)
mods=re.findall(r'\bmod (\w+);',(nav/'mod.rs').read_text())
lib='#![allow(dead_code)]\npub mod navigation {\n'
for name in dict.fromkeys([*mods,'walk_control']):
 p=nav/(name+'.rs')
 if not p.is_file():p=nav/name/'mod.rs'
 lib+=f'#[path="{p.as_posix()}"] pub mod {name};\n'
lib+='}\n';(out/'lib.rs').write_text(lib,encoding='utf-8')
(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-walk-control-check"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/walk-control-target')
raise SystemExit(subprocess.call(['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],cwd=ROOT,env=env))
