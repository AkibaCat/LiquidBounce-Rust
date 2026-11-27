#!/usr/bin/env python3
"""FreeLook at MouseHandler's cancel point and Camera.alignWithEntity's early return.

Apply after no-rotate, camera-control and camera-clip. The existing mouse event
is extended, preserving RotationManager's listener even when FreeLook cancels.
"""

HOOKS = '''    /// MixinCamera's post-initial-setPosition injection. The rotation is final,
    /// including FreeLook's front-view inversion; a spectated entity is valid.
    pub(super) fn native_client_free_look_camera(&self, rotation: &mut [f32; 2], apply: &mut bool) {
        if self.native_window() {
            mod_api::dispatch_client_free_look_camera(self, mod_api::mods(), rotation, apply);
        }
    }

'''


def apply(edit, once):
    def native(text):
        text=once(text,
            '    pub(super) fn native_client_mouse_rotation(&self, cursor_x: f64, cursor_y: f64) {',
            '    pub(super) fn native_client_mouse_rotation(&self, cursor_x: f64, cursor_y: f64, canceled: &mut bool) {')
        text=once(text,
            '        mod_api::dispatch_client_mouse_rotation(self, mod_api::mods(), cursor_x, cursor_y);',
            '        mod_api::dispatch_client_mouse_rotation(self, mod_api::mods(), cursor_x, cursor_y, canceled);')
        return once(text,'    /// Publish Aspect before this frame\'s projections, frusta and shader inputs.\n',
            HOOKS+'    /// Publish Aspect before this frame\'s projections, frusta and shader inputs.\n')
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    edit('crates/horizon-client/src/game/pointer_capture.rs',lambda text:once(text,
        '            self.native_client_mouse_rotation(xo * zoom, yo * zoom);\n            self.turn_player(xo * zoom, yo * zoom);\n',
        '            let mut canceled = false;\n            self.native_client_mouse_rotation(xo * zoom, yo * zoom, &mut canceled);\n            if !canceled { self.turn_player(xo * zoom, yo * zoom); }\n'))

    def camera(text):
        before='''        adjust_position: impl FnOnce([f64; 3]) -> [f64; 3],
        mut clip: impl FnMut([f64; 3], [f64; 3]) -> Option<[f64; 3]>,
    ) {
        self.set_rotation(entity.view_y_rot(partial), entity.view_x_rot(partial));'''
        after='''        adjust_position: impl FnOnce([f64; 3]) -> [f64; 3],
        clip: impl FnMut([f64; 3], [f64; 3]) -> Option<[f64; 3]>,
    ) {
        self.align_with_entity_free_look(entity, partial, camera_type, adjust_distance,
            adjust_position, None, clip);
    }
    /// FreeLook's injection returns before ordinary detached/sleeping alignment.
    /// Its getMaxZoom call is inside the injection method, so the separate
    /// alignWithEntity expression hook does not interpolate noClip distance.
    #[allow(clippy::too_many_arguments)]
    pub fn align_with_entity_free_look(
        &mut self,
        entity: &CameraEntity,
        partial: f32,
        camera_type: CameraType,
        adjust_distance: impl FnOnce(f32) -> f32,
        adjust_position: impl FnOnce([f64; 3]) -> [f64; 3],
        free_look_rotation: Option<[f32; 2]>,
        mut clip: impl FnMut([f64; 3], [f64; 3]) -> Option<[f64; 3]>,
    ) {
        self.set_rotation(entity.view_y_rot(partial), entity.view_x_rot(partial));'''
        text=once(text,before,after)
        text=once(text,'        self.position = adjust_position(self.position);\n',
            '        if free_look_rotation.is_none() { self.position = adjust_position(self.position); }\n')
        return once(text,'        self.detached = !camera_type.is_first_person();\n',
            '''        if let Some([yaw, pitch]) = free_look_rotation {
            self.detached = true;
            self.set_rotation(yaw, pitch);
            let scale = entity.living.as_ref().map_or(1.0, |living| living.scale);
            // EntityExtensions.cameraDistance already includes entity/mount
            // scale; the source injection intentionally multiplies it again.
            let distance = self.perspective_event.distance * scale;
            let zoom = if self.perspective_event.no_clip { distance }
                else { self.max_zoom(distance, &mut clip) };
            self.move_by(-zoom, 0.0, 0.0);
            return;
        }
        self.detached = !camera_type.is_first_person();
''')
    edit('crates/horizon-client-render/src/camera.rs',camera)
    def perspective(text):
        text=once(text,'        let mut view = self.view;\n',
            '''        let mut view = self.view;
        let (mut free_look_rotation, mut free_look_apply) = ([0.0; 2], false);
        self.native_client_free_look_camera(&mut free_look_rotation, &mut free_look_apply);
        let free_look_rotation = free_look_apply.then_some(free_look_rotation);
''')
        text=once(text,'        view.align_with_entity_position(\n','        view.align_with_entity_free_look(\n')
        return once(text,'''                position
            },
            |from, to| {''','''                position
            },
            free_look_rotation,
            |from, to| {''')
    edit('crates/horizon-client/src/game/perspective.rs',perspective)
    return {'hook_names':['client_free_look_camera'],'helper_names':[]}
