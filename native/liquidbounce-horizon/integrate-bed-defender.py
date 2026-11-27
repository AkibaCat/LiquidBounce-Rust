#!/usr/bin/env python3
"""Idempotent BedDefender wiring; depends on the integrated LiquidFiller BlockPlacer."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
BasePlan=runpy.run_path(str(HERE/'integrate-liquid-filler.py'))['Plan']
def integration_applied():
 root=HERE.parents[1]
 checks=(
  (root/'native/liquidbounce-behaviors/src/lib.rs','pub mod bed_defender;'),
  (root/'native/liquidbounce-horizon/engine/src/behaviors.rs','pub bed_defender:'),
  (root/'native/liquidbounce-horizon/engine/src/config.rs','ModuleKind::BedDefender'),
  (root/'native/liquidbounce-horizon/engine/src/actor.rs','RuntimeRequest::BedDefender'),
  (root/'native/liquidbounce-horizon/engine/src/runtime.rs','self.bed_defender_prepare(host);'),
  (root/'native/liquidbounce-horizon/make-behavior-patch.py',"'bed-defender-patch.py'"),
 )
 return all(path.exists() and marker in path.read_text(encoding='utf8') for path,marker in checks)
class Plan(BasePlan):pass
def prepare():
 if integration_applied():
  return Plan()
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod liquid_filler;','pub mod liquid_filler;\npub mod bed_defender;')
 p.edit('lib.rs','mod liquid_filler_runtime;','mod liquid_filler_runtime;\n#[cfg(feature="behavior-hooks")]\nmod bed_defender_runtime;')
 p.edit('behaviors.rs','    pub liquid_filler:crate::liquid_filler_runtime::LiquidFillerRuntime,','    pub liquid_filler:crate::liquid_filler_runtime::LiquidFillerRuntime,\n    pub bed_defender:crate::bed_defender_runtime::BedDefenderRuntime,')
 p.edit('behaviors.rs','            "LiquidFiller",','            "LiquidFiller",\n            "BedDefender",')
 p.edit('behaviors.rs','            liquid_filler:Default::default(),','            liquid_filler:Default::default(),\n            bed_defender:Default::default(),')
 p.edit('behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnDisabled(id) if self.query_ids.get("BedDefender")==Some(&id)=>{self.bed_defender.placer.disable(current_time_millis());self.bed_defender.self_bed.disable();},''')
 p.edit('behaviors.rs','            crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"LiquidFiller"),','            crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"LiquidFiller"),\n            crate::bed_defender_runtime::PLACEMENT_PROVIDER|crate::bed_defender_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"BedDefender"),')
 p.edit('config.rs','pub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};','pub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};\n#[path="config/bed_defender.rs"]\nmod bed_defender;\npub use bed_defender::{BedDefenderSetting,serialize_bed_defender};')
 p.edit('config.rs','    LiquidFiller=>("LiquidFiller","World","liquidFiller"),','    LiquidFiller=>("LiquidFiller","World","liquidFiller"),\n    BedDefender=>("BedDefender","World","bedDefender"),')
 p.edit('config.rs','    LiquidFiller(LiquidFillerSetting),','    LiquidFiller(LiquidFillerSetting),\n    BedDefender(BedDefenderSetting),',2)
 p.edit('config.rs','            | ModuleKind::LiquidFiller','            | ModuleKind::LiquidFiller\n            | ModuleKind::BedDefender')
 p.edit('config.rs','        ModuleKind::LiquidFiller=>liquid_filler::parse_liquid_filler(&mut patch,&root),','        ModuleKind::LiquidFiller=>liquid_filler::parse_liquid_filler(&mut patch,&root),\n        ModuleKind::BedDefender=>bed_defender::parse_bed_defender(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::LiquidFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::LiquidFiller(value.clone())),','        SettingChange::LiquidFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::LiquidFiller(value.clone())),\n        SettingChange::BedDefender(value)=>return ApplyResult::Runtime(RuntimeRequest::BedDefender(value.clone())),')
 p.edit('actor.rs','            ModuleKind::LiquidFiller=>config::serialize_liquid_filler(&b.liquid_filler,&common,format,&describe),','            ModuleKind::LiquidFiller=>config::serialize_liquid_filler(&b.liquid_filler,&common,format,&describe),\n            ModuleKind::BedDefender=>config::serialize_bed_defender(&b.bed_defender,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::LiquidFiller(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::BedDefender(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().bed_defender,&self.state.block_registry){eprintln!("[LiquidBounce] BedDefender config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::LiquidFiller(setting))=>{''')
 p.edit('runtime.rs','        self.liquid_filler_prepare(host); // BlockPlacer RotationUpdate priority -20','        self.liquid_filler_prepare(host); // BlockPlacer RotationUpdate priority -20\n        self.bed_defender_prepare(host);')
 p.edit('runtime.rs','            let liquid_filler_running=modules.query_running(&state.registry,"LiquidFiller");','            let liquid_filler_running=modules.query_running(&state.registry,"LiquidFiller");\n            let bed_defender_running=modules.query_running(&state.registry,"BedDefender");')
 p.edit('runtime.rs','                        crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>liquid_filler_running,','                        crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>liquid_filler_running,\n                        crate::bed_defender_runtime::PLACEMENT_PROVIDER|crate::bed_defender_runtime::CRYSTAL_PROVIDER=>bed_defender_running,')
 anchor='                        } else if let Some((crystal,mode))=crate::block_placer_runtime::BlockPlacerRuntime::processor_mode(processor){'
 p.edit('runtime.rs',anchor,'''                        } else if let Some((crystal,mode))=crate::bed_defender_runtime::BedDefenderRuntime::processor_mode(processor){
                            (if crystal{&mut modules.bed_defender.placer.crystal_rotations}else{&mut modules.bed_defender.placer.rotations},mode)
'''+anchor)
 p.edit('runtime.rs','        self.block_placer_reached(host);','        self.block_placer_reached(host);\n        self.bed_defender_reached(host);')
 p.edit('runtime.rs','        self.block_placer_crystal_tick(host);','        self.block_placer_crystal_tick(host);\n        self.bed_defender_crystal_tick(host);')
 p.edit('runtime.rs','        event.shift=horizon_client::inventory_move::sneak(event.shift);','        if modules.query_running(&state.registry,"BedDefender")&&modules.bed_defender.placer.queue.movement(){event.shift=true;}\n        event.shift=horizon_client::inventory_move::sneak(event.shift);')
 p.edit('runtime.rs','        self.auto_queue_world_change();','        self.auto_queue_world_change();\n        self.bed_defender_world_change();')
 anchor='    #[cfg(feature="behavior-hooks")]\n    fn client_extinguish_success('
 p.edit('runtime.rs',anchor,'''    #[cfg(feature="behavior-hooks")]
    fn client_bed_defender(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBedDefenderEvent<'_>){self.bed_defender_event(event);}
'''+anchor)
 p.edit('placement_esp_runtime.rs','        let liquid_filler=m.query_running(&state.registry,"LiquidFiller");','        let liquid_filler=m.query_running(&state.registry,"LiquidFiller");\n        let bed_defender=m.query_running(&state.registry,"BedDefender");')
 p.edit('placement_esp_runtime.rs','                    m.liquid_filler.placer.placed_render.tick();','                    m.liquid_filler.placer.placed_render.tick();\n                    m.bed_defender.placer.target_render.tick();\n                    m.bed_defender.placer.placed_render.tick();')
 p.edit('placement_esp_runtime.rs','                if m.liquid_filler.placer.placed_render.running(liquid_filler){draws.extend(m.liquid_filler.placer.placed_render.render(time));}','                if m.liquid_filler.placer.placed_render.running(liquid_filler){draws.extend(m.liquid_filler.placer.placed_render.render(time));}\n                if m.bed_defender.placer.target_render.running(bed_defender){draws.extend(m.bed_defender.placer.target_render.render(time));}\n                if m.bed_defender.placer.placed_render.running(bed_defender){draws.extend(m.bed_defender.placer.placed_render.render(time));}')
 p.edit('block_placer_runtime.rs','fn block_placer_run(&self,host:&mut dyn Host,tasks:Vec<DeferredAction>){for action in tasks{match action{','fn block_placer_run(&self,host:&mut dyn Host,tasks:Vec<DeferredAction>){for action in tasks{if matches!(&action,DeferredAction::Place{requester,..}|DeferredAction::Crystal{requester,..} if *requester==crate::bed_defender_runtime::BED_DEFENDER_REQUESTER){self.bed_defender_run(host,vec![action]);continue;}match action{')
 p.edit('block_placer_runtime.rs','fn block_placer_success(&self,requester:u64,position:[i32;3],bounds:[f64;6]){','fn block_placer_success(&self,requester:u64,position:[i32;3],bounds:[f64;6]){self.bed_defender_success(requester,position,bounds);')
 p.edit('block_placer_runtime.rs','fn block_placer_removed(&self,id:i32){','fn block_placer_removed(&self,id:i32){self.bed_defender_removed(id);')
 p.edit('../../make-behavior-patch.py',"'block-placer-patch.py',","'block-placer-patch.py', 'bed-defender-patch.py',")
 return p
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');a=ap.parse_args();p=prepare()
 if not a.check:p.commit()
 print(f'BedDefender {"validated"if a.check else"integrated"}: {len(p.changed)} shared files')
