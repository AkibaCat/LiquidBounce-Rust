from pathlib import Path
import ast,runpy,json,tempfile,hashlib,shutil,sys
H=Path(__file__).resolve().parent
for n in ['debug-patch.py','integrate-debug.py','update-debug-locales.py','test-debug-focused.py']:ast.parse((H/n).read_text(encoding='utf8'))
source=(H/'make-behavior-patch.py').read_text(encoding='utf8');prefix=source[:source.index('fragment = {')];prefix=prefix if "'debug-patch.py'" in prefix else prefix.replace("'step-patch.py'):","'debug-patch.py', 'step-patch.py'):")
scope={'__file__':str(H/'make-behavior-patch.py'),'__name__':'debug_memory_patch'};exec(compile(prefix,str(H/'make-behavior-patch.py'),'exec'),scope)
changes=scope['changes'];assert 'client_debug' in scope['names'];assert 'mod liquidbounce_debug;' in changes['crates/horizon-client/src/game.rs'][1];assert 'client_debug_geometry' in changes['crates/horizon-client/src/game/world_primitives.rs'][1]
sys.path.insert(0,str(H));update=runpy.run_path(str(H/'update-debug-locales.py'))['update'];locale=H.parent/'liquidbounce-interop/resources/localization'
with tempfile.TemporaryDirectory(prefix='debug-locales-')as d:
 p=Path(d)
 for code in ['zh_cn','zh_tw']:shutil.copyfile(locale/f'display_{code}.json',p/f'display_{code}.json')
 update(p);before={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in p.iterdir()};update(p);after={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in p.iterdir()};assert before==after
 for f in p.iterdir():
  data=json.loads(f.read_text(encoding='utf8'));assert data['moduleNames']['Debug']!='Debug';assert data['labels']['Parameters']!='Parameters';assert 'liquidbounce.module.debug.graph.curve.description'in data['descriptions']
print('Debug AST, combined native patch anchors, locale coverage and idempotence passed')
