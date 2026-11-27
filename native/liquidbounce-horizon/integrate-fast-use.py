#!/usr/bin/env python3
"""Transactional one-time FastUse wiring. Validate all anchors before writing."""
from pathlib import Path
import argparse
import runpy

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
E = 'native/liquidbounce-horizon/engine/src/'
Plan = runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']

def build_plan():
    p=Plan()
    def edit(path,old,new,label,count=1): p.replace(path,old,new,label,count)
    edit('native/liquidbounce-behaviors/src/lib.rs','pub mod old_regen;','pub mod old_regen;\npub mod fast_use;','behavior export')
    edit(E+'lib.rs','mod old_regen_runtime;','mod old_regen_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod fast_use_runtime;','runtime export')
    edit(E+'behaviors.rs','    pub old_regen: liquidbounce_behaviors::old_regen::OldRegen,','    pub old_regen: liquidbounce_behaviors::old_regen::OldRegen,\n    pub fast_use: liquidbounce_behaviors::fast_use::FastUse,','behavior field')
    edit(E+'behaviors.rs','            "OldRegen",','            "OldRegen",\n            "FastUse",','registry')
    edit(E+'behaviors.rs','            old_regen: Default::default(),','            old_regen: Default::default(),\n            fast_use: Default::default(),','default')
    edit(E+'behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.timer.set_provider_running(crate::fast_use_runtime::TIMER_PROVIDER, self.query_running(registry, "FastUse"));','timer provider')
    edit(E+'behaviors.rs','        match callback {','        match callback {\n            ModuleCallback::CancelSequences(id) if self.query_ids.get("FastUse") == Some(&id) => { self.fast_use.cancel(); }','cancellation')
    edit(E+'config.rs','pub use old_regen::{OldRegenSetting, serialize_old_regen};','pub use old_regen::{OldRegenSetting, serialize_old_regen};\n#[path = "config/fast_use.rs"]\nmod fast_use;\npub use fast_use::{FastUseSetting, serialize_fast_use};','config export')
    edit(E+'config.rs','    OldRegen => ("OldRegen", "Exploit", "oldRegen"),','    OldRegen => ("OldRegen", "Exploit", "oldRegen"),\n    FastUse => ("FastUse", "Player", "fastUse"),','module kind')
    edit(E+'config.rs','    OldRegen(OldRegenSetting),','    OldRegen(OldRegenSetting),\n    FastUse(FastUseSetting),','setting and runtime variants',2)
    edit(E+'config.rs','            | ModuleKind::OldRegen','            | ModuleKind::OldRegen\n            | ModuleKind::FastUse','dedicated serializer')
    edit(E+'config.rs','        ModuleKind::OldRegen => old_regen::parse_old_regen(&mut patch, &root),','        ModuleKind::OldRegen => old_regen::parse_old_regen(&mut patch, &root),\n        ModuleKind::FastUse => fast_use::parse_fast_use(&mut patch, &root),','parse dispatch')
    edit(E+'config.rs','        SettingChange::OldRegen(value) => {','        SettingChange::FastUse(value) => { return ApplyResult::Runtime(RuntimeRequest::FastUse(*value)); }\n        SettingChange::OldRegen(value) => {','apply dispatch')
    edit(E+'actor.rs','            ModuleKind::OldRegen => {','            ModuleKind::FastUse => config::serialize_fast_use(&b.fast_use,&common,format,&describe),\n            ModuleKind::OldRegen => {','actor serialize')
    edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::OldRegen(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::FastUse(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().fast_use); }\n                ApplyResult::Runtime(RuntimeRequest::OldRegen(setting)) => {','actor apply')
    edit(E+'actor.rs','ModuleKind::OldRegen=>vec!["Regen","Zoot"],','ModuleKind::OldRegen=>vec!["Regen","Zoot"], ModuleKind::FastUse=>vec!["FastEat"],','alias')
    edit(E+'actor.rs','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),\n            ModuleKind::FastUse => Some(self.state.behaviors.borrow().fast_use.settings.mode.tag()),','mode tag')
    edit(E+'coroutine_ticker.rs','    AutoAccount,','    AutoAccount,\n    FastUse,','FIFO owner')
    edit(E+'runtime.rs','                WaitOwner::AutoAccount => self.auto_account_callback(host, task.callback),','                WaitOwner::AutoAccount => self.auto_account_callback(host, task.callback),\n                WaitOwner::FastUse => self.fast_use_callback(host, in_game, task.callback),','FIFO dispatch')
    edit(E+'runtime.rs','        self.old_regen_tick(host);','        self.old_regen_tick(host);\n        self.fast_use_tick(host);','GameTick')
    p.section(E+'runtime.rs','    fn client_directional_input(','    fn client_sprint_queries(',
        '        let mut modules = state.behaviors.borrow_mut();\n',
        '        let mut modules = state.behaviors.borrow_mut();\n        if modules.fast_use.stop_input(modules.query_running(&state.registry,"FastUse"),host.client_fast_use_key()) {\n            event.forward=false; event.backward=false; event.left=false; event.right=false;\n        }\n','critical movement input')
    edit(E+'runtime.rs','            modules.inventory.world_change();','            modules.inventory.world_change();\n            modules.fast_use.cancel();','world and disconnect cancellation',2)
    edit('native/liquidbounce-horizon/make-behavior-patch.py',"'old-regen-patch.py',","'old-regen-patch.py', 'fast-use-patch.py',",'native extension')
    return p

if __name__=='__main__':
    a=argparse.ArgumentParser(description=__doc__)
    g=a.add_mutually_exclusive_group();g.add_argument('--check',action='store_true');g.add_argument('--apply',action='store_true')
    args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} FastUse changes across {len(p.text)} files')
    if args.apply:p.write();print('Applied FastUse complete live wiring')
