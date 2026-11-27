"""Freeze the complete Particles delivery and its focused evidence without shared mutation."""
from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];target=HERE/'particles-freeze.json'
assert not target.exists(),'Historical freeze is immutable'
names=['particles-game.rs','particles-host.rs','particles-render.rs','particles.wgsl','particles-patch.py','integrate-particles.py','update-particles-locales.py','particles-smoke.rs','particles-native-fixture.rs','test-particles.py','particles-integration.md','generate-particles-freeze.py','engine/src/config/particles.rs','engine/src/particles_runtime.rs']
paths=[HERE/name for name in names]+[ROOT/'native/liquidbounce-behaviors/src/particles.rs',ROOT/'build/particles-smoke-01/report.json',ROOT/'build/particles-smoke-01/test.log',ROOT/'build/particles-smoke-02/report.json',ROOT/'build/particles-smoke-02/test.log',ROOT/'build/particles-smoke-02/shared-plan.json']
for language in ['zh_cn','zh_tw']:paths.append(ROOT/f'build/particles-smoke-02/localization/display_{language}.json')
data={'feature':'Particles','date':'2026-11-26','moduleCountDelta':1,'pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','horizonMixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'focused':json.loads((ROOT/'build/particles-smoke-02/report.json').read_text(encoding='utf8')),'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},'limits':'No official stage/build or real-game parity claim. Root applies 18 edits across 7 shared files and copied-dictionary updater serially. Frozen AutoBreak and ItemScroller untouched.'}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
