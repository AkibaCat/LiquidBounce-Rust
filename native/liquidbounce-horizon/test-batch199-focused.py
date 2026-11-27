#!/usr/bin/env python3
"""One Cargo invocation for the two newly added modules, without repeating batch197."""
from pathlib import Path
import argparse,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def prepare(out):
 out.mkdir(parents=True,exist_ok=True)
 reports={}
 for name in ['block-esp','click-gui']:
  target=out/name
  reports[name]=runpy.run_path(str(HERE/f'test-{name}-focused.py'))['prepare'](target)
 model=ROOT/'native/liquidbounce-behaviors/src'
 code='#![allow(dead_code,unused_imports)]\nextern crate self as liquidbounce_behaviors;\n'
 for name in ['breadcrumbs','storage_esp','block_esp','click_gui']:
  code+=f'#[path="{(model/(name+".rs")).as_posix()}"]pub mod {name};\n'
 for name in ['block-esp','click-gui']:
  code+=f'#[path="{name}/config.rs"]mod {name.replace("-","_")}_config;\n'
 (out/'lib.rs').write_text(code,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="batch199-focused"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
 (out/'preparation.json').write_text(json.dumps(reports,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--out',required=True);ap.add_argument('--run',action='store_true');ap.add_argument('--filter');args=ap.parse_args();out=(ROOT/args.out).resolve();prepare(out)
 if not args.run:print(f'Two-module focused crate prepared: {out}');raise SystemExit()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
 command=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')]
 if args.filter:command.append(args.filter)
 command+=['--','--test-threads=1','--skip','breadcrumbs::','--skip','storage_esp::']
 result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
 suffix='-'+args.filter if args.filter else '';log=out/f'test{suffix}.log';log.write_text(result.stdout+result.stderr,encoding='utf8');print(result.stdout+result.stderr)
 report={'command':command,'exitCode':result.returncode,'passed':result.returncode==0,'log':str(log),'scope':'New BlockESP and ClickGUI production models/config parsers only. Batch197 tests are not repeated.','nativeCompile':'separate batch report','gameValidation':'not_run','upstreamDifferentialValidation':'not_run'}
 (out/f'result{suffix}.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');raise SystemExit(result.returncode)
