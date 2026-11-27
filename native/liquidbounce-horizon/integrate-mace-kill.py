#!/usr/bin/env python3
"""Validate MaceKill shared wiring in memory; --apply writes the reviewed plan."""
from pathlib import Path
import argparse
import importlib.util

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("super_knockback_integration", HERE / "integrate-super-knockback.py")
shared = importlib.util.module_from_spec(spec)
spec.loader.exec_module(shared)
ENGINE = "native/liquidbounce-horizon/engine/src/"


def build_plan():
    p = shared.Plan()
    p.replace("native/liquidbounce-behaviors/src/lib.rs", "pub mod super_knockback;\n",
              "pub mod super_knockback;\npub mod mace_kill;\n", "behavior export")
    p.replace(ENGINE + "lib.rs", "mod super_knockback_runtime;\n",
              'mod super_knockback_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod mace_kill_runtime;\n', "runtime export")
    b = ENGINE + "behaviors.rs"
    anchor = "    pub super_knockback: liquidbounce_behaviors::super_knockback::SuperKnockback,\n"
    p.replace(b, anchor, anchor + "    pub mace_kill: liquidbounce_behaviors::mace_kill::MaceKill,\n", "module state")
    p.replace(b, '            "SuperKnockback",\n', '            "SuperKnockback",\n            "MaceKill",\n', "registry")
    p.replace(b, "            super_knockback: Default::default(),\n",
              "            super_knockback: Default::default(),\n            mace_kill: Default::default(),\n", "default state")
    c = ENGINE + "config.rs"
    anchor = "pub use super_knockback::{SuperKnockbackSetting, serialize_super_knockback};\n"
    p.replace(c, anchor, anchor + '#[path = "config/mace_kill.rs"]\nmod mace_kill;\n'
              "pub use mace_kill::{MaceKillSetting, serialize_mace_kill};\n", "configuration export")
    anchor = '    SuperKnockback => ("SuperKnockback", "Combat", "superKnockback"),\n'
    p.replace(c, anchor, anchor + '    MaceKill => ("MaceKill", "Combat", "maceKill"),\n', "module kind")
    p.replace(c, "    SuperKnockback(SuperKnockbackSetting),\n",
              "    SuperKnockback(SuperKnockbackSetting),\n    MaceKill(MaceKillSetting),\n", "setting and runtime variants", count=2)
    p.replace(c, "            | ModuleKind::SuperKnockback\n", "            | ModuleKind::SuperKnockback\n            | ModuleKind::MaceKill\n", "dedicated serializer exclusion")
    anchor = "        ModuleKind::Spider => spider::parse_spider(&mut patch, &root),\n"
    p.replace(c, anchor, "        ModuleKind::MaceKill => mace_kill::parse_mace_kill(&mut patch, &root),\n" + anchor, "parse dispatch")
    anchor = "        SettingChange::SuperKnockback(value) => {\n"
    p.replace(c, anchor, "        SettingChange::MaceKill(value) => {\n"
              "            return ApplyResult::Runtime(RuntimeRequest::MaceKill(*value));\n        }\n" + anchor, "runtime request dispatch")
    a = ENGINE + "actor.rs"
    anchor = "            ModuleKind::Spider => config::serialize_spider(&b.spider, &common, format, &describe),\n"
    p.replace(a, anchor, "            ModuleKind::MaceKill => config::serialize_mace_kill(&b.mace_kill, &common, format, &describe),\n" + anchor, "actor serializer")
    anchor = "                ApplyResult::Runtime(RuntimeRequest::SuperKnockback(setting)) => {\n"
    p.replace(a, anchor, "                ApplyResult::Runtime(RuntimeRequest::MaceKill(setting)) => {\n"
              "                    setting.apply(&mut self.state.behaviors.borrow_mut().mace_kill);\n                }\n" + anchor, "actor setting application")
    p.replace(a, "        let tag = match kind {\n",
              "        let tag = if kind == ModuleKind::MaceKill {\n"
              "            Some(self.state.behaviors.borrow().mace_kill.fall_height.to_string())\n"
              "        } else { match kind {\n", "numeric FallHeight HUD tag")
    p.replace(a, "        };\n            json!({\"name\":kind.name()",
              "        }.map(str::to_owned) };\n            json!({\"name\":kind.name()", "owned mode tags")
    r = ENGINE + "runtime.rs"
    p.section(r, "    fn client_attack_entity(", "    fn client_attack_speed(",
              "        for action in &vehicle_actions {\n",
              "        self.mace_kill_attack(host);\n        for action in &vehicle_actions {\n", "attack after AutoWeapon synchronization")
    p.replace(r, ".context(false, false)",
              '.context(false, state.registry.enabled_without_world(modules.query_ids["MaceKill"]))', "AutoWeapon canMaceSmash enabled dependency", count=2)
    p.replace("native/liquidbounce-horizon/make-behavior-patch.py", "'super-knockback-patch.py',",
              "'super-knockback-patch.py', 'mace-kill-patch.py',", "native patch aggregation")
    return p


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="validate only (default)")
    mode.add_argument("--apply", action="store_true", help="write the validated plan")
    args = parser.parse_args()
    plan = build_plan()
    print(f"Validated {len(plan.steps)} changes across {len(plan.text)} files")
    if args.apply:
        plan.write()
        print("Applied MaceKill integration")
    else:
        print("Check only; shared files unchanged")


if __name__ == "__main__":
    main()
