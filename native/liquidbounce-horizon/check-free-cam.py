#!/usr/bin/env python3
"""Read-only native consumer dry-run; optionally emit a focused Rust consumer fixture."""
from pathlib import Path
import argparse,runpy,ast,json
HERE=Path(__file__).resolve().parent
def check(source_root,out=None):
 changes={}
 def once(text,old,new):
  if text.count(old)!=1:raise ValueError(f'Expected one anchor, got {text.count(old)}: {old[:100]!r}')
  return text.replace(old,new,1)
 def edit(path,transform):
  before=changes.get(path,(source_root/path).read_text(encoding='utf8')if(source_root/path).is_file()else'')
  changes[path]=transform(before)
 runpy.run_path(str(HERE/'free-cam-patch.py'))['apply'](edit,once)
 for path,text in changes.items():
  if path.endswith('.json'):json.loads(text)
  if path.endswith('.py'):ast.parse(text)
 for name in ['free-cam-patch.py','integrate-free-cam.py','publish-free-cam.py','update-free-cam-zh.py']:
  ast.parse((HERE/name).read_text(encoding='utf8'),filename=name)
 if out:
  out.mkdir(parents=True,exist_ok=True)
  camera=changes['crates/horizon-client-render/src/camera.rs']
  start=camera.index('    pub fn align_with_entity_free_cam(')
  end=camera.index('    /// \u0060Camera.calculateFov',start)
  methods=camera[start:end]
  (out/'owners.rs').write_text(changes['crates/horizon-client/src/game/navigation_owners.rs'],encoding='utf8')
  sender=changes['crates/horizon-client/src/player_sync.rs']
  sender=sender[sender.index('/// Only movement'):sender.index('/// Rotate the old velocity')]
  (out/'position_sender.rs').write_text('use crate::packets::{self as play,Serverbound as S};\n'+sender,encoding='utf8')
  (out/'behavior.rs').write_bytes((HERE/'free_cam.rs' if(HERE/'free_cam.rs').is_file()else HERE.parent/'liquidbounce-behaviors/src/free_cam.rs').read_bytes())
  (out/'bridge.rs').write_bytes((HERE/'free-cam-bridge.rs').read_bytes())
  (out/'fixture.rs').write_text((HERE/'free-cam-fixture.rs').read_text(encoding='utf8').replace('__CAMERA_METHODS__',methods),encoding='utf8')
 print(f'FreeCam native dry-run validated: {len(changes)} consumer files')
 return changes
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--source-root',type=Path,required=True);a.add_argument('--prepare-fixture',type=Path);args=a.parse_args()
 check(args.source_root,args.prepare_fixture)
