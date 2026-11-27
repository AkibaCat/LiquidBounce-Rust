from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'crosshair-freeze.json';assert not target.exists(),'Refusing to overwrite historical freeze'
paths=[ROOT/'native/liquidbounce-behaviors/src/crosshair.rs',ROOT/'native/liquidbounce-behaviors/vendor/minecraft-math/sine-table.bin',HERE/'engine/src/config/crosshair.rs',HERE/'engine/src/crosshair_runtime.rs']
paths +=[HERE/name for name in ['crosshair-game.rs','crosshair-host.rs','crosshair-raster.rs','crosshair-patch.py','crosshair-native-fixture.rs','crosshair-smoke.rs','test-crosshair.py','integrate-crosshair.py','update-crosshair-locales.py','crosshair-integration.md','generate-crosshair-freeze.py']]
for rev in range(1,4):paths +=[ROOT/f'build/crosshair-smoke-{rev:02}/{name}'for name in ['report.json','test.log']]
paths +=list((ROOT/'build/crosshair-smoke-03').glob('*.rs'))+[ROOT/'build/crosshair-smoke-03/Cargo.toml']+list((ROOT/'build/crosshair-smoke-03/localization').glob('*.json'))
paths +=list((ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/render/crosshair').rglob('*.kt'))
paths +=[ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/shaders/circle/gui_circle_lut.fsh',ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/render/gui/GuiCircleLutAtlas.kt',ROOT/'upstream/LiquidBounce/src/main/java/net/ccbluex/liquidbounce/render/gui/element/CircleGuiElementRenderState.java',ROOT/'native/liquidbounce-interop/src/localization.rs']
focused=json.loads((ROOT/'build/crosshair-smoke-03/report.json').read_text(encoding='utf8'));assert focused['passed']
data={'module':'Crosshair','modes':['Circle','CS2'],'date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','horizonMixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths}}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
