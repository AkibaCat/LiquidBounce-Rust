// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{client_commands::Reader, xray::Settings};
use serde_json::{Value, json};
#[derive(Default)]
pub struct XRayRuntime {
    pub settings: Settings,
    pub last: Option<String>,
}
fn tr<'a>(en: &'a str, cn: &'a str, tw: &'a str) -> &'a str {
    use liquidbounce_interop::localization::Locale;
    match Locale::from_game_language(&horizon_client::lang::code()) {
        Locale::SimplifiedChinese => cn,
        Locale::TraditionalChinese => tw,
        _ => en,
    }
}
impl LiquidBounceHooks {
    pub(crate) fn xray_frame(&self, host: &mut dyn Host) {
        let text = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let s = &b.xray.settings;
            json!({"active":b.query_running(&state.registry,"XRay"),"fullBright":s.full_bright,"exposed":s.exposed_only,"opacity":s.background_opacity,"blocks":s.blocks}).to_string()
        };
        if self.state().behaviors.borrow().xray.last.as_ref() == Some(&text) {
            return;
        }
        host.client_xray_policy(&text);
        self.state().behaviors.borrow_mut().xray.last = Some(text);
    }
    pub(crate) fn xray_command(&self, host: &mut dyn Host, r: &mut Reader) -> Result<(), String> {
        let action = r.required()?.to_ascii_lowercase();
        let blocks: std::collections::BTreeMap<String, String> =
            serde_json::from_str(&host.client_xray_blocks()).map_err(|_| {
                tr(
                    "Block registry unavailable",
                    "方块注册表不可用",
                    "方塊登錄表無法使用",
                )
                .to_owned()
            })?;
        if action == "list" {
            let p = r
                .word()
                .map(|s| s.parse::<usize>())
                .transpose()
                .map_err(|e| e.to_string())?
                .unwrap_or(1);
            r.finish()?;
            if p == 0 {
                return Err(tr(
                    "Page must be at least 1",
                    "页码必须至少为 1",
                    "頁碼必須至少為 1",
                )
                .into());
            }
            let mut names = self
                .state()
                .behaviors
                .borrow()
                .xray
                .settings
                .blocks
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            names.sort_by_key(|id| {
                let (n, p) = id.split_once(':').unwrap_or(("minecraft", id));
                format!("block.{n}.{p}")
            });
            let count = names.len().div_ceil(8).max(1);
            let p = p.min(count);
            host.chat(tr("XRay blocks", "透视方块列表", "透視方塊清單"));
            for id in names.into_iter().skip((p - 1) * 8).take(8) {
                host.chat(&format!("⬥ {} ({id})", blocks.get(&id).unwrap_or(&id)));
            }
            if count > 1 {
                host.chat(&format!("{p} / {count}"));
            }
            return Ok(());
        }
        let message = match action.as_str() {
            "add" | "remove" => {
                let name = r.required()?;
                r.finish()?;
                let id = if name.contains(':') {
                    name
                } else {
                    format!("minecraft:{name}")
                };
                let display = blocks.get(&id).ok_or_else(|| {
                    format!("{}: {id}", tr("Unknown block", "未知方块", "未知方塊"))
                })?;
                let changed = {
                    let state = self.state();
                    let mut b = state.behaviors.borrow_mut();
                    if action == "add" {
                        b.xray.settings.blocks.insert(id.clone())
                    } else {
                        b.xray.settings.blocks.remove(&id)
                    }
                };
                if !changed {
                    return Err(format!(
                        "{}: {display}",
                        if action == "add" {
                            tr(
                                "Block is already present",
                                "方块已在列表中",
                                "方塊已在清單中",
                            )
                        } else {
                            tr(
                                "Block is not in the list",
                                "列表中没有此方块",
                                "清單中沒有此方塊",
                            )
                        }
                    ));
                }
                format!(
                    "{}: {display}",
                    if action == "add" {
                        tr("Block added", "已添加方块", "已新增方塊")
                    } else {
                        tr("Block removed", "已移除方块", "已移除方塊")
                    }
                )
            }
            "clear" => {
                r.finish()?;
                self.state()
                    .behaviors
                    .borrow_mut()
                    .xray
                    .settings
                    .blocks
                    .clear();
                tr("Blocks cleared", "已清空方块列表", "已清空方塊清單").into()
            }
            "reset" => {
                r.finish()?;
                self.state()
                    .behaviors
                    .borrow_mut()
                    .xray
                    .settings
                    .reset_blocks();
                tr("Blocks reset", "已重置方块列表", "已重設方塊清單").into()
            }
            _ => return Err("xray add/remove <block> | list [page] | clear | reset".into()),
        };
        host.chat(&message);
        self.xray_frame(host);
        Ok(())
    }
}
