"""Append composition-only fix while preserving the passing focused evidence."""
from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'blink-esp-freeze-02.json';assert not target.exists(),'Historical freeze is immutable'
previous=HERE/'blink-esp-freeze.json';data=json.loads(previous.read_text(encoding='utf8'))
data['supersedes']={'file':previous.name,'sha256':hashlib.sha256(previous.read_bytes()).hexdigest()}
data['correction']='Use the stable generic hand_in_world declaration anchor so the independent Blink renderers compose after HoleESP and future visual adapters. Official Check146-01 caught staging failure; no behavior or shader change and no smoke rerun.'
for name in ['blink-esp-patch.py','generate-blink-esp-freeze-02.py']:
 p=HERE/name;data['files'][p.relative_to(ROOT).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
