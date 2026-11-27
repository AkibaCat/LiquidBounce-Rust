"""Append metadata-only correction; preserve all original evidence and freeze."""
from pathlib import Path
import hashlib,json,runpy
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
target=HERE/'hole-esp-freeze-02.json';assert not target.exists(),'Historical freeze is immutable'
previous=HERE/'hole-esp-freeze.json';data=json.loads(previous.read_text(encoding='utf8'))
plan=runpy.run_path(str(HERE/'integrate-hole-esp.py'))['prepare']()
data['supersedes']={'file':previous.name,'sha256':hashlib.sha256(previous.read_bytes()).hexdigest()}
data['correction']='Remove unintended active-mode module tag. Pinned ModuleHoleESP has no tagBy call, so tag remains null. No production scanner/settings/runtime/native or passing smoke changes; no test rerun required.'
data['integration']={'steps':plan.steps,'sharedFiles':list(plan.text)}
for name in ['integrate-hole-esp.py','hole-esp-integration.md','generate-hole-esp-freeze-02.py']:
 path=HERE/name;data['files'][path.relative_to(ROOT).as_posix()]=hashlib.sha256(path.read_bytes()).hexdigest()
target.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
