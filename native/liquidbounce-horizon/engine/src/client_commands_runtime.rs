// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared CommandManager execution boundary. Macro calls do not enter history.
use crate::runtime::{LiquidBounceHooks, raw_options};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::client_commands::Reader;
use liquidbounce_core::{BindAction, InputBind, InputKey, Modifier, Modifiers};
const COMMANDS: [&str; 12] = [
    "autoaccount",
    "autodisable",
    "bind",
    "binds",
    "friend",
    "help",
    "hide",
    "invsee",
    "say",
    "toggle",
    "value",
    "xray",
];
fn tr<'a>(en: &'a str, cn: &'a str, tw: &'a str) -> &'a str {
    match horizon_client::lang::code()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "zh_cn" | "zh_sg" | "zh_hans" => cn,
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => tw,
        _ => en,
    }
}
fn feedback(host: &mut dyn Host, what: &str) {
    host.chat(&format!(
        "{}: {what}",
        tr("Command completed", "命令已执行", "指令已執行")
    ));
}
fn required_string(r: &mut Reader) -> Result<String, String> {
    r.string()?
        .ok_or_else(|| tr("Missing command argument", "缺少命令参数", "缺少指令參數").into())
}
fn modifier(name: &str) -> Option<Modifier> {
    match name.to_ascii_lowercase().as_str() {
        "shift" => Some(Modifier::Shift),
        "control" => Some(Modifier::Control),
        "alt" => Some(Modifier::Alt),
        "super" => Some(Modifier::Super),
        _ => None,
    }
}
fn action(name: &str) -> Option<BindAction> {
    match name.to_ascii_lowercase().as_str() {
        "toggle" => Some(BindAction::Toggle),
        "hold" => Some(BindAction::Hold),
        "smart" => Some(BindAction::Smart),
        _ => None,
    }
}
fn page(r: &mut Reader) -> Result<usize, String> {
    let p = r
        .word()
        .map(|s| s.parse::<i32>())
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or(1);
    r.finish()?;
    if p < 1 {
        return Err(tr(
            "Page must be at least 1",
            "页码必须至少为 1",
            "頁碼必須至少為 1",
        )
        .into());
    }
    Ok(p as usize)
}
fn list(host: &mut dyn Host, title: &str, rows: Vec<String>, page: usize) {
    let count = rows.len().div_ceil(8).max(1);
    let page = page.min(count);
    host.chat(title);
    for row in rows.into_iter().skip((page - 1) * 8).take(8) {
        host.chat(&row);
    }
    if count > 1 {
        host.chat(&format!("{page} / {count}"));
    }
}
pub(crate) fn input_name(name: &str) -> Option<String> {
    let name = name.replace('_', ".").to_ascii_lowercase();
    let name = if name.starts_with("key.keyboard.") || name.starts_with("key.mouse.") {
        name
    } else if name.starts_with("keyboard.") || name.starts_with("mouse.") {
        format!("key.{name}")
    } else {
        format!("key.keyboard.{name}")
    };
    (crate::input::configured_bind(&name).is_some()
        || name == "key.keyboard.unknown"
        || name
            .strip_prefix("key.keyboard.")
            .is_some_and(|v| v.parse::<i32>().is_ok())
        || name
            .strip_prefix("key.mouse.")
            .is_some_and(|v| v.parse::<i32>().is_ok()))
    .then_some(name)
}
fn input_bind(name: &str) -> Option<InputKey> {
    let name = input_name(name)?;
    if name == "key.keyboard.unknown" {
        Some(InputKey::Unbound)
    } else {
        crate::input::configured_bind(&name)
            .or_else(|| {
                name.strip_prefix("key.keyboard.")
                    .and_then(|v| v.parse().ok())
                    .map(InputKey::Keyboard)
            })
            .or_else(|| {
                name.strip_prefix("key.mouse.")
                    .and_then(|v| v.parse().ok())
                    .map(InputKey::Mouse)
            })
    }
}
fn distance(a: &str, b: &str) -> usize {
    let mut row = (0..=b.encode_utf16().count()).collect::<Vec<_>>();
    for (i, a) in a.encode_utf16().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, b) in b.encode_utf16().enumerate() {
            let old = row[j + 1];
            row[j + 1] = (row[j] + 1).min(old + 1).min(prev + usize::from(a != b));
            prev = old;
        }
    }
    *row.last().unwrap()
}
impl LiquidBounceHooks {
    pub(crate) fn client_command_prefix(&self) -> String {
        self.state()
            .behaviors
            .borrow()
            .client_commands
            .prefix
            .clone()
    }
    pub(crate) fn client_command_message(&self, host: &mut dyn Host, message: &str) -> bool {
        let prefix = self.client_command_prefix();
        let Some(body) = message.strip_prefix(&prefix) else {
            return false;
        };
        if let Err(e) = self.client_command_execute(host, body) {
            host.chat(&format!(
                "{}: {e}",
                tr("Client command failed", "客户端命令失败", "用戶端指令失敗")
            ));
        }
        let path = self.state().config_root.join("command_history.txt");
        use std::io::Write;
        if let Err(e) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| {
                f.write_all(body.as_bytes())?;
                f.write_all(b"\n")
            })
        {
            eprintln!("[LiquidBounce] command history: {e}");
        }
        true
    }
    fn command_module(&self, name: &str) -> Result<String, String> {
        self.state()
            .registry
            .modules()
            .find(|m| m.definition().name.eq_ignore_ascii_case(name))
            .map(|m| m.definition().name.clone())
            .ok_or_else(|| format!("{}: {name}", tr("Unknown module", "未知模块", "未知模組")))
    }
    /// MultiSelect silently filters unknown/predicate-failing tokens and rejects
    /// only an empty result, preserving the first occurrence of each member.
    fn command_modules(
        &self,
        text: &str,
        include: impl Fn(&str) -> bool,
    ) -> Result<Vec<String>, String> {
        let mut names = Vec::new();
        for token in text.split(',') {
            if let Ok(name) = self.command_module(token) {
                if include(&name) && !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        if names.is_empty() {
            Err(tr("No matching modules", "没有匹配的模块", "沒有符合的模組").into())
        } else {
            Ok(names)
        }
    }
    fn command_bind(
        &self,
        host: &mut dyn Host,
        name: &str,
        key: &str,
        requested_action: Option<String>,
        requested_modifiers: Option<Vec<String>>,
        multi_select: bool,
    ) -> Result<(), String> {
        let name = self.command_module(name)?;
        let mut s = self.state();
        let m = s
            .registry
            .modules()
            .find(|m| m.definition().name == name)
            .unwrap();
        let id = m.id();
        let old = m
            .definition()
            .bind
            .unwrap_or(InputBind::new(InputKey::Unbound, BindAction::Toggle));
        let action = requested_action
            .as_deref()
            .map(|v| {
                action(v).ok_or_else(|| {
                    tr(
                        "Bind action must be Toggle, Hold or Smart",
                        "绑定动作必须为 Toggle、Hold 或 Smart",
                        "綁定動作必須為 Toggle、Hold 或 Smart",
                    )
                    .to_owned()
                })
            })
            .transpose()?
            .unwrap_or(old.action);
        let modifiers = if let Some(mods) = requested_modifiers {
            let mut parsed = Vec::new();
            for m in mods {
                if let Some(m) = modifier(&m) {
                    parsed.push(m);
                } else if !multi_select {
                    return Err(format!(
                        "{}: {m}",
                        tr("Unknown modifier", "未知修饰键", "未知修飾鍵")
                    ));
                }
            }
            if parsed.is_empty() {
                return Err(tr(
                    "No matching modifiers",
                    "没有匹配的修饰键",
                    "沒有符合的修飾鍵",
                )
                .into());
            }
            Modifiers::new(&parsed)
        } else {
            old.modifiers
        };
        let key = if !multi_select && key.eq_ignore_ascii_case("none") {
            InputKey::Unbound
        } else if let Some(key) = input_bind(key).filter(|k| *k != InputKey::Unbound) {
            key
        } else {
            let message = format!("{}: {key}", tr("Unknown input key", "未知按键", "未知按鍵"));
            if multi_select {
                return Err(message);
            }
            host.chat(&message);
            return Ok(());
        };
        let bind = if key == InputKey::Unbound {
            InputBind::new(key, BindAction::Toggle)
        } else {
            InputBind::new(key, action).with_modifiers(modifiers)
        };
        s.registry
            .set_bind(id, Some(bind))
            .map_err(|e| format!("{e:?}"))?;
        drop(s);
        feedback(host, &format!("{name}: {bind:?}"));
        Ok(())
    }
    #[cfg(feature = "webview")]
    fn command_save(&self, host: &dyn Host) {
        let mut s = self.state();
        let root = s.config_root.clone();
        let actor = crate::actor::Actor {
            state: &mut s,
            options: raw_options(host),
            entity_interaction_attribute: None,
        };
        if let Err(e) = actor
            .save(&root.join("modules.json"))
            .and_then(|()| actor.save_global(&root.join("global.json")))
        {
            eprintln!("[LiquidBounce] client command save: {e}");
        }
    }
    pub(crate) fn client_command_execute(
        &self,
        host: &mut dyn Host,
        body: &str,
    ) -> Result<(), String> {
        let mut r = Reader::new(body);
        let Some(root) = r.word() else {
            return Ok(());
        };
        let root = root.to_ascii_lowercase();
        let result = self.command_run(host, &root, &mut r);
        #[cfg(feature = "webview")]
        if matches!(
            root.as_str(),
            "toggle" | "t" | "bind" | "binds" | "hide" | "value" | "autodisable" | "xray"
        ) {
            self.command_save(host);
        }
        result
    }
    fn command_run(&self, host: &mut dyn Host, root: &str, r: &mut Reader) -> Result<(), String> {
        match root {
            "xray"=>return self.xray_command(host,r),
            "invsee"=>return self.inventory_tracker_command(host,r),
            "toggle" | "t" => {
                let name = self.command_module(&required_string(r)?)?;
                r.finish()?;
                let options = raw_options(host);
                let effects = {
                    let mut s = self.state();
                    let m = s
                        .registry
                        .modules()
                        .find(|m| m.definition().name == name)
                        .unwrap();
                    let (id, enabled) = (m.id(), !m.enabled());
                    s.with_registry(options, |r, c| r.set_enabled_sync(id, enabled, c))
                        .map_err(|e| format!("{e:?}"))?;
                    s.take_effects()
                };
                crate::runtime::apply_effects(self, host, effects);
                feedback(host, &name);
            }
            "bind" => {
                let name = required_string(r)?;
                let key = r.required()?;
                let action = r.string()?;
                let mut mods = vec![];
                while let Some(m) = r.string()? {
                    mods.push(m);
                }
                r.finish()?;
                self.command_bind(
                    host,
                    &name,
                    &key,
                    action,
                    (!mods.is_empty()).then_some(mods),
                    false,
                )?;
            }
            "binds" => match r.required()?.to_ascii_lowercase().as_str() {
                "add" => {
                    let name = required_string(r)?;
                    let key = r.required()?;
                    let action = r.string()?;
                    let mods = r
                        .string()?
                        .map(|s| s.split(',').map(str::to_owned).collect());
                    r.finish()?;
                    self.command_bind(host, &name, &key, action, mods, true)?;
                }
                "clear" => {
                    r.finish()?;
                    let mut s = self.state();
                    let ids = s.registry.modules().map(|m| m.id()).collect::<Vec<_>>();
                    for id in ids {
                        s.registry
                            .set_bind(
                                id,
                                Some(InputBind::new(InputKey::Unbound, BindAction::Toggle)),
                            )
                            .map_err(|e| format!("{e:?}"))?;
                    }
                    drop(s);
                    feedback(host, "binds");
                }
                "remove" => {
                    let text = required_string(r)?;
                    r.finish()?;
                    let names = self.command_modules(&text, |name| {
                        self.state()
                            .registry
                            .modules()
                            .find(|m| m.definition().name == name)
                            .is_some_and(|m| {
                                m.definition()
                                    .bind
                                    .is_some_and(|b| b.key != InputKey::Unbound)
                            })
                    })?;
                    let mut s = self.state();
                    for name in names {
                        let id = s
                            .registry
                            .modules()
                            .find(|m| m.definition().name == name)
                            .unwrap()
                            .id();
                        s.registry
                            .set_bind(
                                id,
                                Some(InputBind::new(InputKey::Unbound, BindAction::Toggle)),
                            )
                            .map_err(|e| format!("{e:?}"))?;
                    }
                    drop(s);
                    feedback(host, "binds");
                }
                "list" => {
                    let p = page(r)?;
                    let rows = self
                        .state()
                        .registry
                        .modules()
                        .filter(|m| {
                            m.definition()
                                .bind
                                .is_some_and(|b| b.key != InputKey::Unbound)
                        })
                        .map(|m| format!("{}: {:?}", m.definition().name, m.definition().bind))
                        .collect();
                    list(host, tr("Bindings", "模块绑定", "模組綁定"), rows, p);
                }
                _ => return Err("binds add | remove | clear | list".into()),
            },
            "hide" => {
                #[cfg(feature = "webview")]
                self.command_hide(host, r)?;
                #[cfg(not(feature = "webview"))]
                return Err("Hide requires the settings actor".into());
            }
            "value" => {
                #[cfg(feature = "webview")]
                self.command_value(host, r)?;
                #[cfg(not(feature = "webview"))]
                return Err("Value requires the settings actor".into());
            }
            "friend" => {
                let op = r.required()?.to_ascii_lowercase();
                let mut s = self.state();
                match op.as_str() {
                    "add" => {
                        let name = r.required()?;
                        let alias = r.word();
                        r.finish()?;
                        if !s.friends.add(name.clone(), alias) {
                            return Err(format!(
                                "{}: {name}",
                                tr("Already friends", "好友已存在", "好友已存在")
                            ));
                        }
                    }
                    "remove" => {
                        let name = r.required()?;
                        r.finish()?;
                        if !s.friends.remove(&name) {
                            return Err(format!(
                                "{}: {name}",
                                tr("Unknown friend", "未知好友", "未知好友")
                            ));
                        }
                    }
                    "alias" => {
                        let name = r.required()?;
                        let alias = r.required()?;
                        r.finish()?;
                        let Some(v) = s.friends.names.get_mut(&name) else {
                            return Err(format!(
                                "{}: {name}",
                                tr("Unknown friend", "未知好友", "未知好友")
                            ));
                        };
                        *v = Some(alias);
                    }
                    "clear" => {
                        r.finish()?;
                        if s.friends.names.is_empty() {
                            return Err(tr("No friends", "好友列表为空", "好友清單為空").into());
                        }
                        s.friends.names.clear();
                    }
                    "list" => {
                        r.finish()?;
                        if s.friends.names.is_empty() {
                            host.chat(tr("No friends", "好友列表为空", "好友清單為空"));
                        } else {
                            let mut entries = s.friends.names.iter().collect::<Vec<_>>();
                            entries
                                .sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
                            for (index,(name, alias)) in entries.into_iter().enumerate() {
                                let default_name=format!("Friend {index}");
                                host.client_name_protect_friend(name,alias.as_deref().unwrap_or(&default_name),&s.behaviors.borrow().client_commands.prefix);
                            }
                        }
                        return Ok(());
                    }
                    _ => return Err("friend add | remove | alias | clear | list".into()),
                }
                crate::friends::save(&s.friends, &s.config_root.join("friends.json"))?;
                drop(s);
                feedback(host, "friend");
            }
            "autoaccount" => {
                let op = r.required()?.to_ascii_lowercase();
                r.finish()?;
                if !matches!(op.as_str(), "login" | "register") {
                    return Err("autoaccount login | register".into());
                }
                if !host
                    .client_state()
                    .is_some_and(|c| c.world.is_some() && c.logged_in && c.eye_position.is_some())
                {
                    return Err(tr(
                        "A player in a world is required",
                        "需要玩家进入世界",
                        "需要玩家進入世界",
                    )
                    .into());
                }
                self.auto_account_command(host, &format!(".autoaccount {op}"));
            }
            "autodisable" => {
                let op = r.required()?.to_ascii_lowercase();
                if op == "list" {
                    let p = page(r)?;
                    let rows = self
                        .state()
                        .behaviors
                        .borrow()
                        .auto_disable
                        .selected()
                        .to_vec();
                    list(host, "AutoDisable", rows, p);
                } else if op == "clear" {
                    r.finish()?;
                    self.auto_disable_command(host, ".autodisable clear");
                } else if op == "add" || op == "remove" {
                    let names = self.command_modules(&required_string(r)?, |_| true)?;
                    r.finish()?;
                    for name in names {
                        let changed = {
                            let s = self.state();
                            let mut b = s.behaviors.borrow_mut();
                            if op == "add" {
                                b.auto_disable.add(&name)
                            } else {
                                b.auto_disable.remove(&name)
                            }
                        };
                        if !changed {
                            return Err(format!(
                                "{}: {name}",
                                tr(
                                    "AutoDisable membership is unchanged",
                                    "自动关闭列表状态未改变",
                                    "自動關閉清單狀態未改變"
                                )
                            ));
                        }
                        feedback(host, &name);
                    }
                } else {
                    return Err("autodisable add | remove | list | clear".into());
                }
            }
            "say" => {
                let message = r.greedy();
                if message.is_empty() {
                    return Err("say <message>".into());
                }
                if !host
                    .client_state()
                    .is_some_and(|c| c.world.is_some() && c.logged_in && c.eye_position.is_some())
                {
                    return Err(tr(
                        "A player in a world is required",
                        "需要玩家进入世界",
                        "需要玩家進入世界",
                    )
                    .into());
                }
                host.client_commands_say(&message);
            }
            "help" => {
                let p = page(r)?;
                let prefix = self.client_command_prefix();
                list(
                    host,
                    tr("Commands", "客户端命令", "用戶端指令"),
                    COMMANDS
                        .iter()
                        .map(|n| format!("{prefix}{n}{}", if *n == "toggle" { ", t" } else { "" }))
                        .collect(),
                    p,
                );
            }
            _ => {
                let count = self
                    .state()
                    .behaviors
                    .borrow()
                    .client_commands
                    .hint_count
                    .max(0) as usize;
                let mut hints = COMMANDS;
                hints.sort_by_key(|n| {
                    if *n == "toggle" {
                        distance(root, n).min(distance(root, "t"))
                    } else {
                        distance(root, n)
                    }
                });
                let hints = hints.into_iter().take(count).collect::<Vec<_>>().join(", ");
                return Err(format!(
                    "{}: {root}{}",
                    tr(
                        "Unknown or not yet ported client command",
                        "未知或尚未移植的客户端命令",
                        "未知或尚未移植的用戶端指令"
                    ),
                    if hints.is_empty() {
                        String::new()
                    } else {
                        format!(" ({hints})")
                    }
                ));
            }
        }
        Ok(())
    }
}

#[cfg(feature = "webview")]
mod settings {
    use super::*;
    use liquidbounce_interop::protocol::ModuleAccess;
    use serde_json::{Value, json};
    fn node<'a>(n: &'a Value, path: &[&str]) -> Option<&'a Value> {
        if path.is_empty() {
            return Some(n);
        }
        if let Some(c) = n["choices"].as_object() {
            let (_, n) = c.iter().find(|(k, _)| k.eq_ignore_ascii_case(path[0]))?;
            return node(n, &path[1..]);
        }
        node(
            n["value"].as_array()?.iter().find(|n| {
                n["name"]
                    .as_str()
                    .is_some_and(|v| v.eq_ignore_ascii_case(path[0]))
            })?,
            &path[1..],
        )
    }
    fn node_mut<'a>(n: &'a mut Value, path: &[&str]) -> Option<&'a mut Value> {
        if path.is_empty() {
            return Some(n);
        }
        if n["choices"].is_object() {
            let c = n["choices"].as_object_mut()?;
            let key = c.keys().find(|k| k.eq_ignore_ascii_case(path[0]))?.clone();
            return node_mut(c.get_mut(&key)?, &path[1..]);
        }
        node_mut(
            n["value"].as_array_mut()?.iter_mut().find(|n| {
                n["name"]
                    .as_str()
                    .is_some_and(|v| v.eq_ignore_ascii_case(path[0]))
            })?,
            &path[1..],
        )
    }
    fn group(n: &Value) -> bool {
        matches!(
            n["valueType"].as_str(),
            Some("CONFIGURABLE" | "TOGGLEABLE" | "CHOICE")
        )
    }
    /// Apply only the selected value/group. Replaying sibling leaves would
    /// invoke unrelated onChanged listeners and mutate live list iterators.
    fn partial(n: &Value, path: &[&str]) -> Value {
        if path.is_empty() {
            return n.clone();
        }
        let mut out = n.clone();
        if let Some(choices) = out["choices"].as_object_mut() {
            let key = choices
                .keys()
                .find(|k| k.eq_ignore_ascii_case(path[0]))
                .unwrap()
                .clone();
            let child = partial(&choices[&key], &path[1..]);
            choices.clear();
            choices.insert(key, child);
        } else {
            let child = n["value"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| {
                    v["name"]
                        .as_str()
                        .is_some_and(|v| v.eq_ignore_ascii_case(path[0]))
                })
                .unwrap();
            out["value"] = json!([partial(child, &path[1..])]);
        }
        out
    }
    fn restore(n: &mut Value, default: &Value, all: bool) {
        if n["name"]
            .as_str()
            .is_some_and(|n| n.eq_ignore_ascii_case("Bind"))
            && all
        {
            return;
        }
        if !all {
            if n["valueType"] == "CHOICE" {
                n["active"] = default["active"].clone();
            } else {
                *n = default.clone();
            }
            return;
        }
        if let Some(a) = n["choices"].as_object_mut() {
            if let Some(b) = default["choices"].as_object() {
                for (name, a) in a {
                    if let Some(b) = b.get(name) {
                        restore(a, b, true);
                    }
                }
            }
            n["active"] = default["active"].clone();
        }
        if group(default) {
            if let (Some(a), Some(b)) = (n["value"].as_array_mut(), default["value"].as_array()) {
                for a in a {
                    if let Some(b) = b.iter().find(|b| b["name"] == a["name"]) {
                        restore(a, b, true);
                    }
                }
            }
            return;
        }
        *n = default.clone();
    }
    fn assign(n: &mut Value, text: &str, items: &[String]) -> Result<(), String> {
        let kind = n["valueType"].as_str().unwrap_or("INVALID");
        let v = match kind {
            "TEXT" | "FILE" => json!(text),
            "KEY" => json!(input_name(text).unwrap_or_else(|| "key.keyboard.unknown".into())),
            "BIND" => {
                let mut bind = n["value"].clone();
                bind["boundKey"] =
                    json!(input_name(text).unwrap_or_else(|| "key.keyboard.unknown".into()));
                bind
            }
            "BOOLEAN" => json!(match text.to_ascii_lowercase().as_str() {
                "true" | "on" | "yes" => true,
                "false" | "off" | "no" => false,
                _ =>
                    return Err(tr(
                        "Boolean must be true/on/yes or false/off/no",
                        "布尔值必须为 true/on/yes 或 false/off/no",
                        "布林值必須為 true/on/yes 或 false/off/no"
                    )
                    .into()),
            }),
            "INT" => json!(text.parse::<i32>().map_err(|e| e.to_string())?),
            "FLOAT" => json!(text.parse::<f32>().map_err(|e| e.to_string())?),
            "INT_RANGE" | "FLOAT_RANGE" => {
                let (a, b) = text.split_once("..").ok_or("Range requires min..max")?;
                if kind == "INT_RANGE" {
                    json!({"from":a.parse::<i32>().map_err(|e|e.to_string())?,"to":b.parse::<i32>().map_err(|e|e.to_string())?})
                } else {
                    json!({"from":a.parse::<f32>().map_err(|e|e.to_string())?,"to":b.parse::<f32>().map_err(|e|e.to_string())?})
                }
            }
            "CHOOSE" => n["choices"]
                .as_array()
                .and_then(|a| {
                    a.iter()
                        .find(|v| v.as_str().is_some_and(|v| v.eq_ignore_ascii_case(text)))
                })
                .ok_or("Unknown choice")?
                .clone(),
            "CHOICE" => {
                let tag = n["choices"]
                    .as_object()
                    .and_then(|a| a.keys().find(|v| v.as_str() == text))
                    .ok_or("Unknown mode")?
                    .clone();
                n["active"] = json!(tag);
                return Ok(());
            }
            "MULTI_CHOOSE" => {
                let choices = n["choices"].as_array().ok_or("Missing choices")?;
                let mut a = vec![];
                for text in text.split(',') {
                    a.push(
                        choices
                            .iter()
                            .find(|v| v.as_str().is_some_and(|v| v.eq_ignore_ascii_case(text)))
                            .ok_or("Unknown multi choice")?
                            .clone(),
                    );
                }
                json!(a)
            }
            "MUTABLE_LIST" | "LIST" => {
                if n["innerValueType"] != "TEXT" {
                    return Err(format!(
                        "Cannot deserialize list component {}",
                        n["innerValueType"]
                    ));
                }
                json!(text.split(',').collect::<Vec<_>>())
            }
            "REGISTRY_LIST" | "REGISTRY_MUTABLE_LIST" => {
                if n["registry"] != "item" {
                    return Err(format!("Registry is not yet connected: {}", n["registry"]));
                }
                let mut a = vec![];
                for text in text.split(',') {
                    let text = if text.contains(':') {
                        text.to_owned()
                    } else {
                        format!("minecraft:{text}")
                    };
                    if !items.contains(&text) {
                        return Err(format!("Unknown item: {text}"));
                    }
                    a.push(text);
                }
                json!(a)
            }
            _ => {
                return Err(format!(
                    "{} {kind}",
                    tr(
                        "Cannot deserialize value type",
                        "无法从文本解析此设置类型",
                        "無法從文字解析此設定類型"
                    )
                ));
            }
        };
        n["value"] = v;
        Ok(())
    }
    impl LiquidBounceHooks {
        pub(super) fn command_hide(&self, host: &mut dyn Host, r: &mut Reader) -> Result<(), String> {
            match r.required()?.to_ascii_lowercase().as_str() {
                op @ ("hide" | "unhide") => {
                    let text = required_string(r)?;
                    r.finish()?;
                    let names = self.command_modules(&text, |name| {
                        self.state().hidden.get(name).copied().unwrap_or(false) != (op == "hide")
                    })?;
                    let mut s = self.state();
                    for name in names {
                        s.hidden.insert(name, op == "hide");
                    }
                }
                "clear" => {
                    r.finish()?;
                    self.state().hidden.clear();
                }
                "list" => {
                    let p = page(r)?;
                    let s = self.state();
                    let rows = s
                        .registry
                        .modules()
                        .filter(|m| s.hidden.get(&m.definition().name).copied().unwrap_or(false))
                        .map(|m| m.definition().name.clone())
                        .collect();
                    list(
                        host,
                        tr("Hidden modules", "隐藏的模块", "隱藏的模組"),
                        rows,
                        p,
                    );
                    return Ok(());
                }
                _ => return Err("hide hide | unhide | clear | list".into()),
            }
            feedback(host, "hide");
            Ok(())
        }
        pub(super) fn command_value(
            &self,
            host: &mut dyn Host,
            r: &mut Reader,
        ) -> Result<(), String> {
            let op = r.required()?.to_ascii_lowercase();
            let path = r.required()?;
            let input = if op == "set" {
                Some(required_string(r)?)
            } else {
                None
            };
            r.finish()?;
            if !matches!(op.as_str(), "set" | "reset" | "reset-all") {
                return Err("value set | reset | reset-all".into());
            }
            let path = path
                .split_once('.')
                .filter(|(prefix, _)| prefix.eq_ignore_ascii_case("liquidbounce"))
                .map_or(path.as_str(), |(_, path)| path);
            let parts = path.split('.').collect::<Vec<_>>();
            if parts[0].eq_ignore_ascii_case("commandmanager") {
                let s = self.state();
                let mut b = s.behaviors.borrow_mut();
                let settings = &mut b.client_commands;
                match (
                    parts.get(1).map(|v| v.to_ascii_lowercase()).as_deref(),
                    op.as_str(),
                    parts.len(),
                ) {
                    (Some("prefix"), "set", 2) => settings.prefix = input.unwrap(),
                    (Some("hintcount"), "set", 2) => {
                        settings.hint_count =
                            input.unwrap().parse::<i32>().map_err(|e| e.to_string())?
                    }
                    (Some("prefix"), "reset", 2) => settings.prefix = ".".into(),
                    (Some("hintcount"), "reset", 2) => settings.hint_count = 5,
                    (None, "reset-all", 1) => *settings = Default::default(),
                    _ => return Err("Unknown CommandManager value path".into()),
                }
                drop(b);
                drop(s);
                feedback(host, path);
                return Ok(());
            }
            if !parts[0].eq_ignore_ascii_case("module") {
                return Err(tr("Unknown value path", "未知设置路径", "未知設定路徑").into());
            }
            let name = self.command_module(parts.get(1).ok_or("Missing module name")?)?;
            let tail = &parts[2..];
            let options = raw_options(host);
            let effects = {
                let mut s = self.state();
                let items = s.item_registry.clone();
                let enabled = s
                    .registry
                    .modules()
                    .find(|m| m.definition().name == name)
                    .unwrap()
                    .enabled();
                let mut a = crate::actor::Actor {
                    state: &mut s,
                    options,
                    entity_interaction_attribute: None,
                };
                let mut tree = a.settings(&name).ok_or("Missing actual module settings")?;
                tree["value"]
                    .as_array_mut()
                    .ok_or("Missing settings values")?
                    .push(json!({"name":"Enabled","valueType":"BOOLEAN","value":enabled}));
                let target = node_mut(&mut tree, tail).ok_or("Unknown value path")?;
                if op == "reset-all" {
                    if !group(target) {
                        return Err("reset-all requires a value group".into());
                    }
                } else if target["valueType"] == "CONFIGURABLE"
                    || (group(target) && target["valueType"] != "CHOICE")
                {
                    return Err("Path is not a configurable value".into());
                }
                if let Some(input) = input {
                    assign(target, &input, &items)?;
                } else {
                    let mut defaults = crate::runtime::State::default();
                    let enabled = defaults
                        .registry
                        .modules()
                        .find(|m| m.definition().name == name)
                        .unwrap()
                        .enabled();
                    let mut default = crate::actor::Actor {
                        state: &mut defaults,
                        options,
                        entity_interaction_attribute: None,
                    }
                    .settings(&name)
                    .ok_or("Missing default settings")?;
                    default["value"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"name":"Enabled","valueType":"BOOLEAN","value":enabled}));
                    restore(
                        target,
                        node(&default, tail).ok_or("Missing default value")?,
                        op == "reset-all",
                    );
                }
                let tree = partial(&tree, tail);
                let kind =
                    crate::config::ModuleKind::from_name(&name).ok_or("Unknown module kind")?;
                let check =
                    crate::config::parse_settings(kind, &tree).map_err(|e| e.to_string())?;
                if let Some(e) = check.issues.first() {
                    return Err(e.to_string());
                }
                if name == "Macros" {
                    a.state.behaviors.borrow_mut().macros.text_values = true;
                }
                let result = a.set_settings(&name, &tree);
                if name == "Macros" {
                    a.state.behaviors.borrow_mut().macros.text_values = false;
                }
                result?;
                s.take_effects()
            };
            crate::runtime::apply_effects(self, host, effects);
            feedback(host, path);
            Ok(())
        }
    }
}
