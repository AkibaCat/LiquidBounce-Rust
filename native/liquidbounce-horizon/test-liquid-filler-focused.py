#!/usr/bin/env python3
"""Prepare production LiquidFiller/BlockPlacer model smoke; Cargo only with --run."""
from pathlib import Path
import argparse,hashlib,json,os,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def prepare(out):
 out.mkdir(parents=True,exist_ok=True)
 model=HERE.parent/'liquidbounce-behaviors/src/liquid_filler.rs';placer=model.with_name('block_placer.rs')
 smoke=(HERE/'liquid-filler-smoke.rs').read_text(encoding='utf8')
 smoke=smoke.replace('../liquidbounce-behaviors/src/liquid_filler.rs',model.as_posix()).replace('../liquidbounce-behaviors/src/block_placer.rs',placer.as_posix()).replace('fn main(){','#[test]\nfn liquid_filler_and_block_placer_source_smoke(){',1)
 (out/'lib.rs').write_text(smoke,encoding='utf8')
 (out/'Cargo.toml').write_text('''[package]
name="liquid-filler-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
''',encoding='utf8')
 sources=[model,placer,HERE/'liquid-filler-smoke.rs',HERE/'engine/src/config/liquid_filler.rs',HERE/'engine/src/block_placer_runtime.rs',HERE/'engine/src/liquid_filler_runtime.rs']
 report={'status':'prepared-no-compiler-invoked','tests':1,'scope':'Actual LiquidFiller and BlockPlacer models via existing smoke: source/filter/slots, five orders, sponge BFS limits, retained queue/cooldown, executor phases and crystal selection. Two production config tests remain in the unified engine test batch (filter liquid_filler_config); native world/packet/render execution is not exercised.','deferredConfigTests':['liquid_filler_config_roundtrip_keeps_inactive_modes_and_renderers','liquid_filler_config_toggle_preserves_target_and_rejects_unknown_blocks'],'sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}}
 (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report
if __name__=='__main__':
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--out',required=True);ap.add_argument('--run',action='store_true');a=ap.parse_args();out=ROOT/a.out;prepare(out)
 if not a.run:print(f'LiquidFiller fixture prepared: {out}; no compiler invoked');raise SystemExit()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
 r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
 (out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':r.returncode==0,'exitCode':r.returncode},indent=2)+'\n',encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
