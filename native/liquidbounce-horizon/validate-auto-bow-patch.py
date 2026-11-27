"""Generate AutoBow native seams and display locales only inside owned scratch."""
from pathlib import Path
import argparse,runpy,json,shutil,sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
scratch=HERE.parents[1]/'build/auto-bow-smoke-01';scratch.mkdir(parents=True,exist_ok=True)
parser=argparse.ArgumentParser();parser.add_argument('--locales-only',action='store_true');args=parser.parse_args()
if not args.locales_only:
 path=HERE/'make-behavior-patch.py';source=path.read_text(encoding='utf8')
 if "'auto-bow-patch.py'"not in source:source=source.replace("'auto-buff-patch.py',","'auto-buff-patch.py', 'auto-bow-patch.py',")
 assert "'auto-bow-patch.py'"in source
 source=source.replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(scratch/"hooks-preview.patch")!r})')
 exec(compile(source,str(path),'exec'),{'__file__':str(path),'__name__':'__main__'})
folder=scratch/'localization';folder.mkdir(exist_ok=True)
for locale in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{locale}.json',folder/f'display_{locale}.json')
update=runpy.run_path(str(HERE/'update-auto-bow-locales.py'))['update'];update(folder)
first=[p.read_bytes()for p in sorted(folder.glob('*.json'))];update(folder)
assert first==[p.read_bytes()for p in sorted(folder.glob('*.json'))]
for p in folder.glob('*.json'):
 data=json.loads(p.read_text(encoding='utf8'));assert '\ufffd'not in p.read_text(encoding='utf8')
 keys=[k for k in data['descriptions']if k.startswith('liquidbounce.module.autoBow.')];assert len(keys)>=54
 old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
 for key,value in data['descriptions'].items():
  if key.startswith(old):assert data['descriptions'][key.replace(old,'liquidbounce.module.autoBow.bowAimbot.rotations')]==value
 assert data['moduleNames']['AutoBow'] in ['自动弓箭','自動弓箭']
 print(p.name,len(keys),'AutoBow descriptions; valid UTF-8; idempotent')
