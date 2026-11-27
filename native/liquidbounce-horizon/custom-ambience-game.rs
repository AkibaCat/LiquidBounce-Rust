// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use serde_json::{Value, json};
use std::cell::RefCell;
// Published at client/frame boundaries before dispatching any other mod. Native
// provider reads never dispatch back into the behavior actor or borrow its state.
thread_local! {static SNAPSHOT:RefCell<Value>=const{RefCell::new(Value::Null)};}
pub(super) fn publish_snapshot(value: Value) {
    SNAPSHOT.with_borrow_mut(|s| *s = value);
}
pub(super) fn snapshot() -> Value {
    SNAPSHOT.with_borrow(Clone::clone)
}
pub(super) fn snapshot_weather(s: &Value, original: (f32, f32)) -> (f32, f32) {
    match s["weather"].as_str().unwrap_or("") {
        "Sunny" => (0., 0.),
        "Rainy" => (1., 0.),
        "Snowy" => (0.9, 0.),
        "Thunder" => (1., 1.),
        _ => original,
    }
}
pub(super) fn snapshot_time(s: &Value, original: i64, definition: &str) -> i64 {
    if definition != "minecraft:overworld" {
        return original;
    }
    match s["time"].as_str().unwrap_or("") {
        "Dawn" => 23041,
        "Day" => 1000,
        "Noon" => 6000,
        "Dusk" => 12610,
        "Night" => 13000,
        "MidNight" => 18000,
        _ => original,
    }
}
pub(super) fn snapshot_precipitation(s: &Value, intensity: f32) -> (f32, bool) {
    let snow = s["weather"] == "Snowy";
    let intensity = if snow { 0. } else { intensity };
    (
        if intensity != 0. {
            s["precipitation"].as_f64().map_or(intensity, |v| v as f32)
        } else {
            intensity
        },
        snow,
    )
}
impl Game<'_> {
    pub(super) fn client_custom_ambience_refresh(&self) {
        let mut output = String::new();
        if self.native_window() && self.title.is_none() {
            self.native_client_custom_ambience(r#"{"action":"settings"}"#, &mut output);
        }
        publish_snapshot(serde_json::from_str(&output).unwrap_or(Value::Null));
    }
    fn client_custom_ambience_query(&self, input: Value) -> Option<Value> {
        if !self.native_window() || self.title.is_some() {
            return None;
        }
        if input["action"] == "frame" {
            let mut output = String::new();
            self.native_client_custom_ambience(&input.to_string(), &mut output);
            serde_json::from_str(&output).ok()
        } else {
            Some(snapshot())
        }
    }
    pub(super) fn client_custom_ambience_weather(&self, original: (f32, f32)) -> (f32, f32) {
        if !self.native_window() || self.title.is_some() {
            return original;
        }
        SNAPSHOT.with_borrow(|v| snapshot_weather(v, original))
    }
    pub(super) fn client_custom_ambience_time(&self, original: i64) -> i64 {
        self.client_custom_ambience_clock(
            original,
            self.dimension_type().default_clock.as_deref().unwrap_or(""),
        )
    }
    pub(super) fn client_custom_ambience_clock(&self, original: i64, definition: &str) -> i64 {
        if !self.native_window() || self.title.is_some() {
            return original;
        }
        SNAPSHOT.with_borrow(|v| snapshot_time(v, original, definition))
    }
    pub(super) fn client_custom_ambience_precipitation(&self, intensity: f32) -> (f32, bool) {
        if !self.native_window() || self.title.is_some() {
            return (intensity, false);
        }
        SNAPSHOT.with_borrow(|v| snapshot_precipitation(v, intensity))
    }
    pub(super) fn client_custom_ambience_sky(&self, sky: &mut horizon_client::environment::Sky) {
        if let Some(v) = self.client_custom_ambience_query(json!({"action":"settings"}))
            && let Ok(color) = serde_json::from_value::<[f32; 4]>(v["sky_color"].clone())
        {
            sky.sky_color = [color[0], color[1], color[2]];
        }
    }
    pub(super) fn client_custom_ambience_fog(
        &self,
        mut fog: horizon_client::fog::FogData,
    ) -> horizon_client::fog::FogData {
        if let Some(v) = self.client_custom_ambience_query(json!({"action":"settings"}))
            && v["fog"].is_object()
        {
            let f = &v["fog"];
            if let Ok(color) = serde_json::from_value(f["color"].clone()) {
                fog.color = color;
            }
            let number =
                |v: &Value, fallback: f32| v.as_f64().unwrap_or(f64::from(fallback)) as f32;
            fog.environmental_start = number(&f["environmental"]["from"], fog.environmental_start);
            fog.environmental_end = number(&f["environmental"]["to"], fog.environmental_end);
            fog.render_distance_start =
                number(&f["render_distance"]["from"], fog.render_distance_start);
            fog.render_distance_end = number(&f["render_distance"]["to"], fog.render_distance_end);
            fog.sky_end = number(&f["sky_end"], fog.sky_end);
            fog.cloud_end = number(&f["cloud_end"], fog.cloud_end);
            if f["disable"] == true {
                fog.environmental_start = f32::MAX;
                fog.environmental_end = f32::MAX;
                fog.render_distance_start = f32::MAX;
                fog.render_distance_end = f32::MAX;
                fog.sky_end = f32::MAX;
                fog.cloud_end = f32::MAX;
            }
        }
        fog
    }
    pub(super) fn client_custom_ambience_frame(&self, partial: f64) {
        let Some(player) = &self.player else {
            horizon_client::gfx::custom_ambience::begin_frame("null");
            return;
        };
        let q = self.camera_orientation();
        let right = q * glam::Vec3::X;
        let up = q * glam::Vec3::Y;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as i64);
        let input = json!({"action":"frame","now":now,"partial":partial as f32,"player":[player.position.x,player.position.y,player.position.z],"right":right.to_array(),"up":up.to_array()});
        let output = self
            .client_custom_ambience_query(input)
            .unwrap_or(Value::Null);
        horizon_client::gfx::custom_ambience::begin_frame(&output.to_string());
    }
}
