#!/usr/bin/env python3
"""Read-only ElytraTarget display schema, full renderer coverage and idempotence checks."""
from pathlib import Path
import ast,sys,runpy,re,json,tempfile,shutil,hashlib
H=Path(__file__).resolve().parent
sys.path.insert(0,str(H));source=(H/'update-elytra-target-locales.py').read_text(encoding='utf8');ast.parse(source)
u=runpy.run_path(str(H/'update-elytra-target-locales.py'));r=runpy.run_path(str(H/'update-target-renderer-locales.py'))
spec=(H/'engine/src/config/elytra_target.rs').read_text(encoding='utf8')
paths=re.findall(r'\(\s*"([^"]+)",\s*"(?:BOOLEAN|FLOAT|INT|FLOAT_RANGE|INT_RANGE|CHOOSE|MULTI_CHOOSE)"',spec)
assert len(paths)==20,(len(paths),paths)
expected={p for p,*_ in u['TEXTS']};assert set(paths)<=expected,set(paths)-expected
locale=H.parent/'liquidbounce-interop/resources/localization'
with tempfile.TemporaryDirectory(prefix='elytra-target-locales-')as d:
 folder=Path(d)
 for code in ['zh_cn','zh_tw']:shutil.copyfile(locale/f'display_{code}.json',folder/f'display_{code}.json')
 u['update'](folder);before={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in folder.iterdir()};u['update'](folder);after={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in folder.iterdir()};assert before==after
 for f in folder.iterdir():
  code='zh_tw'if'zh_tw'in f.name else'zh_cn';data=json.loads(f.read_text(encoding='utf8'));prefix=u['PREFIX']
  assert data['moduleNames']['ElytraTarget']!='ElytraTarget'
  for p,*_ in u['TEXTS']:
   key=prefix+('.'+u['camel'](p)if p else'');assert key+'.description'in data['descriptions'],key
   if p:assert key in data['labels'],key
  for p,options in u['CHOICES'].items():
   for n in options:assert prefix+'.'+p+'.'+n in data['labels'],(p,n)
  render_prefix=prefix+'.targetRendering'
  for p in r['paths']():
   key=render_prefix+('.'+u['camel'](p)if p else'');assert key in data['labels'],key;assert key+'.description'in data['descriptions'],key
  for field in ['moduleNames','labels','descriptions']:
   for k,v in data[field].items():
    if k=='ElytraTarget'or k.startswith(prefix):assert any('\u3400'<=c<='\u9fff'for c in v),(f.name,k,v)
  assert data['labels'][prefix+'.target.fOV']==data['labels'][prefix+'.target.fov']
print(f'ElytraTarget locale AST, {len(paths)} config leaves, {len(u["TEXTS"])} descriptions, {len(r["paths"]())} complete renderer paths, scoped option labels and zh_cn/zh_tw updater idempotence passed; production locale files were not modified')
