"""Generate AutoBuff native seams and localized strings only in owned scratch."""
from pathlib import Path
import runpy,json,shutil,sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
scratch=HERE.parents[1]/'build/auto-buff-smoke-01';scratch.mkdir(parents=True,exist_ok=True)
path=HERE/'make-behavior-patch.py';source=path.read_text(encoding='utf8')
if "'auto-buff-patch.py'"not in source:source=source.replace("'fast-exp-patch.py',","'fast-exp-patch.py', 'auto-buff-patch.py',")
source=source.replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(scratch/"hooks-preview.patch")!r})')
exec(compile(source,str(path),'exec'),{'__file__':str(path),'__name__':'__main__'})
folder=scratch/'localization';folder.mkdir(exist_ok=True)
for locale in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{locale}.json',folder/f'display_{locale}.json')
update=runpy.run_path(str(HERE/'update-auto-buff-locales.py'))['update'];update(folder)
first=[p.read_bytes()for p in sorted(folder.glob('*.json'))];update(folder);assert first==[p.read_bytes()for p in sorted(folder.glob('*.json'))]
for p in folder.glob('*.json'):
 data=json.loads(p.read_text(encoding='utf8'));assert '\ufffd'not in p.read_text(encoding='utf8');keys=[k for k in data['descriptions']if k.startswith('liquidbounce.module.autoBuff.')];assert len(keys)>90;print(p.name,len(keys),'AutoBuff descriptions, valid UTF-8, idempotent')
