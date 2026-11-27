// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientBetterChatEvent;
use liquidbounce_behaviors::better_chat::{
    self, Settings, Translation, TranslationCache, TranslationSettings,
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
struct Reply {
    settings: TranslationSettings,
    result: Result<Translation, &'static str>,
}
#[derive(Default)]
pub struct BetterChatRuntime {
    pub settings: Settings,
    pub translation: TranslationSettings,
    cache: TranslationCache,
    pending: VecDeque<(TranslationSettings, String)>,
    workers: Vec<Receiver<Reply>>,
    last_provider: Option<(usize, String, String)>,
}
impl BetterChatRuntime {
    pub fn cancel(&mut self) {
        self.pending.clear();
        self.workers.clear();
    }
    pub fn provider_changed(&mut self) {
        let key = (
            self.translation.provider,
            self.translation.base_url.clone(),
            self.translation.api_key.clone(),
        );
        if self.last_provider.as_ref() != Some(&key) {
            self.cache.clear();
            self.last_provider = Some(key);
        }
    }
    fn translate(&mut self, text: String, now: i64) -> Option<Translation> {
        self.provider_changed();
        if let Some(value) = self.cache.get(&self.translation, &text, now) {
            return Some(value);
        }
        self.pending.push_back((self.translation.clone(), text));
        None
    }
    fn poll(&mut self, now: i64) -> Vec<Translation> {
        self.provider_changed();
        let mut out = Vec::new();
        let mut i = 0;
        while i < self.workers.len() {
            match self.workers[i].try_recv() {
                Ok(reply) => {
                    self.workers.remove(i);
                    if let Ok(result) = reply.result {
                        self.cache.put(reply.settings, result.clone(), now);
                        out.push(result);
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.workers.remove(i);
                }
                Err(mpsc::TryRecvError::Empty) => i += 1,
            }
        }
        while self.workers.len() < 4 {
            let Some((settings, text)) = self.pending.pop_front() else {
                break;
            };
            let (tx, rx) = mpsc::channel();
            if std::thread::Builder::new()
                .name("LiquidBounce-ChatTranslate".into())
                .spawn(move || {
                    let result = request(&settings, &text);
                    let _ = tx.send(Reply { settings, result });
                })
                .is_ok()
            {
                self.workers.push(rx);
            }
        }
        out
    }
}
pub(crate) fn request(s: &TranslationSettings, text: &str) -> Result<Translation, &'static str> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(20))
        .timeout_write(Duration::from_secs(20))
        .build();
    let response = if s.provider == 0 {
        let mut endpoint =
            url::Url::parse("https://translate.googleapis.com/translate_a/t?client=gtx&dt=t")
                .unwrap();
        endpoint
            .query_pairs_mut()
            .append_pair("sl", "auto")
            .append_pair("tl", &s.language_code)
            .append_pair("q", text);
        agent.get(endpoint.as_str()).call()
    } else {
        let endpoint = format!("{}/translate", s.base_url.trim().trim_end_matches('/'));
        let url = url::Url::parse(&endpoint).map_err(|_| "Invalid translator URL")?;
        agent
            .post(url.as_str())
            .set("Content-Type", "application/json")
            .send_string(&better_chat::libre_body(s, text).to_string())
    }
    .map_err(|_| "Translation request failed")?;
    let body = response
        .into_string()
        .map_err(|_| "Could not read translation response")?;
    let value = serde_json::from_str(&body).map_err(|_| "Could not decode translation response")?;
    better_chat::translation_response(s, text, &value)
}
fn copy_text() -> (&'static str, &'static str) {
    match horizon_client::lang::code()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "zh_cn" | "zh_sg" | "zh_hans" => ("聊天复制", "已复制这条消息"),
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => ("聊天複製", "已複製這則訊息"),
        _ => ("ChatCopy", "The line is copied"),
    }
}
impl LiquidBounceHooks {
    pub(crate) fn better_chat(&self, event: &mut ClientBetterChatEvent<'_>) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let running = b.query_running(&state.registry, "BetterChat");
        let prefix = b.client_commands.prefix.clone();
        if !running {
            b.better_chat.cancel();
            *event.output = json!({"running":false}).to_string();
            return;
        }
        let now = crate::behaviors::current_time_millis();
        let r = &mut b.better_chat;
        let s = &r.settings;
        let mut translations = Vec::new();
        let response = match input["action"].as_str().unwrap_or("") {
            "policy" => {
                json!({"running":true,"infinite":s.feature(1),"antiClear":s.feature(2),"keepAfterDeath":s.feature(4),"copy":s.copy,"notify":s.notify,"highlight":s.highlight,"prefix":prefix})
            }
            "outgoing" => json!({"message":s.outgoing(input["message"].as_str().unwrap_or(""))}),
            "receive" => {
                let kind = input["kind"].as_u64().unwrap_or(0) as usize;
                let plain = input["plain"].as_str().unwrap_or("");
                let decorated = input["message"].as_str().unwrap_or(plain);
                let filtered = s.filtered(plain);
                let id = (!filtered).then(|| s.stack_id(kind, plain)).flatten();
                let text = s.translation_text(kind, decorated);
                if let Some(text) = text {
                    if let Some(result) = r.translate(text, now) {
                        translations.push(result);
                    }
                }
                json!({"cancel":filtered,"stack":id})
            }
            "poll" => {
                translations = r.poll(now);
                json!({"running":true})
            }
            "copied" => {
                #[cfg(feature = "webview")]
                if s.copy && s.notify {
                    let (title, message) = copy_text();
                    state.events.borrow_mut().push((
                        "notification".into(),
                        json!({"title":title,"message":message,"severity":"SUCCESS"}),
                    ));
                }
                json!({})
            }
            _ => json!({}),
        };
        let mut response = response;
        response["translations"] = json!(
            translations
                .into_iter()
                .filter(Translation::valid)
                .map(|r| json!({"text":r.display(),"copy":r.translation}))
                .collect::<Vec<_>>()
        );
        *event.output = response.to_string();
    }
}
