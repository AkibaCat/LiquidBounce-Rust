#!/usr/bin/env python3
"""Freeze TargetLock owned production and focused evidence at handoff."""
from pathlib import Path
import argparse,hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def hashes(paths):return {p.relative_to(ROOT).as_posix():sha(p)for p in sorted(set(paths))}
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',default='native/liquidbounce-horizon/target-lock-freeze.json');a=p.parse_args();out=ROOT/a.out
 if out.exists():raise SystemExit('Refusing to overwrite freeze')
 owned=[ROOT/'native/liquidbounce-behaviors/src/target_lock.rs',HERE/'engine/src/config/target_lock.rs',HERE/'engine/src/target_lock_runtime.rs',HERE/'target-lock-host.rs',HERE/'target-lock-patch.py']
 reports=[]
 for name in ['source-01','tests-01']:
  path=ROOT/f'build/target-lock-{name}/report.json';report=json.loads(path.read_text(encoding='utf8'));assert report['passed'];reports.append({'path':path.relative_to(ROOT).as_posix(),'sha256':sha(path),'scope':report['scope']})
  for relative,digest in report['sources'].items():assert sha(ROOT/relative)==digest,relative
 support=[p for p in HERE.glob('*target-lock*')if p.is_file()and p not in owned and p!=out];support.extend(p for p in(HERE/'target-lock-source-fixture').rglob('*')if p.is_file())
 source=json.loads((ROOT/'build/target-lock-source-01/report.json').read_text(encoding='utf8'))
 data={'frozenDate':'2026-11-26','revision':1,'pinnedSources':{'LiquidBounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','Horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','Horizon-Mixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'files':hashes(owned),'supportingFiles':hashes(support),'reports':reports,'vectors':{'scenarios':source['scenarios'],'steps':source['steps']},'scope':'Owned implementation frozen before shared registration. One focused compile and 10 smoke tests, short untouched original Kotlin/ListValue oracle. State/Host/Game/mode/events are explicit providers. No complete engine/concrete Game or actual game proof. Shared files and localization dictionaries unchanged.','pendingChecks':['Root shared registration/config/actor/cache and original event-order integration.','Root complete cumulative concrete Game/engine/package build and final full game validation.']}
 out.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps({'path':str(out.relative_to(ROOT)),'sha256':sha(out),'steps':source['steps']}))
if __name__=='__main__':main()
