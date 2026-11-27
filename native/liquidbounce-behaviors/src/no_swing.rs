// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleNoSwing.kt, MixinConnection.java and MixinLivingEntity.java at ae0eb396.
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU8, Ordering};

const RUNNING: u8 = 1;
const CLIENT: u8 = 2;
const SERVER: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HideFor(u8);

impl Default for HideFor {
    fn default() -> Self {
        Self::BOTH
    }
}

impl HideFor {
    pub const NONE: Self = Self(0);
    pub const CLIENT: Self = Self(CLIENT);
    pub const SERVER: Self = Self(SERVER);
    pub const BOTH: Self = Self(CLIENT | SERVER);

    pub const fn client(self) -> bool {
        self.0 & CLIENT != 0
    }
    pub const fn server(self) -> bool {
        self.0 & SERVER != 0
    }

    pub fn tags(self) -> Vec<&'static str> {
        let mut tags = Vec::new();
        if self.client() {
            tags.push("Client");
        }
        if self.server() {
            tags.push("Server");
        }
        tags
    }
}

/// ModuleNoSwing.HideFor defaults to the two enum entries. No world, packet,
/// render state or settings tree is fabricated by this transport-facing state.
#[derive(Debug)]
pub struct NoSwing {
    state: AtomicU8,
}

impl Default for NoSwing {
    fn default() -> Self {
        Self {
            state: AtomicU8::new(CLIENT | SERVER),
        }
    }
}

impl NoSwing {
    pub fn set_running(&self, running: bool) -> bool {
        let before = if running {
            self.state.fetch_or(RUNNING, Ordering::AcqRel)
        } else {
            self.state.fetch_and(!RUNNING, Ordering::AcqRel)
        };
        (before & RUNNING != 0) != running
    }

    pub fn set_hide_for(&self, value: HideFor) -> bool {
        let mut before = self.state.load(Ordering::Acquire);
        loop {
            match self.state.compare_exchange_weak(
                before,
                (before & RUNNING) | value.0,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return before & (CLIENT | SERVER) != value.0,
                Err(current) => before = current,
            }
        }
    }

    pub fn hide_for(&self) -> HideFor {
        HideFor(self.state.load(Ordering::Acquire) & (CLIENT | SERVER))
    }
    pub fn should_hide_for_client(&self) -> bool {
        self.state.load(Ordering::Acquire) & (RUNNING | CLIENT) == (RUNNING | CLIENT)
    }
    pub fn should_hide_for_server(&self) -> bool {
        self.state.load(Ordering::Acquire) & (RUNNING | SERVER) == (RUNNING | SERVER)
    }
}

/// MultiChoiceListValue.deserializeFrom clears first, accepts a primitive or
/// array, looks up tags without case, ignores unknown tags and permits no choice.
pub fn parse_no_swing_hide_for(value: &Value) -> Result<HideFor, String> {
    fn tag(value: &Value) -> Result<Option<&str>, String> {
        match value {
            Value::String(text) => Ok(Some(text)),
            Value::Number(_) | Value::Bool(_) => Ok(None),
            _ => Err("NoSwing.HideFor entries must be JSON primitives".into()),
        }
    }
    let mut bits = 0;
    let mut include = |value: &Value| -> Result<(), String> {
        match tag(value)? {
            Some(text) if text.eq_ignore_ascii_case("Client") => bits |= CLIENT,
            Some(text) if text.eq_ignore_ascii_case("Server") => bits |= SERVER,
            _ => {}
        }
        Ok(())
    };
    match value {
        Value::Array(values) => {
            for value in values {
                include(value)?;
            }
        }
        Value::String(_) | Value::Number(_) | Value::Bool(_) => include(value)?,
        // Gson's non-array/non-primitive cases leave the already-cleared set.
        Value::Null | Value::Object(_) => {}
    }
    Ok(HideFor(bits))
}

/// Root config integration adds this leaf after the common Bind/Hidden values.
/// File JSON excludes protocol metadata, as the original ConfigSystem does.
pub fn serialize_no_swing_hide_for(value: HideFor, interop: bool, description: &str) -> Value {
    let mut result = json!({"name":"HideFor","value":value.tags()});
    if interop {
        result["valueType"] = json!("MULTI_CHOOSE");
        result["key"] = json!("liquidbounce.module.noSwing.hideFor");
        result["description"] = json!(description);
        result["choices"] = json!(["Client", "Server"]);
        result["canBeNone"] = json!(true);
        result["isOrderSensitive"] = json!(false);
    }
    result
}
