"""Transactional ProphuntESP/VoidESP wiring; validates all files before writing."""
from pathlib import Path
import argparse
ROOT=Path(__file__).resolve().parents[2]
E='native/liquidbounce-horizon/engine/src/'

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args();originals={};changes={}
    def read(path):
        if path not in changes:originals[path]=(ROOT/path).read_text(encoding='utf8');changes[path]=originals[path]
        return changes[path]
    def edit(path,old,new,count=1):
        t=read(path)
        if t.count(old)!=count:raise ValueError(f'{path}: expected {count} of {old[:80]!r}, got {t.count(old)}')
        changes[path]=t.replace(old,new)
    def absent(path,token):
        if token in read(path):raise ValueError(f'{path}: already contains {token}')
    absent('native/liquidbounce-behaviors/src/lib.rs','pub mod placement_renderer;')
    absent(E+'behaviors.rs','    pub prophunt_esp:')
    absent(E+'config.rs','    ProphuntEsp =>')
    edit('native/liquidbounce-behaviors/src/lib.rs','pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod placement_renderer;\npub mod prophunt_esp;\npub mod void_esp;\npub mod void_search;')
    edit(E+'lib.rs','mod breadcrumbs_runtime;','mod breadcrumbs_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod placement_esp_runtime;')
    edit(E+'behaviors.rs','    pub breadcrumbs:','    pub prophunt_esp: liquidbounce_behaviors::prophunt_esp::ProphuntEsp,\n    pub void_esp: liquidbounce_behaviors::void_esp::VoidEsp,\n    pub breadcrumbs:')
    edit(E+'behaviors.rs','            "Breadcrumbs",','            "Breadcrumbs",\n            "ProphuntESP",\n            "VoidESP",')
    edit(E+'behaviors.rs','            breadcrumbs: Default::default(),','            prophunt_esp: Default::default(),\n            void_esp: Default::default(),\n            breadcrumbs: Default::default(),')
    edit(E+'config.rs','#[path = "config/breadcrumbs.rs"]','#[path = "config/placement_esp.rs"]\nmod placement_esp;\npub use placement_esp::{ProphuntEspSetting,VoidEspSetting,serialize_prophunt_esp,serialize_void_esp};\n#[path = "config/breadcrumbs.rs"]')
    edit(E+'config.rs','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),\n    ProphuntEsp => ("ProphuntESP", "Render", "prophuntESP"),\n    VoidEsp => ("VoidESP", "Render", "voidEsp"),')
    edit(E+'config.rs','"BlockOverlay" => Some(Self::BlockOutline),','"BlockOverlay" => Some(Self::BlockOutline), "BlockUpdateDetector" | "FallingBlockESP" => Some(Self::ProphuntEsp),')
    edit(E+'config.rs','    Breadcrumbs(BreadcrumbsSetting),','    Breadcrumbs(BreadcrumbsSetting),\n    ProphuntEsp(ProphuntEspSetting),\n    VoidEsp(VoidEspSetting),',2)
    edit(E+'config.rs','            | ModuleKind::Breadcrumbs','            | ModuleKind::Breadcrumbs\n            | ModuleKind::ProphuntEsp\n            | ModuleKind::VoidEsp')
    edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::ProphuntEsp => placement_esp::parse_prophunt_esp(&mut patch,&root),\n        ModuleKind::VoidEsp => placement_esp::parse_void_esp(&mut patch,&root),')
    edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::ProphuntEsp(value) => { return ApplyResult::Runtime(RuntimeRequest::ProphuntEsp(*value)); }\n        SettingChange::VoidEsp(value) => { return ApplyResult::Runtime(RuntimeRequest::VoidEsp(*value)); }\n        SettingChange::Damage(value) => {')
    edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::ProphuntEsp => config::serialize_prophunt_esp(&b.prophunt_esp,&common,format,&describe),\n            ModuleKind::VoidEsp => config::serialize_void_esp(&b.void_esp,&common,format,&describe),')
    edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::ProphuntEsp(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().prophunt_esp); }\n                ApplyResult::Runtime(RuntimeRequest::VoidEsp(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().void_esp); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
    edit(E+'behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {','            ModuleCallback::OnDisabled(id) if self.query_ids.get("ProphuntESP") == Some(&id) => { self.prophunt_esp.render.clear_silently(current_time_millis()); }\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("VoidESP") == Some(&id) => { self.void_esp.disabled(current_time_millis()); }\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {')
    anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
    edit(E+'runtime.rs',anchor,'    #[cfg(feature = "behavior-hooks")]\n    fn client_placement_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientPlacementEspEvent<\'_>){self.placement_esp(host,event);}\n'+anchor)
    edit(E+'runtime.rs',"    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {","    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {\n        { let state=self.state();let mut m=state.behaviors.borrow_mut();m.prophunt_esp.render.clear();m.void_esp.world_change(); }")
    patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'placement-esp-patch.py'")
    edit(patch,"'semantic-pointer-patch.py'","'placement-esp-patch.py', 'semantic-pointer-patch.py'")
    for path,original in originals.items():
        if (ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation, nothing written')
    if not args.check:
        for path,t in changes.items():(ROOT/path).write_text(t,encoding='utf8')
    print(f'{"Validated" if args.check else "Integrated"} ProphuntESP/VoidESP across {len(changes)} shared files')

if __name__=='__main__':main()
