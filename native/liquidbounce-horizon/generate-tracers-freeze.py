from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 target=HERE/'tracers-freeze.json'
 if target.exists():raise SystemExit('Refusing to overwrite historical freeze')
 focused=json.loads((ROOT/'build/tracers-smoke-02/report.json').read_text(encoding='utf8'));native=json.loads((ROOT/'build/tracers-native-01/report.json').read_text(encoding='utf8'));assert focused['passed']and native['passed']
 paths=[ROOT/'native/liquidbounce-behaviors/src/tracers.rs',ROOT/'native/liquidbounce-behaviors/src/tracers_curve.rs',HERE/'engine/src/config/tracers.rs',HERE/'engine/src/tracers_runtime.rs']
 paths += [HERE/name for name in ['tracers-game.rs','tracers-host.rs','tracers-render.py','tracers-patch.py','tracers-smoke.rs','test-tracers.py','integrate-tracers.py','update-tracers-locales.py','tracers-integration.md','generate-tracers-freeze.py']]
 paths += [ROOT/'build/tracers-smoke-02/report.json',ROOT/'build/tracers-smoke-02/test.log',ROOT/'build/tracers-native-01/report.json']
 record={'module':'Tracers','date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},'focused':focused,'finalNativeComposition':native,'separateAuthorizedTestFix':'targeting.rs TargetInfo initializer adds tag_color: None; production unchanged, cumulative testing deferred','files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths}}
 target.write_text(json.dumps(record,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
if __name__=='__main__':main()
