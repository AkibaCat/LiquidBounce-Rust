// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSnapTap declares only common ClientModule settings.
use super::*;

pub fn serialize_snap_tap(
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    Ok(tree.group(
        "SnapTap",
        "TOGGLEABLE",
        tree.common(ModuleKind::SnapTap, common),
    ))
}
