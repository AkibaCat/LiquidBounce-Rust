#!/usr/bin/env python3
"""One-time integration of frozen TargetLock production sources."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def edit(path,old,new,count=1):
 p=ROOT/path;s=p.read_text(encoding='utf8')
 if s.count(old)!=count:raise ValueError(f'{path}: expected {count} copies of {old[:70]!r}, got {s.count(old)}')
 p.write_text(s.replace(old,new),encoding='utf8')
E='native/liquidbounce-horizon/engine/src/'
edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;','pub mod auto_leave;\npub mod target_lock;')
edit(E+'lib.rs','mod one_shots_runtime;','mod one_shots_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod target_lock_runtime;')
edit(E+'behaviors.rs','    pub auto_leave:','    pub target_lock: liquidbounce_behaviors::target_lock::TargetLock,\n    pub auto_leave:')
edit(E+'behaviors.rs','            "TextFieldProtect",','            "TextFieldProtect",\n            "TargetLock",')
edit(E+'behaviors.rs','            auto_leave: Default::default(),','            target_lock: Default::default(),\n            auto_leave: Default::default(),')
edit(E+'targeting.rs','    pub player_name: Option<String>,','    pub player_name: Option<String>,\n    pub abstract_client_player: bool,')
edit(E+'targeting.rs','        player_name: value["player_name"].as_str().map(str::to_owned),','        player_name: value["player_name"].as_str().map(str::to_owned),\n        abstract_client_player: value["abstract_client_player"].as_bool().unwrap_or(false),')
edit(E+'runtime.rs','''        let info = self
            .friends
            .tag(snapshot.entity.kind, snapshot.player_name.as_deref());
        self.target_tags.get(id, || info)''','''        self.target_tags.get(id, || {
            let mut info = self.friends.tag(snapshot.entity.kind, snapshot.player_name.as_deref());
            let player = snapshot.abstract_client_player.then(|| snapshot.player_name.as_ref().map(|name|
                liquidbounce_behaviors::target_lock::Player { id, name: name.clone() })).flatten();
            let modules = self.behaviors.borrow();
            crate::target_lock_runtime::apply_tag(&modules.target_lock,
                modules.query_running(&self.registry,"TargetLock"), player.as_ref(), &mut info);
            info
        })''')
edit(E+'runtime.rs','''        self.reset_vl_tick(host);
        self.auto_leave_tick(host);''','''        self.auto_leave_tick(host);
        self.reset_vl_tick(host);
        self.target_lock_tick(host);''')
edit(E+'runtime.rs','''        #[cfg(feature = "webview")]
        self.state().probe("clientAttackEntity",''','''        self.target_lock_attack(host, event.target as i32);
        #[cfg(feature = "webview")]
        self.state().probe("clientAttackEntity",''')
edit(E+'config.rs','mod one_shots;','mod one_shots;\n#[path = "config/target_lock.rs"]\nmod target_lock;\npub use target_lock::{TargetLockSetting,serialize_target_lock};')
edit(E+'config.rs','    TextFieldProtect => ("TextFieldProtect", "Misc", "textFieldProtect"),','    TextFieldProtect => ("TextFieldProtect", "Misc", "textFieldProtect"),\n    TargetLock => ("TargetLock", "Misc", "targetLock"),')
edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    TargetLock(TargetLockSetting),',2)
edit(E+'config.rs','            | ModuleKind::TextFieldProtect','            | ModuleKind::TextFieldProtect\n            | ModuleKind::TargetLock')
edit(E+'config.rs','        ModuleKind::ItemScroller => item_scroller::parse_item_scroller(&mut patch, &root),','        ModuleKind::ItemScroller => item_scroller::parse_item_scroller(&mut patch, &root),\n        ModuleKind::TargetLock => target_lock::parse_target_lock(&mut patch, &root),')
edit(E+'config.rs','        SettingChange::ResetVl(value) => {','        SettingChange::TargetLock(value) => { return ApplyResult::Runtime(RuntimeRequest::TargetLock(value.clone())); }\n        SettingChange::ResetVl(value) => {')
edit(E+'actor.rs','            ModuleKind::ResetVl => {','            ModuleKind::TargetLock => config::serialize_target_lock(&b.target_lock,&common,format,&describe),\n            ModuleKind::ResetVl => {')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::TargetLock(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().target_lock); }\n                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {')
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'hitbox-patch.py',","'hitbox-patch.py', 'target-lock-patch.py',")
print('Integrated TargetLock shared source')
