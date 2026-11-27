"""Freeze full StorageESP implementation and focused historical evidence."""
from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];target=HERE/'storage-esp-freeze.json'
assert not target.exists(),'Historical freeze is immutable'
names=['storage-esp-game.rs','storage-esp-model.wgsl','storage-esp-sobel.frag','storage-esp-render.py','storage-esp-patch.py','integrate-storage-esp.py','update-storage-esp-locales.py','test-storage-esp.py','storage-esp-smoke.rs','storage-esp-integration.md','generate-storage-esp-freeze.py','engine/src/config/storage_esp.rs','engine/src/storage_esp_runtime.rs']
paths=[HERE/n for n in names]+[ROOT/f'native/liquidbounce-behaviors/src/{n}.rs'for n in ['storage_esp','storage_esp_surface']]
for attempt in range(1,6):
 paths.append(ROOT/f'build/storage-esp-smoke-{attempt:02}/report.json')
 if attempt<=2:paths.append(ROOT/f'build/storage-esp-smoke-{attempt:02}/test.log')
for lang in ['zh_cn','zh_tw']:paths.append(ROOT/f'build/storage-esp-smoke-02/localization/display_{lang}.json')
data={'feature':'StorageESP','date':'2026-11-26','moduleCountDelta':1,'pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','horizonMixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'focused':json.loads((ROOT/'build/storage-esp-smoke-05/report.json').read_text(encoding='utf8')),'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},'limits':'No shared/stage/upstream writes. Native Game compilation, actual game parity and performance remain root aggregate verification. Current native map snapshot refreshes per frame instead of source scanner caching. AutoBreak/ItemScroller untouched.'}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
