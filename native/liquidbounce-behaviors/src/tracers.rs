// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleTracers / GenericColorMode, LiquidBounce ae0eb396.
use crate::breadcrumbs::{Breadcrumbs, ColoredPoint, Effect, rainbow, rgba};
#[path = "tracers_curve.rs"]
pub mod curve;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    #[default]
    Distance,
    Health,
    Static,
    Rainbow,
}
impl ColorMode {
    pub const ALL: [Self; 4] = [Self::Distance, Self::Health, Self::Static, Self::Rainbow];
    pub fn tag(self) -> &'static str {
        match self {
            Self::Distance => "Distance",
            Self::Health => "Health",
            Self::Static => "Static",
            Self::Rainbow => "Rainbow",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Tracers {
    pub color_mode: ColorMode,
    pub saturation: f32,
    pub brightness: f32,
    pub hue: Vec<[f32; 2]>,
    pub alpha: f32,
    pub health_alpha: i32,
    pub color: i32,
    pub line_width: f32,
    pub maximum_distance: f32,
    pub render: Breadcrumbs,
    pub selected: std::collections::BTreeSet<u64>,
    pub perspective: Option<i32>,
}
impl Default for Tracers {
    fn default() -> Self {
        let mut render = Breadcrumbs::default();
        for style in &mut render.styles {
            style.values.insert("Fill".into(), 1.);
        }
        Self {
            color_mode: ColorMode::Distance,
            saturation: 1.,
            brightness: 1.,
            hue: vec![[0., 0.], [100., 120.], [200., 120.]],
            alpha: 1.,
            health_alpha: 255,
            color: 0xff00a0ffu32 as i32,
            line_width: 1.,
            maximum_distance: 128.,
            render,
            selected: Default::default(),
            perspective: None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entity {
    pub position: [f64; 3],
    pub interpolated: [f64; 3],
    pub height: f32,
    pub health: f32,
    pub max_health: f32,
    pub friend: bool,
    pub tag: Option<u32>,
}
fn hsb(h: f32, s: f32, b: f32, a: f32) -> [f32; 4] {
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
    let rgb = rgb.map(|v| (v * 255. + 0.5) as i32);
    [
        rgb[0] as f32 / 255.,
        rgb[1] as f32 / 255.,
        rgb[2] as f32 / 255.,
        (a * 255.) as i32 as f32 / 255.,
    ]
}
impl Tracers {
    pub fn clear(&mut self) {
        self.selected.clear();
        self.perspective = None;
    }
    pub fn effect(&self) -> Option<Effect> {
        self.render.effect()
    }
    pub fn lines(
        &self,
        camera: [f64; 3],
        eye: [f32; 3],
        entities: &[Entity],
        nanos: f64,
    ) -> Vec<[ColoredPoint; 2]> {
        let mut lines = vec![];
        for e in entities {
            let sq64 = (0..3)
                .map(|i| (e.position[i] - camera[i]) * (e.position[i] - camera[i]))
                .sum::<f64>();
            let sq = sq64 as f32;
            if sq > self.maximum_distance * self.maximum_distance {
                continue;
            }
            let mut color = if e.friend {
                rgba(0xff0000ff)
            } else if let Some(c) = e.tag {
                rgba(c)
            } else {
                match self.color_mode {
                    ColorMode::Static => rgba(self.color as u32),
                    ColorMode::Rainbow => {
                        let mut c = rainbow(nanos);
                        c[3] = 50. / 255.;
                        c
                    }
                    ColorMode::Health => {
                        let health = e.health.min(e.max_health);
                        let part = health / e.max_health;
                        [
                            ((255. * (1. - part)) as i32).clamp(0, 255) as f32 / 255.,
                            ((255. * part) as i32).clamp(0, 255) as f32 / 255.,
                            0.,
                            self.health_alpha as f32 / 255.,
                        ]
                    }
                    ColorMode::Distance => hsb(
                        curve::sample(&self.hue, sq64.sqrt() as f32) / 360.,
                        self.saturation,
                        self.brightness,
                        self.alpha,
                    ),
                }
            };
            if self.render.shader {
                color[3] = 1.;
            }
            let relative: [f32; 3] =
                std::array::from_fn(|i| (e.interpolated[i] - camera[i]) as f32);
            let base = std::array::from_fn(|i| camera[i] + f64::from(relative[i]));
            let top = std::array::from_fn(|i| {
                camera[i] + f64::from(relative[i] + if i == 1 { e.height } else { 0. })
            });
            let start = std::array::from_fn(|i| camera[i] + f64::from(eye[i]));
            for (a, b) in [(start, base), (base, top)] {
                lines.push([
                    ColoredPoint { position: a, color },
                    ColoredPoint { position: b, color },
                ]);
            }
        }
        lines
    }
}
