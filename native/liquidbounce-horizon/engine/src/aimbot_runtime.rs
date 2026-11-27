// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{auto_rod_runtime::NativeSamples, runtime::LiquidBounceHooks};
use horizon_client::mod_api::{ClientAimbotRenderEvent, Host};
use liquidbounce_behaviors::{
    aimbot::Aimbot,
    auto_bow::{self, Candidate},
    auto_bow_math::{B, V},
    clicking::JavaRandom,
    point_tracker::PointTracker,
    rotations::{Rotation, modern_mouse_gcd},
    target_renderer::{TargetRendererRuntime, TargetRendererSettings},
};
use liquidbounce_core::InputKey;
use serde_json::{Value, json};
pub struct AimbotRuntime {
    pub behavior: Aimbot,
    pub point_tracker: PointTracker,
    pub random: JavaRandom,
    pub target_renderer: TargetRendererSettings,
    pub target_render_runtime: TargetRendererRuntime,
}
impl Default for AimbotRuntime {
    fn default() -> Self {
        let mut r = Self {
            behavior: Default::default(),
            point_tracker: Default::default(),
            random: Default::default(),
            target_renderer: Default::default(),
            target_render_runtime: Default::default(),
        };
        r.point_tracker
            .initialize(&mut NativeSamples(&mut r.random));
        r
    }
}
impl AimbotRuntime {
    pub fn clear(&mut self) {
        self.behavior.clear();
    }
}
fn query(host: &mut dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_aimbot_query(&v.to_string())).unwrap_or(Value::Null)
}
impl LiquidBounceHooks {
    fn aimbot_pressed(&self, host: &dyn Host, key: &Value, recent: bool) -> bool {
        match key.as_str().and_then(crate::input::configured_bind) {
            Some(InputKey::Keyboard(scan)) => host.client_keyboard_scan_down(scan),
            Some(InputKey::Mouse(button)) => {
                let s = self.state();
                let b = s.behaviors.borrow();
                if recent {
                    b.auto_rod
                        .input
                        .mouse_click(button, crate::behaviors::current_time_millis())
                } else {
                    usize::try_from(button)
                        .ok()
                        .and_then(|i| b.auto_rod.input.mouse.get(i))
                        .copied()
                        .unwrap_or(false)
                }
            }
            _ => false,
        }
    }
    pub(crate) fn aimbot_prepare(&self, host: &mut dyn Host) {
        if !{
            let s = self.state();
            s.behaviors.borrow().query_running(&s.registry, "Aimbot")
        } {
            return;
        }
        let [yaw, pitch] = host.client_player_rotation();
        let current = Rotation::new(yaw, pitch);
        let facts = query(host, json!({"op":"facts"}));
        let click = self.aimbot_pressed(host, &facts["attackKey"], true);
        let using = self.aimbot_pressed(host, &facts["useKey"], false);
        let (settings, previous, met) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let moved = b
                .auto_rod
                .input
                .mouse_moved(crate::behaviors::current_time_millis());
            let requirements = [
                click,
                moved,
                facts["weapon"] == true,
                facts["emptyHand"] == true,
                facts["vanillaName"] == true,
                facts["notBreaking"] == true,
            ];
            let met = facts.is_object()
                && b.aimbot
                    .behavior
                    .requirements_met(facts["screen"] == true, requirements);
            b.aimbot.behavior.prepare(current, met);
            let previous = b
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .previous_rotation_target();
            (b.aimbot.behavior.settings.clone(), previous, met)
        };
        if !met {
            return;
        }
        let mut targets = Vec::<(i32, bool)>::new();
        let range = f64::from(settings.range * settings.range);
        let crystal_range = f64::from(settings.range) * f64::from(settings.range);
        let fov = if settings.fov >= 180. {
            None
        } else {
            Some((f64::from(settings.fov) * f64::from(std::f32::consts::PI / 180.)).cos())
        };
        if settings.crystals && (!settings.crystals_using || using) {
            let mut crystals = facts["crystals"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|v| {
                    v["distance"].as_f64().is_some_and(|d| d <= crystal_range)
                        && fov.is_none_or(|f| v["cosine"].as_f64().unwrap_or(-1.) >= f)
                })
                .collect::<Vec<_>>();
            crystals.sort_by(|a, b| {
                liquidbounce_behaviors::auto_weapon::java_double_compare(
                    b["cosine"].as_f64().unwrap_or(-1.),
                    a["cosine"].as_f64().unwrap_or(-1.),
                )
            });
            targets.extend(
                crystals
                    .into_iter()
                    .filter_map(|v| Some((v["id"].as_i64()? as i32, false))),
            );
        }
        if !settings.crystals || !settings.crystals_only {
            let mut candidates = vec![];
            for row in facts["candidates"].as_array().into_iter().flatten() {
                let Some(id) = row["id"].as_i64().map(|i| i as i32) else {
                    continue;
                };
                let target = if row["target"].is_object() {
                    row["target"].to_string()
                } else {
                    host.client_targeting_entity(id)
                };
                let Some(t) = crate::targeting::snapshot(&target) else {
                    continue;
                };
                if !t.entity.kind.living {
                    continue;
                }
                let mut s = self.state();
                let info = s.target_info(id, &t);
                if !s.global_targets.combat.should_be_attacked(&t.entity, info) {
                    continue;
                }
                let distance = row["distance"].as_f64().unwrap_or(f64::INFINITY);
                if !(distance <= range) {
                    continue;
                }
                candidates.push(Candidate {
                    id,
                    type_weight: row["type"].as_i64().unwrap_or(i64::from(i32::MAX)) as i32,
                    health: row["health"].as_f64().unwrap_or(0.) as f32,
                    distance,
                    cosine: row["cosine"].as_f64().unwrap_or(-1.),
                    hurt: row["hurt"].as_i64().unwrap_or(0) as i32,
                    age: row["age"].as_i64().unwrap_or(0) as i32,
                });
            }
            let s = auto_bow::Settings {
                fov: settings.fov,
                hurt_time: settings.hurt_time,
                priority: settings.priority.clone(),
                ..Default::default()
            };
            targets.extend(
                auto_bow::targets(&s, candidates)
                    .into_iter()
                    .map(|c| (c.id, true)),
            );
        }
        let mut chosen = None;
        for (id, living) in targets {
            if settings.lazy
                && query(
                    host,
                    json!({"op":"lazy","id":id,"range":settings.range,"walls":settings.wall_range}),
                ) == true
            {
                chosen = Some((id, living, current));
                break;
            }
            let q = query(host, json!({"op":"point","id":id}));
            let Some(eyes) = serde_json::from_value::<[f64; 3]>(q["eye"].clone()).ok() else {
                continue;
            };
            let Some(bounds) = serde_json::from_value::<[f64; 6]>(q["bounds"].clone()).ok() else {
                continue;
            };
            let point = {
                let s = self.state();
                let mut b = s.behaviors.borrow_mut();
                let r = &mut b.aimbot;
                r.point_tracker.find(
                    V(eyes[0], eyes[1], eyes[2]),
                    B::from_array(bounds),
                    &mut NativeSamples(&mut r.random),
                )
            };
            let b = point.bounds;
            let q = query(
                host,
                json!({"op":"raytrace","point":point.pos.array(),"bounds":[b.lo.0,b.lo.1,b.lo.2,b.hi.0,b.hi.1,b.hi.2],"range":settings.range,"walls":settings.wall_range}),
            );
            if let Ok([yaw, pitch]) = serde_json::from_value::<[f32; 2]>(q) {
                chosen = Some((id, living, Rotation::new(yaw, pitch)));
                break;
            }
        }
        let target = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.aimbot;
            r.behavior.target = chosen.filter(|(_, living, _)| *living).map(|(id, _, _)| id);
            if let Err(e) = r.behavior.aim(
                chosen.map(|(_, _, r)| r),
                previous,
                &mut liquidbounce_behaviors::random::random_unit_f32,
            ) {
                eprintln!("[LiquidBounce] Aimbot smoothing: {e}");
            }
            r.behavior.target
        };
        // Called even on the no-target branch; requirements failure returns above.
        self.aimbot_auto_weapon(host, target);
    }
    fn aimbot_auto_weapon(&self, host: &mut dyn Host, target: Option<i32>) {
        use liquidbounce_behaviors::auto_weapon::{REQUESTER, Snapshot, Target};
        let Some(id) = target else { return };
        let weapon = host
            .client_weapon_context()
            .as_ref()
            .and_then(Snapshot::parse);
        let owns = host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);
        let target = host.client_crosshair_clicker_target(id);
        let switch = {
            let s = self.state();
            let b = s.behaviors.borrow();
            weapon.as_ref().and_then(|w| {
                b.auto_weapon.on_target(
                    b.query_running(&s.registry, "AutoWeapon"),
                    &w.context(
                        owns,
                        s.registry.enabled_without_world(b.query_ids["MaceKill"]),
                    ),
                    Some(Target {
                        living: target[0],
                        would_block_hit: target[1],
                    }),
                )
            })
        };
        if let Some(switch) = switch
            && let Err(e) = host.client_select_slot_silently(REQUESTER, switch.slot, switch.ticks)
        {
            eprintln!("[LiquidBounce] Aimbot AutoWeapon: {e}");
        }
    }
    pub(crate) fn aimbot_render(&self, host: &dyn Host, event: &mut ClientAimbotRenderEvent) {
        let sensitivity = host.client_mouse_sensitivity();
        let s = self.state();
        let b = s.behaviors.borrow();
        if !b.query_running(&s.registry, "Aimbot") {
            return;
        }
        let container = event.container || b.inventory.manager.inventory_open_server_side;
        let speed = b.timer.speed();
        let Some(r) = b
            .aimbot
            .behavior
            .render(event.partial, speed, event.screen, container)
        else {
            return;
        };
        let current = b
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .current
            .unwrap_or(Rotation::new(event.yaw, event.pitch));
        match r.normalize(current, modern_mouse_gcd(sensitivity)) {
            Ok(r) => {
                event.yaw = r.yaw;
                event.pitch = r.pitch;
                event.apply = true
            }
            Err(e) => eprintln!("[LiquidBounce] Aimbot normalization: {e:?}"),
        }
    }
}
