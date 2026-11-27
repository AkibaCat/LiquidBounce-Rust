// SPDX-License-Identifier: GPL-3.0-or-later
//! The two wire formats from the pinned AiChatClient, without network effects.
use crate::auto_chat_game::{PROTOCOLS, Settings};
use serde_json::{Value, json};
pub fn body(s: &Settings, question: &str) -> Value {
    let prompt = s.prompt.replace("{SERVER_NAME}", &s.server_name);
    if s.protocol == 1 {
        json!({"model":s.model,"max_tokens":1024,"system":prompt,"messages":[{"role":"user","content":question}]})
    } else {
        json!({"model":s.model,"messages":[{"role":"system","content":prompt},{"role":"user","content":question}]})
    }
}
pub fn answer(protocol: u8, value: &Value) -> Result<String, String> {
    let result = if protocol == 1 {
        value["content"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter(|v| v["type"] == "text")
                    .filter_map(|v| v["text"].as_str())
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty())
    } else {
        value["choices"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v["message"]["content"].as_str())
            .map(str::to_owned)
    };
    result
        .map(|s| s.trim_end_matches('.').to_owned())
        .ok_or_else(|| {
            format!(
                "{} returned no answer",
                PROTOCOLS[usize::from(protocol).min(1)]
            )
        })
}
pub fn provider_error(protocol: u8, content: &str) -> String {
    let parsed = serde_json::from_str::<Value>(content).ok();
    let text = parsed
        .as_ref()
        .and_then(|v| {
            v["error"]["message"]
                .as_str()
                .or_else(|| v["error"].as_str())
        })
        .filter(|s| !super::auto_chat_game::is_blank(s));
    let fallback = || {
        let mut units = 0;
        content
            .chars()
            .take_while(|c| {
                units += c.len_utf16();
                units <= 200
            })
            .collect::<String>()
    };
    format!(
        "{} returned an error: {}",
        PROTOCOLS[usize::from(protocol).min(1)],
        text.map(str::to_owned).unwrap_or_else(fallback)
    )
}
