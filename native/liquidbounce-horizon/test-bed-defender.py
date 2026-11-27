#!/usr/bin/env python3
"""Static BedDefender fixture; never invokes Cargo or applies shared edits."""
from pathlib import Path
import ast,runpy
HERE=Path(__file__).resolve().parent

def main():
 for name in ['integrate-bed-defender.py','bed-defender-patch.py',Path(__file__).name]:ast.parse((HERE/name).read_text(encoding='utf8'))
 model=(HERE.parent/'liquidbounce-behaviors/src/bed_defender.rs').read_text(encoding='utf8')
 for value in ['max_layers','allow_chests','requires_sneak','bed_plates::scan::layers','hardness(b).total_cmp','b.count.cmp','nearby(a.index,selected)','SelfMode::None']:
  assert value in model,value
 runtime=(HERE/'engine/src/bed_defender_runtime.rs').read_text(encoding='utf8')
 for value in ['PLACEMENT_PROVIDER:u64=34','CRYSTAL_PROVIDER:u64=35','PLACEMENT_PROCESSOR:u64=512','CRYSTAL_PROCESSOR:u64=528','BED_DEFENDER_REQUESTER','bed_defender_prepare','bed_defender_reached','bed_defender_crystal_tick','requires_sneak','b.bed_defender.self_bed.world_change']:
  assert value in runtime,value
 config=(HERE/'engine/src/config/bed_defender.rs').read_text(encoding='utf8')
 for value in ['serialize_bed_defender','parse_bed_defender','SelfBed','Place','inactive_selfbed_and_placer_roundtrip','mode_switch_resets_manual']:
  assert value in config,value
 native=(HERE/'bed-defender-native.rs').read_text(encoding='utf8')
 for value in ['BedBlock','scaffold_block_facts','client_hotbar_slot','bed_defender_beds','"slots"','"targets"']:
  assert value in native,value
 plan=runpy.run_path(str(HERE/'integrate-bed-defender.py'))['prepare']()
 joined='\n'.join(plan.changed.values())
 joined+='\n'+'\n'.join((HERE/'engine/src'/n).read_text(encoding='utf8') for n in ['lib.rs','behaviors.rs','config.rs','actor.rs','runtime.rs','placement_esp_runtime.rs','block_placer_runtime.rs'])+(HERE/'make-behavior-patch.py').read_text(encoding='utf8')
 joined+='\n'+(HERE.parent/'liquidbounce-behaviors/src/lib.rs').read_text(encoding='utf8')
 for value in ['BedDefender','bed_defender_runtime','bed-defender-patch.py','PLACEMENT_PROVIDER|crate::bed_defender_runtime::CRYSTAL_PROVIDER']:
  assert value in joined,value
 patch=(HERE/'bed-defender-patch.py').read_text(encoding='utf8')
 host=(HERE/'bed-defender-host.rs').read_text(encoding='utf8')
 for value in ['client_bed_defender','client_bed_defender_query','BedDefenderPosition','Event::BedDefenderPosition(raw)']:
  assert value in patch,value
 assert 'native_client_bed_defender' in host
 print(f'BedDefender static fixture passed: {len(plan.changed)} shared seams prepared; no compiler invoked')
if __name__=='__main__':main()
