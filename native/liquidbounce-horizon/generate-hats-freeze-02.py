from pathlib import Path
import hashlib,json,runpy
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'hats-freeze-02.json'
assert not target.exists(),'Refusing to overwrite historical freeze'
original=json.loads((HERE/'hats-freeze.json').read_text(encoding='utf8'))
paths=list(original['files'])+['native/liquidbounce-horizon/hats-freeze.json','native/liquidbounce-horizon/hats-staging-fix-02.md','native/liquidbounce-horizon/generate-hats-freeze-02.py','native/liquidbounce-behaviors/vendor/minecraft-math/README.md','native/liquidbounce-behaviors/vendor/minecraft-math/sine-table.bin']
original['revision']=2
original['correction']='Package-owned identical sine table; no behavioral changes or repeated smoke. Official Check remains root-owned.'
original['files']={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest()for p in paths}
target.write_text(json.dumps(original,indent=2)+'\n',encoding='utf8')
print(hashlib.sha256(target.read_bytes()).hexdigest())
