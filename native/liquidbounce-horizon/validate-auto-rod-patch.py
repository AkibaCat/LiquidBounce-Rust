"""Generate native AutoRod seams and locale resources inside owned scratch only."""
from pathlib import Path
import argparse,json,runpy,shutil,sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
scratch=HERE.parents[1]/'build/auto-rod-static-01';scratch.mkdir(parents=True,exist_ok=True)
p=argparse.ArgumentParser();p.add_argument('--locales-only',action='store_true');a=p.parse_args()
if not a.locales_only:
 path=HERE/'make-behavior-patch.py';source=path.read_text(encoding='utf8')
 if "'auto-rod-patch.py'"not in source:source=source.replace("'auto-bow-patch.py',","'auto-bow-patch.py', 'auto-rod-patch.py',")
 assert "'auto-rod-patch.py'"in source
 source=source.replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(scratch/"hooks-preview.patch")!r})')
 exec(compile(source,str(path),'exec'),{'__file__':str(path),'__name__':'__main__'})
folder=scratch/'localization';folder.mkdir(exist_ok=True)
for locale in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{locale}.json',folder/f'display_{locale}.json')
update=runpy.run_path(str(HERE/'update-auto-rod-locales.py'))['update'];update(folder)
first=[p.read_bytes()for p in sorted(folder.glob('*.json'))];update(folder)
assert first==[p.read_bytes()for p in sorted(folder.glob('*.json'))]
for path in folder.glob('*.json'):
 data=json.loads(path.read_text(encoding='utf8'));assert '\ufffd'not in path.read_text(encoding='utf8')
 descriptions=[k for k in data['descriptions']if k.startswith('liquidbounce.module.autoRod.')]
 assert len(descriptions)>=190,len(descriptions)
 old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
 for k,v in data['descriptions'].items():
  if k.startswith(old):assert data['descriptions'][k.replace(old,'liquidbounce.module.autoRod.rotations')]==v
 assert data['moduleNames']['AutoRod']in['自动鱼竿','自動釣竿']
 print(path.name,len(descriptions),'AutoRod descriptions; valid UTF-8; idempotent')
