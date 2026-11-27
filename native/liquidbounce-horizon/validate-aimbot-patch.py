"""Generate Aimbot patch and locale previews inside owned scratch; no Rust runs."""
from pathlib import Path
import argparse,json,runpy,shutil,sys
HERE=Path(__file__).resolve().parent;sys.path.insert(0,str(HERE))
scratch=HERE.parents[1]/'build/aimbot-static-01';scratch.mkdir(parents=True,exist_ok=True)
p=argparse.ArgumentParser();p.add_argument('--locales-only',action='store_true');a=p.parse_args()
if not a.locales_only:
 path=HERE/'make-behavior-patch.py';source=path.read_text(encoding='utf8')
 if "'aimbot-patch.py'"not in source:source=source.replace("'auto-rod-patch.py',","'auto-rod-patch.py', 'aimbot-patch.py',")
 assert "'aimbot-patch.py'"in source
 source=source.replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(scratch/"hooks-preview.patch")!r})')
 exec(compile(source,str(path),'exec'),{'__file__':str(path),'__name__':'__main__'})
folder=scratch/'localization';folder.mkdir(exist_ok=True)
for locale in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{locale}.json',folder/f'display_{locale}.json')
update=runpy.run_path(str(HERE/'update-aimbot-locales.py'))['update'];update(folder)
first=[p.read_bytes()for p in sorted(folder.glob('*.json'))];update(folder)
assert first==[p.read_bytes()for p in sorted(folder.glob('*.json'))]
for path in folder.glob('*.json'):
 data=json.loads(path.read_text(encoding='utf8'));assert '\ufffd'not in path.read_text(encoding='utf8')
 descriptions={k:v for k,v in data['descriptions'].items()if k.startswith('liquidbounce.module.aimbot.')}
 assert len(descriptions)>=180,len(descriptions)
 for suffix in ['range','wallRange','requires.emptyHand','crystals.onlyWhileUsing','axis.horizontal','ignore.container','aimPoint.delay.delay','angleSmooth.interpolation.directionChangeFactor','angleSmooth.sigmoid.steepness']:
  assert f'liquidbounce.module.aimbot.{suffix}.description'in descriptions,suffix
 assert data['moduleNames']['Aimbot']in['自动瞄准','自動瞄準']
 print(path.name,len(descriptions),'Aimbot descriptions; valid UTF-8; idempotent')
