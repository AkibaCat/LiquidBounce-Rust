from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'hats-freeze-04.json'
assert not target.exists(),'Refusing to overwrite historical freeze'
data=json.loads((HERE/'hats-freeze-03.json').read_text(encoding='utf8'))
paths=list(data['files'])+['native/liquidbounce-horizon/hats-freeze-03.json','native/liquidbounce-horizon/hats-locales-fix-04.md','native/liquidbounce-horizon/generate-hats-freeze-04.py','build/hats-locales-04/report.json']
data['revision']=4
data['correction']='Populate actual nested display dictionaries; remove ignored legacy Hats top-level keys. Match mutable Host declaration. No gameplay/GPU smoke repeated. Official Check remains root-owned.'
data['files']={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest()for p in paths}
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8')
print(hashlib.sha256(target.read_bytes()).hexdigest())
