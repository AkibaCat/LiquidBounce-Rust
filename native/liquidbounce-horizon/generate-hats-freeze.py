from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 target=HERE/'hats-freeze.json';assert not target.exists(),'Refusing to overwrite historical freeze'
 files=[ROOT/'native/liquidbounce-behaviors/src/hats.rs',HERE/'engine/src/config/hats.rs',HERE/'engine/src/hats_runtime.rs']
 files +=[HERE/name for name in['hats-host.rs','hats-game.rs','hats-patch.py','hats-render.py','hats-image-render.rs','hats-image.wgsl','hats-smoke.rs','hats-native-fixture.rs','test-hats.py','integrate-hats.py','update-hats-locales.py','hats-integration.md','generate-hats-freeze.py']]
 for revision in range(1,5):files +=[ROOT/f'build/hats-smoke-{revision:02}/{name}'for name in['report.json','test.log']]
 files +=[ROOT/'build/hats-smoke-04/Cargo.toml',ROOT/'build/hats-smoke-04/hud.rs',ROOT/'native/liquidbounce-behaviors/src/navigation/simulation/sine-table.bin',ROOT/'native/liquidbounce-behaviors/src/tracers_curve.rs',HERE/'breadcrumbs-render.rs',HERE/'breadcrumbs.wgsl']
 files +=list((ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/render/hats').rglob('*.kt'))
 focused=json.loads((ROOT/'build/hats-smoke-04/report.json').read_text(encoding='utf8'));assert focused['passed']
 data={'module':'Hats','modes':['Cone','Halo','Orbs','Flower','Star','Image'],'date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0','horizonMixins':'5a64af938ec86b00d53d16a408aabc0038fb3e57'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files}}
 target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
if __name__=='__main__':main()
