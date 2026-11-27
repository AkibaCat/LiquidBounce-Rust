#!/usr/bin/env python3
"""Read-only TargetStrafe owned checks; native dependencies are explicitly scaffolded."""
from pathlib import Path
import ast,sys,runpy,json,tempfile,hashlib,shutil
H=Path(__file__).resolve().parent
for n in ['target-strafe-patch.py','integrate-target-strafe.py','update-target-strafe-locales.py','test-target-strafe-focused.py']:
 ast.parse((H/n).read_text(encoding='utf8'))
p=runpy.run_path(str(H/'integrate-target-strafe.py'))['prepare']();prepared=dict(p.changed)
original_read=Path.read_text
try:
 Path.read_text=lambda path,*a,**kw:prepared.get(path,original_read(path,*a,**kw))
 q=runpy.run_path(str(H/'integrate-target-strafe.py'))['prepare']();assert not q.changed
finally:Path.read_text=original_read
changes={}
def once(s,old,new):
 count=s.count(old)
 if count!=1:raise ValueError(f'Expected one native TargetStrafe anchor, got {count}: {old[:120]!r}')
 return s.replace(old,new,1)
def edit(path,transform):
 p=H.parents[1]/'upstream/Horizon'/path
 before=changes.get(path,p.read_text(encoding='utf8')if p.is_file()else'')
 changes[path]=transform(before)
# Run the actual Strafe precursor in memory, so move_self ordering is verified.
runpy.run_path(str(H/'strafe-patch.py'))['apply'](edit,once)
# Other dependency extensions need the full make prefix. Supply only their named
# module/clear anchors here. This is not a full extension-composition check.
assert 'mod aimbot;' in (H/'aimbot-patch.py').read_text(encoding='utf8')
assert '        super::edge_assistance::clear();' in (H/'edge-assistance-patch.py').read_text(encoding='utf8')
edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod activity;','mod aimbot;\nmod activity;'))
anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,'        super::edge_assistance::clear();\n'+anchor))
additions=runpy.run_path(str(H/'target-strafe-patch.py'))['apply'](edit,once)
assert additions=={'hook_names':[],'helper_names':['client_target_strafe_query']}
for path,needle in [
 ('crates/horizon-client/src/client_lib.rs','pub mod target_strafe_math;'),
 ('crates/horizon-client/src/game.rs','mod target_strafe;'),
 ('crates/horizon-client/src/game/native_hooks.rs','horizon_client::target_strafe_bridge::clear();'),
 ('crates/horizon-client/src/motion.rs','include!("target-strafe-motion.rs");'),
 ('crates/horizon-client/src/world_primitives.rs','LiquidBounceGradientQuad'),
 ('crates/horizon-client/src/game/world_primitives.rs','self.client_target_strafe_frame(&mut primitives,alpha);'),
 ('crates/horizon-client/src/target_strafe_math.rs','pub fn projected_points_limit')]:assert needle in changes[path],(path,needle)
motion=changes['crates/horizon-client/src/motion.rs']
head=motion.index('    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {')
assert motion.index('let movement = self.target_strafe_move(movement);',head)<motion.index('let movement = self.client_strafe_movement(movement);',head)
assert 'crate::game::' not in changes['crates/horizon-client/src/target_strafe.rs']
assert 'crate::game::' not in changes['crates/horizon-client/src/target_strafe_bridge.rs']
sys.path.insert(0,str(H));locale=H.parent/'liquidbounce-interop/resources/localization'
u=runpy.run_path(str(H/'update-target-strafe-locales.py'));update=u['update'];prefix=u['PREFIX'];camel=u['camel']
assert len(u['TEXTS'])==44
with tempfile.TemporaryDirectory(prefix='target-strafe-locales-')as d:
 p=Path(d)
 for code in ['zh_cn','zh_tw']:shutil.copyfile(locale/f'display_{code}.json',p/f'display_{code}.json')
 update(p);before={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in p.iterdir()};update(p);after={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in p.iterdir()};assert before==after
 for f in p.iterdir():
  data=json.loads(f.read_text(encoding='utf8'));assert data['moduleNames']['TargetStrafe']!='TargetStrafe'
  for path,en,cn,tw in u['TEXTS']:
   key=prefix+'.'+camel(path)+'.description'if path else prefix+'.description'
   assert key in data['descriptions'],key
   if path:assert prefix+'.'+camel(path) in data['labels'],path
  for path,options in [('mode',['Motion','Input']),('requirements',['Space','Speed','KillAura','Ground']),('target.priority',['Type','Health','Distance','Direction','HurtTime','Age'])]:
   for option in options:assert prefix+'.'+path+'.'+option in data['labels']
print('TargetStrafe AST, integrator no-op simulation, native standalone anchors with explicit dependency scaffolds, 44 descriptions, scoped Simplified/Traditional labels and locale idempotence passed; no compiler or full extension composition invoked')
