from pathlib import Path
import json,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 target=HERE/'breadcrumbs-freeze.json'
 if target.exists():raise SystemExit('Refusing to overwrite historical freeze')
 files=[ROOT/'native/liquidbounce-behaviors/src/breadcrumbs.rs',HERE/'engine/src/config/breadcrumbs.rs',HERE/'engine/src/breadcrumbs_runtime.rs']
 files+=[HERE/name for name in ['breadcrumbs-host.rs','breadcrumbs-game.rs','breadcrumbs-render.rs','breadcrumbs.wgsl','breadcrumbs-patch.py','breadcrumbs-smoke.rs','test-breadcrumbs.py','integrate-breadcrumbs.py','update-breadcrumbs-locales.py','breadcrumbs-integration.md','generate-breadcrumbs-freeze.py']]
 files+=list(sorted((HERE/'breadcrumbs-shaders').iterdir()))
 files+=[ROOT/'build/breadcrumbs-smoke-03/report.json',ROOT/'build/breadcrumbs-smoke-03/test.log']
 focused=json.loads((ROOT/'build/breadcrumbs-smoke-03/report.json').read_text(encoding='utf8'));assert focused['passed']
 report={'module':'Breadcrumbs','date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files}}
 target.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
if __name__=='__main__':main()
