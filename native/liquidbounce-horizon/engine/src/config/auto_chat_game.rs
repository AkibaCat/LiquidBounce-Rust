// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::auto_chat_game::{PROTOCOLS, Settings};
#[derive(Clone, Debug, PartialEq)]
pub struct AutoChatGameSetting {
    pub name: String,
    pub value: Value,
}
impl AutoChatGameSetting {
    pub fn apply(self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.name.as_str();
        let v = &self.value;
        let text = || scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected text"));
        match p {
            "Protocol" => {
                let tag = text()?;
                s.protocol = PROTOCOLS
                    .iter()
                    .position(|s| s.eq_ignore_ascii_case(&tag))
                    .ok_or_else(|| ConfigError::new(p, "unknown AI protocol"))?
                    as u8;
            }
            "BaseUrl" => s.base_url = text()?,
            "ApiKey" => s.api_key = text()?,
            "Model" => s.model = text()?,
            "ReactionTime" => {
                let pair = parse_int_pair(v, p)?;
                if pair[0] > pair[1] || pair[0] < 0 || pair[1] > 10000 {
                    return Err(ConfigError::new(
                        p,
                        "expected ordered range from 0 to 10000",
                    ));
                }
                s.reaction_time = pair;
            }
            "Cooldown" => {
                let n = parse_int(v, p)?;
                if !(0..=60).contains(&n) {
                    return Err(ConfigError::new(p, "expected 0 to 60"));
                }
                s.cooldown = n;
            }
            "BufferTime" => {
                let n = parse_int(v, p)?;
                if !(0..=500).contains(&n) {
                    return Err(ConfigError::new(p, "expected 0 to 500"));
                }
                s.buffer_time = n;
            }
            "TriggerSentence" => s.trigger_sentence = text()?,
            "IncludeTrigger" => s.include_trigger = parse_bool(v, p)?,
            "ServerName" => s.server_name = text()?,
            "AnswerTemplate" => s.answer_template = text()?,
            "Prompt" => s.prompt = text()?,
            _ => return Err(ConfigError::new(p, "unknown AutoChatGame setting")),
        }
        Ok(())
    }
}
pub fn serialize_auto_chat_game(
    s: &Settings,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::AutoChatGame, common);
    let key = |name: &str| {
        format!(
            "liquidbounce.module.autoChatGame.{}{}",
            name[..1].to_ascii_lowercase(),
            &name[1..]
        )
    };
    let mut protocol = t.leaf(
        "Protocol",
        "CHOOSE",
        json!(PROTOCOLS[usize::from(s.protocol).min(1)]),
        &key("Protocol"),
        None,
    );
    if format == JsonFormat::Interop {
        protocol["choices"] = json!(PROTOCOLS);
    }
    values.push(protocol);
    for (name, value) in [
        ("BaseUrl", &s.base_url),
        ("ApiKey", &s.api_key),
        ("Model", &s.model),
    ] {
        values.push(t.leaf(name, "TEXT", json!(value), &key(name), None));
    }
    values.push(t.ranged(
        "ReactionTime",
        "INT_RANGE",
        json!({"from":s.reaction_time[0],"to":s.reaction_time[1]}),
        json!({"from":0,"to":10000}),
        "ms",
        &key("ReactionTime"),
    ));
    values.push(t.ranged(
        "Cooldown",
        "INT",
        json!(s.cooldown),
        json!({"from":0,"to":60}),
        "minutes",
        &key("Cooldown"),
    ));
    values.push(t.ranged(
        "BufferTime",
        "INT",
        json!(s.buffer_time),
        json!({"from":0,"to":500}),
        "ms",
        &key("BufferTime"),
    ));
    values.push(t.leaf(
        "TriggerSentence",
        "TEXT",
        json!(s.trigger_sentence),
        &key("TriggerSentence"),
        None,
    ));
    values.push(t.leaf(
        "IncludeTrigger",
        "BOOLEAN",
        json!(s.include_trigger),
        &key("IncludeTrigger"),
        None,
    ));
    for (name, value) in [
        ("ServerName", &s.server_name),
        ("AnswerTemplate", &s.answer_template),
        ("Prompt", &s.prompt),
    ] {
        values.push(t.leaf(name, "TEXT", json!(value), &key(name), None));
    }
    Ok(t.group("AutoChatGame", "TOGGLEABLE", values))
}
pub(super) fn parse_auto_chat_game(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in [
        "Protocol",
        "BaseUrl",
        "ApiKey",
        "Model",
        "ReactionTime",
        "Cooldown",
        "BufferTime",
        "TriggerSentence",
        "IncludeTrigger",
        "ServerName",
        "AnswerTemplate",
        "Prompt",
    ] {
        leaf(
            patch,
            root,
            name,
            &format!("AutoChatGame.{name}"),
            |v, _| {
                let setting = AutoChatGameSetting {
                    name: name.into(),
                    value: v.clone(),
                };
                setting.clone().apply(&mut Settings::default())?;
                Ok(SettingChange::AutoChatGame(setting))
            },
        );
    }
}
