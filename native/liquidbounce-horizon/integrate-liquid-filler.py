#!/usr/bin/env python3
"""Idempotently integrate LiquidFiller and its shared non-support BlockPlacer."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
BasePlan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
class Plan(BasePlan):
 def edit(self,name,old,new,count=1):
  path=HERE/'engine/src'/name
  source=self.changed.get(path)
  if source is None:source=path.read_text(encoding='utf8')
  if new in source:return
  super().edit(name,old,new,count)
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod extinguish;','pub mod extinguish;\npub mod liquid_filler;\npub mod block_placer;')
 p.edit('../../../liquidbounce-behaviors/src/rotations.rs','    pub theoretical_server: Rotation,','    pub theoretical_server: Rotation,\n    /// The requested plan processed this update (none during reset/inventory suspension).\n    pub processed_request: Option<u64>,')
 p.edit('../../../liquidbounce-behaviors/src/rotations.rs','        self.last_model = self.current;','        self.processed_request = None;\n        self.last_model = self.current;')
 p.edit('../../../liquidbounce-behaviors/src/rotations.rs','                self.current = Some(rotation);','                self.current = Some(rotation);\n                if requested.is_some(){self.processed_request=Some(plan.processor);}')
 p.edit('lib.rs','mod extinguish_runtime;','mod extinguish_runtime;\n#[cfg(feature="behavior-hooks")]\nmod block_placer_runtime;\n#[cfg(feature="behavior-hooks")]\nmod liquid_filler_runtime;')
 p.edit('behaviors.rs','    pub extinguish:crate::extinguish_runtime::ExtinguishRuntime,','    pub extinguish:crate::extinguish_runtime::ExtinguishRuntime,\n    pub liquid_filler:crate::liquid_filler_runtime::LiquidFillerRuntime,\n    pub block_placer_executor:crate::block_placer_runtime::Executor,')
 p.edit('behaviors.rs','            "Extinguish",','            "Extinguish",\n            "LiquidFiller",')
 p.edit('behaviors.rs','            extinguish:Default::default(),','            extinguish:Default::default(),\n            liquid_filler:Default::default(),\n            block_placer_executor:Default::default(),')
 p.edit('behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnDisabled(id) if self.query_ids.get("LiquidFiller")==Some(&id)=>{self.liquid_filler.placer.disable(current_time_millis());},''')
 p.edit('behaviors.rs','            crate::extinguish_runtime::ROTATION_PROVIDER=>self.query_running(registry,"Extinguish"),','            crate::extinguish_runtime::ROTATION_PROVIDER=>self.query_running(registry,"Extinguish"),\n            crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>self.query_running(registry,"LiquidFiller"),')
 p.edit('config.rs','pub use extinguish::{ExtinguishSetting,serialize_extinguish};','pub use extinguish::{ExtinguishSetting,serialize_extinguish};\n#[path="config/liquid_filler.rs"]\nmod liquid_filler;\npub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};')
 p.edit('config.rs','    Extinguish=>("Extinguish","World","extinguish"),','    Extinguish=>("Extinguish","World","extinguish"),\n    LiquidFiller=>("LiquidFiller","World","liquidFiller"),')
 p.edit('config.rs','    Extinguish(ExtinguishSetting),','    Extinguish(ExtinguishSetting),\n    LiquidFiller(LiquidFillerSetting),',2)
 p.edit('config.rs','            | ModuleKind::Extinguish','            | ModuleKind::Extinguish\n            | ModuleKind::LiquidFiller')
 p.edit('config.rs','        ModuleKind::Extinguish=>extinguish::parse_extinguish(&mut patch,&root),','        ModuleKind::Extinguish=>extinguish::parse_extinguish(&mut patch,&root),\n        ModuleKind::LiquidFiller=>liquid_filler::parse_liquid_filler(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Extinguish(value)=>return ApplyResult::Runtime(RuntimeRequest::Extinguish(value.clone())),','        SettingChange::Extinguish(value)=>{return ApplyResult::Runtime(RuntimeRequest::Extinguish(value.clone()));}\n        SettingChange::LiquidFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::LiquidFiller(value.clone())),')
 p.edit('actor.rs','            ModuleKind::Extinguish=>config::serialize_extinguish(&b.extinguish,&common,format,&describe),','            ModuleKind::Extinguish=>config::serialize_extinguish(&b.extinguish,&common,format,&describe),\n            ModuleKind::LiquidFiller=>config::serialize_liquid_filler(&b.liquid_filler,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Extinguish(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::LiquidFiller(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().liquid_filler,&self.state.block_registry){eprintln!("[LiquidBounce] LiquidFiller config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::Extinguish(setting))=>{''')
 p.edit('runtime.rs','        self.custom_ambience_tick(); // ExpiringList FIRST_PRIORITY','        self.block_placer_first_tick(host); // PostRotationExecutor FIRST_PRIORITY\n        self.custom_ambience_tick(); // ExpiringList FIRST_PRIORITY')
 p.edit('runtime.rs','        self.scaffold_prepare(host);','        self.scaffold_prepare(host);\n        self.liquid_filler_prepare(host); // BlockPlacer RotationUpdate priority -20')
 p.edit('runtime.rs','            let extinguish_running=modules.query_running(&state.registry,"Extinguish");','            let extinguish_running=modules.query_running(&state.registry,"Extinguish");\n            let liquid_filler_running=modules.query_running(&state.registry,"LiquidFiller");')
 p.edit('runtime.rs','                        crate::extinguish_runtime::ROTATION_PROVIDER=>extinguish_running,','                        crate::extinguish_runtime::ROTATION_PROVIDER=>extinguish_running,\n                        crate::block_placer_runtime::PLACEMENT_PROVIDER|crate::block_placer_runtime::CRYSTAL_PROVIDER=>liquid_filler_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::extinguish_runtime::ExtinguishRuntime::processor_mode(processor){','''                        } else if let Some((crystal,mode))=crate::block_placer_runtime::BlockPlacerRuntime::processor_mode(processor){
                            (if crystal{&mut modules.liquid_filler.placer.crystal_rotations}else{&mut modules.liquid_filler.placer.rotations},mode)
                        } else if let Some(mode)=crate::extinguish_runtime::ExtinguishRuntime::processor_mode(processor){''')
 p.edit('runtime.rs','        self.kill_aura_ai_notification(host);','        self.block_placer_reached(host);\n        self.block_placer_crystal_tick(host);\n        self.kill_aura_ai_notification(host);')
 p.edit('runtime.rs','        event.shift=horizon_client::inventory_move::sneak(event.shift);','        if modules.query_running(&state.registry,"LiquidFiller")&&modules.liquid_filler.placer.queue.movement(){event.shift=true;}\n        event.shift=horizon_client::inventory_move::sneak(event.shift);')
 p.edit('runtime.rs','        self.auto_queue_world_change();','        self.auto_queue_world_change();\n        {let s=self.state();let mut b=s.behaviors.borrow_mut();b.liquid_filler.placer.reset_world();b.block_placer_executor.clear();}')
 anchor='    #[cfg(feature="behavior-hooks")]\n    fn client_extinguish_success('
 p.edit('runtime.rs',anchor,'''    #[cfg(feature="behavior-hooks")]
    fn client_block_placer_success(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBlockPlacerSuccessEvent){self.block_placer_success(event.requester,event.position,[0.,0.,0.,1.,1.,1.]);}
    #[cfg(feature="behavior-hooks")]
    fn client_block_placer_after_move(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientBlockPlacerAfterMoveEvent){self.block_placer_after_move(host);}
    #[cfg(feature="behavior-hooks")]
    fn client_block_placer_removed(&self,_host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientBlockPlacerRemovedEvent){self.block_placer_removed(event.id);}
'''+anchor)
 p.edit('placement_esp_runtime.rs','        let scaffold=m.query_running(&state.registry,"Scaffold");','        let scaffold=m.query_running(&state.registry,"Scaffold");\n        let liquid_filler=m.query_running(&state.registry,"LiquidFiller");')
 p.edit('placement_esp_runtime.rs','                    m.void_esp.render.tick();','                    m.void_esp.render.tick();\n                    m.liquid_filler.placer.target_render.tick();\n                    m.liquid_filler.placer.placed_render.tick();')
 p.edit('placement_esp_runtime.rs','                if m.scaffold.render.running(scaffold){draws.extend(m.scaffold.render.render(time));}','                if m.scaffold.render.running(scaffold){draws.extend(m.scaffold.render.render(time));}\n                if m.liquid_filler.placer.target_render.running(liquid_filler){draws.extend(m.liquid_filler.placer.target_render.render(time));}\n                if m.liquid_filler.placer.placed_render.running(liquid_filler){draws.extend(m.liquid_filler.placer.placed_render.render(time));}')
 p.edit('../../make-behavior-patch.py',"'scaffold-special-techniques-patch.py',","'scaffold-special-techniques-patch.py', 'block-placer-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('LiquidFiller '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
