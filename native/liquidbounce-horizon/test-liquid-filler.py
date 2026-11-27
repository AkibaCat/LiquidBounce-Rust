#!/usr/bin/env python3
"""Check integration and the full in-memory native patch stack; never stage or compile."""
from pathlib import Path
import ast,runpy,tempfile
HERE=Path(__file__).resolve().parent
def main():
 for name in ['integrate-liquid-filler.py','block-placer-patch.py',Path(__file__).name,'update-liquid-filler-locales.py']:
  path=HERE/name
  if path.exists():ast.parse(path.read_text(encoding='utf8'))
 plan=runpy.run_path(str(HERE/'integrate-liquid-filler.py'))['prepare']()
 generator=plan.changed.get(HERE/'engine/src/../../make-behavior-patch.py',(HERE/'make-behavior-patch.py').read_text(encoding='utf8'))
 if "'block-placer-patch.py'" not in generator:
  generator=generator.replace("'scaffold-special-techniques-patch.py',","'scaffold-special-techniques-patch.py', 'block-placer-patch.py',")
 scope={'__file__':str(HERE/'make-behavior-patch.py')}
 exec(compile(generator.split('fragment = {',1)[0],str(HERE/'make-behavior-patch.py'),'exec'),scope)
 changes=scope['changes'];remote=changes['crates/horizon-client/src/game/remote.rs'][1]
 a=remote.index('        self.no_slow_vanilla_send_state(input);')
 assert remote.index('        self.native_client_block_placer_after_move();',a)<remote.index('        self.native_client_no_slow_network(false);',a)
 assert remote.count('self.native_client_block_placer_after_move();')==1
 net=changes['crates/horizon-client/src/net.rs'][1]
 observation=net.index('send.send(Event::BlockPlacerRemoved(p.ids.clone()))')
 assert observation<net.index('        if cancelled {continue;}',observation)
 assert 'Event::BlockPlacerRemoved(ids)=>' in changes['crates/horizon-client/src/game/remote_poll.rs'][1]
 assert 'self.native_client_block_placer_removed(id)' not in remote
 attack=changes['crates/horizon-client/src/game/kill_aura.rs'][1]
 assert 'self.kill_aura_attack_with_swing(id,keep,critical,0)' in attack
 assert 'self.client_projectile_puncher_swing(swing);' in attack[attack.index('pub(super) fn kill_aura_attack_with_swing'):]
 for name in ['client_liquid_filler','client_block_placer','block_placer_crystal','liquid_filler_core','liquid_filler_random']:
  assert changes[f'crates/horizon-client/src/game/{name}.rs'][1]
 for name in ['client_block_placer_success','client_block_placer_after_move','client_block_placer_removed']:
  assert name in scope['names']
 updater=runpy.run_path(str(HERE/'update-liquid-filler-locales.py'));texts,labels=updater['entries']()
 with tempfile.TemporaryDirectory(prefix='liquid-filler-locales-') as folder:
  folder=Path(folder)
  for locale in ['zh_cn','zh_tw']:
   source=HERE.parent/'liquidbounce-interop/resources/localization'/f'display_{locale}.json'
   (folder/source.name).write_text(source.read_text(encoding='utf8'),encoding='utf8')
  updater['update'](folder);before={p:p.read_text(encoding='utf8') for p in folder.iterdir()};updater['update'](folder)
  for p,content in before.items():assert p.read_text(encoding='utf8')==content
 print(f'LiquidFiller static fixture passed: {len(plan.changed)} integration seams; {len(changes)} native source files checked in memory')
 print(f'LiquidFiller locale fixture passed: {len(texts)} descriptions and {len(labels)} scoped labels; both updates idempotent')
if __name__=='__main__':main()
