#!/usr/bin/env python3
"""Read-only full native composition and KillAura transactional anchor checks.

Rust compilation is deliberately left to the official combined module batch.
"""
from pathlib import Path
import ast,json,runpy,shutil,subprocess,tempfile
HERE=Path(__file__).resolve().parent

def compose():
 source=(HERE/'make-behavior-patch.py').read_text(encoding='utf8')
 source=source[:source.index('\noutput = []')]
 source=source.replace("'step-patch.py'):","'kill-aura-patch.py', 'kill-aura-block-input-patch.py', 'fightbot-navigation-patch.py', 'step-patch.py'):")
 context={'__file__':str(HERE/'make-behavior-patch.py'),'__name__':'kill_aura_static'}
 exec(compile(source,str(HERE/'make-behavior-patch.py'),'exec'),context)
 return context

def main():
 for name in ['integrate-kill-aura.py','kill-aura-patch.py','kill-aura-extra-patch.py','kill-aura-block-input-patch.py','fightbot-navigation-patch.py','update-kill-aura-locales.py']:
  ast.parse((HERE/name).read_text(encoding='utf8'))
 plan=runpy.run_path(str(HERE/'integrate-kill-aura.py'))['prepare']()
 context=compose();changes=context['changes']
 expected=['crates/horizon-client/src/game/kill_aura.rs','crates/horizon-client/src/game/native_hooks.rs','crates/horizon-client/src/game/hand.rs','crates/horizon-client/src/game/remote.rs','crates/horizon-client/src/game/target_renderer.rs','crates/horizon-client/src/game/dragon.rs','crates/horizon-client/src/game/mace.rs']
 with tempfile.TemporaryDirectory(prefix='liquidbounce-kill-aura-static-')as temporary:
  temporary=Path(temporary)
  for name in expected:
   text=changes[name][1];path=temporary/Path(name).name;path.write_text(text,encoding='utf8')
   check=subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',str(path)],capture_output=True,text=True)
   if check.returncode:
    raise RuntimeError(f'{name}:\n{check.stderr}')
  aura=changes[expected[0]][1]
  assert 'super::click::mine'not in aura and 'kill_aura_start_destroy'in aura
  assert 'BlockCollisionScan::any_collision'in aura and 'kill_aura_fight_goal'in aura
  assert 'self.kill_aura_extra_candidates(&mut candidates)'in aura
  assert 'self.kill_aura_attack_candidate(id)'in aura
  assert 'self.kill_aura_extra_local_attack(c).is_none()'in aura
  assert 'return Some((false, physical.distance(eye), None))'in aura
  assert 'matches!(hit.kind, 10 | 11 | KILL_AURA_GUEST)'in aura
  assert 'pub(super) fn knockback('in changes[expected[5]][1]
  assert 'pub(super) fn mace_smash_self('in changes[expected[6]][1]
  assert 'client_kill_aura_pick'in changes[expected[2]][1]
  assert context['names'].count('client_kill_aura_state')==1
  assert context['helper_names'].count('client_kill_aura_query')==1
  for field in ['let tick_hit = self.tick_hit.take();','let tick_mob = self.tick_mob.take();']:
   assert aura.count(field)==2,field
  runtime=(HERE/'engine/src/kill_aura_runtime.rs').read_text(encoding='utf8')
  snapshot=runtime[runtime.index('pub(crate) fn kill_aura_block_state'):runtime.index('fn kill_aura_auto_weapon')]
  assert 'self.state()'not in snapshot and 'self.kill_aura_blocking.snapshot'in snapshot
  locale_source=HERE.parent/'liquidbounce-interop/resources/localization'
  locale_temporary=temporary/'locales';locale_temporary.mkdir()
  for locale in ['zh_cn','zh_tw']:
   shutil.copyfile(locale_source/f'display_{locale}.json',locale_temporary/f'display_{locale}.json')
  update=runpy.run_path(str(HERE/'update-kill-aura-locales.py'))['update']
  update(locale_temporary)
  first={p.name:p.read_bytes()for p in locale_temporary.iterdir()}
  update(locale_temporary)
  assert first=={p.name:p.read_bytes()for p in locale_temporary.iterdir()}
  for locale in ['zh_cn','zh_tw']:
   data=json.loads(first[f'display_{locale}.json'])
   for suffix in ['autoBlocking.blockMode','fightBot.leader.username','failSwing.notifyWhenFail.sound.pitch','rangeIndicator.colorMode']:
    assert 'liquidbounce.module.killAura.'+suffix+'.description' in data['descriptions'],suffix
 print(f'KillAura static composition passed: {len(plan.changed)} transactional shared files; {len(expected)} native sources parsed in temp; simplified/traditional Chinese updater idempotent; no production/staging writes')

if __name__=='__main__':main()
