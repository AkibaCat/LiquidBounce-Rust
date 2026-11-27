from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'wings-freeze.json'
assert not target.exists(),'Refusing to overwrite historical freeze'
paths=[ROOT/'native/liquidbounce-behaviors/src/wings.rs',HERE/'engine/src/config/wings.rs',HERE/'engine/src/wings_runtime.rs']
paths +=[HERE/name for name in['wings-game.rs','wings-host.rs','wings-patch.py','wings-render.py','wings.wgsl','wings-smoke.rs','wings-native-fixture.rs','test-wings.py','integrate-wings.py','update-wings-locales.py','wings-integration.md','generate-wings-freeze.py','breadcrumbs-freeze-02.json','breadcrumbs-render.rs','breadcrumbs.wgsl','hats-freeze-04.json']]
paths +=[ROOT/'native/liquidbounce-behaviors/src/breadcrumbs.rs',ROOT/'native/liquidbounce-behaviors/vendor/minecraft-math/sine-table.bin',HERE/'engine/src/config/breadcrumbs.rs',ROOT/'native/liquidbounce-interop/src/localization.rs']
for revision in range(1,4):paths +=[ROOT/f'build/wings-smoke-{revision:02}/{name}'for name in['report.json','test.log']]
paths +=[ROOT/'build/wings-smoke-03'/name for name in['Cargo.toml','config.rs','lib.rs','render.rs','localization.rs','localization/display_zh_cn.json','localization/display_zh_tw.json']]
paths +=list((ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/render/wings').rglob('*.kt'))
focused=json.loads((ROOT/'build/wings-smoke-03/report.json').read_text(encoding='utf8'));assert focused['passed']
data={'module':'Wings','modes':['Lines'],'date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','horizonMixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths}}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
