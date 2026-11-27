from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'hole-esp-freeze.json';assert not target.exists(),'Historical freeze is immutable'
paths=[ROOT/f'native/liquidbounce-behaviors/src/{name}.rs'for name in ['hole_tracker','hole_esp']]
paths +=[HERE/'engine/src/config/hole_esp.rs',HERE/'engine/src/hole_esp_runtime.rs']
paths +=[HERE/name for name in ['hole-esp-game.rs','hole-esp-host.rs','hole-esp-render.py','hole-esp-patch.py','integrate-hole-esp.py','update-hole-esp-locales.py','hole-esp-smoke.rs','hole-esp-native-fixture.rs','test-hole-esp.py','hole-esp-integration.md','generate-hole-esp-freeze.py','breadcrumbs-render.rs','breadcrumbs.wgsl']]
for revision in [1,2,3]:
 out=ROOT/f'build/hole-esp-smoke-{revision:02}';paths +=[out/'report.json',out/'test.log',out/'Cargo.toml',out/'shared-plan.json']+list(out.glob('*.rs'))+list((out/'localization').glob('*.json'))+list((out/'patched').rglob('*.rs'))
base=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
paths +=[base/'features/module/modules/render/ModuleHoleESP.kt',base/'utils/block/MovableRegionScanner.kt',base/'utils/math/BlockBoxExtensions.kt',base/'render/RenderShortcuts.kt',base/'render/engine/type/Color4b.kt']+[base/f'utils/block/hole/{name}.kt'for name in ['Hole','HoleTracker','HoleManager']]
report=json.loads((ROOT/'build/hole-esp-smoke-03/report.json').read_text(encoding='utf8'));assert report['passed']
data={'feature':'HoleESP, HoleTracker and HoleManager','date':'2026-11-26','moduleCountDelta':1,'pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','horizonMixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'focused':report,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths}}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
