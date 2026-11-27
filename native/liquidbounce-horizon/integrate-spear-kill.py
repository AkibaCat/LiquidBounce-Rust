#!/usr/bin/env python3
"""Validate SpearKill wiring in memory; integration owner uses --apply."""
from pathlib import Path
import argparse,importlib.util
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('spear_shared',HERE/'integrate-super-knockback.py')
shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
ENGINE='native/liquidbounce-horizon/engine/src/'
def build_plan():
    p=shared.Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod mace_kill;\n','pub mod mace_kill;\npub mod spear_kill;\n','behavior export')
    p.replace(ENGINE+'lib.rs','mod mace_kill_runtime;\n','mod mace_kill_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod spear_kill_runtime;\n','runtime export')
    b=ENGINE+'behaviors.rs'
    anchor='    pub mace_kill: liquidbounce_behaviors::mace_kill::MaceKill,\n'
    p.replace(b,anchor,anchor+'    pub spear_kill: liquidbounce_behaviors::spear_kill::SpearKill,\n','state')
    p.replace(b,'            "MaceKill",\n','            "MaceKill",\n            "SpearKill",\n','registry')
    p.replace(b,'            mace_kill: Default::default(),\n','            mace_kill: Default::default(),\n            spear_kill: Default::default(),\n','default')
    c=ENGINE+'config.rs'
    anchor='pub use mace_kill::{MaceKillSetting, serialize_mace_kill};\n'
    p.replace(c,anchor,anchor+'#[path = "config/spear_kill.rs"]\nmod spear_kill;\npub use spear_kill::{SpearKillSetting, serialize_spear_kill};\n','config export')
    anchor='    MaceKill => ("MaceKill", "Combat", "maceKill"),\n'
    p.replace(c,anchor,anchor+'    SpearKill => ("SpearKill", "Combat", "spearKill"),\n','kind')
    p.replace(c,'    MaceKill(MaceKillSetting),\n','    MaceKill(MaceKillSetting),\n    SpearKill(SpearKillSetting),\n','setting/runtime variants',count=2)
    p.replace(c,'            | ModuleKind::MaceKill\n','            | ModuleKind::MaceKill\n            | ModuleKind::SpearKill\n','dedicated serializer')
    anchor='        ModuleKind::MaceKill => mace_kill::parse_mace_kill(&mut patch, &root),\n'
    p.replace(c,anchor,anchor+'        ModuleKind::SpearKill => spear_kill::parse_spear_kill(&mut patch, &root),\n','parse')
    anchor='        SettingChange::MaceKill(value) => {\n'
    p.replace(c,anchor,'        SettingChange::SpearKill(value) => {\n            return ApplyResult::Runtime(RuntimeRequest::SpearKill(*value));\n        }\n'+anchor,'apply dispatch')
    a=ENGINE+'actor.rs'
    anchor='            ModuleKind::MaceKill => config::serialize_mace_kill(&b.mace_kill, &common, format, &describe),\n'
    p.replace(a,anchor,anchor+'            ModuleKind::SpearKill => config::serialize_spear_kill(&b.spear_kill, &common, format, &describe),\n','actor serializer')
    anchor='                ApplyResult::Runtime(RuntimeRequest::MaceKill(setting)) => {\n'
    p.replace(a,anchor,'                ApplyResult::Runtime(RuntimeRequest::SpearKill(setting)) => {\n                    setting.apply(&mut self.state.behaviors.borrow_mut().spear_kill);\n                }\n'+anchor,'actor application')
    p.replace(a,'ModuleKind::SuperKnockback=>vec!["WTap"],','ModuleKind::SuperKnockback=>vec!["WTap"], ModuleKind::SpearKill=>vec!["AutoSpear"],','alias')
    r=ENGINE+'runtime.rs'
    p.replace(r,'        self.edge_assistance_prepare(host);\n','        self.edge_assistance_prepare(host);\n        self.spear_kill_tick(host); // FINAL_DECISION (-500)\n','final tick priority')
    p.replace(r,'        self.edge_assistance_render(host,event.primitives);\n','        self.edge_assistance_render(host,event.primitives);\n        self.spear_kill_render(host,event);\n','preview')
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'mace-kill-patch.py',","'mace-kill-patch.py', 'spear-kill-patch.py',",'native patch')
    return p
def main():
    parser=argparse.ArgumentParser(description=__doc__);mode=parser.add_mutually_exclusive_group()
    mode.add_argument('--check',action='store_true');mode.add_argument('--apply',action='store_true');args=parser.parse_args()
    p=build_plan();print(f'Validated {len(p.steps)} SpearKill changes across {len(p.text)} files')
    if args.apply:p.write();print('Applied SpearKill shared integration')
    else:print('Check only; no shared file writes')
if __name__=='__main__':main()
