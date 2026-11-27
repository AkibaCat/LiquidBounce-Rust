#!/usr/bin/env python3
"""Full BlockBounce at PlayerJumpEvent's getJumpPower expression boundary.

Apply after movement-patch and mining-patch. The module configuration is captured from the real
client, but bouncy-block detection uses the actual position at each jump.
Shared living entities inherit an identity modifier and preserve their power.
"""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HOOKS='''    /// Configuration snapshot for the actual local player's jump-power event.
    pub(super) fn native_client_block_bounce(&self, motion: &mut f32, apply: &mut bool) {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|player| player.native_client_queries_allowed()) { return; }
        mod_api::dispatch_client_block_bounce(self, mod_api::mods(), motion, apply);
    }

'''
MODIFIER='''    fn modify_jump_power(&mut self, power: f32) -> Result<f32> {
        if !self.player.client_side || !self.player.native_client_queries_allowed() { return Ok(power); }
        let Some(motion) = self.player.client_block_bounce_motion else { return Ok(power); };
        let bounds = self.player.bounds();
        let min = bounds.min_position();
        let max = bounds.max_position();
        let on_bouncy = crate::block_bounce::standing_on_bouncy_block(
            [min.x, min.y, min.z], [max.x, max.y, max.z], |[x, y, z]| {
                self.assets.registry.state(self.block(BlockPos::new(x, y, z)))
                    .is_some_and(|state| crate::block_bounce::is_bouncy_block(&state.name))
            });
        Ok(crate::block_bounce::BlockBounce { motion }.jump_power(true, on_bouncy, power))
    }
'''


def apply(edit,once):
    edit('crates/horizon-client/src/block_bounce.rs',lambda _: (HERE.parent/'liquidbounce-behaviors/src/block_bounce.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs',lambda text:once(text,'pub mod block_contact;\n','pub mod block_contact;\npub mod block_bounce;\n'))
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,anchor,HOOKS+anchor))
    def motion(text):
        text=once(text,'    pub no_jump_delay: i32,\n','    pub no_jump_delay: i32,\n    pub client_block_bounce_motion: Option<f32>,\n')
        return once(text,'    fn jump_strength(&mut self) -> Result<f64> {\n',MODIFIER+'    fn jump_strength(&mut self) -> Result<f64> {\n')
    edit('crates/horizon-client/src/motion.rs',motion)
    edit('crates/horizon-client/src/motion/body.rs',lambda text:once(text,'            no_jump_delay: 0,\n',
        '            no_jump_delay: 0,\n            client_block_bounce_motion: None,\n'))
    def living(text):
        text=once(text,'    fn jump_strength(&mut self) -> Result<f64, Self::Error>;\n',
            '''    fn jump_strength(&mut self) -> Result<f64, Self::Error>;
    /// PlayerJumpEvent changes the final Float getJumpPower value. Ordinary
    /// living worlds keep the original value through this default method.
    fn modify_jump_power(&mut self, power: f32) -> Result<f32, Self::Error> { Ok(power) }
''')
        return once(text,'    let power = jump_power(world, 1.0)?;\n',
            '    let power = jump_power(world, 1.0)?;\n    let power = world.modify_jump_power(power)?;\n')
    edit('crates/horizon-world/src/living_motion.rs',living)
    def remote(text):
        text=once(text,'        self.sync_player_body();\n',
            '''        self.sync_player_body();
        let (mut block_bounce_motion, mut block_bounce_apply) = (0.0, false);
        self.native_client_block_bounce(&mut block_bounce_motion, &mut block_bounce_apply);
        let block_bounce_motion = block_bounce_apply.then_some(block_bounce_motion);
''')
        return once(text,'        let was_gliding = player.fall_flying;\n',
            '        player.client_block_bounce_motion = block_bounce_motion;\n        let was_gliding = player.fall_flying;\n')
    edit('crates/horizon-client/src/game/remote.rs',remote)
    return {'hook_names':['client_block_bounce'],'helper_names':[]}
