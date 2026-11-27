#!/usr/bin/env python3
"""Validate all SuperKnockback shared wiring before any write (default: check).

Run with --apply only from the integration owner after reviewing the plan.
Every anchor and all pre-write file snapshots are checked before files change.
"""
from pathlib import Path
import argparse
import os

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ENGINE = "native/liquidbounce-horizon/engine/src/"


class Plan:
    def __init__(self):
        self.original = {}
        self.text = {}
        self.steps = []

    def read(self, name):
        if name not in self.text:
            raw = (ROOT / name).read_bytes()
            self.original[name] = raw
            self.text[name] = raw.decode("utf8").replace("\r\n", "\n")
        return self.text[name]

    def replace(self, name, old, new, label, count=1):
        text = self.read(name)
        actual = text.count(old)
        if actual != count:
            raise RuntimeError(f"{name}: {label}: expected {count} anchors, got {actual}")
        self.text[name] = text.replace(old, new)
        self.steps.append(label)

    def section(self, name, start, end, old, new, label):
        text = self.read(name)
        if text.count(start) != 1 or text.count(end) != 1:
            raise RuntimeError(f"{name}: {label}: section boundary is not unique")
        a = text.index(start)
        b = text.index(end, a + len(start))
        body = text[a:b]
        if body.count(old) != 1:
            raise RuntimeError(f"{name}: {label}: section anchor is not unique")
        self.text[name] = text[:a] + body.replace(old, new) + text[b:]
        self.steps.append(label)

    def write(self):
        # Detect concurrent integration edits before starting any file write.
        for name, raw in self.original.items():
            if (ROOT / name).read_bytes() != raw:
                raise RuntimeError(f"{name}: changed since validation; no files written")
        pending = []
        try:
            for name, text in self.text.items():
                path = ROOT / name
                newline = "\r\n" if b"\r\n" in self.original[name] else "\n"
                raw = text.replace("\n", newline).encode("utf8")
                temporary = path.with_name(path.name + ".super-knockback.tmp")
                with temporary.open("xb") as stream:
                    stream.write(raw)
                pending.append((temporary, path))
            for temporary, path in pending:
                os.replace(temporary, path)
        finally:
            for temporary, _ in pending:
                if temporary.exists():
                    temporary.unlink()


def build_plan():
    p = Plan()
    p.replace("native/liquidbounce-behaviors/src/lib.rs", "pub mod middle_click_action;\n",
              "pub mod middle_click_action;\npub mod super_knockback;\n", "behavior export")
    p.replace(ENGINE + "lib.rs", 'mod middle_click_action_runtime;\n',
              'mod middle_click_action_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod super_knockback_runtime;\n', "runtime export")
    b = ENGINE + "behaviors.rs"
    p.replace(b, "    pub middle_click_action: liquidbounce_behaviors::middle_click_action::MiddleClickAction,\n",
              "    pub middle_click_action: liquidbounce_behaviors::middle_click_action::MiddleClickAction,\n"
              "    pub super_knockback: liquidbounce_behaviors::super_knockback::SuperKnockback,\n"
              "    pub combat_cooldown: std::sync::Arc<crate::super_knockback_runtime::CombatCooldown>,\n", "module and shared service fields")
    p.replace(b, '            "MiddleClickAction",\n', '            "MiddleClickAction",\n            "SuperKnockback",\n', "registry")
    p.replace(b, "        let coroutine_ticker = crate::coroutine_ticker::CoroutineTicker::default();\n",
              "        let coroutine_ticker = crate::coroutine_ticker::CoroutineTicker::default();\n"
              "        let timer = std::sync::Arc::new(crate::timer_runtime::TimerService::default());\n", "one Timer service owner")
    p.replace(b, "            timer: Default::default(),\n", "            timer: timer.clone(),\n", "Timer field")
    p.replace(b, "            middle_click_action: Default::default(),\n",
              "            middle_click_action: Default::default(),\n"
              "            super_knockback: Default::default(),\n"
              "            combat_cooldown: std::sync::Arc::new(crate::super_knockback_runtime::CombatCooldown::new(timer)),\n", "module and service initialization")
    anchor = '            ModuleCallback::CancelSequences(id) if self.query_ids.get("Spider") == Some(&id) => {\n'
    p.replace(b, anchor,
              '            ModuleCallback::CancelSequences(id) if self.query_ids.get("SuperKnockback") == Some(&id) => {\n'
              "                self.super_knockback.cancel();\n            }\n" + anchor, "sequence cancellation")
    c = ENGINE + "config.rs"
    anchor = "pub use middle_click_action::{MiddleClickActionSetting, serialize_middle_click_action};\n"
    p.replace(c, anchor, anchor + '#[path = "config/super_knockback.rs"]\nmod super_knockback;\n'
              "pub use super_knockback::{SuperKnockbackSetting, serialize_super_knockback};\n", "configuration export")
    anchor = '    MiddleClickAction => ("MiddleClickAction", "Misc", "middleClickAction"),\n'
    p.replace(c, anchor, anchor + '    SuperKnockback => ("SuperKnockback", "Combat", "superKnockback"),\n', "module kind")
    p.replace(c, "    MiddleClickAction(MiddleClickActionSetting),\n",
              "    MiddleClickAction(MiddleClickActionSetting),\n    SuperKnockback(SuperKnockbackSetting),\n", "setting and runtime enum variants", count=2)
    p.replace(c, "            | ModuleKind::MiddleClickAction\n",
              "            | ModuleKind::MiddleClickAction\n            | ModuleKind::SuperKnockback\n", "dedicated serializer exclusion")
    p.replace(c, "        ModuleKind::Spider => spider::parse_spider(&mut patch, &root),\n",
              "        ModuleKind::SuperKnockback => super_knockback::parse_super_knockback(&mut patch, &root),\n"
              "        ModuleKind::Spider => spider::parse_spider(&mut patch, &root),\n", "parse dispatch")
    anchor = "        SettingChange::MiddleClickAction(value) => {\n"
    p.replace(c, anchor, "        SettingChange::SuperKnockback(value) => {\n"
              "            return ApplyResult::Runtime(RuntimeRequest::SuperKnockback(*value));\n        }\n" + anchor, "runtime request dispatch")
    a = ENGINE + "actor.rs"
    anchor = "            ModuleKind::Spider => config::serialize_spider(&b.spider, &common, format, &describe),\n"
    p.replace(a, anchor, "            ModuleKind::SuperKnockback => config::serialize_super_knockback(&b.super_knockback, &common, format, &describe),\n" + anchor, "actor serializer")
    anchor = 'ModuleKind::MiddleClickAction=>vec!["FriendClicker","MiddleClickPearl"],'
    p.replace(a, anchor, anchor + ' ModuleKind::SuperKnockback=>vec!["WTap"],', "alias")
    anchor = '            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),\n'
    p.replace(a, anchor, anchor + '            ModuleKind::SuperKnockback => Some(self.state.behaviors.borrow().super_knockback.settings.mode.tag()),\n', "mode tag")
    anchor = "                ApplyResult::Runtime(RuntimeRequest::MiddleClickAction(setting)) => {\n"
    p.replace(a, anchor, "                ApplyResult::Runtime(RuntimeRequest::SuperKnockback(setting)) => {\n"
              "                    setting.apply(&mut self.state.behaviors.borrow_mut().super_knockback);\n                }\n" + anchor, "actor setting application")
    p.replace(ENGINE + "coroutine_ticker.rs", "    Spider,\n", "    Spider,\n    SuperKnockback,\n", "FIFO owner")
    r = ENGINE + "runtime.rs"
    anchor = "            pipeline.add_listener(0, modules.timer.clone());\n"
    p.replace(r, anchor, anchor + "            pipeline.add_listener(0, modules.combat_cooldown.clone());\n", "constant PacketListener")
    anchor = "                WaitOwner::Spider => self.spider_callback(host, in_game, task.callback),\n"
    p.replace(r, anchor, anchor + "                WaitOwner::SuperKnockback => self.super_knockback_callback(host, in_game, task.callback),\n", "FIFO dispatch")
    p.section(r, "    pub(crate) fn observe_world(", "    fn feed_keyboard(",
              "            modules.middle_click_action.world_change();\n",
              "            modules.middle_click_action.world_change();\n            modules.combat_cooldown.world_change();\n", "world disappearance reset")
    p.section(r, "    fn client_world_change(", "    fn client_timer_scale(",
              "            timer.world_change();\n", "            timer.world_change();\n"
              "            self.state().behaviors.borrow().combat_cooldown.world_change();\n", "dimension/world event reset")
    p.section(r, "    fn client_disconnecting(", "    fn settings(",
              "            modules.middle_click_action.world_change();\n",
              "            modules.middle_click_action.world_change();\n            modules.combat_cooldown.disconnect();\n", "disconnect TPS reset")
    p.section(r, "    fn client_smart_eat_wait(", "    fn client_use_key_pressed(",
              "            let has_client = host.client_state().is_some();\n",
              "            let has_client = host.client_state().is_some();\n"
              "            if has_client {\n"
              "                self.state().behaviors.borrow().combat_cooldown.drain();\n            }\n", "tick actor work before FIFO")
    p.section(r, "    fn client_attack_entity(", "    fn client_attack_speed(",
              "        use liquidbounce_behaviors::auto_weapon::{REQUESTER, Snapshot, Target};\n",
              "        self.super_knockback_attack(host, event.target);\n"
              "        use liquidbounce_behaviors::auto_weapon::{REQUESTER, Snapshot, Target};\n", "attack before AutoWeapon snapshots")
    p.section(r, "    fn client_directional_input(", "    fn client_sprint_queries(",
              "        let mut modules = state.behaviors.borrow_mut();\n",
              "        let mut modules = state.behaviors.borrow_mut();\n"
              '        let knockback_running = modules.query_running(&state.registry, "SuperKnockback");\n'
              "        if modules.super_knockback.block_movement(knockback_running) {\n"
              "            event.forward = false;\n            event.backward = false;\n"
              "            event.left = false;\n            event.right = false;\n        }\n", "WTap directional input before other modules")
    p.section(r, "    fn client_directional_input(", "    fn client_sprint_queries(",
              "        drop(modules);\n", "        if modules.super_knockback.stop_sprint(knockback_running) {\n"
              "            event.sprint = false;\n        }\n        drop(modules);\n", "SprintTap INPUT after Sprint and AntiHunger")
    anchor = "                *event.managed_jump_yaw = Some(yaw);\n            }\n"
    p.section(r, "    fn client_sprint_queries(", "    fn client_sprint_network(", anchor,
              anchor + '            let knockback_running = modules.query_running(&state.registry, "SuperKnockback");\n'
              "            if modules.super_knockback.stop_sprint(knockback_running) {\n"
              "                event.movement_override = 0;\n            }\n", "SprintTap MOVEMENT_TICK override")
    p.replace("native/liquidbounce-horizon/make-behavior-patch.py", "'middle-click-action-patch.py',",
              "'middle-click-action-patch.py', 'super-knockback-patch.py',", "native patch aggregation")
    return p


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="validate only (default)")
    mode.add_argument("--apply", action="store_true", help="write the completely validated plan")
    args = parser.parse_args()
    plan = build_plan()
    print(f"Validated {len(plan.steps)} changes across {len(plan.text)} files")
    for name in plan.text:
        print(f"  {name}")
    if args.apply:
        plan.write()
        print("Applied SuperKnockback integration")
    else:
        print("Check only; shared files unchanged")


if __name__ == "__main__":
    main()
