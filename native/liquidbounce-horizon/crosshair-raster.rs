// SPDX-License-Identifier: GPL-3.0-or-later
//! CPU evaluation of gui_circle_lut.fsh at physical pixel centers. HUD fills
//! then use the existing native alpha blend pipeline, in the original layer.
#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub bounds: [f32; 4],
    pub color: u32,
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
pub fn circle(center: [f32; 2], inner: f32, outer: f32, lut: &[i32], scale: f32) -> Vec<Pixel> {
    if outer <= 0. || !outer.is_finite() || scale <= 0. || !scale.is_finite() || lut.len() != 256 {
        return vec![];
    }
    let inner = ((inner / outer).clamp(0., 1.) * 32767.).round() / 32767.;
    let dist = |x: i32, y: i32| {
        let p = [
            ((x as f32 + 0.5) / scale - center[0]) / outer,
            ((y as f32 + 0.5) / scale - center[1]) / outer,
        ];
        p[0].hypot(p[1])
    };
    let mut out = vec![];
    for y in ((center[1] - outer) * scale - 0.5).ceil() as i32
        ..((center[1] + outer) * scale - 0.5).ceil() as i32
    {
        for x in ((center[0] - outer) * scale - 0.5).ceil() as i32
            ..((center[0] + outer) * scale - 0.5).ceil() as i32
        {
            let d = dist(x, y);
            let aa = ((dist(x | 1, y) - dist(x & !1, y)).abs()
                + (dist(x, y | 1) - dist(x, y & !1)).abs())
            .max(1e-4);
            let alpha = (1. - smooth(-aa, 0., d - 1.))
                * if inner > 0. {
                    1. - smooth(-aa, 0., inner - d)
                } else {
                    1.
                };
            if alpha <= 0.001 {
                continue;
            }
            let px = (x as f32 + 0.5) / scale - center[0];
            let py = (y as f32 + 0.5) / scale - center[1];
            let at = (px.atan2(py) / std::f32::consts::TAU + 1.).fract() * 256. - 0.5;
            let a = at.floor() as i32;
            let t = at - at.floor();
            let ca = (lut[a.rem_euclid(256) as usize] as u32).to_be_bytes();
            let cb = (lut[(a + 1).rem_euclid(256) as usize] as u32).to_be_bytes();
            let rgba = std::array::from_fn::<u8, 4, _>(|i| {
                ((ca[i] as f32 + (cb[i] as f32 - ca[i] as f32) * t)
                    * if i == 0 { alpha } else { 1. })
                .round()
                .clamp(0., 255.) as u8
            });
            if rgba[0] == 0 {
                continue;
            }
            out.push(Pixel {
                bounds: [
                    x as f32 / scale,
                    y as f32 / scale,
                    (x + 1) as f32 / scale,
                    (y + 1) as f32 / scale,
                ],
                color: u32::from_be_bytes(rgba),
            });
        }
    }
    out
}
