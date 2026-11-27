#!/usr/bin/env python3
"""Transactional registry/config wiring for Surround and BlockIn pure planners."""
from pathlib import Path
import argparse, runpy
HERE=Path(__file__).resolve().parent
BasePlan=runpy.run_path(str(HERE/'integrate-liquid-filler.py'))['Plan']
class Plan(BasePlan): pass
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod liquid_filler;','pub mod liquid_filler;\npub mod surround;\npub mod block_in;')
 p.edit('behaviors.rs','    pub liquid_filler:crate::liquid_filler_runtime::LiquidFillerRuntime,','    pub liquid_filler:crate::liquid_filler_runtime::LiquidFillerRuntime,\n    pub surround:liquidbounce_behaviors::surround::Settings,\n    pub block_in:liquidbounce_behaviors::block_in::Settings,')
 p.edit('behaviors.rs','            "LiquidFiller",','            "LiquidFiller",\n            "Surround",\n            "BlockIn",')
 p.edit('behaviors.rs','            liquid_filler:Default::default(),','            liquid_filler:Default::default(),\n            surround:Default::default(),\n            block_in:Default::default(),')
 p.edit('config.rs','#[path="config/liquid_filler.rs"]\nmod liquid_filler;\npub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};','#[path="config/liquid_filler.rs"]\nmod liquid_filler;\npub use liquid_filler::{LiquidFillerSetting,serialize_liquid_filler};\n#[path="config/surround.rs"]\nmod surround;\npub use surround::{SurroundSetting,serialize_surround};\n#[path="config/block_in.rs"]\nmod block_in;\npub use block_in::{BlockInSetting,serialize_block_in};',1)
 p.edit('config.rs','    LiquidFiller=>("LiquidFiller","World","liquidFiller"),','    LiquidFiller=>("LiquidFiller","World","liquidFiller"),\n    Surround=>("Surround","World","surround"),\n    BlockIn=>("BlockIn","World","blockIn"),')
 p.edit('config.rs','    LiquidFiller(LiquidFillerSetting),','    LiquidFiller(LiquidFillerSetting),\n    Surround(SurroundSetting),\n    BlockIn(BlockInSetting),',2)
 p.edit('config.rs','            | ModuleKind::LiquidFiller','            | ModuleKind::LiquidFiller\n            | ModuleKind::Surround\n            | ModuleKind::BlockIn')
 p.edit('config.rs','        ModuleKind::LiquidFiller=>liquid_filler::parse_liquid_filler(&mut patch,&root),','        ModuleKind::LiquidFiller=>liquid_filler::parse_liquid_filler(&mut patch,&root),\n        ModuleKind::Surround=>surround::parse_surround(&mut patch,&root),\n        ModuleKind::BlockIn=>block_in::parse_block_in(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::LiquidFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::LiquidFiller(value.clone())),','        SettingChange::LiquidFiller(value)=>return ApplyResult::Runtime(RuntimeRequest::LiquidFiller(value.clone())),\n        SettingChange::Surround(value)=>return ApplyResult::Runtime(RuntimeRequest::Surround(value.clone())),\n        SettingChange::BlockIn(value)=>return ApplyResult::Runtime(RuntimeRequest::BlockIn(value.clone())),' )
 p.edit('actor.rs','            ModuleKind::LiquidFiller=>config::serialize_liquid_filler(&b.liquid_filler,&common,format,&describe),','            ModuleKind::LiquidFiller=>config::serialize_liquid_filler(&b.liquid_filler,&common,format,&describe),\n            ModuleKind::Surround=>config::serialize_surround(&b.surround,&common,format,&describe),\n            ModuleKind::BlockIn=>config::serialize_block_in(&b.block_in,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::LiquidFiller(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::Surround(setting))=>{setting.apply(&mut self.state.behaviors.borrow_mut().surround,&self.state.block_registry).ok();}\n                ApplyResult::Runtime(RuntimeRequest::BlockIn(setting))=>{setting.apply(&mut self.state.behaviors.borrow_mut().block_in,&self.state.block_registry).ok();}\n                ApplyResult::Runtime(RuntimeRequest::LiquidFiller(setting))=>{')
 return p
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');a=ap.parse_args();p=prepare()
 if not a.check:p.commit()
 print(f'Surround/BlockIn {"validated" if a.check else "integrated"}: {len(p.changed)} shared files')
