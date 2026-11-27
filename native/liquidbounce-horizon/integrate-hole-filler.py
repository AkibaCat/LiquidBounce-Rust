#!/usr/bin/env python3
"""Idempotent HoleFiller wiring; depends on the integrated LiquidFiller BlockPlacer."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
BasePlan=runpy.run_path(str(HERE/'integrate-liquid-filler.py'))['Plan']
class Plan(BasePlan):pass
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod liquid_filler;','pub mod liquid_filler;\npub mod hole_filler;')
 p.edit('lib.rs','mod liquid_filler_runtime;','mod liquid_filler_runtime;\n#[cfg(feature="behavior-hooks")]\nmod hole_filler_runtime;')
 p.edit('behaviors.rs','    pub liquid_filler:crate::liquid_filler_runtime::LiquidFillerRuntime,','    pub liquid_filler:crate::liquid_filler_runtime::LiquidFillerRuntime,\n    pub hole_filler:crate::hole_filler_runtime::HoleFillerRuntime,')
 p.edit('behaviors.rs','            "LiquidFiller",','            "LiquidFiller",\n            "HoleFiller",')
 p.edit('behaviors.rs','            liquid_filler:Default::default(),','            liquid_filler:Default::default(),\n            hole_filler:Default::default(),')
 p.edit('behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnDisabled(id) if self.query_ids.get("HoleFiller")==Some(&id)=>{self.hole_filler.placer.disable(current_time_millis());},''')
 p.edit('behaviors.rs','            crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"LiquidFiller"),','            crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"LiquidFiller"),\n            crate::hole_filler_runtime::PLACEMENT_PROVIDER|crate::hole_filler_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"HoleFiller"),')
 p.edit('config.rs','pub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};','pub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};\n#[path="config/hole_filler.rs"]\nmod hole_filler;\npub use hole_filler::{HoleFillerSetting,serialize_hole_filler};')
 p.edit('config.rs','    LiquidFiller=>("LiquidFiller","World","liquidFiller"),','    LiquidFiller=>("LiquidFiller","World","liquidFiller"),\n    HoleFiller=>("HoleFiller","World","holeFiller"),')
 p.edit('config.rs','    LiquidFiller(LiquidFillerSetting),','    LiquidFiller(LiquidFillerSetting),\n    HoleFiller(HoleFillerSetting),',2)
 p.edit('config.rs','            | ModuleKind::LiquidFiller','            | ModuleKind::LiquidFiller\n            | ModuleKind::HoleFiller')
 p.edit('config.rs','        ModuleKind::LiquidFiller=>liquid_filler::parse_liquid_filler(&mut patch,&root),','        ModuleKind::LiquidFiller=>liquid_filler::parse_liquid_filler(&mut patch,&root),\n        ModuleKind::HoleFiller=>hole_filler::parse_hole_filler(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::LiquidFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::LiquidFiller(value.clone())),','        SettingChange::LiquidFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::LiquidFiller(value.clone())),\n        SettingChange::HoleFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::HoleFiller(value.clone())),')
 p.edit('actor.rs','            ModuleKind::LiquidFiller=>config::serialize_liquid_filler(&b.liquid_filler,&common,format,&describe),','            ModuleKind::LiquidFiller=>config::serialize_liquid_filler(&b.liquid_filler,&common,format,&describe),\n            ModuleKind::HoleFiller=>config::serialize_hole_filler(&b.hole_filler,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::LiquidFiller(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::HoleFiller(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().hole_filler,&self.state.block_registry){eprintln!("[LiquidBounce] HoleFiller config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::LiquidFiller(setting))=>{''')
 p.edit('runtime.rs','        self.liquid_filler_prepare(host); // BlockPlacer RotationUpdate priority -20','        self.liquid_filler_prepare(host); // BlockPlacer RotationUpdate priority -20\n        self.hole_filler_prepare(host);')
 p.edit('runtime.rs','            let liquid_filler_running=modules.query_running(&state.registry,"LiquidFiller");','            let liquid_filler_running=modules.query_running(&state.registry,"LiquidFiller");\n            let hole_filler_running=modules.query_running(&state.registry,"HoleFiller");')
 p.edit('runtime.rs','                        crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>liquid_filler_running,','                        crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>liquid_filler_running,\n                        crate::hole_filler_runtime::PLACEMENT_PROVIDER|crate::hole_filler_runtime::CRYSTAL_PROVIDER=>hole_filler_running,')
 anchor='                        } else if let Some((crystal,mode))=crate::block_placer_runtime::BlockPlacerRuntime::processor_mode(processor){'
 p.edit('runtime.rs',anchor,'''                        } else if let Some((crystal,mode))=crate::hole_filler_runtime::HoleFillerRuntime::processor_mode(processor){
                            (if crystal{&mut modules.hole_filler.placer.crystal_rotations}else{&mut modules.hole_filler.placer.rotations},mode)
'''+anchor)
 p.edit('runtime.rs','        self.block_placer_reached(host);','        self.block_placer_reached(host);\n        self.hole_filler_reached(host);')
 p.edit('runtime.rs','        self.block_placer_crystal_tick(host);','        self.block_placer_crystal_tick(host);\n        self.hole_filler_crystal_tick(host);')
 p.edit('runtime.rs','        event.shift=horizon_client::inventory_move::sneak(event.shift);','        if modules.query_running(&state.registry,"HoleFiller")&&modules.hole_filler.placer.queue.movement(){event.shift=true;}\n        event.shift=horizon_client::inventory_move::sneak(event.shift);')
 p.edit('runtime.rs','        self.auto_queue_world_change();','        self.auto_queue_world_change();\n        self.hole_filler_world_change();')
 anchor='    #[cfg(feature="behavior-hooks")]\n    fn client_extinguish_success('
 p.edit('runtime.rs',anchor,'''    #[cfg(feature="behavior-hooks")]
    fn client_hole_filler(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientHoleFillerEvent<'_>){self.hole_filler_event(event);}
'''+anchor)
 p.edit('placement_esp_runtime.rs','        let liquid_filler=m.query_running(&state.registry,"LiquidFiller");','        let liquid_filler=m.query_running(&state.registry,"LiquidFiller");\n        let hole_filler=m.query_running(&state.registry,"HoleFiller");')
 p.edit('placement_esp_runtime.rs','                    m.liquid_filler.placer.placed_render.tick();','                    m.liquid_filler.placer.placed_render.tick();\n                    m.hole_filler.placer.target_render.tick();\n                    m.hole_filler.placer.placed_render.tick();')
 p.edit('placement_esp_runtime.rs','                if m.liquid_filler.placer.placed_render.running(liquid_filler){draws.extend(m.liquid_filler.placer.placed_render.render(time));}','                if m.liquid_filler.placer.placed_render.running(liquid_filler){draws.extend(m.liquid_filler.placer.placed_render.render(time));}\n                if m.hole_filler.placer.target_render.running(hole_filler){draws.extend(m.hole_filler.placer.target_render.render(time));}\n                if m.hole_filler.placer.placed_render.running(hole_filler){draws.extend(m.hole_filler.placer.placed_render.render(time));}')
 p.edit('block_placer_runtime.rs','fn block_placer_run(&self,host:&mut dyn Host,tasks:Vec<DeferredAction>){for action in tasks{','fn block_placer_run(&self,host:&mut dyn Host,tasks:Vec<DeferredAction>){for action in tasks{if matches!(&action,DeferredAction::Place{requester,..}|DeferredAction::Crystal{requester,..} if *requester==crate::hole_filler_runtime::HOLE_FILLER_REQUESTER){self.hole_filler_run(host,vec![action]);continue;}')
 p.edit('block_placer_runtime.rs','fn block_placer_success(&self,requester:u64,position:[i32;3],bounds:[f64;6]){self.bed_defender_success(requester,position,bounds);','fn block_placer_success(&self,requester:u64,position:[i32;3],bounds:[f64;6]){self.bed_defender_success(requester,position,bounds);self.hole_filler_success(requester,position,bounds);')
 p.edit('block_placer_runtime.rs','fn block_placer_removed(&self,id:i32){self.bed_defender_removed(id);','fn block_placer_removed(&self,id:i32){self.bed_defender_removed(id);self.hole_filler_removed(id);')
 p.edit('../../make-behavior-patch.py',"'block-placer-patch.py',","'block-placer-patch.py', 'hole-filler-patch.py',")
 return p
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');a=ap.parse_args();p=prepare()
 if not a.check:p.commit()
 print(f'HoleFiller {"validated"if a.check else"integrated"}: {len(p.changed)} shared files')
