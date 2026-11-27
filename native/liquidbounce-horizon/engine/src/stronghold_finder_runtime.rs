// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientStrongholdFinderEvent;
use liquidbounce_behaviors::stronghold_finder::{Eye, Notice};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn stronghold_finder(&self, event: &mut ClientStrongholdFinderEvent) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        if !m.query_running(&state.registry, "StrongholdFinder") {
            return;
        }
        let s = &mut m.stronghold_finder;
        match v["action"].as_str() {
            Some("options") => {
                *event.output = "true".into();
            }
            Some("used") => {
                if let Ok(pos) = serde_json::from_value(v["position"].clone()) {
                    s.used(pos, v["tick"].as_i64().unwrap_or(0) as i32, 0);
                }
            }
            Some("spawn") => {
                if let Ok(pos) = serde_json::from_value(v["position"].clone()) {
                    s.spawn(
                        v["id"].as_i64().unwrap_or(0) as i32,
                        pos,
                        v["tick"].as_i64().unwrap_or(0) as i32,
                        0,
                    );
                }
            }
            Some("blocks") => {
                if let Some(blocks) = v["blocks"].as_array() {
                    for b in blocks {
                        if let (Ok(pos), Some(name)) = (
                            serde_json::from_value(b["position"].clone()),
                            b["name"].as_str(),
                        ) {
                            s.block(pos, name);
                        }
                    }
                }
            }
            Some("forget") => {
                if let Ok(chunk) = serde_json::from_value(v["chunk"].clone()) {
                    s.forget(chunk);
                }
            }
            Some("tick") => {
                let eyes = v["eyes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|e| {
                        Some(Eye {
                            id: e["id"].as_i64()? as i32,
                            position: serde_json::from_value(e["position"].clone()).ok()?,
                            velocity: serde_json::from_value(e["velocity"].clone()).ok()?,
                        })
                    })
                    .collect::<Vec<_>>();
                let notices = s.tick(v["tick"].as_i64().unwrap_or(0) as i32, &eyes);
                drop(m);
                #[cfg(feature = "webview")]
                for notice in notices {
                    let (key, args) = match notice {
                        Notice::Sample(n) => ("sampleCaptured", vec![n.to_string()]),
                        Notice::Best(c) => (
                            "bestChunk",
                            vec![
                                c.chunk[0].to_string(),
                                c.chunk[1].to_string(),
                                format!("{:.1}", c.probability * 100.),
                            ],
                        ),
                    };
                    let mut message = liquidbounce_interop::localization::describe(
                        &horizon_client::lang::code(),
                        &format!("liquidbounce.module.strongholdFinder.messages.{key}"),
                    );
                    for arg in args {
                        message = message.replacen("%s", &arg, 1);
                    }
                    message = message.replace("%%", "%");
                    state.events.borrow_mut().push((
                        "notification".into(),
                        json!({"title":"StrongholdFinder","message":message,"severity":"INFO"}),
                    ));
                }
                #[cfg(not(feature = "webview"))]
                let _ = notices;
            }
            Some("render") => {
                let Ok(player) = serde_json::from_value::<[f64; 3]>(v["player"].clone()) else {
                    return;
                };
                let mut lines = Vec::new();
                let mut planes = Vec::new();
                let mut hud = Vec::<String>::new();
                if !s.portals.is_empty() {
                    for (p, portal) in &s.portals {
                        planes.push(json!({"position":[f64::from(p[0]),f64::from(p[1])+0.01,f64::from(p[2])],"size":1.,"color":if *portal{0xaa00dcffu32}else{0xaaffd700u32}}));
                    }
                    let distance = |p: &[i32; 3]| {
                        (0..3)
                            .map(|i| (f64::from(p[i]) + 0.5 - player[i]).powi(2))
                            .sum::<f64>()
                    };
                    if let Some(p) = s
                        .portals
                        .keys()
                        .min_by(|a, b| distance(a).total_cmp(&distance(b)))
                    {
                        let start = [player[0], player[1] + 0.05, player[2]];
                        let target = p.map(|n| f64::from(n) + 0.5);
                        let color = 0xdcff5050u32;
                        lines.push(json!({"start":start,"end":target,"color":color}));
                        let dx = target[0] - start[0];
                        let dz = target[2] - start[2];
                        let length = dx.hypot(dz);
                        if length > 1e-6 {
                            lines.push(json!({"start":start,"end":[start[0]+dx/length*2.,start[1],start[2]+dz/length*2.],"color":color}));
                        }
                    }
                } else {
                    if s.settings.render_rays {
                        for sample in &s.measurements {
                            // Vec3.directionFromRotation(0, yaw), as float operations.
                            let direction = liquidbounce_behaviors::wings::look(sample.angle);
                            let end = std::array::from_fn::<_, 3, _>(|i| {
                                direction[i].mul_add(2048., sample.position[i])
                            });
                            lines.push(
                                json!({"start":sample.position,"end":end,"color":0xaaffffffu32}),
                            );
                        }
                    }
                    if let Some(posterior) = &s.posterior {
                        for (i, c) in posterior
                            .candidates
                            .iter()
                            .take(s.settings.top_candidates.max(0) as usize)
                            .enumerate()
                        {
                            if (i == 0 && s.settings.render_best)
                                || (i > 0 && s.settings.render_top)
                            {
                                let alpha = (45. + c.probability * 170.) as u32;
                                let color = (alpha.clamp(30, 200) << 24)
                                    | if i == 0 { 0x00aaff } else { 0xffaa00 };
                                planes.push(json!({"position":[f64::from(c.chunk[0])*16.,player[1],f64::from(c.chunk[1])*16.],"size":16.,"color":color}));
                            }
                        }
                        if let Some(c) = posterior.candidates.first() {
                            let lang = horizon_client::lang::code();
                            let (title, sample, best) = if lang == "zh_tw" {
                                ("要塞定位", "樣本", "最佳區塊")
                            } else if lang.starts_with("zh_") {
                                ("要塞定位", "样本", "最佳区块")
                            } else {
                                ("StrongholdFinder", "Samples", "Best chunk")
                            };
                            hud = vec![
                                title.into(),
                                format!(
                                    "{sample}: {} | Sigma: {:.3}°",
                                    posterior.sample_count, s.settings.sigma
                                ),
                                format!(
                                    "{best}: {}, {} ({:.1}%)",
                                    c.chunk[0],
                                    c.chunk[1],
                                    c.probability * 100.
                                ),
                                format!(
                                    "/tp {} ~ {}",
                                    c.chunk[0].wrapping_mul(16).wrapping_add(8),
                                    c.chunk[1].wrapping_mul(16).wrapping_add(8)
                                ),
                            ];
                        }
                    }
                }
                *event.output = json!({"lines":lines,"planes":planes,"hud":hud}).to_string();
            }
            _ => {}
        }
    }
}
