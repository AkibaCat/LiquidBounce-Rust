// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game, Screen};
use horizon_client::{
    chat::ChatSource,
    hud::HudBatch,
    text::{ClickEvent, Style, Text},
};
use serde_json::{Value, json};
use winit::{event::MouseButton, keyboard::KeyCode};
impl Game<'_> {
    pub(super) fn client_better_chat_query(&self, input: Value) -> Value {
        let mut output = String::new();
        self.native_client_better_chat(&input.to_string(), &mut output);
        serde_json::from_str(&output).unwrap_or(Value::Null)
    }
    pub(super) fn client_better_chat_policy(&mut self) -> Value {
        let p = self.client_better_chat_query(json!({"action":"policy"}));
        let running = p["running"] == true;
        self.ui.chat.liquidbounce_policy(
            running && p["infinite"] == true,
            running && p["antiClear"] == true,
        );
        p
    }
    fn client_better_chat_translations(&mut self, result: &Value) {
        for translation in result["translations"].as_array().into_iter().flatten() {
            let Some(text) = translation["text"].as_str() else {
                continue;
            };
            let copy = translation["copy"].as_str().unwrap_or(text);
            let line = Text::literal(text).with_style(&Style {
                click: Some(ClickEvent::CopyToClipboard(copy.into())),
                ..Style::default()
            });
            let focused = self.screen == Screen::Chat;
            self.ui.chat.add(
                &self.gui.atlas,
                line,
                None,
                ChatSource::Client,
                self.visual_ticks(),
                focused,
            );
        }
    }
    pub(super) fn client_better_chat_tick(&mut self) {
        if !self.native_window() {
            return;
        }
        self.client_better_chat_policy();
        let result = self.client_better_chat_query(json!({"action":"poll"}));
        self.client_better_chat_translations(&result);
    }
    pub(super) fn client_better_chat_outgoing(&self, message: &str) -> String {
        if message.starts_with('/') {
            return message.into();
        }
        self.client_better_chat_query(json!({"action":"outgoing","message":message}))["message"]
            .as_str()
            .unwrap_or(message)
            .into()
    }
    /// Receives the undecorated component separately, as ChatReceiveEvent.textData does.
    pub(super) fn client_better_chat_receive(
        &mut self,
        kind: usize,
        plain: &str,
        line: &Text,
        source: ChatSource,
    ) -> bool {
        self.client_better_chat_policy();
        let stripped = Text::literal(plain).strip_legacy().plain();
        let result = self.client_better_chat_query(
            json!({"action":"receive","kind":kind,"plain":stripped,"message":plain}),
        );
        self.client_better_chat_translations(&result);
        if result["cancel"] == true {
            return false;
        }
        if let Some(id) = result["stack"].as_str() {
            let focused = self.screen == Screen::Chat;
            self.ui.chat.liquidbounce_add_stack(
                &self.gui.atlas,
                line.clone(),
                source,
                self.visual_ticks(),
                focused,
                id.into(),
            );
            return false;
        }
        true
    }
    pub(super) fn client_better_chat_copy(&mut self, button: MouseButton) {
        if self.screen != Screen::Chat {
            return;
        }
        let p = self.client_better_chat_policy();
        if p["running"] != true || p["copy"] != true {
            return;
        }
        let height = self.renderer.as_ref().map_or(720., |r| {
            HudBatch::new([r.config.width, r.config.height]).height
        });
        let Some(content) = self.ui.chat.liquidbounce_copy_at(
            height as i32,
            self.cursor.0 as f64,
            self.cursor.1 as f64,
        ) else {
            return;
        };
        if button == MouseButton::Left && self.has_shift_down() {
            use horizon_client::clipboard::Clipboard;
            self.clipboard.set(&content);
            if p["notify"] == true {
                let _ = self.client_better_chat_query(json!({"action":"copied"}));
                let message = match horizon_client::lang::code()
                    .to_ascii_lowercase()
                    .replace('-', "_")
                    .as_str()
                {
                    "zh_cn" | "zh_sg" | "zh_hans" => "已复制这条消息",
                    "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => "已複製這則訊息",
                    _ => "The line is copied",
                };
                self.titles.set_overlay(Text::literal(message), false);
            }
        } else if button == MouseButton::Right {
            self.open_chat(&content, false);
        }
    }
    pub(super) fn client_better_chat_highlight(&mut self, batch: &mut HudBatch) {
        if self.screen != Screen::Chat {
            return;
        }
        let p = self.client_better_chat_policy();
        if p["running"] == true && p["copy"] == true && p["highlight"] == true {
            self.ui.chat.liquidbounce_highlight(
                batch,
                &self.gui.atlas,
                self.cursor.0 as f64,
                self.cursor.1 as f64,
            );
        }
    }
    pub(super) fn client_better_chat_death_key(
        &mut self,
        key: KeyCode,
        typed: Option<&str>,
    ) -> bool {
        if self.screen != Screen::Dead {
            return false;
        }
        let p = self.client_better_chat_policy();
        if p["running"] != true || p["keepAfterDeath"] != true {
            return false;
        }
        let prefix = p["prefix"].as_str().unwrap_or(".");
        let first = prefix.chars().next().map(String::from).unwrap_or_default();
        let initial = if horizon_client::keybinds::key_code(self.settings.keys.get("key.chat"))
            == Some(key)
        {
            Some(String::new())
        } else if horizon_client::keybinds::key_code(self.settings.keys.get("key.command"))
            == Some(key)
        {
            Some("/".into())
        } else if !first.is_empty() && typed.is_some_and(|t| t.starts_with(&first)) {
            Some(first)
        } else {
            None
        };
        if let Some(initial) = initial {
            self.open_chat(&initial, false);
            true
        } else {
            false
        }
    }
}
