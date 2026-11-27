#!/usr/bin/env python3
"""One-time canonical wiring for frozen MobOwners, CombineMobs and BetterTab.

Every replacement is validated in memory before any shared source is written.
Use --check to validate the complete plan without writing canonical files.
The shared localization dictionaries remain a separate root-owned update.
"""
from pathlib import Path
import argparse

ROOT = Path(__file__).resolve().parents[2]
E = 'native/liquidbounce-horizon/engine/src/'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    originals = {}
    changes = {}

    def source(path):
        if path not in changes:
            originals[path] = (ROOT / path).read_text(encoding='utf8')
            changes[path] = originals[path]
        return changes[path]

    def absent(path, token):
        if token in source(path):
            raise ValueError(f'{path}: already contains {token!r}; refuse partial/repeated integration')

    def edit(path, old, new, count=1):
        current = source(path)
        actual = current.count(old)
        if actual != count:
            raise ValueError(f'{path}: expected {count} copies of {old[:90]!r}, got {actual}')
        changes[path] = current.replace(old, new)

    modules = [
        ('mob_owners', 'MobOwners', 'mobOwners', False),
        ('combine_mobs', 'CombineMobs', 'combineMobs', False),
        ('better_tab', 'BetterTab', 'betterTab', True),
    ]
    for snake, kind, key, clone in modules:
        absent('native/liquidbounce-behaviors/src/lib.rs', f'pub mod {snake};')
        absent(E + 'lib.rs', f'mod {snake}_runtime;')
        absent(E + 'behaviors.rs', f'    pub {snake}:')
        absent(E + 'config.rs', f'    {kind} =>')
        absent(E + 'runtime.rs', f'    fn client_{snake}(')
        edit('native/liquidbounce-behaviors/src/lib.rs', 'pub mod auto_leave;',
             f'pub mod auto_leave;\npub mod {snake};')
        edit(E + 'lib.rs', 'mod auto_break_runtime;',
             f'mod auto_break_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod {snake}_runtime;')
        edit(E + 'behaviors.rs', '    pub auto_leave:',
             f'    pub {snake}: liquidbounce_behaviors::{snake}::{kind},\n    pub auto_leave:')
        edit(E + 'behaviors.rs', '            "AutoLeave",',
             f'            "AutoLeave",\n            "{kind}",')
        edit(E + 'behaviors.rs', '            auto_leave: Default::default(),',
             f'            {snake}: Default::default(),\n            auto_leave: Default::default(),')
        edit(E + 'config.rs', '#[path = "config/auto_weapon.rs"]',
             f'#[path = "config/{snake}.rs"]\nmod {snake};\n'
             f'pub use {snake}::{{{kind}Setting, serialize_{snake}}};\n'
             '#[path = "config/auto_weapon.rs"]')
        edit(E + 'config.rs', '    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),',
             '    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n'
             f'    {kind} => ("{kind}", "Render", "{key}"),')
        edit(E + 'config.rs', '    OneShot(OneShotSetting),',
             f'    OneShot(OneShotSetting),\n    {kind}({kind}Setting),', 2)
        edit(E + 'config.rs', '            | ModuleKind::AutoLeave',
             f'            | ModuleKind::AutoLeave\n            | ModuleKind::{kind}')
        edit(E + 'config.rs', '        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),',
             '        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n'
             f'        ModuleKind::{kind} => {snake}::parse_{snake}(&mut patch, &root),')
        value = 'value.clone()' if clone else '*value'
        edit(E + 'config.rs', '        SettingChange::Damage(value) => {',
             f'        SettingChange::{kind}(value) => {{\n'
             f'            return ApplyResult::Runtime(RuntimeRequest::{kind}({value}));\n'
             '        }\n        SettingChange::Damage(value) => {')
        edit(E + 'actor.rs',
             '            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),',
             '            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n'
             f'            ModuleKind::{kind} => config::serialize_{snake}(&b.{snake}, &common, format, &describe),')
        edit(E + 'actor.rs', '                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {',
             f'                ApplyResult::Runtime(RuntimeRequest::{kind}(setting)) => {{\n'
             f'                    setting.apply(&mut self.state.behaviors.borrow_mut().{snake});\n'
             '                }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')

    # CombineMobs has no host-side disable effect: clear both trackers while the
    # callback already owns the behavior state, avoiding a nested RefCell borrow.
    edit(E + 'behaviors.rs', '        match callback {', '''        match callback {
            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {
                self.combine_mobs.disabled();
            }''')
    anchor = '    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
    edit(E + 'runtime.rs', anchor, '''    #[cfg(feature = "behavior-hooks")]
    fn client_mob_owners(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientMobOwnersEvent) {
        self.mob_owners(host, event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_combine_mobs_frame(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientCombineMobsFrameEvent) {
        self.combine_mobs_frame(host, event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_combine_mobs(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientCombineMobsEvent<'_>) {
        self.combine_mobs(host, event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_better_tab(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientBetterTabEvent<'_>) {
        self.better_tab(host, event);
    }
''' + anchor)
    patch = 'native/liquidbounce-horizon/make-behavior-patch.py'
    for name in ('mob-owners', 'combine-mobs', 'better-tab'):
        absent(patch, f"'{name}-patch.py'")
    edit(patch, "'true-sight-patch.py',",
         "'true-sight-patch.py', 'mob-owners-patch.py', 'combine-mobs-patch.py', 'better-tab-patch.py',")

    # Detect a concurrent canonical change after validation before writing any
    # file. The root runs this script only when its other shared edits are done.
    for path, original in originals.items():
        if (ROOT / path).read_text(encoding='utf8') != original:
            raise ValueError(f'{path}: changed during integration validation; no files written')
    if not args.check:
        for path, content in changes.items():
            (ROOT / path).write_text(content, encoding='utf8')
    print(f'{"Validated" if args.check else "Integrated"} MobOwners, CombineMobs and BetterTab across {len(changes)} shared files')


if __name__ == '__main__':
    main()
