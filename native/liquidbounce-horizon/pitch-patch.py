#!/usr/bin/env python3
"""NoPitchLimit's exact-source Entity hooks; never mutates upstream on import."""
import json

TURN = '''fn native_turn_rotation(rotation: &mut Rotation, xo: f64, yo: f64) {
    if !horizon_client::packet_bridge::pitch_limit_removed() {
        rotation.turn(xo, yo);
        return;
    }
    // Entity.turn: casts happen before * 0.15. MixinEntity removes both
    // clamps and writes setXRot's original raw argument, without % 360.
    let x_delta = (yo as f32) * 0.15;
    let y_delta = (xo as f32) * 0.15;
    let pitch = rotation.x_rot + x_delta;
    if pitch.is_finite() {
        rotation.x_rot = pitch;
    }
    let yaw = rotation.y_rot + y_delta;
    if yaw.is_finite() {
        rotation.y_rot = yaw;
    }
    rotation.x_rot_o += x_delta;
    rotation.y_rot_o += y_delta;
}

'''

RESET = '''    /// ModuleNoPitchLimit.onDisabled writes only the client's current xRot.
    /// Do not wrap modulo 360, reset xRotO, or emit a rotation packet here.
    pub(super) fn native_client_reset_pitch(&mut self) -> bool {
        if !self.native_window() || self.player.is_none() { return false; }
        self.pitch = self.pitch.clamp(-90.0, 90.0);
        if let Some(model) = self.player_model.as_mut() {
            model.head_pitch = self.pitch;
        }
        true
    }

'''


def apply(edit, replace_once) -> dict[str, str]:
    """Cumulative transformations; caller owns added-file patch assembly."""
    def perspective(text):
        original = "    rotation.turn(xo, yo);\n"
        if text.count(original) != 2:
            raise ValueError("Expected Entity.turn at both view and rope old rotation paths")
        text = text.replace(original, "    native_turn_rotation(&mut rotation, xo, yo);\n")
        return replace_once(text, "fn turn_rotation(yaw: f32,", TURN + "fn turn_rotation(yaw: f32,")

    edit("crates/horizon-client/src/game/perspective.rs", perspective)

    def entities(text):
        original = "pitch.clamp(-90.0, 90.0)"
        if text.count(original) != 11:
            raise ValueError("Expected eleven Entity.setXRot storage paths")
        return text.replace(original, "horizon_client::packet_bridge::pitch_limit(pitch, -90.0, 90.0)")

    edit("crates/horizon-client/src/game/entity.rs", entities)
    edit("crates/horizon-client/src/game/commands.rs", lambda text: replace_once(text,
        "fn entity_pitch(pitch: f32) -> f32 {\n    (pitch % 360.0).clamp(-90.0, 90.0)\n}",
        "fn entity_pitch(pitch: f32) -> f32 {\n    horizon_client::packet_bridge::set_x_rot(pitch)\n}"))

    def remote(text):
        text = replace_once(text, "    m.head_pitch = (at.pitch % 360.0).clamp(-90.0, 90.0);", "    m.head_pitch = horizon_client::packet_bridge::set_x_rot(at.pitch);")
        return replace_once(text, "                        m.head_pitch = (pitch % 360.0).clamp(-90.0, 90.0);", "                        m.head_pitch = horizon_client::packet_bridge::set_x_rot(pitch);")

    edit("crates/horizon-client/src/game/remote.rs", remote)
    edit("crates/horizon-client/src/game/remote_abilities.rs", lambda text: replace_once(text,
        '''        self.pitch = (if p.relative_pitch {
            self.pitch + p.pitch
        } else {
            p.pitch
        })
        .clamp(-90.0, 90.0);''',
        '''        self.pitch = horizon_client::packet_bridge::pitch_limit(if p.relative_pitch {
            self.pitch + p.pitch
        } else {
            p.pitch
        }, -90.0, 90.0);'''))
    edit("crates/horizon-client/src/game/control.rs", lambda text: replace_once(text,
        "                self.pitch = arg(2)?.parse::<f32>()?.clamp(-90.0, 90.0);",
        "                self.pitch = horizon_client::packet_bridge::pitch_limit(arg(2)?.parse::<f32>()?, -90.0, 90.0);"))
    edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: replace_once(text,
        "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n",
        RESET + "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n"))
    fragment = {
        "helpers": [{"name": "client_reset_pitch", "target": "horizon_client::game::native_hooks::Game<'_>::native_client_reset_pitch"}],
        "hooks": [],
        "rust_summary": "NoPitchLimit raw Entity.turn, Entity setXRot and client rotation setter paths; actor-only disable clamp.",
        "doc_summary": "Exact pinned-source extension. PositionMoveRotation.calculateAbsolute keeps its independent pre-setXRot clamp, matching the original mixin targets.",
    }
    return {"tools/native-hook-fragments/liquidbounce_pitch.json": json.dumps(fragment, indent=2) + "\n"}
