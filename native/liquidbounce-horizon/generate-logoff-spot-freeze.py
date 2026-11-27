"""Freeze complete LogoffSpot delivery and focused historical evidence."""
from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];target=HERE/'logoff-spot-freeze.json'
assert not target.exists(),'Historical freeze is immutable'
names=['logoff-spot-game.rs','logoff-spot-host.rs','logoff-spot-remote.rs','logoff-spot-patch.py','integrate-logoff-spot.py','update-logoff-spot-locales.py','logoff-spot-smoke.rs','logoff-spot-native-fixture.rs','test-logoff-spot.py','logoff-spot-integration.md','generate-logoff-spot-freeze.py','engine/src/config/logoff_spot.rs','engine/src/logoff_spot_runtime.rs']
paths=[HERE/n for n in names]
for attempt in range(1,5):
 for name in ['report.json','test.log']:paths.append(ROOT/f'build/logoff-spot-smoke-{attempt:02}/{name}')
paths.append(ROOT/'build/logoff-spot-smoke-04/shared-plan.json')
for lang in ['zh_cn','zh_tw']:paths.append(ROOT/f'build/logoff-spot-smoke-04/interop/resources/localization/display_{lang}.json')
data={'feature':'LogoffSpot','date':'2026-11-26','moduleCountDelta':1,'pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','horizonMixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'focused':json.loads((ROOT/'build/logoff-spot-smoke-04/report.json').read_text(encoding='utf8')),'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},'limits':'No official build/stage or real-game parity claim. Root applies 20 edits across seven shared files and copied dictionary updater serially. AutoBreak and ItemScroller untouched.'}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
