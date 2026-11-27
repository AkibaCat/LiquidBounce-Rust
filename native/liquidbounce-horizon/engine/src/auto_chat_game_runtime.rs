// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    behaviors::current_time_millis as now,
    runtime::{LiquidBounceHooks, raw_options},
};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    auto_chat_game::{self, Settings, State},
    auto_chat_game_wire as wire,
    random::random_i32_inclusive,
};
use std::{
    sync::mpsc::{self, Receiver},
    time::Duration,
};
struct Reply {
    result: Result<String, String>,
    started: i64,
}
#[derive(Default)]
pub struct AutoChatGameRuntime {
    pub state: State,
    worker: Option<Receiver<Reply>>,
}
impl AutoChatGameRuntime {
    pub fn cancel(&mut self) {
        self.worker = None;
        self.state.cancel();
    }
    fn start(&mut self, question: String) -> Result<(), String> {
        let settings = self.state.settings.clone();
        let started = now();
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("LiquidBounce-AutoChatGame".into())
            .spawn(move || {
                let result =
                    request(&settings, &question).map_err(|e| redact(e, &settings.api_key));
                let _ = tx.send(Reply { result, started });
            })
            .map_err(|_| "Could not start AutoChatGame request worker".to_owned())?;
        self.worker = Some(rx);
        Ok(())
    }
}
fn redact(message: String, key: &str) -> String {
    if key.is_empty() {
        message
    } else {
        message.replace(key, "[redacted]")
    }
}
pub(crate) fn request(s: &Settings, question: &str) -> Result<String, String> {
    let base = if auto_chat_game::is_blank(&s.base_url) {
        if s.protocol == 1 {
            "https://api.anthropic.com/v1/"
        } else {
            "https://api.openai.com/v1/"
        }
    } else {
        &s.base_url
    };
    let root = if base.ends_with('/') {
        base.to_owned()
    } else {
        format!("{base}/")
    };
    let root = url::Url::parse(&root).map_err(|_| "Invalid API base URL".to_owned())?;
    let endpoint = root
        .join(if s.protocol == 1 {
            "messages"
        } else {
            "chat/completions"
        })
        .map_err(|_| "Invalid API endpoint".to_owned())?;
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(20))
        .timeout_write(Duration::from_secs(20))
        .build();
    let mut request = agent
        .post(endpoint.as_str())
        .set("Content-Type", "application/json");
    if s.protocol == 1 {
        request = request
            .set("x-api-key", &s.api_key)
            .set("anthropic-version", "2023-06-01");
    } else if !auto_chat_game::is_blank(&s.api_key) {
        request = request.set("Authorization", &format!("Bearer {}", s.api_key));
    }
    let response = match request.send_string(&wire::body(s, question).to_string()) {
        Ok(r) => r,
        Err(ureq::Error::Status(_, r)) => {
            return Err(wire::provider_error(
                s.protocol,
                &r.into_string().unwrap_or_default(),
            ));
        }
        // Transport errors may include the configured URL. Never log a URL or
        // configuration credential in a worker failure.
        Err(ureq::Error::Transport(_)) => return Err("AI request transport failed".into()),
    };
    let text = response
        .into_string()
        .map_err(|_| "Could not read AI answer".to_owned())?;
    let json = serde_json::from_str(&text).map_err(|_| "Could not decode AI answer".to_owned())?;
    wire::answer(s.protocol, &json)
}
fn language() -> u8 {
    match horizon_client::lang::code()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "zh_cn" | "zh_sg" | "zh_hans" => 1,
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => 2,
        _ => 0,
    }
}
fn text(english: &str, cn: &str, tw: &str) -> String {
    match language() {
        1 => cn,
        2 => tw,
        _ => english,
    }
    .into()
}
impl LiquidBounceHooks {
    pub(crate) fn auto_chat_game_enabled(&self, host: &mut dyn Host) {
        if !auto_chat_game::is_blank(
            &self
                .state()
                .behaviors
                .borrow()
                .auto_chat_game
                .state
                .settings
                .api_key,
        ) {
            return;
        }
        host.chat(&text(
            "§cPlease enter your API key in the module settings.",
            "§c请在模块设置中输入 API 密钥。",
            "§c請在模組設定中輸入 API 金鑰。",
        ));
        let options = raw_options(host);
        let effects = {
            let mut state = self.state();
            let id = state.behaviors.borrow().query_ids["AutoChatGame"];
            let _ = state.with_registry(options, |r, c| r.set_enabled_sync(id, false, c));
            state.take_effects()
        };
        crate::runtime::apply_effects(self, host, effects);
    }
    pub(crate) fn auto_chat_game_message(&self, message: &str) {
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        if b.query_running(&state.registry, "AutoChatGame") {
            b.auto_chat_game
                .state
                .message(now(), message, random_i32_inclusive);
        }
    }
    pub(crate) fn auto_chat_game_tick(&self, host: &mut dyn Host) {
        self.auto_chat_game_poll(host, true);
    }
    pub(crate) fn auto_chat_game_schedule(&self, host: &dyn Host) {
        let state = self.state();
        let b = state.behaviors.borrow();
        let r = &b.auto_chat_game;
        let deadline = if b.query_running(&state.registry, "AutoChatGame") {
            r.state
                .gg
                .iter()
                .copied()
                .chain(r.state.answer.as_ref().map(|(due, _)| *due))
                .chain(r.worker.is_some().then(|| now().saturating_add(10)))
                .min()
        } else {
            None
        };
        host.client_auto_chat_game_schedule(deadline);
    }
    pub(crate) fn auto_chat_game_poll(&self, host: &mut dyn Host, game_tick: bool) {
        let mut local = Vec::new();
        let mut outgoing = Vec::new();
        {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            if !b.query_running(&state.registry, "AutoChatGame") {
                b.auto_chat_game.cancel();
                host.client_auto_chat_game_schedule(None);
                return;
            }
            let r = &mut b.auto_chat_game;
            let time = now();
            for _ in 0..r.state.due_gg(time) {
                outgoing.push("gg".to_owned());
            }
            if r.state.answer.as_ref().is_some_and(|(due, _)| *due <= time) {
                let (_, answer) = r.state.answer.take().unwrap();
                match liquidbounce_behaviors::report_helper_format::format_command(
                    &r.state.settings.answer_template,
                    &answer,
                ) {
                    Ok(s) => outgoing.push(s),
                    Err(e) => eprintln!("[LiquidBounce] AutoChatGame format failed: {e}"),
                }
            }
            let reply = r.worker.as_ref().and_then(|rx| rx.try_recv().ok());
            if let Some(reply) = reply {
                r.worker = None;
                r.state.asking = false;
                match reply.result {
                    Ok(answer) => {
                        let elapsed = time.wrapping_sub(reply.started);
                        let delay = random_i32_inclusive(
                            r.state.settings.reaction_time[0],
                            r.state.settings.reaction_time[1],
                        );
                        local.push(match language() {
                            1 => format!("§a答案：{answer}，耗时 {elapsed} 毫秒。"),
                            2 => format!("§a答案：{answer}，耗時 {elapsed} 毫秒。"),
                            _ => format!("§aAnswer: {answer}, took {elapsed}ms."),
                        });
                        local.push(match language() {
                            1 => format!("§a准备回答：{answer}，等待 {delay} 毫秒。"),
                            2 => format!("§a準備回答：{answer}，等待 {delay} 毫秒。"),
                            _ => format!("§aAnswering question: {answer}, waiting for {delay}ms."),
                        });
                        r.state.answer = Some((time.wrapping_add(i64::from(delay)), answer));
                    }
                    Err(error) => {
                        eprintln!("[LiquidBounce] GPT AutoChatGame failed: {error}");
                        local.push(format!(
                            "{}{error}",
                            text(
                                "§cFailed to answer question: ",
                                "§c回答问题失败：",
                                "§c回答問題失敗："
                            )
                        ));
                    }
                }
            }
            if let Some(question) = game_tick.then(|| r.state.question(time)).flatten() {
                local.push(format!(
                    "{}{question}",
                    text(
                        "§aUnderstood question: ",
                        "§a已识别问题：",
                        "§a已識別問題："
                    )
                ));
                if let Err(error) = r.start(question) {
                    r.state.asking = false;
                    local.push(error);
                }
            }
        }
        for message in local {
            host.chat(&message);
        }
        for message in outgoing {
            host.client_spammer_send(&message);
        }
        self.auto_chat_game_schedule(host);
    }
}
