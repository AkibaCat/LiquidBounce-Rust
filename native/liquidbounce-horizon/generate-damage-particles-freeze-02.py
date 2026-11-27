from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'damage-particles-freeze-02.json'
if target.exists():raise SystemExit('Refusing to overwrite historical freeze')
prior=json.loads((HERE/'damage-particles-freeze.json').read_text(encoding='utf8'))
paths=[ROOT/p for p in prior['files'] if not p.startswith('build/damage-particles-smoke-02/')]
paths += [Path(__file__),HERE/'damage-particles-revision-02.md',ROOT/'build/damage-particles-smoke-03/report.json',ROOT/'build/damage-particles-smoke-03/test.log']
prior['revision']=2;prior['focused']=json.loads((ROOT/'build/damage-particles-smoke-03/report.json').read_text(encoding='utf8'));assert prior['focused']['passed']
prior['files']={p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in paths}
target.write_text(json.dumps(prior,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
