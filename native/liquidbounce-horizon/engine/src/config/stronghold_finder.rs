// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::stronghold_finder::StrongholdFinder;
#[derive(Clone, Debug, PartialEq)]
pub struct StrongholdFinderSetting(pub String, pub Value);
pub const FIELDS: [(&str, &str, f64, f64, &str); 13] = [
    ("Sigma", "FLOAT", 0.005, 0.20, "°"),
    ("HypothesisCount", "INT", 2000., 100000., ""),
    ("RequireSameStrongholdAcrossThrows", "BOOLEAN", 0., 0., ""),
    ("SampleDelayTicks", "INT", 0., 10., ""),
    ("MinEyeHorizontalSpeed", "FLOAT", 0.001, 0.2, ""),
    ("MaxSampleAgeTicks", "INT", 5., 100., ""),
    ("MaxEyeSpawnDistance", "FLOAT", 1., 32., ""),
    ("ShowTopCandidates", "INT", 1., 10., ""),
    ("RenderRays", "BOOLEAN", 0., 0., ""),
    ("RenderBestChunk", "BOOLEAN", 0., 0., ""),
    ("RenderTopChunks", "BOOLEAN", 0., 0., ""),
    ("AnnouncePrediction", "BOOLEAN", 0., 0., ""),
    ("ResetOnWorldChange", "BOOLEAN", 0., 0., ""),
];
impl StrongholdFinderSetting {
    pub fn apply(&self, m: &mut StrongholdFinder) -> Result<(), ConfigError> {
        let path = format!("StrongholdFinder.{}", self.0);
        let s = &mut m.settings;
        let v = &self.1;
        match self.0.as_str() {
            "Sigma" => s.sigma = parse_float(v, &path)?,
            "HypothesisCount" => s.hypothesis_count = parse_int(v, &path)?,
            "RequireSameStrongholdAcrossThrows" => s.require_same = parse_bool(v, &path)?,
            "SampleDelayTicks" => s.sample_delay = parse_int(v, &path)?,
            "MinEyeHorizontalSpeed" => s.min_speed = parse_float(v, &path)?,
            "MaxSampleAgeTicks" => s.max_age = parse_int(v, &path)?,
            "MaxEyeSpawnDistance" => s.max_spawn_distance = parse_float(v, &path)?,
            "ShowTopCandidates" => s.top_candidates = parse_int(v, &path)?,
            "RenderRays" => s.render_rays = parse_bool(v, &path)?,
            "RenderBestChunk" => s.render_best = parse_bool(v, &path)?,
            "RenderTopChunks" => s.render_top = parse_bool(v, &path)?,
            "AnnouncePrediction" => s.announce = parse_bool(v, &path)?,
            "ResetOnWorldChange" => s.reset_on_world_change = parse_bool(v, &path)?,
            _ => return Err(ConfigError::new(&path, "unknown StrongholdFinder setting")),
        }
        if matches!(
            self.0.as_str(),
            "Sigma" | "HypothesisCount" | "RequireSameStrongholdAcrossThrows" | "ShowTopCandidates"
        ) {
            m.settings_changed();
        }
        Ok(())
    }
}
pub fn serialize_stronghold_finder(
    m: &StrongholdFinder,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = &m.settings;
    let mut values = t.common(ModuleKind::StrongholdFinder, c);
    let data = [
        float_json(s.sigma, "StrongholdFinder.Sigma")?,
        json!(s.hypothesis_count),
        json!(s.require_same),
        json!(s.sample_delay),
        float_json(s.min_speed, "StrongholdFinder.MinEyeHorizontalSpeed")?,
        json!(s.max_age),
        float_json(s.max_spawn_distance, "StrongholdFinder.MaxEyeSpawnDistance")?,
        json!(s.top_candidates),
        json!(s.render_rays),
        json!(s.render_best),
        json!(s.render_top),
        json!(s.announce),
        json!(s.reset_on_world_change),
    ];
    for ((name, kind, from, to, suffix), v) in FIELDS.into_iter().zip(data) {
        let key = format!(
            "liquidbounce.module.strongholdFinder.{}{}",
            name[..1].to_ascii_lowercase(),
            &name[1..]
        );
        values.push(if kind == "BOOLEAN" {
            t.leaf(name, kind, v, &key, None)
        } else {
            let range = if kind == "INT" {
                json!({"from":from as i32,"to":to as i32})
            } else {
                json!({"from":from,"to":to})
            };
            t.ranged(name, kind, v, range, suffix, &key)
        });
    }
    Ok(t.group("StrongholdFinder", "TOGGLEABLE", values))
}
pub(super) fn parse_stronghold_finder(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for (name, kind, _, _, _) in FIELDS {
        leaf(
            p,
            root,
            name,
            &format!("StrongholdFinder.{name}"),
            |v, path| {
                match kind {
                    "BOOLEAN" => {
                        parse_bool(v, path)?;
                    }
                    "INT" => {
                        parse_int(v, path)?;
                    }
                    _ => {
                        parse_float(v, path)?;
                    }
                }
                Ok(SettingChange::StrongholdFinder(StrongholdFinderSetting(
                    name.into(),
                    v.clone(),
                )))
            },
        );
    }
}
