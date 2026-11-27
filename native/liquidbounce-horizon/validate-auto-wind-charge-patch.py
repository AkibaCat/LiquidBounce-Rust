"""Generate a normal patch into owned scratch without changing staging."""
from pathlib import Path
import runpy,json,shutil
HERE=Path(__file__).resolve().parent
scratch=HERE.parents[1]/'build/auto-wind-charge-smoke-01'
scratch.mkdir(parents=True,exist_ok=True)
path=HERE/'make-behavior-patch.py';source=path.read_text(encoding='utf8')
if "'auto-wind-charge-patch.py'"not in source:source=source.replace("'fast-exp-patch.py',","'fast-exp-patch.py', 'auto-wind-charge-patch.py',")
destination=scratch/'hooks-preview.patch'
source=source.replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(destination)!r})')
exec(compile(source,str(path),'exec'),{'__file__':str(path),'__name__':'__main__'})
folder=scratch/'localization';folder.mkdir(exist_ok=True)
for locale in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{locale}.json',folder/f'display_{locale}.json')
update=runpy.run_path(str(HERE/'update-auto-wind-charge-locales.py'))['update'];update(folder)
first=[p.read_bytes()for p in sorted(folder.glob('*.json'))];update(folder);assert first==[p.read_bytes()for p in sorted(folder.glob('*.json'))]
for p in folder.glob('*.json'):
 data=json.loads(p.read_text(encoding='utf8'));assert '\ufffd'not in p.read_text(encoding='utf8')
 keys=[k for k in data['descriptions']if k.startswith('liquidbounce.module.autoWindCharge.')]
 assert len(keys)>=9
 print(p.name,len(keys),'AutoWindCharge descriptions; idempotent, valid UTF-8')
