// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::auto_account::{AutoAccount, Pattern, SOURCES};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AutoAccountSetting {
    Password(String),
    Delay([i32; 2]),
    RegisterCommand(String),
    LoginCommand(String),
    RegisterRegex(Pattern),
    LoginRegex(Pattern),
    RepeatPasswordOnRegister(bool),
    MessageSource(u32),
}
impl AutoAccountSetting {
    pub fn apply(self, state: &mut AutoAccount) {
        match self {
            Self::Password(v) => state.password = v,
            Self::Delay(v) => state.delay = v,
            Self::RegisterCommand(v) => state.register_command = v,
            Self::LoginCommand(v) => state.login_command = v,
            Self::RegisterRegex(v) => state.register_regex = v,
            Self::LoginRegex(v) => state.login_regex = v,
            Self::RepeatPasswordOnRegister(v) => state.repeat_password_on_register = v,
            Self::MessageSource(v) => state.message_sources = v,
        }
    }
}
pub fn serialize_auto_account(
    state: &AutoAccount,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::AutoAccount, common);
    let text = |name: &str, value: &str, key: &str| {
        tree.leaf(
            name,
            "TEXT",
            json!(value),
            &format!("liquidbounce.module.autoAccount.{key}"),
            None,
        )
    };
    values.extend([
        text("Password", &state.password, "password"),
        tree.ranged(
            "Delay",
            "INT_RANGE",
            json!({"from":state.delay[0],"to":state.delay[1]}),
            json!({"from":0,"to":50}),
            "ticks",
            "liquidbounce.module.autoAccount.delay",
        ),
        text(
            "RegisterCommand",
            &state.register_command,
            "registerCommand",
        ),
        text("LoginCommand", &state.login_command, "loginCommand"),
        text(
            "RegisterRegex",
            state.register_regex.source(),
            "registerRegex",
        ),
        text("LoginRegex", state.login_regex.source(), "loginRegex"),
        tree.leaf(
            "RepeatPasswordOnRegister",
            "BOOLEAN",
            json!(state.repeat_password_on_register),
            "liquidbounce.module.autoAccount.repeatPasswordOnRegister",
            None,
        ),
        tree.multi(
            "MessageSource",
            state.message_sources,
            &SOURCES,
            "liquidbounce.module.autoAccount.messageSource",
        ),
    ]);
    Ok(tree.group("AutoAccount", "TOGGLEABLE", values))
}
pub(super) fn parse_auto_account(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Password", "AutoAccount.Password", |v, p| {
        scalar_string(v)
            .map(|v| SettingChange::AutoAccount(AutoAccountSetting::Password(v)))
            .ok_or_else(|| ConfigError::new(p, "expected text"))
    });
    leaf(patch, root, "Delay", "AutoAccount.Delay", |v, p| {
        parse_int_pair(v, p).map(|v| SettingChange::AutoAccount(AutoAccountSetting::Delay(v)))
    });
    for (name, create) in [
        (
            "RegisterCommand",
            AutoAccountSetting::RegisterCommand as fn(String) -> AutoAccountSetting,
        ),
        ("LoginCommand", AutoAccountSetting::LoginCommand),
    ] {
        leaf(patch, root, name, &format!("AutoAccount.{name}"), |v, p| {
            scalar_string(v)
                .map(|v| SettingChange::AutoAccount(create(v)))
                .ok_or_else(|| ConfigError::new(p, "expected text"))
        });
    }
    for (name, create) in [
        (
            "RegisterRegex",
            AutoAccountSetting::RegisterRegex as fn(Pattern) -> AutoAccountSetting,
        ),
        ("LoginRegex", AutoAccountSetting::LoginRegex),
    ] {
        leaf(patch, root, name, &format!("AutoAccount.{name}"), |v, p| {
            let source =
                scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected regex text"))?;
            Pattern::new(&source)
                .map(|v| SettingChange::AutoAccount(create(v)))
                .map_err(|e| ConfigError::new(p, e.to_string()))
        });
    }
    leaf(
        patch,
        root,
        "RepeatPasswordOnRegister",
        "AutoAccount.RepeatPasswordOnRegister",
        |v, p| {
            parse_bool(v, p).map(|v| {
                SettingChange::AutoAccount(AutoAccountSetting::RepeatPasswordOnRegister(v))
            })
        },
    );
    leaf(
        patch,
        root,
        "MessageSource",
        "AutoAccount.MessageSource",
        |v, p| {
            parse_multi_mask(v, p, &SOURCES).map(|v| {
                SettingChange::AutoAccount(AutoAccountSetting::MessageSource(if v == 0 {
                    7
                } else {
                    v
                }))
            })
        },
    );
}
