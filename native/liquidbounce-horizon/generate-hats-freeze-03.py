from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'hats-freeze-03.json'
assert not target.exists(),'Refusing to overwrite historical freeze'
data=json.loads((HERE/'hats-freeze-02.json').read_text(encoding='utf8'))
paths=list(data['files'])+['native/liquidbounce-horizon/hats-freeze-02.json','native/liquidbounce-horizon/hats-host-fix-03.md','native/liquidbounce-horizon/generate-hats-freeze-03.py']
data['revision']=3
data['correction']='Drain image errors through mutable ClientTick.Pre Host; read-only frame snapshot preserved. No repeated smoke. Official Check remains root-owned.'
data['files']={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest()for p in paths}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8')
print(hashlib.sha256(target.read_bytes()).hexdigest())
