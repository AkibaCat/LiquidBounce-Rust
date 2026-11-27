from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']

def integration_applied():
    """Return true when Eagle's registry, config, actor and native seams exist."""
    root=HERE.parents[1]
    checks=(
        (root/'native/liquidbounce-behaviors/src/lib.rs','pub mod eagle;'),
        (root/'native/liquidbounce-horizon/engine/src/behaviors.rs','pub eagle:'),
        (root/'native/liquidbounce-horizon/engine/src/config.rs','ModuleKind::Eagle'),
        (root/'native/liquidbounce-horizon/engine/src/actor.rs','RuntimeRequest::Eagle'),
        (root/'native/liquidbounce-horizon/engine/src/edge_assistance_runtime.rs','"eagle":{"running"'),
        (root/'native/liquidbounce-horizon/make-behavior-patch.py',"'eagle-patch.py'"),
    )
    return all(path.exists() and marker in path.read_text(encoding='utf8') for path,marker in checks)

def prepare():
 if integration_applied():
  return Plan()
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod chest_cleaner;','pub mod chest_cleaner;\npub mod eagle;','behavior')
 edit(E+'behaviors.rs','    pub chest_cleaner:crate::chest_cleaner_runtime::ChestCleanerRuntime,','    pub chest_cleaner:crate::chest_cleaner_runtime::ChestCleanerRuntime,\n    pub eagle:liquidbounce_behaviors::eagle::Settings,','state')
 edit(E+'behaviors.rs','            "ChestCleaner",','            "ChestCleaner",\n            "Eagle",','registry')
 edit(E+'behaviors.rs','            chest_cleaner:Default::default(),','            chest_cleaner:Default::default(),\n            eagle:Default::default(),','default')
 edit(E+'config.rs','pub use chest_cleaner::{ChestCleanerSetting,serialize_chest_cleaner};','pub use chest_cleaner::{ChestCleanerSetting,serialize_chest_cleaner};\n#[path="config/eagle.rs"]\nmod eagle;\npub use eagle::{EagleSetting,serialize_eagle};','config')
 edit(E+'config.rs','    ChestCleaner => ("ChestCleaner","Player","chestCleaner"),','    ChestCleaner => ("ChestCleaner","Player","chestCleaner"),\n    Eagle => ("Eagle","Player","eagle"),','kind')
 edit(E+'config.rs','"ContainerCleaner" => Some(Self::ChestCleaner),','"ContainerCleaner" => Some(Self::ChestCleaner), "FastBridge"|"BridgeAssistant"|"LegitScaffold"=>Some(Self::Eagle),','aliases')
 edit(E+'config.rs','    ChestCleaner(ChestCleanerSetting),','    ChestCleaner(ChestCleanerSetting),\n    Eagle(EagleSetting),','variants',2)
 edit(E+'config.rs','            | ModuleKind::ChestCleaner','            | ModuleKind::ChestCleaner\n            | ModuleKind::Eagle','serialization')
 edit(E+'config.rs','        ModuleKind::ChestCleaner => chest_cleaner::parse_chest_cleaner(&mut patch,&root),','        ModuleKind::ChestCleaner => chest_cleaner::parse_chest_cleaner(&mut patch,&root),\n        ModuleKind::Eagle => eagle::parse_eagle(&mut patch,&root),','parser')
 edit(E+'config.rs','        SettingChange::ChestCleaner(value) => {','        SettingChange::Eagle(value) => {return ApplyResult::Runtime(RuntimeRequest::Eagle(*value));}\n        SettingChange::ChestCleaner(value) => {','dispatch')
 edit(E+'actor.rs','            ModuleKind::ChestCleaner => config::serialize_chest_cleaner(&b.chest_cleaner.settings,&common,format,&describe),','            ModuleKind::ChestCleaner => config::serialize_chest_cleaner(&b.chest_cleaner.settings,&common,format,&describe),\n            ModuleKind::Eagle => config::serialize_eagle(&b.eagle,&common,format,&describe),','tree')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::ChestCleaner(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Eagle(setting)) => {setting.apply(&mut self.state.behaviors.borrow_mut().eagle);}\n                ApplyResult::Runtime(RuntimeRequest::ChestCleaner(setting)) => {','settings')
 edit(E+'edge_assistance_runtime.rs','"safeWalk":{"running":m.query_running(&state.registry,"SafeWalk")','"eagle":{"running":m.query_running(&state.registry,"Eagle"),"distance":m.eagle.distance,"conditional":m.eagle.conditional,"conditions":m.eagle.conditions,"pitch":m.eagle.pitch},"safeWalk":{"running":m.query_running(&state.registry,"SafeWalk")','real input preparation')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'no-web-patch.py',","'eagle-patch.py', 'no-web-patch.py',",'native patch')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare();print(f'Validated {len(p.steps)} Eagle edits')
 if args.apply and p.steps:p.write();print('Applied Eagle')
