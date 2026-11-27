#!/usr/bin/env python3
"""PerspectiveEvent singleton, CameraClip collision phase and raw wheel input.

Apply after camera-control-patch.py. The engine's client_camera_tick handler
executes CameraClip, AutoF5, QuickPerspectiveSwap, then ScrollAdjust final reset.
The former per-query perspective hook remains an unused compatibility surface.
"""

HOOK_NAMES = ['client_camera_tick', 'client_mouse_scroll_raw']
HELPER_NAMES = ['client_keyboard_scan_down']
HOOKS = '''    /// PerspectiveEvent.update at Camera.tick HEAD, once per real client tick.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn native_client_camera_tick(&self, raw_perspective: u8, managed_camera: bool,
        perspective: &mut u8, last_perspective: u8, last_distance: &mut f32,
        distance: &mut f32, no_clip: &mut bool) {
        if self.native_window() {
            mod_api::dispatch_client_camera_tick(self, mod_api::mods(), raw_perspective,
                managed_camera, perspective, last_perspective, last_distance, distance, no_clip);
        }
    }
    /// MouseScrollEvent precedes discrete/sensitivity scaling and all screen branches.
    pub(super) fn native_client_mouse_scroll_raw(&self, vertical: f64) {
        if self.native_window() {
            mod_api::dispatch_client_mouse_scroll_raw(self, mod_api::mods(), vertical);
        }
    }
    /// InputConstants.isKeyDown reads SDL's keyboard array, even for a Key
    /// whose category is MOUSE. Do not use GLFW IDs or mouse-button state here.
    pub(super) fn native_client_keyboard_scan_down(&self, scan: i32) -> bool {
        let Some((_, name)) = horizon_client::key_names::KEYBOARD.iter().find(|(code, _)| *code == scan) else {
            return false;
        };
        self.physical_keys.iter().any(|key| horizon_client::keybinds::key_name(*key) == Some(*name))
    }

'''

SNAPSHOT = '''/// LiquidBounce PerspectiveEvent's singleton fields. Updated only by Camera.tick;
/// every render consumer reads the same result until the next tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PerspectiveSnapshot {
    pub perspective: u8,
    pub distance: f32,
    pub no_clip: bool,
    pub last_perspective: u8,
    pub last_distance: f32,
}
impl PerspectiveSnapshot {
    pub fn update(&mut self, raw_perspective: u8, entity: Option<&CameraEntity>) {
        self.last_distance = self.distance;
        self.last_perspective = self.perspective;
        self.perspective = raw_perspective;
        self.no_clip = false;
        self.distance = perspective_distance(entity);
    }
    /// MixinCamera modifies the getMaxZoom result only when noClip is true.
    /// Equal endpoints skip lerp exactly, including equal signed zero.
    pub fn render_distance(self, partial: f32) -> f32 {
        if self.distance != self.last_distance {
            lerp(partial, self.last_distance, self.distance)
        } else {
            self.distance
        }
    }
}
/// EntityExtensions.cameraDistance takes separate maxima, then multiplies;
/// vanilla Camera instead takes the maximum of the already scaled distances.
pub fn perspective_distance(entity: Option<&CameraEntity>) -> f32 {
    let Some(entity) = entity else { return 4.0; };
    let (mut scale, mut distance) = entity.living.as_ref()
        .map_or((1.0, 4.0), |living| (living.scale, living.camera_distance as f32));
    if let Some((mount_scale, mount_distance)) = entity.mount {
        scale = java_max(scale, mount_scale);
        distance = java_max(distance, mount_distance as f32);
    }
    scale * distance
}

'''

TICK = '''    /// Camera.tick HEAD: preserve the preceding final event before resetting it.
    pub(super) fn tick_native_camera_perspective(&mut self) {
        if !self.native_window() { return; }
        let entity = self.camera_entity().or_else(|| self.player_camera_entity());
        let raw = self.native_raw_camera_perspective();
        let mut event = self.view.perspective_event;
        event.update(raw, entity.as_ref());
        self.native_client_camera_tick(raw, self.studio.flying(), &mut event.perspective,
            event.last_perspective, &mut event.last_distance, &mut event.distance, &mut event.no_clip);
        self.view.perspective_event = event;
    }

'''


def apply(edit, once):
    def native(text):
        anchor = '    /// Publish Aspect before this frame\'s projections, frusta and shader inputs.\n'
        text = once(text, anchor, HOOKS + anchor)
        before = '''        let original = self.native_raw_camera_perspective();
        let mut perspective = original;
        self.native_client_camera_perspective(original, self.studio.flying(), &mut perspective);
        match perspective {'''
        after = '''        if !self.native_window() { return self.camera_type; }
        match self.view.perspective_event.perspective {'''
        text = once(text, before, after)
        return once(text, '''    /// Original PerspectiveEvent consumers share a temporary view; the raw
    /// option is read afresh so F5 continues to work while a HOLD bind is down.
''', '''    /// All PerspectiveEvent consumers share the cached Camera.tick result.
    /// The stored F5 option stays independent and is sampled on the next tick.
''')
    edit('crates/horizon-client/src/game/native_hooks.rs', native)

    def camera(text):
        anchor = '/// `Camera`: the eye-height and field-of-view easing and the view placement.\n'
        text = once(text, anchor, SNAPSHOT + anchor)
        text = once(text, 'pub struct Camera {\n', 'pub struct Camera {\n    pub perspective_event: PerspectiveSnapshot,\n')
        text = once(text, '            eye_height: 0.0,\n', '            perspective_event: PerspectiveSnapshot::default(),\n            eye_height: 0.0,\n')
        return once(text, '''            let zoom = self.max_zoom(distance, &mut clip);
''', '''            let zoom = if self.perspective_event.no_clip {
                self.perspective_event.render_distance(partial)
            } else {
                self.max_zoom(distance, &mut clip)
            };
''')
    edit('crates/horizon-client-render/src/camera.rs', camera)
    edit('crates/horizon-client/src/game/perspective.rs', lambda t: once(t,
        '    pub(super) fn camera_view(&mut self, partial: f32)', TICK + '    pub(super) fn camera_view(&mut self, partial: f32)'))
    edit('crates/horizon-client/src/game/fov.rs', lambda t: once(t,
        '    pub(super) fn tick_fov(&mut self) {\n', '    pub(super) fn tick_fov(&mut self) {\n        self.tick_native_camera_perspective();\n'))
    def remote_camera(text):
        # Remote/spectated entities need the same living-vehicle attributes as
        # the local player. The original module's extension has no local guard.
        text = once(text, '''    pub(super) fn camera_entity(&self) -> Option<CameraEntity> {
        if let Some(m) = self.camera_mob() {
            return Some(mob_camera_entity(m));
        }
''', '''    pub(super) fn camera_entity(&self) -> Option<CameraEntity> {
        let mount = |id: Option<u32>| id.and_then(|id| self.mobs.iter().find(|m| m.id == id))
            .map(|vehicle| (
                vehicle.kind.attributes.get("minecraft:scale").copied().unwrap_or(1.0) as f32,
                vehicle.kind.attributes.get("minecraft:camera_distance").copied().unwrap_or(4.0)));
        if let Some(m) = self.camera_mob() {
            let mut entity = mob_camera_entity(m);
            entity.mount = mount(m.extra.mount);
            return Some(entity);
        }
''')
        return once(text, '            .map(stand_camera_entity)\n', '''            .map(|stand| {
                let mut entity = stand_camera_entity(stand);
                entity.mount = mount(stand.vehicle);
                entity
            })
''')
    edit('crates/horizon-client/src/game/remote_camera.rs', remote_camera)
    edit('crates/horizon-client/src/game.rs', lambda t: once(t,
        '                    // MouseHandler.onScroll: discrete scrolling and the wheel sensitivity.\n',
        '''                    if !self.pad_synthetic_input() {
                        self.native_client_mouse_scroll_raw(raw_y);
                    }
                    // MouseHandler.onScroll: discrete scrolling and the wheel sensitivity.
'''))
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
