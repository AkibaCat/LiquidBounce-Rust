// SPDX-License-Identifier: GPL-3.0-or-later
// IndicatorRenderer/IndicatorGeometry/WorldToScreen, LiquidBounce ae0eb396.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Projected {
    pub screen: Option<[f32; 2]>,
    pub offset: Option<[f32; 2]>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub in_view: bool,
    pub anchor: [f32; 2],
    pub outward: [f32; 2],
    pub tag: [f32; 2],
    pub arrow_origin: [f32; 2],
    pub angle: f32,
    pub length: f32,
}
pub fn project(clip: [f32; 4], framebuffer: [f32; 2], gui_scale: f32) -> Projected {
    let [x, y, z, w] = clip;
    let mul = 0.5 / gui_scale;
    let screen = (clip.iter().all(|v| v.is_finite()) && w > 0.).then(|| {
        [
            (x / w + 1.) * mul * framebuffer[0],
            (-y / w + 1.) * mul * framebuffer[1],
        ]
    });
    let offset = (x.is_finite() && y.is_finite() && w.is_finite() && w != 0.).then(|| {
        let sign = if w < 0. { -1. } else { 1. };
        [
            x / w * sign * mul * framebuffer[0],
            -y / w * sign * mul * framebuffer[1],
        ]
    });
    let _ = z;
    Projected { screen, offset }
}
pub fn border_point(x: f32, y: f32, hw: f32, hh: f32, margin: f32) -> [f32; 2] {
    let lx = (hw - margin).max(0.);
    let ly = (hh - margin).max(0.);
    let scale = (if x == 0. { f32::INFINITY } else { lx / x.abs() }).min(if y == 0. {
        f32::INFINITY
    } else {
        ly / y.abs()
    });
    if scale.is_finite() {
        [x * scale, y * scale]
    } else {
        [0., 0.]
    }
}
pub fn clamp_to_border(x: f32, y: f32, hw: f32, hh: f32, margin: f32) -> [f32; 2] {
    if x.abs() <= (hw - margin).max(0.) && y.abs() <= (hh - margin).max(0.) {
        [x, y]
    } else {
        border_point(x, y, hw, hh, margin)
    }
}
pub fn fit_inside(min: f32, max: f32, extent: f32) -> f32 {
    if min < 0. {
        -min
    } else if max > extent {
        extent - max
    } else {
        0.
    }
}
#[allow(clippy::too_many_arguments)]
pub fn layout(
    projected: Projected,
    viewport: [f32; 2],
    tag: [f32; 4],
    edge_inset: f32,
    visible: [bool; 2],
    style_length: f32,
    in_view_style_length: f32,
) -> Option<Layout> {
    let center = viewport.map(|v| v * 0.5);
    let offset = projected
        .screen
        .map(|v| [v[0] - center[0], v[1] - center[1]]);
    let in_view = offset.is_some_and(|v| v[0].abs() <= center[0] && v[1].abs() <= center[1]);
    if !visible[usize::from(!in_view)] {
        return None;
    }
    let inset = edge_inset.clamp(0., 0.9);
    let hw = center[0] * (1. - inset);
    let hh = center[1] * (1. - inset);
    let normalize = |v: [f32; 2]| {
        let length = (v[0] * v[0] + v[1] * v[1]).sqrt();
        [v[0] / length, v[1] / length]
    };
    let (anchor, outward) = if let Some(offset) = offset {
        (
            clamp_to_border(offset[0], offset[1], hw, hh, 1.),
            if offset[0] * offset[0] + offset[1] * offset[1] == 0. {
                [0., 1.]
            } else {
                normalize(offset)
            },
        )
    } else {
        let direction = projected.offset?;
        if direction[0] * direction[0] + direction[1] * direction[1] == 0. {
            return None;
        }
        let direction = normalize(direction);
        (
            border_point(direction[0], direction[1], hw, hh, 1.),
            direction,
        )
    };
    let anchor = [anchor[0] + center[0], anchor[1] + center[1]];
    let length = if in_view {
        in_view_style_length
    } else {
        style_length
    };
    let [xmin, ymin, xmax, ymax] = tag;
    let side_x = if outward[0] > 0.1 {
        -xmax
    } else if outward[0] < -0.1 {
        -xmin
    } else {
        0.
    };
    let side_y = if outward[1] > 0.1 {
        -ymax
    } else if outward[1] < -0.1 {
        -ymin
    } else {
        0.
    };
    let base = [
        anchor[0] - outward[0] * (length + 2.) + side_x,
        anchor[1] - outward[1] * (length + 2.) + side_y,
    ];
    Some(Layout {
        in_view,
        anchor,
        outward,
        tag: [
            base[0] + fit_inside(base[0] + xmin, base[0] + xmax, viewport[0]),
            base[1] + fit_inside(base[1] + ymin, base[1] + ymax, viewport[1]),
        ],
        arrow_origin: [
            anchor[0] - outward[0] * length,
            anchor[1] - outward[1] * length,
        ],
        angle: (-outward[0]).atan2(outward[1]),
        length,
    })
}
pub fn transform(origin: [f32; 2], angle: f32, p: [f32; 2]) -> [f32; 2] {
    let (s, c) = (f64::from(angle).sin() as f32, f64::from(angle).cos() as f32);
    [
        origin[0] + c * p[0] - s * p[1],
        origin[1] + s * p[0] + c * p[1],
    ]
}
pub fn arrow_quad(origin: [f32; 2], angle: f32, width: f32, height: f32) -> [[f32; 2]; 4] {
    [
        [-width * 0.5, 0.],
        [width * 0.5, 0.],
        [width * 0.5, height],
        [-width * 0.5, height],
    ]
    .map(|p| transform(origin, angle, p))
}
