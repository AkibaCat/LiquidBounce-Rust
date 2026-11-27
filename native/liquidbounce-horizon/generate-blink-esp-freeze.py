from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'blink-esp-freeze.json';assert not target.exists(),'Historical freeze is immutable'
paths=[ROOT/'native/liquidbounce-behaviors/src/blink_esp.rs',HERE/'engine/src/config/blink_esp.rs',HERE/'engine/src/blink_esp_runtime.rs']
paths +=[HERE/name for name in['blink-esp-host.rs','blink-esp-game.py','blink-esp-render.py','blink-esp-patch.py','integrate-blink-esp.py','update-blink-esp-locales.py','blink-esp-smoke.rs','test-blink-esp.py','blink-esp-integration.md','generate-blink-esp-freeze.py','breadcrumbs-render.rs','breadcrumbs.wgsl']]
for revision in[1,2]:
 out=ROOT/f'build/blink-esp-smoke-{revision:02}'
 paths +=[out/'report.json',out/'test.log',out/'Cargo.toml']+list(out.glob('*.rs'))+list((out/'localization').glob('*.json'))+[out/'blink_esp.wgsl']
 paths +=list((out/'patched').rglob('*.rs'))+list((out/'patched').rglob('*.wgsl'))
paths +=[ROOT/'upstream/Horizon/crates/horizon-client/src/game/perspective.rs',ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/blink/BlinkManager.kt',ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/blink/esp/BlinkEspMode.kt',ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/render/WireframePlayer.kt',ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/render/RenderStateExtensions.kt',ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/shaders/circle/gradient_circle.fsh',ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/shaders/circle/rounded_rect.fsh']
report=json.loads((ROOT/'build/blink-esp-smoke-02/report.json').read_text(encoding='utf8'));assert report['passed']
data={'feature':'BlinkManager global ESP','date':'2026-11-26','moduleCountDelta':0,'pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','horizonMixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'focused':report,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths}}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
