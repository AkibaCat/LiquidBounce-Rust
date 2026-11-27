// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleRadar and GenericColorMode, LiquidBounce ae0eb396.
use crate::{
    breadcrumbs::rainbow,
    tracers::{ColorMode, curve},
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerMode {
    #[default]
    Triangle,
    Image1,
    Image2,
}
impl PointerMode {
    pub const ALL: [Self; 3] = [Self::Triangle, Self::Image1, Self::Image2];
    pub fn tag(self) -> &'static str {
        match self {
            Self::Triangle => "Triangle",
            Self::Image1 => "Image1",
            Self::Image2 => "Image2",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Radar {
    pub only_players: bool,
    pub hide_teammates: bool,
    pub show: u32,
    pub edge_inset: f32,
    pub pointer_mode: PointerMode,
    pub width: f32,
    pub height: f32,
    pub tail_concave_size: f32,
    pub image_sizes: [f32; 2],
    pub color_mode: ColorMode,
    pub saturation: f32,
    pub brightness: f32,
    pub hue: Vec<[f32; 2]>,
    pub health_alpha: i32,
    pub color: i32,
    pub alpha: Vec<[f32; 2]>,
    pub selected: BTreeSet<u64>,
    pub perspective: Option<i32>,
}
impl Default for Radar {
    fn default() -> Self {
        Self {
            only_players: false,
            hide_teammates: false,
            show: 3,
            edge_inset: 0.,
            pointer_mode: PointerMode::Triangle,
            width: 8.,
            height: 10.,
            tail_concave_size: 2.,
            image_sizes: [10.; 2],
            color_mode: ColorMode::Distance,
            saturation: 1.,
            brightness: 1.,
            hue: vec![[0., 0.], [100., 120.], [200., 120.]],
            health_alpha: 255,
            color: 0x64ffffff,
            alpha: vec![[0., 1.], [200., 1.]],
            selected: Default::default(),
            perspective: None,
        }
    }
}
fn hsb(h: f32, s: f32, b: f32) -> [i32; 3] {
    let mut rgb = [b; 3];
    if s != 0. {
        let h = (h - h.floor()) * 6.;
        let f = h - h.floor();
        let p = b * (1. - s);
        let q = b * (1. - s * f);
        let t = b * (1. - s * (1. - f));
        rgb = match h as i32 {
            0 => [b, t, p],
            1 => [q, b, p],
            2 => [p, b, t],
            3 => [p, q, b],
            4 => [t, p, b],
            _ => [b, p, q],
        };
    }
    rgb.map(|v| (v * 255. + 0.5) as i32)
}
impl Radar {
    pub fn clear(&mut self) {
        self.selected.clear();
        self.perspective = None;
    }
    /// The source changes concavity when this setting is assigned, not when height changes.
    pub fn set_concavity(&mut self, v: f32) {
        self.tail_concave_size = if v.is_nan() || self.height.is_nan() {
            f32::NAN
        } else {
            v.min(self.height)
        };
    }
    /// Alpha uses interpolated distance; Distance color uses the entity's current position.
    pub fn indicator_color(
        &self,
        interpolated_distance: f32,
        current_distance: f32,
        health: f32,
        max_health: f32,
        nanos: f64,
    ) -> Option<u32> {
        let alpha = (curve::sample(&self.alpha, interpolated_distance) * 255.).floor() as i32;
        if alpha == 0 {
            return None;
        }
        let rgb = match self.color_mode {
            ColorMode::Distance => hsb(
                curve::sample(&self.hue, current_distance) / 360.,
                self.saturation,
                self.brightness,
            ),
            ColorMode::Health => {
                let h = if health.is_nan() || max_health.is_nan() {
                    f32::NAN
                } else {
                    health.min(max_health)
                };
                let part = h / max_health;
                [
                    ((255. * (1. - part)) as i32).clamp(0, 255),
                    ((255. * part) as i32).clamp(0, 255),
                    0,
                ]
            }
            ColorMode::Static => [
                (self.color >> 16) & 255,
                (self.color >> 8) & 255,
                self.color & 255,
            ],
            ColorMode::Rainbow => {
                let c = rainbow(nanos);
                [c[0], c[1], c[2]].map(|v| (v * 255.) as i32)
            }
        };
        // Color4b.alpha replaces the original alpha, including the static/health setting.
        Some(
            ((alpha as u32) << 24)
                | ((rgb[0] as u32) << 16)
                | ((rgb[1] as u32) << 8)
                | rgb[2] as u32,
        )
    }
}
