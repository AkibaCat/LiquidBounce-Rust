#!/usr/bin/env python3
"""Validate complete Offhand shared wiring in memory; integration owner applies."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod murder_mystery;','pub mod murder_mystery;\npub mod offhand;','behavior export')
 p.replace(E+'lib.rs','mod murder_mystery_runtime;','mod murder_mystery_runtime;\n#[cfg(feature="behavior-hooks")]\nmod offhand_runtime;','runtime export')
 b=E+'behaviors.rs'
 p.replace(b,'    pub murder_mystery:liquidbounce_behaviors::murder_mystery::State,','    pub murder_mystery:liquidbounce_behaviors::murder_mystery::State,\n    pub offhand:crate::offhand_runtime::OffhandRuntime,','state')
 p.replace(b,'            "MurderMystery",','            "MurderMystery",\n            "Offhand",','registration')
 p.replace(b,'            murder_mystery:Default::default(),','            murder_mystery:Default::default(),\n            offhand:Default::default(),','initialization')
 anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("MurderMystery")==Some(&id)=>'
 p.replace(b,anchor,'            ModuleCallback::OnEnabled(id) if self.query_ids.get("Offhand")==Some(&id)=>{let ca=self.query_running(registry,"CrystalAura");self.offhand.enable(ca);}\n            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("Offhand")==Some(&id)=>{self.offhand.cancel(&mut self.inventory);self.item_policy.offhand_operating=false;}\n'+anchor,'persistent state lifecycle and immediate shared policy')
 c=E+'config.rs'
 p.replace(c,'pub use murder_mystery::{MurderMysterySetting,serialize_murder_mystery};','pub use murder_mystery::{MurderMysterySetting,serialize_murder_mystery};\n#[path="config/offhand.rs"]\nmod offhand;\npub use offhand::{OffhandSetting,serialize_offhand};','config export')
 p.replace(c,'    MurderMystery => ("MurderMystery", "Render", "murderMystery"),','    MurderMystery => ("MurderMystery", "Render", "murderMystery"),\n    Offhand => ("Offhand","Player","offhand"),','kind')
 p.replace(c,'"ContainerStorer"=>Some(Self::AutoDeposit),','"AutoTotem"=>Some(Self::Offhand), "ContainerStorer"=>Some(Self::AutoDeposit),','alias parsing')
 p.replace(c,'    MurderMystery(MurderMysterySetting),','    MurderMystery(MurderMysterySetting),\n    Offhand(OffhandSetting),','setting and runtime variants',count=2)
 p.replace(c,'            | ModuleKind::MurderMystery','            | ModuleKind::MurderMystery\n            | ModuleKind::Offhand','dedicated serializer')
 p.replace(c,'        ModuleKind::MurderMystery=>murder_mystery::parse_murder_mystery(&mut patch,&root),','        ModuleKind::MurderMystery=>murder_mystery::parse_murder_mystery(&mut patch,&root),\n        ModuleKind::Offhand=>offhand::parse_offhand(&mut patch,&root),','tree parser')
 p.replace(c,'        SettingChange::MurderMystery(value)=>','        SettingChange::Offhand(value)=>return ApplyResult::Runtime(RuntimeRequest::Offhand(value.clone())),\n        SettingChange::MurderMystery(value)=>','apply runtime request')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::MurderMystery=>config::serialize_murder_mystery(&b.murder_mystery,&common,format,&describe),','            ModuleKind::MurderMystery=>config::serialize_murder_mystery(&b.murder_mystery,&common,format,&describe),\n            ModuleKind::Offhand=>config::serialize_offhand(&b.offhand.state.settings,&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::MurderMystery(setting))=>','                ApplyResult::Runtime(RuntimeRequest::Offhand(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().offhand.state.settings){eprintln!("[LiquidBounce] Offhand config: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::MurderMystery(setting))=>','apply full live tree')
 p.replace(a,'            ModuleKind::SuperKnockback => Some(self.state.behaviors.borrow().super_knockback.settings.mode.tag()),','            ModuleKind::Offhand=>Some(self.state.behaviors.borrow().offhand.state.active.name()),\n            ModuleKind::SuperKnockback => Some(self.state.behaviors.borrow().super_knockback.settings.mode.tag()),','active mode tag')
 p.replace(a,'ModuleKind::MoreCarry=>vec!["XCarry"],','ModuleKind::Offhand=>vec!["AutoTotem"], ModuleKind::MoreCarry=>vec!["XCarry"],','source alias')
 r=E+'runtime.rs'
 p.replace(r,'        self.macros_key(host,code);','        self.macros_key(host,code);\n        #[cfg(feature="behavior-hooks")]\n        if action==KeyAction::Press{self.offhand_key(host,code);}','press only no screen guard')
 i=E+'inventory_runtime.rs'
 p.replace(i,'        modules.replenish.world_change();','        modules.replenish.world_change();\n        modules.offhand.world_change();\n        modules.item_policy.offhand_operating=false;','world cancellation preserves source mode state')
 p.replace(i,'        self.inventory_cleaner_plan(host,&snapshot.context);','        self.offhand_plan(host,&snapshot.context);\n        self.inventory_cleaner_plan(host,&snapshot.context);','source planner and real shared policy')
 p.replace(i,'            let tools = self.tool_snapshot(host, [0; 3], false);','            self.offhand_refresh(host);\n            let tools = self.tool_snapshot(host, [0; 3], false);','fresh requires hazards at each continuation')
 p.replace(i,'                modules.inventory.observe_context(&context);','                owners.push(modules.offhand.owner(modules.query_running(&state.registry,"Offhand"),&context));\n                modules.inventory.observe_context(&context);','shared constraints and dynamic requires')
 p.replace(i,'                    self.auto_armor_outcome(host,outcome,&context);','                    self.auto_armor_outcome(host,outcome,&context);\n                    {let s=self.state();let mut b=s.behaviors.borrow_mut();let b=&mut*b;b.offhand.outcome(&mut b.inventory,outcome);}','finish timing after actual performed sequence')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'offhand-patch.py', 'step-patch.py'):",'native patch after dependencies')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--apply',action='store_true');args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} Offhand changes across {len(p.text)} shared files')
 if args.apply:p.write();print('Integrated Offhand')
