"""Append the official native-compile private-helper correction; no smoke rerun."""
from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
previous=HERE/'blink-esp-freeze-02.json';target=HERE/'blink-esp-freeze-03.json';assert not target.exists(),'Historical freeze is immutable'
data=json.loads(previous.read_text(encoding='utf8'))
data['supersedes']={'file':previous.name,'sha256':hashlib.sha256(previous.read_bytes()).hexdigest()}
data['correction']='Official Check149-02 caught cloned perspective::own_entity_id private to sibling module. Substitute its exact actual remote.entity_id-or-integrated-0 expression at each copied use. No rendering behavior/shader/config change; no smoke/stage rerun.'
for name in ['blink-esp-game.py','generate-blink-esp-freeze-03.py','blink-esp-revision-03.md']:
 p=HERE/name;data['files'][p.relative_to(ROOT).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
