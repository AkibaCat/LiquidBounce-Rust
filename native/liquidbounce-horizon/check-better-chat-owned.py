from pathlib import Path
import ast,sys,runpy,json,tempfile,hashlib,shutil
H=Path(__file__).resolve().parent
for n in ['better-chat-patch.py','integrate-better-chat.py','update-better-chat-locales.py','test-better-chat-focused.py']:ast.parse((H/n).read_text(encoding='utf8'))
p=runpy.run_path(str(H/'integrate-better-chat.py'))['prepare']();prepared=dict(p.changed)
original_read=Path.read_text
try:
 Path.read_text=lambda path,*a,**kw:prepared.get(path,original_read(path,*a,**kw))
 q=runpy.run_path(str(H/'integrate-better-chat.py'))['prepare']();assert not q.changed
finally:Path.read_text=original_read
changes={}
def once(s,old,new):
 if s.count(old)!=1:raise ValueError(f'Expected one native BetterChat anchor, got {s.count(old)}: {old[:120]!r}')
 return s.replace(old,new,1)
def edit(path,transform):
 p=H.parents[1]/'upstream/Horizon'/path
 before=changes.get(path,(p.read_text(encoding='utf8')if p.is_file()else''))
 changes[path]=transform(before)
runpy.run_path(str(H/'flag-check-patch.py'))['apply'](edit,once)
additions=runpy.run_path(str(H/'better-chat-patch.py'))['apply'](edit,once);assert additions['hook_names']==['client_better_chat']
render=changes['crates/horizon-client-render/src/chat.rs']
assert render.count('liquidbounce_id: None, liquidbounce_count: 1,')==4
assert 'client_id: None,' in changes['crates/horizon-client-render/src/liquidbounce_better_chat.rs']
for path,needle in [('crates/horizon-client/src/game.rs','mod liquidbounce_better_chat;'),('crates/horizon-client/src/game/chat.rs','self.client_better_chat_highlight(batch);'),('crates/horizon-client-render/src/chat.rs','liquidbounce_id: Option<String>'),('crates/horizon-client/src/game/remote.rs','better_chat_plain: None')]:assert needle in changes[path],(path,needle)
sys.path.insert(0,str(H));update=runpy.run_path(str(H/'update-better-chat-locales.py'))['update'];locale=H.parent/'liquidbounce-interop/resources/localization'
with tempfile.TemporaryDirectory(prefix='better-chat-locales-')as d:
 p=Path(d)
 for code in ['zh_cn','zh_tw']:shutil.copyfile(locale/f'display_{code}.json',p/f'display_{code}.json')
 update(p);before={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in p.iterdir()};update(p);after={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in p.iterdir()};assert before==after
 for f in p.iterdir():
  data=json.loads(f.read_text(encoding='utf8'));assert data['moduleNames']['BetterChat']!='BetterChat'
  for suffix in ['features','appendPrefix.prefix','appendSuffix.suffix','autoTranslate','antiSpam.stackMessages','antiSpam.filters','copy.notify','copy.highlight']:assert 'liquidbounce.module.betterChat.'+suffix+'.description'in data['descriptions']
  for suffix in ['languageCode','provider','provider.libreTranslate.baseUrl','provider.libreTranslate.apiKey']:assert 'liquidbounce.autoTranslate.'+suffix+'.description'in data['descriptions']
print('BetterChat AST, integrator no-op simulation, native source patch anchors, zh_cn/zh_tw coverage and locale updater idempotence passed')
