// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{hud::HudBatch, text::Text};
use std::cell::RefCell;
#[path = "indicator_math.rs"]
mod math;
pub(super) enum Arrow {
    None,
    Triangle { width: f32, height: f32, notch: f32 },
    Atlas { size: f32, uv: [[f32; 2]; 4] },
}
impl Arrow {
    fn length(&self) -> f32 {
        match self {
            Self::None => 0.,
            Self::Triangle { height, .. } => *height,
            Self::Atlas { size, .. } => *size,
        }
    }
}
pub(super) enum Primitive {
    Text {
        text: Text,
        origin: [f32; 2],
        scale: f32,
        color: u32,
        shadow: bool,
    },
    Quad {
        corners: [[f32; 2]; 4],
        uv: [[f32; 2]; 4],
        color: u32,
    },
}
pub(super) struct Entry {
    pub position: [f64; 3],
    pub priority: i32,
    pub color: u32,
    pub visible: [bool; 2],
    pub edge_inset: f32,
    pub style: Arrow,
    pub in_view_style: Arrow,
    pub bounds: [f32; 4],
    pub tag: Vec<Primitive>,
}
thread_local! {static SUBMITTED:RefCell<Vec<Entry>>=const{RefCell::new(Vec::new())};}
pub(super) fn clear() {
    SUBMITTED.with_borrow_mut(Vec::clear);
    horizon_client::gfx::indicator::clear();
}
pub(super) fn submit(entry: Entry) {
    SUBMITTED.with_borrow_mut(|v| v.push(entry));
}
fn native_color(argb: u32) -> u32 {
    let [a, r, g, b] = argb.to_be_bytes();
    u32::from_le_bytes([r, g, b, a])
}
impl Game<'_> {
    pub(super) fn client_indicator_overlay(&self, batch: &mut HudBatch) {
        let mut entries = SUBMITTED.with_borrow_mut(std::mem::take);
        let Some(camera) = self.native_camera() else {
            return;
        };
        let Some(window) = self.native_physical_window_size() else {
            return;
        };
        let matrix = glam::Mat4::from_cols_array(&camera.projection)
            * glam::Mat4::from_cols_array(&camera.view);
        entries.sort_by_key(|v| v.priority);
        for entry in entries {
            let clip = matrix
                * glam::Vec4::new(
                    (entry.position[0] - camera.position[0]) as f32,
                    (entry.position[1] - camera.position[1]) as f32,
                    (entry.position[2] - camera.position[2]) as f32,
                    1.,
                );
            let projected = math::project(clip.to_array(), window.map(|v| v as f32), batch.scale);
            let Some(layout) = math::layout(
                projected,
                [batch.width, batch.height],
                entry.bounds,
                entry.edge_inset,
                entry.visible,
                entry.style.length(),
                entry.in_view_style.length(),
            ) else {
                continue;
            };
            for primitive in entry.tag {
                match primitive {
                    Primitive::Text {
                        text,
                        origin,
                        scale,
                        color,
                        shadow,
                    } => {
                        let first = batch.vertices.len();
                        batch.text_styled(
                            &self.gui.atlas,
                            &text,
                            origin[0],
                            origin[1],
                            native_color(color),
                            shadow,
                        );
                        for v in &mut batch.vertices[first..] {
                            v.pos = [
                                layout.tag[0] + v.pos[0] * scale,
                                layout.tag[1] + v.pos[1] * scale,
                            ];
                        }
                    }
                    Primitive::Quad { corners, uv, color } => batch.quad(
                        corners.map(|p| [p[0] + layout.tag[0], p[1] + layout.tag[1]]),
                        uv,
                        native_color(color),
                    ),
                }
            }
            let style = if layout.in_view {
                entry.in_view_style
            } else {
                entry.style
            };
            match style {
                Arrow::None => {}
                Arrow::Triangle {
                    width,
                    height,
                    notch,
                } => horizon_client::gfx::indicator::submit(
                    layout.arrow_origin,
                    layout.angle,
                    [width, height],
                    notch,
                    entry.color,
                    [batch.width, batch.height],
                ),
                Arrow::Atlas { size, uv } => batch.quad(
                    math::arrow_quad(layout.arrow_origin, layout.angle, size, size),
                    uv,
                    native_color(entry.color),
                ),
            }
        }
        horizon_client::gfx::indicator::finish_layer(batch.vertices.len());
    }
}
