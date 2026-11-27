#!/usr/bin/env python3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
E='native/liquidbounce-horizon/engine/src/'
def edit(path,old,new,count=1):
 p=ROOT/path;s=p.read_text(encoding='utf8')
 if s.count(old)!=count:raise ValueError(f'{path}: expected {count} copies of {old[:65]!r}, got {s.count(old)}')
 p.write_text(s.replace(old,new),encoding='utf8')
edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;','pub mod auto_leave;\npub mod spider;')
edit(E+'lib.rs','mod auto_break_runtime;','mod auto_break_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod spider_runtime;')
edit(E+'behaviors.rs','    pub auto_leave:','    pub spider: liquidbounce_behaviors::spider::Spider,\n    pub auto_leave:')
edit(E+'behaviors.rs','            "AutoLeave",','            "AutoLeave",\n            "Spider",')
edit(E+'behaviors.rs','            auto_leave: Default::default(),','            spider: Default::default(),\n            auto_leave: Default::default(),')
edit(E+'behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::CancelSequences(id) if self.query_ids.get("Spider") == Some(&id) => self.spider.cancel(),''')
edit(E+'config.rs','#[path = "config/auto_weapon.rs"]','#[path = "config/spider.rs"]\nmod spider;\npub use spider::{SpiderSetting,serialize_spider};\n#[path = "config/auto_weapon.rs"]')
edit(E+'config.rs','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n    Spider => ("Spider", "Movement", "spider"),')
edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    Spider(SpiderSetting),',2)
edit(E+'config.rs','            | ModuleKind::AutoLeave','            | ModuleKind::AutoLeave\n            | ModuleKind::Spider')
edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::Spider => spider::parse_spider(&mut patch, &root),')
edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::Spider(value) => { return ApplyResult::Runtime(RuntimeRequest::Spider(*value)); }\n        SettingChange::Damage(value) => {')
edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::Spider => config::serialize_spider(&b.spider,&common,format,&describe),')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Spider(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().spider); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
edit(E+'actor.rs','ModuleKind::GuiCloser=>vec!["AutoClose","ContainerCloser"],','ModuleKind::GuiCloser=>vec!["AutoClose","ContainerCloser"], ModuleKind::Spider=>vec!["WallClimb"],')
edit(E+'coroutine_ticker.rs','    AutoLeave,','    AutoLeave,\n    Spider,')
edit(E+'runtime.rs','                WaitOwner::AutoLeave => self.auto_leave_callback(host, in_game, task.callback),','                WaitOwner::AutoLeave => self.auto_leave_callback(host, in_game, task.callback),\n                WaitOwner::Spider => self.spider_callback(host,in_game,task.callback),')
edit(E+'runtime.rs','        self.no_clip_tick(host);','        self.no_clip_tick(host);\n        self.spider_tick(host);')
anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
edit(E+'runtime.rs',anchor,'''    #[cfg(feature = "behavior-hooks")]
    fn client_spider_jump(&self, event: &mut horizon_client::mod_api::ClientSpiderJumpEvent) {
        self.spider_jump(event.horizontal_collision,&mut event.motion);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_spider_shape_active(&self, event: &mut horizon_client::mod_api::ClientSpiderShapeActiveEvent) {
        event.active |= self.spider_shape_active();
    }
'''+anchor)
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'high-jump-patch.py',", "'high-jump-patch.py', 'spider-patch.py',")
print('Integrated Spider')
