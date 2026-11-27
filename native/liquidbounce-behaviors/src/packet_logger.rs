// SPDX-License-Identifier: GPL-3.0-or-later
//! PacketLogger settings and output, derived from LiquidBounce ae0eb396.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub whitelist: bool,
    pub c2s: Vec<String>,
    pub s2c: Vec<String>,
    pub show_type: bool,
    pub chat: bool,
    pub file: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            whitelist: false,
            c2s: vec![],
            s2c: vec![],
            show_type: true,
            chat: true,
            file: false,
        }
    }
}
impl Settings {
    pub fn accepts(&self, incoming: bool, id: &str) -> bool {
        let present = if incoming { &self.s2c } else { &self.c2s }
            .iter()
            .any(|s| s == id);
        present == self.whitelist
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub kind: String,
    pub value: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub time: i64,
    pub incoming: bool,
    pub class: String,
    pub id: String,
    pub cancelled: bool,
    pub fields: Vec<Field>,
}
impl Record {
    pub fn csv(&self) -> String {
        let fields = self
            .fields
            .iter()
            .map(|f| format!("{}:{}={};", f.name, f.kind, f.value))
            .collect::<String>();
        format!(
            "{},{},{},{},{},\"{}\"\n",
            self.time,
            if self.incoming {
                "Incoming"
            } else {
                "Outgoing"
            },
            self.class,
            self.id,
            self.cancelled,
            fields.replace('"', "\"\"")
        )
    }
}
