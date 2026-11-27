#!/usr/bin/env python3
"""Aspect, SmoothCamera and QuickPerspectiveSwap at actual camera consumers.

The stored F5 option stays separate from the temporary effective perspective.
SmoothCamera transforms the interpolated eye before detached collision rays.
Aspect changes world/culling projections; the independent hand projection and
Camera.getNearPlane's fluid samples keep their original dimensions.
"""


def policy_source():
    return '''// SPDX-License-Identifier: GPL-3.0-or-later
//! Aspect snapshot shared by the world renderer and its shader/culling inputs.
use std::sync::atomic::{AtomicI32, Ordering};
static RATIO: AtomicI32 = AtomicI32::new(100);
pub fn publish(ratio_percentage: i32) { RATIO.store(ratio_percentage, Ordering::Release); }
pub fn projection_width(width: f32) -> f32 {
    width / (RATIO.load(Ordering::Acquire) as f32 / 100.0)
}
pub fn aspect(width: f32, height: f32) -> f32 { projection_width(width) / height }
'''


def apply(edit, replace_once):
    def native(text):
        anchor = '    /// The camera this frame is drawn from, `partial` ticks past the last tick, after vanilla\n'
        return replace_once(text, anchor, '''    /// Publish Aspect before this frame's projections, frusta and shader inputs.
    pub(super) fn sync_native_camera_aspect(&self) {
        let mut ratio = 100;
        self.native_client_camera_aspect(&mut ratio);
        horizon_client::camera_control::publish(ratio);
    }
    pub(super) fn native_client_camera_aspect(&self, ratio_percentage: &mut i32) {
        if self.native_window() {
            mod_api::dispatch_client_camera_aspect(self, mod_api::mods(), ratio_percentage);
        }
    }
    /// The unmodified F5 option (0 first, 1 back, 2 front); never stores overrides.
    pub(super) fn native_raw_camera_perspective(&self) -> u8 {
        match self.camera_type {
            super::perspective::CameraType::FirstPerson => 0,
            super::perspective::CameraType::ThirdPersonBack => 1,
            super::perspective::CameraType::ThirdPersonFront => 2,
        }
    }
    pub(super) fn native_client_camera_position(&self, raw_perspective: u8, position: &mut [f64; 3]) {
        if self.native_window() {
            mod_api::dispatch_client_camera_position(self, mod_api::mods(), raw_perspective, position);
        }
    }
    pub(super) fn native_client_camera_perspective(&self, original: u8,
        managed_camera: bool, perspective: &mut u8) {
        if self.native_window() {
            mod_api::dispatch_client_camera_perspective(self, mod_api::mods(), original,
                managed_camera, perspective);
        }
    }
    /// Original PerspectiveEvent consumers share a temporary view; the raw
    /// option is read afresh so F5 continues to work while a HOLD bind is down.
    pub(super) fn effective_camera_type(&self) -> super::perspective::CameraType {
        let original = self.native_raw_camera_perspective();
        let mut perspective = original;
        self.native_client_camera_perspective(original, self.studio.flying(), &mut perspective);
        match perspective {
            0 => super::perspective::CameraType::FirstPerson,
            1 => super::perspective::CameraType::ThirdPersonBack,
            2 => super::perspective::CameraType::ThirdPersonFront,
            _ => self.camera_type,
        }
    }

''' + anchor)

    edit('crates/horizon-client-render/src/camera_control.rs', lambda _: policy_source())
    edit('crates/horizon-client-render/src/lib.rs', lambda t: replace_once(t, 'pub mod camera;\n', 'pub mod camera;\npub mod camera_control;\n'))
    edit('crates/horizon-client/src/client_lib.rs', lambda t: replace_once(t, 'breaking, brewing, camera, chat,', 'breaking, brewing, camera, camera_control, chat,'))
    edit('crates/horizon-client/src/game/native_hooks.rs', native)

    def camera(text):
        signature = '''    pub fn align_with_entity_distance(
        &mut self,
        entity: &CameraEntity,
        partial: f32,
        camera_type: CameraType,
        adjust_distance: impl FnOnce(f32) -> f32,
        mut clip: impl FnMut([f64; 3], [f64; 3]) -> Option<[f64; 3]>,
    ) {'''
        text = replace_once(text, signature, '''    pub fn align_with_entity_distance(
        &mut self,
        entity: &CameraEntity,
        partial: f32,
        camera_type: CameraType,
        adjust_distance: impl FnOnce(f32) -> f32,
        clip: impl FnMut([f64; 3], [f64; 3]) -> Option<[f64; 3]>,
    ) {
        self.align_with_entity_position(entity, partial, camera_type, adjust_distance,
            |position| position, clip);
    }
    /// Modify the entity-aligned eye before third-person collision and sleeping offsets.
    pub fn align_with_entity_position(
        &mut self,
        entity: &CameraEntity,
        partial: f32,
        camera_type: CameraType,
        adjust_distance: impl FnOnce(f32) -> f32,
        adjust_position: impl FnOnce([f64; 3]) -> [f64; 3],
        mut clip: impl FnMut([f64; 3], [f64; 3]) -> Option<[f64; 3]>,
    ) {''')
        text = replace_once(text, '        self.eye = self.position;\n', '        self.position = adjust_position(self.position);\n        self.eye = self.position;\n')
        return replace_once(text,
            '        glam::Mat4::from_cols_array(&perspective(fov_degrees, width, height, 0.05, far, true));',
            '        glam::Mat4::from_cols_array(&perspective(fov_degrees, crate::camera_control::projection_width(width), height, 0.05, far, true));')
    edit('crates/horizon-client-render/src/camera.rs', camera)

    def perspective(text):
        text = replace_once(text, '        let kind = match self.camera_type {', '        let kind = match self.effective_camera_type() {')
        text = replace_once(text, '        view.align_with_entity_distance(\n', '        view.align_with_entity_position(\n')
        text = replace_once(text, '''                distance
            },
            |from, to| {''', '''                distance
            },
            |mut position| {
                self.native_client_camera_position(self.native_raw_camera_perspective(), &mut position);
                position
            },
            |from, to| {''')
        return replace_once(text, '!self.camera_type.first_person() || self.sleeping.is_some() || self.studio.flying()',
            '!self.effective_camera_type().first_person() || self.sleeping.is_some() || self.studio.flying()')
    edit('crates/horizon-client/src/game/perspective.rs', perspective)

    # These are visual reads only. F5 cycling, emote/replay option updates,
    # input sensitivity and the stored camera option continue using the raw field.
    for relative in ('creatures/mob_geometry.rs', 'debug_keys.rs', 'fishing.rs',
            'frame_geometry.rs', 'leads.rs', 'names.rs', 'trident.rs', 'text_displays.rs',
            'render_tick.rs', 'remote_camera.rs'):
        edit('crates/horizon-client/src/game/' + relative,
            lambda t: t.replace('self.camera_type', 'self.effective_camera_type()'))
    edit('crates/horizon-client/src/game/fov.rs', lambda t: replace_once(t,
        '        if !self.camera_type.first_person() {\n', '        if !self.effective_camera_type().first_person() {\n'))

    def game(text):
        text = text.replace('self.camera_type', 'self.effective_camera_type()')
        anchor = '    fn render_frame(&mut self) -> Result<bool> {'
        text = replace_once(text, anchor, anchor + '\n        self.sync_native_camera_aspect();')
        # The two matrices supplied to native shader/post consumers must match
        # the actual renderer's lens. Item activation's independent HUD aspect stays.
        before = '\n                    size[0] as f32 / size[1].max(1) as f32,\n'
        if text.count(before) != 1:
            raise ValueError('Expected one NativeCamera projection aspect')
        text = text.replace(before, '\n                    horizon_client::camera_control::aspect(size[0] as f32, size[1].max(1) as f32),\n')
        before = '\n                        size[0] as f32 / size[1].max(1) as f32,\n'
        return replace_once(text, before, '\n                        horizon_client::camera_control::aspect(size[0] as f32, size[1].max(1) as f32),\n')
    edit('crates/horizon-client/src/game.rs', game)

    edit('crates/horizon-client-render/src/gfx.rs', lambda t: replace_once(t,
        '        let aspect = self.config.width as f32 / self.config.height.max(1) as f32;\n',
        '        let aspect = crate::camera_control::aspect(self.config.width as f32, self.config.height.max(1) as f32);\n'))
    edit('crates/horizon-client-render/src/gfx/native_post.rs', lambda t: replace_once(t,
        '    let aspect = size[0] as f32 / size[1].max(1) as f32;\n',
        '    let aspect = crate::camera_control::aspect(size[0] as f32, size[1].max(1) as f32);\n'))

    return {'hook_names': ['client_camera_aspect', 'client_camera_position', 'client_camera_perspective'],
            'helper_names': ['raw_camera_perspective']}
