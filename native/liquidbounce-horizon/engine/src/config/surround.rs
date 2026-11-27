// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::surround::{self, Settings};

#[derive(Clone, Debug, PartialEq)]
pub enum SurroundSetting { Value(String, Value) }
impl SurroundSetting {
    pub fn apply(self, state: &mut liquidbounce_behaviors::surround::Settings, blocks: &[String]) -> Result<(), ConfigError> {
        let Self::Value(path, value) = self;
        let p = format!("Surround.{path}");
        match path.as_str() {
            "Features" => state.features = parse_multi_mask(&value, &p, &surround::FEATURES)?,
            "DisableOn" => state.disable_on = parse_multi_mask(&value, &p, &surround::DISABLE_ON)?,
            "Instant" => state.instant = parse_bool(&value, &p)?,
            "Filter" => state.whitelist = scalar_string(&value).is_some_and(|v| v.eq_ignore_ascii_case("Whitelist")),
            "Blocks" => {
                let a = value.as_array().ok_or_else(|| ConfigError::new(&p, "expected blocks array"))?;
                let mut out = Vec::new();
                for item in a {
                    let id = scalar_string(item).ok_or_else(|| ConfigError::new(&p, "expected block id"))?;
                    let id = if id.contains(':') { id } else { format!("minecraft:{id}") };
                    if !blocks.is_empty() && !blocks.contains(&id) { return Err(ConfigError::new(&p, "unknown block id")); }
                    if !out.contains(&id) { out.push(id); }
                }
                state.blocks = out;
            }
            _ => return Err(ConfigError::new(&p, "unknown Surround setting")),
        }
        Ok(())
    }
}

pub fn serialize_surround(state: &liquidbounce_behaviors::surround::Settings, common: &CommonSettings, format: JsonFormat, describe: &dyn Fn(&str)->String) -> Result<Value, ConfigError> {
    let t=Tree{format,describe}; let mut values=t.common(ModuleKind::Surround,common);
    let mut features=t.leaf("Features","MULTI_CHOOSE",json!(surround::FEATURES.into_iter().enumerate().filter_map(|(i,n)|(state.features&(1<<i)!=0).then_some(n)).collect::<Vec<_>>()),"liquidbounce.module.surround.features",None);
    if format==JsonFormat::Interop { features["choices"]=json!(surround::FEATURES); features["canBeNone"]=json!(true); features["isOrderSensitive"]=json!(false); }
    values.push(features);
    let mut disable=t.leaf("DisableOn","MULTI_CHOOSE",json!(surround::DISABLE_ON.into_iter().enumerate().filter_map(|(i,n)|(state.disable_on&(1<<i)!=0).then_some(n)).collect::<Vec<_>>()),"liquidbounce.module.surround.disableOn",None);
    if format==JsonFormat::Interop { disable["choices"]=json!(surround::DISABLE_ON); disable["canBeNone"]=json!(true); disable["isOrderSensitive"]=json!(false); }
    values.push(disable);
    values.push(t.leaf("Instant","BOOLEAN",json!(state.instant),"liquidbounce.module.surround.instant",None));
    values.push(t.leaf("Filter","CHOOSE",json!(if state.whitelist{"Whitelist"}else{"Blacklist"}),"liquidbounce.module.surround.filter",None));
    values.push(t.leaf("Blocks","BLOCK",json!(state.blocks),"liquidbounce.module.surround.blocks",None));
    Ok(t.group("Surround","TOGGLEABLE",values))
}

pub(super) fn parse_surround(p:&mut SettingsPatch,root:&Nodes<'_>) {
    for name in ["Features","DisableOn","Instant","Filter","Blocks"] { if let Some(v)=root.get(name) { match value_of(v,&format!("Surround.{name}")) { Ok(v)=>match SurroundSetting::Value(name.into(),v.clone()).apply(&mut Settings::default(),&[]) { Ok(())=>p.changes.push(SettingChange::Surround(SurroundSetting::Value(name.into(),v.clone()))), Err(e)=>p.issues.push(e) }, Err(e)=>p.issues.push(e) } } }
}
