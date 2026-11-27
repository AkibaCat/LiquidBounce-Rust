from pathlib import Path
import hashlib, json
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]

def main():
    target=HERE/'new-chunks-freeze.json'
    assert not target.exists(), 'Refusing to overwrite historical freeze'
    files=[ROOT/'native/liquidbounce-behaviors/src/new_chunks.rs',HERE/'engine/src/config/new_chunks.rs',HERE/'engine/src/new_chunks_runtime.rs']
    files += [HERE/name for name in ['new-chunks-host.rs','new-chunks-game.rs','new-chunks-patch.py','new-chunks-smoke.rs','new-chunks-native-fixture.rs','test-new-chunks.py','integrate-new-chunks.py','update-new-chunks-locales.py','new-chunks-integration.md','generate-new-chunks-freeze.py']]
    files += [ROOT/'build/new-chunks-smoke-01/report.json',ROOT/'build/new-chunks-smoke-02/report.json',ROOT/'build/new-chunks-smoke-02/test.log']
    focused=json.loads((ROOT/'build/new-chunks-smoke-02/report.json').read_text(encoding='utf8'))
    assert focused['passed']
    data={'module':'NewChunks','date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files}}
    target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8')
    print(hashlib.sha256(target.read_bytes()).hexdigest())

if __name__=='__main__':main()
