#!/usr/bin/env python3
"""Focused static fixture; does not invoke Cargo, rustc or full staging."""
from pathlib import Path
import ast,runpy
HERE=Path(__file__).resolve().parent
def main():
 for name in ['integrate-auto-mob-heal.py','auto-mob-heal-patch.py',Path(__file__).name]:
  ast.parse((HERE/name).read_text(encoding='utf8'))
 native=(HERE/'auto-mob-heal-native.rs').read_text(encoding='utf8')
 for value in ['m.attrs.max_health','m.extra.sitting_pose','m.extra.body_extra.clone()','stations::is_valid_repair_item','components_view(&view)','distance_squared_to_point(eye)','remote_mob_takes(i, hand)','remote_mob_swings(i, hand)','location: [0.; 3]']:
  assert value in native,value
 assert 'native_client_use_item(' not in native
 runtime=(HERE/'engine/src/auto_mob_heal_runtime.rs').read_text(encoding='utf8')
 assert runtime.index('"op":"interact"')<runtime.index('random_i32_inclusive')
 assert 'facts["usingItem"] == true || facts["screenOpen"] != false' in runtime
 assert 'WaitOwner::AutoMobHeal' in runtime and 'plan.slot != 40' in runtime
 config=(HERE/'engine/src/config/auto_mob_heal.rs').read_text(encoding='utf8')
 for value in ['Range','Delay','SlotResetDelay','SwingMode','HealthThreshold','NoWaste','AllowBucketFood','OnlyWhenSitting','RepairThreshold','AllowEnchantedGoldenApple']:
  assert f'"{value}"' in config or value in config,value
 # Dry-run validates every actual shared registration seam; the parent owns commit.
 plan=runpy.run_path(str(HERE/'integrate-auto-mob-heal.py'))['prepare']()
 combined='\n'.join(plan.changed.values())
 combined+='\n'+'\n'.join((HERE/'engine/src'/name).read_text(encoding='utf8') for name in ['lib.rs','behaviors.rs','config.rs','actor.rs','runtime.rs','coroutine_ticker.rs'])
 combined+='\n'+(HERE/'make-behavior-patch.py').read_text(encoding='utf8')
 combined+='\n'+(HERE.parent/'liquidbounce-behaviors/src/lib.rs').read_text(encoding='utf8')
 for value in ['AutoMobHeal','AutoGolemRepair','auto_mob_heal_tick(host)','auto_mob_heal_callback(in_game,task.callback)','auto-mob-heal-patch.py','pub mod auto_mob_heal;']:
  assert value in combined,value
 print(f'AutoMobHeal static fixture passed; {len(plan.changed)} shared source seams prepared')
if __name__=='__main__':main()
