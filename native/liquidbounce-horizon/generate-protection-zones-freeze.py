from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 target=HERE/'protection-zones-freeze.json';assert not target.exists(),'Refusing to overwrite historical freeze'
 files=[ROOT/'native/liquidbounce-behaviors/src/protection_zones.rs',HERE/'engine/src/config/protection_zones.rs',HERE/'engine/src/protection_zones_runtime.rs']
 files +=[HERE/name for name in['protection-zones-host.rs','protection-zones-game.rs','protection-zones-patch.py','protection-zones-smoke.rs','protection-zones-native-fixture.rs','test-protection-zones.py','integrate-protection-zones.py','update-protection-zones-locales.py','protection-zones-integration.md','generate-protection-zones-freeze.py']]
 files +=[ROOT/'build/protection-zones-smoke-01/report.json',ROOT/'build/protection-zones-smoke-01/test.log']
 focused=json.loads((ROOT/'build/protection-zones-smoke-01/report.json').read_text(encoding='utf8'));assert focused['passed']
 data={'module':'ProtectionZones','date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files}}
 target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
if __name__=='__main__':main()
