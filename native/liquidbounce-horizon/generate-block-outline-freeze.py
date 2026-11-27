from pathlib import Path
import json,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 target=HERE/'block-outline-freeze.json'
 if target.exists():raise SystemExit('Refusing to overwrite historical freeze')
 files=[ROOT/'native/liquidbounce-behaviors/src/block_outline.rs',ROOT/'native/liquidbounce-behaviors/src/block_outline_surface.rs',HERE/'engine/src/config/block_outline.rs',HERE/'engine/src/block_outline_runtime.rs']
 files += [HERE/name for name in ['block-outline-host.rs','block-outline-game.rs','block-outline-patch.py','block-outline-smoke.rs','test-block-outline.py','integrate-block-outline.py','update-block-outline-locales.py','block-outline-integration.md','generate-block-outline-freeze.py']]
 files += [ROOT/'build/block-outline-smoke-01/report.json',ROOT/'build/block-outline-smoke-01/test.log']
 focused=json.loads((ROOT/'build/block-outline-smoke-01/report.json').read_text(encoding='utf8'));assert focused['passed']
 report={'module':'BlockOutline','date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files}}
 target.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
if __name__=='__main__':main()
