// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleWings/WingsLines, LiquidBounce ae0eb396, CCBlueX 2015–2026.
use crate::breadcrumbs::{Breadcrumbs, ColoredPoint, Effect, rgba};
const TABLE: &[u8; 262144] = include_bytes!("../vendor/minecraft-math/sine-table.bin");
fn trig(angle: f64, cos: bool) -> f32 {
    let index =
        ((angle * 10430.378350470453 + if cos { 16384. } else { 0. }) as i64 & 65535) as usize * 4;
    f32::from_le_bytes(TABLE[index..index + 4].try_into().unwrap())
}
pub fn look(yaw: f32) -> [f64; 3] {
    let a = f64::from(-yaw * (std::f32::consts::PI / 180.) - std::f32::consts::PI);
    [f64::from(-trig(a, false)), 0., f64::from(-trig(a, true))]
}
#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub local: bool,
    pub friend: bool,
    pub position: [f64; 3],
    pub interpolated: [f64; 3],
    pub body_yaw: f32,
    pub scale: f32,
    pub height: f32,
    pub chest: bool,
    pub hurt: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Wings {
    pub height: f32,
    pub back_offset: f32,
    pub equipment_offset: f32,
    pub show_damage: bool,
    pub first_person: bool,
    pub view_on_friend: bool,
    pub distance: i32,
    pub length: f32,
    pub width: f32,
    pub fade_start: f32,
    pub count: i32,
    pub angles: [i32; 2],
    pub color: i32,
    pub render: Breadcrumbs,
}
impl Default for Wings {
    fn default() -> Self {
        let mut render = Breadcrumbs::default();
        for s in &mut render.styles {
            s.values.insert("Fill".into(), 1.);
        }
        Self {
            height: 0.3,
            back_offset: 0.25,
            equipment_offset: 0.1,
            show_damage: true,
            first_person: false,
            view_on_friend: true,
            distance: 64,
            length: 1.,
            width: 0.05,
            fade_start: 0.5,
            count: 4,
            angles: [10, 37],
            color: 0xff4677ffu32 as i32,
            render,
        }
    }
}
fn rotate(mut p: [f32; 3], axis: usize, degrees: f32) -> [f32; 3] {
    let a = f64::from(degrees * (std::f32::consts::PI / 180.));
    let (s, c) = (a.sin() as f32, a.cos() as f32);
    let (i, j) = match axis {
        0 => (1, 2),
        1 => (2, 0),
        _ => (0, 1),
    };
    let (x, y) = (p[i], p[j]);
    p[i] = c * x - s * y;
    p[j] = s * x + c * y;
    p
}
impl Wings {
    pub fn effect(&self) -> Option<Effect> {
        self.render.effect()
    }
    pub fn selected(&self, e: &Entity, local: [f64; 3], first: bool, free_look: bool) -> bool {
        if e.local {
            !first || self.first_person || free_look
        } else {
            self.view_on_friend
                && e.friend
                && e.position
                    .iter()
                    .zip(local)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f64>()
                    < f64::from(self.distance).powi(2)
        }
    }
    pub fn mesh(
        &self,
        entities: &[Entity],
        local: [f64; 3],
        first: bool,
        free_look: bool,
        local_shift: bool,
    ) -> Vec<[ColoredPoint; 4]> {
        let mut out = vec![];
        let shifting = if local_shift { 0.1f32 } else { 0. };
        let step = if self.count > 1 {
            (self.angles[1] - self.angles[0]) as f32 / (self.count - 1) as f32
        } else {
            0.
        };
        for e in entities
            .iter()
            .filter(|e| self.selected(e, local, first, free_look))
        {
            let mut color = rgba(if e.hurt && self.show_damage {
                ((self.color as u32) & 0xff000000) | 0x00ff0000
            } else {
                self.color as u32
            });
            if self.render.shader {
                color[3] = 1.;
            }
            let transparent = [color[0], color[1], color[2], 0.];
            let behind = (f64::from(self.back_offset)
                + if e.chest {
                    f64::from(self.equipment_offset)
                } else {
                    0.
                })
                * f64::from(e.scale);
            let look = look(e.body_yaw);
            let mut origin =
                std::array::from_fn::<_, 3, _>(|i| (-behind).mul_add(look[i], e.interpolated[i]));
            origin[1] += f64::from(e.height) / 2. + f64::from(self.height) - f64::from(shifting);
            for i in 0..self.count.max(0) {
                let angle = self.angles[0] as f32 + i as f32 * step;
                for side in [1f32, -1.] {
                    let tx = ((0.1 + 0.0375 * f64::from(i)) * f64::from(side)) as f32;
                    let tz = (-0.1 - 0.0375 * f64::from(i)) as f32;
                    let transform = |mut p: [f32; 3]| {
                        p = rotate(p, 1, angle * side);
                        p[0] += tx;
                        p[2] += tz;
                        p = rotate(p, 0, if shifting == 0. { 0. } else { 27.5 });
                        p = rotate(p, 2, e.body_yaw);
                        p = rotate(p, 0, 90.);
                        std::array::from_fn(|j| origin[j] + f64::from(p[j] * e.scale))
                    };
                    let (first_x, second_x) = if side > 0. {
                        (self.length - 0.25, self.length)
                    } else {
                        (self.length, self.length - 0.25)
                    };
                    for extent in [first_x, -second_x] {
                        let solid = extent * self.fade_start;
                        for (a, b, c1, c2) in [
                            (0., solid, color, color),
                            (solid, extent, color, transparent),
                        ] {
                            let h = self.width / 2.;
                            out.push(
                                [[a, 0., -h], [b, 0., -h], [b, 0., h], [a, 0., h]]
                                    .into_iter()
                                    .enumerate()
                                    .map(|(j, p)| ColoredPoint {
                                        position: transform(p),
                                        color: if j == 0 || j == 3 { c1 } else { c2 },
                                    })
                                    .collect::<Vec<_>>()
                                    .try_into()
                                    .unwrap(),
                            );
                        }
                    }
                }
            }
        }
        out
    }
}
