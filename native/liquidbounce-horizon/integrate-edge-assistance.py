#!/usr/bin/env python3
"""Transactional Parkour/SafeWalk shared wiring. --check never writes."""
from pathlib import Path
import argparse
import runpy

HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']

def prepare():
 p=Plan()
 p.edit('lib.rs','mod auto_walk_runtime;','mod auto_walk_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod edge_assistance_runtime;')
 p.edit('behaviors.rs','    pub auto_walk: crate::auto_walk_runtime::AutoWalkSettings,','    pub auto_walk: crate::auto_walk_runtime::AutoWalkSettings,\n    pub parkour: crate::edge_assistance_runtime::ParkourSettings,\n    pub safe_walk: crate::edge_assistance_runtime::SafeWalkSettings,')
 p.edit('behaviors.rs','            "AutoWalk",','            "AutoWalk",\n            "Parkour",\n            "SafeWalk",')
 p.edit('behaviors.rs','            auto_walk: Default::default(),','            auto_walk: Default::default(),\n            parkour: Default::default(),\n            safe_walk: Default::default(),')
 p.edit('behaviors.rs','            crate::auto_walk_runtime::ROTATION_PROVIDER => self.query_running(registry, "AutoWalk"),','            crate::auto_walk_runtime::ROTATION_PROVIDER => self.query_running(registry, "AutoWalk"),\n            crate::edge_assistance_runtime::ROTATION_PROVIDER => self.query_running(registry, "Parkour"),')
 p.edit('config.rs','pub use auto_walk::{AutoWalkSetting, serialize_auto_walk};','pub use auto_walk::{AutoWalkSetting, serialize_auto_walk};\n#[path = "config/edge_assistance.rs"]\nmod edge_assistance;\npub use edge_assistance::{ParkourSetting,SafeWalkSetting,serialize_parkour,serialize_safe_walk};')
 p.edit('config.rs','    AutoWalk => ("AutoWalk", "Player", "autoWalk"),','    AutoWalk => ("AutoWalk", "Player", "autoWalk"),\n    Parkour => ("Parkour", "Movement", "parkour"),\n    SafeWalk => ("SafeWalk", "Movement", "safeWalk"),')
 p.edit('config.rs','    AutoWalk(AutoWalkSetting),','    AutoWalk(AutoWalkSetting),\n    Parkour(ParkourSetting),\n    SafeWalk(SafeWalkSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoWalk','            | ModuleKind::AutoWalk\n            | ModuleKind::Parkour\n            | ModuleKind::SafeWalk')
 p.edit('config.rs','        ModuleKind::AutoWalk => auto_walk::parse_auto_walk(&mut patch, &root),','        ModuleKind::AutoWalk => auto_walk::parse_auto_walk(&mut patch, &root),\n        ModuleKind::Parkour => edge_assistance::parse_parkour(&mut patch, &root),\n        ModuleKind::SafeWalk => edge_assistance::parse_safe_walk(&mut patch, &root),')
 p.edit('config.rs','''        SettingChange::AutoWalk(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoWalk(value.clone()));
        }''','''        SettingChange::AutoWalk(value) => {
            return ApplyResult::Runtime(RuntimeRequest::AutoWalk(value.clone()));
        }
        SettingChange::Parkour(value) => { return ApplyResult::Runtime(RuntimeRequest::Parkour(value.clone())); }
        SettingChange::SafeWalk(value) => { return ApplyResult::Runtime(RuntimeRequest::SafeWalk(value.clone())); }''')
 p.edit('actor.rs','''            ModuleKind::AutoWalk => {
                config::serialize_auto_walk(&b.auto_walk, &common, format, &describe)
            }''','''            ModuleKind::AutoWalk => {
                config::serialize_auto_walk(&b.auto_walk, &common, format, &describe)
            }
            ModuleKind::Parkour => config::serialize_parkour(&b.parkour,&common,format,&describe),
            ModuleKind::SafeWalk => config::serialize_safe_walk(&b.safe_walk,&common,format,&describe),''')
 p.edit('actor.rs','''                ApplyResult::Runtime(RuntimeRequest::AutoWalk(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().auto_walk);
                }''','''                ApplyResult::Runtime(RuntimeRequest::AutoWalk(setting)) => {
                    setting.apply(&mut self.state.behaviors.borrow_mut().auto_walk);
                }
                ApplyResult::Runtime(RuntimeRequest::Parkour(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().parkour); }
                ApplyResult::Runtime(RuntimeRequest::SafeWalk(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().safe_walk); }''')
 p.edit('runtime.rs','            let auto_walk_running = modules.query_running(&state.registry, "AutoWalk");','            let auto_walk_running = modules.query_running(&state.registry, "AutoWalk");\n            let parkour_running = modules.query_running(&state.registry, "Parkour");')
 p.edit('runtime.rs','                        crate::auto_walk_runtime::ROTATION_PROVIDER => auto_walk_running,','                        crate::auto_walk_runtime::ROTATION_PROVIDER => auto_walk_running,\n                        crate::edge_assistance_runtime::ROTATION_PROVIDER => parkour_running,')
 p.edit('runtime.rs','''                            (&mut modules.auto_walk.rotations, mode)
                        } else {''','''                            (&mut modules.auto_walk.rotations, mode)
                        } else if let Some(mode) = crate::edge_assistance_runtime::ParkourSettings::processor_mode(processor) {
                            (&mut modules.parkour.rotations,mode)
                        } else {''')
 p.edit('runtime.rs','        self.auto_walk_prepare(host);','        self.auto_walk_prepare(host);\n        self.edge_assistance_prepare(host);')
 p.edit('runtime.rs','        self.auto_walk_render(host, event.partial, event.primitives);','        self.auto_walk_render(host, event.partial, event.primitives);\n        self.edge_assistance_render(host,event.primitives);')
 p.edit('runtime.rs','        if self.auto_walk_result(host) {','        let parkour_sprint = self.edge_assistance_result(host);\n        if self.auto_walk_result(host) {')
 p.edit('runtime.rs','        if auto_walk_sprint && transformed.is_moving() && modules.query_running(&state.registry, "AutoWalk") {','        if parkour_sprint && transformed.is_moving() { event.sprint = true; }\n        if auto_walk_sprint && transformed.is_moving() && modules.query_running(&state.registry, "AutoWalk") {')
 p.edit('../../make-behavior-patch.py',"'auto-walk-patch.py',","'auto-walk-patch.py', 'edge-assistance-patch.py',")
 return p

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();plan=prepare()
 if not args.check:plan.commit()
 print('Parkour + SafeWalk '+('validated' if args.check else 'integrated')+f': {len(plan.changed)} shared files')
