#!/usr/bin/env python3
"""Prepare production BedDefender model tests; Cargo is invoked only with --run."""
from pathlib import Path
import argparse,hashlib,json,os,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def prepare(out):
 out.mkdir(parents=True,exist_ok=True);core=HERE.parent/'liquidbounce-behaviors';model=core/'src/bed_defender.rs'
 (out/'lib.rs').write_text(f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
pub use behavior_core::bed_plates;
#[path="{model.as_posix()}"]pub mod bed_defender;
''',encoding='utf8')
 (out/'Cargo.toml').write_text('''[package]
name="bed-defender-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
'''+f'behavior_core={{package="liquidbounce-behaviors",path="{core.as_posix()}"}}\n',encoding='utf8')
 sources=[model,core/'src/bed_plates.rs',core/'src/bed_plates_scan.rs',HERE/'engine/src/config/bed_defender.rs',HERE/'engine/src/bed_defender_runtime.rs',HERE/'bed-defender-native.rs']
 report={'status':'prepared-no-compiler-invoked','tests':4,'scope':'Four exact production BedDefender model tests: block hardness/count/selected-slot order, layer and stable distance order, self-bed selection and inclusive box reach. The production BedPlates self-bed/scan code is a dependency. Two full configuration tests remain in the unified engine batch; native packet/input/game/render behavior is not executed.','deferredConfigFilter':'bed_defender_','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}}
 (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report
if __name__=='__main__':
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--out',required=True);ap.add_argument('--run',action='store_true');a=ap.parse_args();out=ROOT/a.out;prepare(out)
 if not a.run:print(f'BedDefender focused fixture prepared: {out}; no compiler invoked');raise SystemExit()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
 r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':r.returncode==0,'exitCode':r.returncode},indent=2)+'\n',encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
