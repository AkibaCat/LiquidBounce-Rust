from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]

def main():
    target=HERE/'placement-esp-freeze.json';assert not target.exists()
    files=[ROOT/f'native/liquidbounce-behaviors/src/{name}.rs'for name in ['placement_renderer','prophunt_esp','void_esp','void_search']]
    files+=[HERE/'engine/src/config/placement_esp.rs',HERE/'engine/src/placement_esp_runtime.rs']
    files+=[HERE/name for name in ['placement-esp-host.rs','placement-esp-game.rs','placement-esp-patch.py','placement-esp-native-fixture.rs','placement-esp-smoke.rs','test-placement-esp.py','integrate-placement-esp.py','update-placement-esp-locales.py','placement-esp-integration.md','generate-placement-esp-freeze.py']]
    files+=[ROOT/'build/placement-esp-smoke-01/report.json',ROOT/'build/placement-esp-smoke-01/test.log']
    focused=json.loads((ROOT/'build/placement-esp-smoke-01/report.json').read_text(encoding='utf8'));assert focused['passed']
    data={'modules':['ProphuntESP','VoidESP'],'date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files}}
    target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())

if __name__=='__main__':main()
