#!/usr/bin/env python3
"""NoRotateSet exact post-vanilla correction phases and local old-rot writes."""
import json

HELPERS = '''    pub(super) fn native_client_no_rotate_capture(&self, running: &mut bool) {
        mod_api::dispatch_client_no_rotate_capture(self, mod_api::mods(), running);
    }
    /// WrapMethod(handleRotatePlayer) captures eligibility before vanilla.
    pub(super) fn native_no_rotate_previous(&self) -> Option<[f32; 2]> {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|player| player.native_client_queries_allowed())
            || self.remote.as_ref().is_some_and(|remote| remote.level_load.is_active()) {
            return None;
        }
        let mut running = false;
        self.native_client_no_rotate_capture(&mut running);
        running.then_some([self.yaw, self.pitch])
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn native_client_server_rotation(&self, previous_yaw: f32, previous_pitch: f32, corrected_yaw: f32, corrected_pitch: f32, preaccepted: bool, yaw: &mut f32, pitch: &mut f32, restore: &mut bool) {
        mod_api::dispatch_client_server_rotation(self, mod_api::mods(), previous_yaw, previous_pitch,
            corrected_yaw, corrected_pitch, preaccepted, yaw, pitch, restore);
    }
    /// Position checks running/loading at RETURN. Rotation already checked
    /// them in its wrapper, before vanilla and its synchronous acknowledgement.
    pub(super) fn native_no_rotate_apply(&mut self, previous_yaw: f32, previous_pitch: f32, preaccepted: bool) {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|player| player.native_client_queries_allowed()) {
            return;
        }
        let loading = self.remote.as_ref().is_some_and(|remote| remote.level_load.is_active());
        if !preaccepted && loading { return; }
        let (mut yaw, mut pitch, mut restore) = (self.yaw, self.pitch, false);
        self.native_client_server_rotation(previous_yaw, previous_pitch,
            self.yaw, self.pitch, preaccepted, &mut yaw, &mut pitch, &mut restore);
        if !restore { return; }
        // Entity.setYRot/setXRot retain finite writes, then setOldRot writes
        // both old angles to the actual restored current angles.
        if yaw.is_finite() { self.yaw = yaw; }
        if pitch.is_finite() { self.pitch = horizon_client::packet_bridge::set_x_rot(pitch); }
        if let Some(player) = self.player.as_mut() {
            player.yaw = self.yaw;
            player.pitch = self.pitch;
        }
        self.rope_rot_old = (self.yaw, self.pitch);
        if let Some(model) = self.player_model.as_mut() {
            model.yaw = self.yaw;
            model.previous_yaw = self.yaw;
            model.head_pitch = self.pitch;
            model.previous_head_pitch = self.pitch;
        }
    }
    pub(super) fn native_client_player_last_rotation(&self) -> [f32; 2] {
        [self.rope_rot_old.0, self.rope_rot_old.1]
    }
    /// MouseRotationEvent fires only for MouseHandler, after its sensitivity
    /// and zoom scaling, immediately before the actual Entity.turn.
    pub(super) fn native_client_mouse_rotation(&self, cursor_x: f64, cursor_y: f64) {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|player| player.native_client_queries_allowed()) {
            return;
        }
        mod_api::dispatch_client_mouse_rotation(self, mod_api::mods(), cursor_x, cursor_y);
    }
    pub(super) fn native_client_change_look(&self, partial: f32, yaw: &mut f32, pitch: &mut f32, apply: &mut bool) {
        mod_api::dispatch_client_change_look(self, mod_api::mods(), partial, yaw, pitch, apply);
    }
    /// GameRenderer.update HEAD, once before this frame's camera extraction.
    pub(super) fn native_change_look_now(&mut self, partial: f32) {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|player| player.native_client_queries_allowed()) {
            return;
        }
        let (mut yaw, mut pitch, mut apply) = (self.yaw, self.pitch, false);
        self.native_client_change_look(partial, &mut yaw, &mut pitch, &mut apply);
        if !apply { return; }
        // LocalPlayer.setRotation snapshots both old angles and yBob before
        // applying the normalized result. No body/head-yaw solver is run.
        let (old_yaw, old_pitch) = (self.yaw, self.pitch);
        self.rope_rot_old = (old_yaw, old_pitch);
        self.sway.reset_yaw(old_yaw);
        if yaw.is_finite() { self.yaw = yaw; }
        if pitch.is_finite() { self.pitch = horizon_client::packet_bridge::set_x_rot(pitch); }
        if let Some(player) = self.player.as_mut() {
            player.yaw = self.yaw;
            player.pitch = self.pitch;
        }
        if let Some(model) = self.player_model.as_mut() {
            model.previous_yaw = old_yaw;
            model.yaw = self.yaw;
            model.previous_head_pitch = old_pitch;
            model.head_pitch = self.pitch;
        }
    }

'''


def function(text, signature):
    start = text.index(signature)
    end = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return start, end


def apply(edit, replace_once):
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: replace_once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HELPERS + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))

    def position(text):
        start, end = function(text, '    fn remote_teleport(')
        body = text[start:end]
        body = replace_once(body, '        self.yaw = yaw;\n        self.pitch = pitch;\n',
            '        let no_rotate_previous = (self.yaw, self.pitch);\n        self.yaw = yaw;\n        self.pitch = pitch;\n')
        # Attack's AbortBreaking extension may already own the vanilla clear.
        # Keep the correction at the same successful handler return in either
        # the standalone fixture or the complete production transform.
        clear = '        if !retain_break { self.mining = None; }\n' if '        if !retain_break { self.mining = None; }\n' in body else '        self.mining = None;\n'
        body = replace_once(body, clear + '        Ok(())\n',
            clear + '        self.native_no_rotate_apply(no_rotate_previous.0, no_rotate_previous.1, false);\n        Ok(())\n')
        return text[:start] + body + text[end:]
    edit('crates/horizon-client/src/game/remote.rs', position)

    def rotation(text):
        start, end = function(text, '    pub(super) fn remote_rotation(')
        body = text[start:end]
        body = replace_once(body, '        self.yaw = if p.relative_yaw {\n',
            '        let no_rotate_previous = self.native_no_rotate_previous();\n        self.yaw = if p.relative_yaw {\n')
        body = replace_once(body, '            flags: 0,\n        }));\n',
            '            flags: 0,\n        }));\n        if let Some([yaw, pitch]) = no_rotate_previous {\n            self.native_no_rotate_apply(yaw, pitch, true);\n        }\n')
        return text[:start] + body + text[end:]
    edit('crates/horizon-client/src/game/remote_abilities.rs', rotation)
    edit('crates/horizon-client/src/game/pointer_capture.rs', lambda text: replace_once(text,
        '            self.look_turn(xo, yo);\n',
        '            self.tutorial.mouse(xo, yo);\n            let zoom = self.zoom_turn_scale();\n            self.native_client_mouse_rotation(xo * zoom, yo * zoom);\n            self.turn_player(xo * zoom, yo * zoom);\n'))
    edit('crates/horizon-client/src/game.rs', lambda text: replace_once(text,
        '        let vanilla = self.camera_view(camera_alpha as f32);\n',
        '        self.native_change_look_now(camera_alpha as f32);\n        let vanilla = self.camera_view(camera_alpha as f32);\n'))
    hooks = ['client_no_rotate_capture', 'client_server_rotation', 'client_mouse_rotation', 'client_change_look']
    fragment = {
        'hooks': [{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}",'side':'client','cancellable':False,'mod_api':{'contract':'listen'}} for name in hooks],
        'helpers': [{'name':'client_player_last_rotation','target':"horizon_client::game::native_hooks::Game<'_>::native_client_player_last_rotation"}],
        'rust_summary':'NoRotateSet after accepted position and rotation corrections, preserving vanilla acknowledgement and local old angles.',
        'doc_summary':'Rotation eligibility is captured before vanilla; position eligibility is checked at return. Loading screens and guest/server/replay players bypass it.',
    }
    return {'tools/native-hook-fragments/liquidbounce_no_rotate.json':json.dumps(fragment,indent=2)+'\n'}
