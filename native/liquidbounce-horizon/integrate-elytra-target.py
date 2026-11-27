"""Transactionally integrate ElytraTarget into the native grouped build."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']

def integration_applied():
 root=HERE.parents[1]
 checks=(
  (root/'native/liquidbounce-behaviors/src/lib.rs','pub mod elytra_target;'),
  (root/'native/liquidbounce-horizon/engine/src/behaviors.rs','pub elytra_target:'),
  (root/'native/liquidbounce-horizon/engine/src/config.rs','ModuleKind::ElytraTarget'),
  (root/'native/liquidbounce-horizon/engine/src/actor.rs','RuntimeRequest::ElytraTarget'),
  (root/'native/liquidbounce-horizon/engine/src/runtime.rs','self.elytra_target_tick(host);'),
  (root/'native/liquidbounce-horizon/make-behavior-patch.py',"'elytra-target-patch.py'"),
 )
 return all(path.exists() and marker in path.read_text(encoding='utf8') for path,marker in checks)

def prepare():
 if integration_applied():
  return Plan()
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_bow;','pub mod auto_bow;\npub mod elytra_target;')
 p.edit('lib.rs','mod auto_bow_runtime;','mod auto_bow_runtime;\n#[cfg(feature="behavior-hooks")]\nmod elytra_target_runtime;')
 p.edit('behaviors.rs','    pub auto_bow:crate::auto_bow_runtime::AutoBowRuntime,','    pub auto_bow:crate::auto_bow_runtime::AutoBowRuntime,\n    pub elytra_target:crate::elytra_target_runtime::ElytraTargetRuntime,')
 p.edit('behaviors.rs','            "AutoBow",','            "AutoBow",\n            "ElytraTarget",')
 p.edit('behaviors.rs','            auto_bow:Default::default(),','            auto_bow:Default::default(),\n            elytra_target:Default::default(),')
 anchor='            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBow")==Some(&id)=>self.auto_bow.clear(),'
 p.edit('behaviors.rs',anchor,'            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("ElytraTarget")==Some(&id)=>{self.elytra_target.behavior.disabled();self.pending_slot_resets.push(crate::elytra_target_runtime::REQUESTER);},\n'+anchor)
 p.edit('behaviors.rs','    pub fn query_running(&self, registry: &ModuleRegistry, name: &str) -> bool {','    pub fn query_running(&self, registry: &ModuleRegistry, name: &str) -> bool {\n        if name=="ElytraTarget"&&!self.elytra_target.behavior.gliding{return false;}')
 anchor='            crate::auto_bow_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBow"),'
 p.edit('behaviors.rs',anchor,anchor+'\n            crate::elytra_target_runtime::ROTATION_PROVIDER=>self.query_running(registry,"ElytraTarget"),')
 p.edit('config.rs','pub use auto_bow::{AutoBowSetting,serialize_auto_bow};','pub use auto_bow::{AutoBowSetting,serialize_auto_bow};\n#[path="config/elytra_target.rs"]\nmod elytra_target;\npub use elytra_target::{ElytraTargetSetting,serialize_elytra_target};')
 p.edit('config.rs','    AutoBow=>("AutoBow","Combat","autoBow"),','    AutoBow=>("AutoBow","Combat","autoBow"),\n    ElytraTarget=>("ElytraTarget","Combat","elytraTarget"),')
 p.edit('config.rs','    AutoBow(AutoBowSetting),','    AutoBow(AutoBowSetting),\n    ElytraTarget(ElytraTargetSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoBow','            | ModuleKind::AutoBow\n            | ModuleKind::ElytraTarget')
 p.edit('config.rs','        ModuleKind::AutoBow=>auto_bow::parse_auto_bow(&mut patch,&root),','        ModuleKind::AutoBow=>auto_bow::parse_auto_bow(&mut patch,&root),\n        ModuleKind::ElytraTarget=>elytra_target::parse_elytra_target(&mut patch,&root),')
 anchor='        SettingChange::AutoBow(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBow(value.clone())),'
 p.edit('config.rs',anchor,anchor+'\n        SettingChange::ElytraTarget(value)=>return ApplyResult::Runtime(RuntimeRequest::ElytraTarget(value.clone())),')
 anchor='            ModuleKind::AutoBow=>config::serialize_auto_bow(&b.auto_bow,&common,format,&describe),'
 p.edit('actor.rs',anchor,anchor+'\n            ModuleKind::ElytraTarget=>config::serialize_elytra_target(&b.elytra_target,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoBow(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::ElytraTarget(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().elytra_target){eprintln!("[LiquidBounce] ElytraTarget config: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AutoBow(setting))=>{')
 p.edit('runtime.rs','        self.kill_aura_prepare(host);','        self.elytra_target_prepare(host);\n        self.kill_aura_prepare(host);')
 p.edit('runtime.rs','        self.auto_bow_tick(host);','        self.auto_bow_tick(host);\n        self.elytra_target_tick(host);')
 anchor='            let auto_bow_running=modules.query_running(&state.registry,"AutoBow");'
 p.edit('runtime.rs',anchor,anchor+'\n            let elytra_target_running=modules.query_running(&state.registry,"ElytraTarget");')
 anchor='                        crate::auto_bow_runtime::ROTATION_PROVIDER=>auto_bow_running,'
 p.edit('runtime.rs',anchor,anchor+'\n                        crate::elytra_target_runtime::ROTATION_PROVIDER=>elytra_target_running,')
 p.edit('runtime.rs','                        if processor == 0 {','                        if processor==crate::elytra_target_runtime::PROCESSOR{return Ok(modules.elytra_target.behavior.process(from,to,crate::behaviors::current_time_millis()));}\n                        if processor == 0 {')
 p.edit('kill_aura_runtime.rs','        let a = &b.kill_aura;\n        let t = &a.rotations.basic;','        if b.query_running(&s.registry,"ElytraTarget")&&b.elytra_target.behavior.settings.ignore_kill_aura{return;}\n        let a = &b.kill_aura;\n        let t = &a.rotations.basic;')
 p.edit('target_renderer_runtime.rs','        owner!("KillAura",kill_aura,true);','        owner!("KillAura",kill_aura,!(m.query_running(&state.registry,"ElytraTarget")&&m.elytra_target.target_renderer.boolean("Enabled")&&m.kill_aura.behavior.target==m.elytra_target.behavior.target));')
 p.edit('target_renderer_runtime.rs','        // TARGET_RENDERER_OWNERS_END','        owner!("ElytraTarget",elytra_target,true);\n        // TARGET_RENDERER_OWNERS_END')
 p.edit('../../make-behavior-patch.py',"'auto-bow-patch.py',","'auto-bow-patch.py', 'elytra-target-patch.py',")
 return p
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');a=ap.parse_args();p=prepare()
 if not a.check:p.commit()
 print('ElytraTarget '+('checked'if a.check else'integrated')+f': {len(p.changed)} shared files')
