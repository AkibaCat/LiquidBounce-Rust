// SPDX-License-Identifier: GPL-3.0-or-later
//! ElytraRecast has no custom settings or mode group.
use super::*;

pub fn serialize_elytra_recast(
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    Ok(tree.group(
        "ElytraRecast",
        "TOGGLEABLE",
        tree.common(ModuleKind::ElytraRecast, common),
    ))
}
