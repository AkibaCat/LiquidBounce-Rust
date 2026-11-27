// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::dank_bobbing::DankBobbing;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DankBobbingSetting(pub f32);
impl DankBobbingSetting {
    pub fn apply(self, module: &mut DankBobbing) {
        module.motion = self.0;
    }
}
pub fn serialize_dank_bobbing(
    state: &DankBobbing,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::DankBobbing, common);
    values.push(tree.ranged(
        "Motion",
        "FLOAT",
        float_json(state.motion, "DankBobbing.Motion")?,
        json!({"from":1.0,"to":50.0}),
        "",
        "liquidbounce.module.dankBobbing.motion",
    ));
    Ok(tree.group("DankBobbing", "TOGGLEABLE", values))
}
pub(super) fn parse_dank_bobbing(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Motion", "DankBobbing.Motion", |v, p| {
        parse_float(v, p).map(|v| SettingChange::DankBobbing(DankBobbingSetting(v)))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"TOGGLE"}),
        }
    }
    #[test]
    fn motion_roundtrips_without_imposing_the_ui_range() {
        for motion in [-10.0, 0.0, 5.0, 99.25] {
            let state = DankBobbing { motion };
            for format in [JsonFormat::File, JsonFormat::Interop] {
                let file =
                    serialize_dank_bobbing(&state, &common(), format, &str::to_owned).unwrap();
                let patch = parse_settings(ModuleKind::DankBobbing, &file).unwrap();
                assert!(patch.issues.is_empty());
                let changes: Vec<_> = patch
                    .changes
                    .iter()
                    .filter_map(|change| match change {
                        SettingChange::DankBobbing(value) => Some(*value),
                        _ => None,
                    })
                    .collect();
                assert_eq!(changes, vec![DankBobbingSetting(motion)]);
                let mut restored = DankBobbing::default();
                changes[0].apply(&mut restored);
                assert_eq!(restored, state);
            }
        }
    }
    #[test]
    fn motion_uses_float_metadata_and_rejects_nonfinite_output() {
        let file = serialize_dank_bobbing(
            &DankBobbing::default(),
            &common(),
            JsonFormat::Interop,
            &str::to_owned,
        )
        .unwrap();
        let motion = file["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == "Motion")
            .unwrap();
        assert_eq!(motion["valueType"], "FLOAT");
        assert_eq!(motion["range"], json!({"from":1.0,"to":50.0}));
        assert!(
            serialize_dank_bobbing(
                &DankBobbing { motion: f32::NAN },
                &common(),
                JsonFormat::File,
                &str::to_owned
            )
            .is_err()
        );
    }
}
