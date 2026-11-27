// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoChatGame, pinned to LiquidBounce ae0eb396.
pub const PROTOCOLS: [&str; 2] = ["OpenAiCompatible", "Anthropic"];
include!("auto_chat_game_prompt.rs");
#[derive(Clone, PartialEq)]
pub struct Settings {
    pub protocol: u8,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub reaction_time: [i32; 2],
    pub cooldown: i32,
    pub buffer_time: i32,
    pub trigger_sentence: String,
    pub include_trigger: bool,
    pub server_name: String,
    pub answer_template: String,
    pub prompt: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            protocol: 0,
            base_url: String::new(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
            reaction_time: [1000, 5000],
            cooldown: 2,
            buffer_time: 200,
            trigger_sentence: "Chat Game".into(),
            include_trigger: true,
            server_name: "Minecraft".into(),
            answer_template: "%s".into(),
            prompt: DEFAULT_PROMPT.into(),
        }
    }
}
// Kotlin Char.isWhitespace also includes Unicode space separators, but Java's
// default Regex("\\s+") only collapses these six ASCII whitespace characters.
pub fn is_blank(s: &str) -> bool {
    s.chars().all(kotlin_whitespace)
}
fn kotlin_whitespace(c: char) -> bool {
    matches!(c,'\t'..='\r'|'\u{001c}'..='\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}')
}
pub fn normalize_question(lines: &[String]) -> String {
    let joined = lines.join(" ");
    let mut out = String::new();
    let mut whitespace = false;
    for c in joined.chars() {
        if matches!(c, ' ' | '\t' | '\n' | '\u{000b}' | '\u{000c}' | '\r') {
            if !whitespace {
                out.push(' ');
            }
            whitespace = true;
        } else {
            out.push(c);
            whitespace = false;
        }
    }
    out.trim_matches(kotlin_whitespace).into()
}
#[derive(Default)]
pub struct State {
    pub settings: Settings,
    pub buffer: Vec<String>,
    pub trigger_at: i64,
    pub cooldown_at: i64,
    pub gg: Vec<i64>,
    pub answer: Option<(i64, String)>,
    pub asking: bool,
}
impl State {
    pub fn message(&mut self, now: i64, message: &str, mut random: impl FnMut(i32, i32) -> i32) {
        if message.contains("Show some love by typing") {
            self.gg.push(now.wrapping_add(i64::from(random(
                self.settings.reaction_time[0],
                self.settings.reaction_time[1],
            ))));
            return;
        }
        if self
            .cooldown_at
            .wrapping_add(i64::from(self.settings.cooldown) * 60000)
            < now
        {
            if message.contains(&self.settings.trigger_sentence) {
                self.trigger_at = now;
                self.buffer.clear();
                if !self.settings.include_trigger {
                    return;
                }
            }
            if self
                .trigger_at
                .wrapping_add(i64::from(self.settings.buffer_time))
                >= now
            {
                self.buffer.push(message.into());
            }
        } else {
            self.buffer.clear();
        }
    }
    pub fn question(&mut self, now: i64) -> Option<String> {
        if self.asking
            || self.answer.is_some()
            || self.buffer.is_empty()
            || self
                .trigger_at
                .wrapping_add(i64::from(self.settings.buffer_time))
                >= now
        {
            return None;
        }
        let question = normalize_question(&self.buffer);
        self.buffer.clear();
        self.cooldown_at = now;
        self.asking = true;
        Some(question)
    }
    pub fn cancel(&mut self) {
        self.gg.clear();
        self.answer = None;
        self.asking = false;
    }
    pub fn due_gg(&mut self, now: i64) -> usize {
        let mut count = 0;
        self.gg.retain(|due| {
            if *due <= now {
                count += 1;
                false
            } else {
                true
            }
        });
        count
    }
}
