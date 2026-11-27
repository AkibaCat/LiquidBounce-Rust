from pathlib import Path
import ast,sys,runpy,json,tempfile,hashlib,shutil
H=Path('native/liquidbounce-horizon').resolve()
for n in ['block-esp-patch.py','integrate-block-esp.py','update-block-esp-locales.py','test-block-esp-focused.py']:ast.parse((H/n).read_text(encoding='utf8'))
p=runpy.run_path(str(H/'integrate-block-esp.py'))['prepare']();prepared=dict(p.changed)
# Re-evaluate against in-memory published files to check no-op semantics without shared writes.
cls=type(p);q=cls();original_read=Path.read_text
try:
 Path.read_text=lambda path,*a,**kw:prepared.get(path,original_read(path,*a,**kw))
 q=runpy.run_path(str(H/'integrate-block-esp.py'))['prepare']();assert not q.changed
finally:Path.read_text=original_read
sys.path.insert(0,str(H));update=runpy.run_path(str(H/'update-block-esp-locales.py'))['update'];locale=H.parent/'liquidbounce-interop/resources/localization'
with tempfile.TemporaryDirectory(prefix='block-esp-locales-')as d:
 p=Path(d)
 for code in ['zh_cn','zh_tw']:shutil.copyfile(locale/f'display_{code}.json',p/f'display_{code}.json')
 update(p);before={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in p.iterdir()};update(p);after={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in p.iterdir()};assert before==after
 for f in p.iterdir():
  data=json.loads(f.read_text(encoding='utf8'));assert data['moduleNames']['BlockESP']!='BlockESP'
  for suffix in ['targets','mode.box.outline','mode.glow.style','colorMode.static.color','distanceFade.farEnd','mode.glow.style.neon.coreMix','mode.glow.style.aura.fill']:assert 'liquidbounce.module.blockESP.'+suffix+'.description'in data['descriptions']
print('BlockESP AST, integrator no-op simulation, zh_cn/zh_tw schema coverage and updater idempotence passed')
