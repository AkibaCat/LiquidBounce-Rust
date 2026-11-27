#!/usr/bin/env python3
"""Static HoleFiller fixture; does not invoke a compiler or apply shared wiring."""
from pathlib import Path
import ast,runpy
HERE=Path(__file__).resolve().parent
def main():
 for name in['integrate-hole-filler.py','hole-filler-patch.py',Path(__file__).name]:ast.parse((HERE/name).read_text(encoding='utf8'))
 model=(HERE.parent/'liquidbounce-behaviors/src/hole_filler.rs').read_text(encoding='utf8')
 for value in['ONLY_WHEN_SELF_IN_HOLE','ONLY_ONE_BY_ONE','angle>=0.866','if items==0{return None;}','wrapping_sub(size)','java_compare','range.wrapping_mul(range)']:assert value in model,value
 runtime=(HERE/'engine/src/hole_filler_runtime.rs').read_text(encoding='utf8')
 for value in['PLACEMENT_PROVIDER:u64=37','CRYSTAL_PROVIDER:u64=38','PLACEMENT_PROCESSOR:u64=560','CRYSTAL_PROCESSOR:u64=576','if crystal{35}else{0}','should_be_attacked','core::targets','range.ceil()as i32']:assert value in runtime,value
 assert runtime.index('let snapshot=query')<runtime.index('let mut r=std::mem::take')
 config=(HERE/'engine/src/config/hole_filler.rs').read_text(encoding='utf8')
 for value in['serialize_hole_filler','parse_hole_filler','Placing','complete_inactive_placing_roundtrip','reject_unknown_blocks']:assert value in config,value
 plan=runpy.run_path(str(HERE/'integrate-hole-filler.py'))['prepare']()
 generator=plan.changed.get(HERE/'engine/src/../../make-behavior-patch.py',(HERE/'make-behavior-patch.py').read_text(encoding='utf8'))
 scope={'__file__':str(HERE/'make-behavior-patch.py')};exec(compile(generator.split('fragment = {',1)[0],str(HERE/'make-behavior-patch.py'),'exec'),scope)
 assert 'client_hole_filler' in scope['names'];assert 'client_hole_filler_query' in scope['helper_names']
 game=scope['changes']['crates/horizon-client/src/game/hole_esp.rs'][1]
 for value in['client_hole_subscriptions','subscriptions.push((1,','cache.manager.unsubscribe(id)','cache.subscribers=next','if self.client_hole_subscriptions().is_empty()']:assert value in game,value
 assert game.index('cache.manager.subscribe(id,h,v,')<game.index('cache.manager.unsubscribe(id)')
 assert 'native_client_hole_filler(&self,input:&str,output:&mut String)' in scope['changes']['crates/horizon-client/src/game/native_hooks.rs'][1]
 print(f'HoleFiller static fixture passed: {len(plan.changed)} shared files prepared; no compiler invoked')
if __name__=='__main__':main()
