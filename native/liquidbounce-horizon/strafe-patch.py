#!/usr/bin/env python3
"""Complete SELF PlayerMoveEvent HEAD modifier. Apply after mining identity hook."""
from pathlib import Path
HERE = Path(__file__).resolve().parent
HOOKS = '''    /// Capture full raw settings and the untransformed directional flags.
    pub(super) fn native_client_strafe(&self, air: &mut f32, ground: &mut f32, strict: &mut bool, input: &mut u8, apply: &mut bool) {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed()) { return; }
        mod_api::dispatch_client_strafe(self, mod_api::mods(), air, ground, strict, input, apply);
    }

'''
MODIFIER = '''    /// PlayerMoveEvent runs before Entity.move, including noPhysics moves.
    fn client_strafe_movement(&self, movement: Vec3) -> Vec3 {
        if !self.player.client_side || !self.player.native_client_queries_allowed() { return movement; }
        let Some((strafe, input)) = self.player.client_strafe else { return movement; };
        let [x,y,z] = strafe.move_event(true, crate::strafe::StrafeContext {
            self_movement: true,
            on_ground: self.player.ground.on_ground,
            moving: self.player.move_vector[0] != 0.0 || self.player.move_vector[1] != 0.0,
            input,
            yaw: self.player.yaw,
        }, [movement.x,movement.y,movement.z], |angle|
            [horizon_core::jmath::sin(angle),horizon_core::jmath::cos(angle)]);
        Vec3::new(x,y,z)
    }
'''
CAPTURE = '''        let (mut strafe_air, mut strafe_ground, mut strafe_strict, mut strafe_input, mut strafe_apply) = (1.0, 1.0, false, 0, false);
        self.native_client_strafe(&mut strafe_air, &mut strafe_ground, &mut strafe_strict, &mut strafe_input, &mut strafe_apply);
        let client_strafe = strafe_apply.then_some((horizon_client::strafe::Strafe {
            strength_in_air: strafe_air, strength_on_ground: strafe_ground, strict_movement: strafe_strict,
        }, strafe_input));
'''


def apply(edit, once):
    edit('crates/horizon-client/src/strafe.rs', lambda _: (HERE.parent/'liquidbounce-behaviors/src/strafe.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs', lambda t: once(t,'pub mod block_contact;\n','pub mod block_contact;\npub mod strafe;\n'))
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda t: once(t,anchor,HOOKS+anchor))
    def motion(text):
        text = once(text,'    pub no_jump_delay: i32,\n','    pub no_jump_delay: i32,\n    pub client_strafe: Option<(crate::strafe::Strafe, u8)>,\n')
        anchor = '    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {\n'
        text = once(text,anchor,MODIFIER+anchor)
        return once(text,anchor,anchor+'        let movement = self.client_strafe_movement(movement);\n')
    edit('crates/horizon-client/src/motion.rs',motion)
    edit('crates/horizon-client/src/motion/body.rs',lambda t: once(t,'            no_jump_delay: 0,\n','            no_jump_delay: 0,\n            client_strafe: None,\n'))
    def remote(text):
        text = once(text,'        self.sync_player_body();\n','        self.sync_player_body();\n'+CAPTURE)
        return once(text,'        let was_gliding = player.fall_flying;\n',
            '        player.client_strafe = client_strafe;\n        let was_gliding = player.fall_flying;\n')
    edit('crates/horizon-client/src/game/remote.rs',remote)
    return {'hook_names':['client_strafe'],'helper_names':[]}
