#!/usr/bin/env python3
"""HighJump: synchronous PlayerJumpEvent motion and real GameTick state helpers."""
from pathlib import Path
import json
import runpy
HERE=Path(__file__).resolve().parent
method=runpy.run_path(str(HERE/'terrain-speed-order-patch.py'))['method']
HOOK_NAMES=[]
HELPER_NAMES=['client_high_jump_state','client_high_jump_vertical']
def apply(edit,once):
    edit('crates/horizon-client/src/high_jump_bridge.rs',lambda _:(HERE/'high-jump-bridge.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod block_contact;\n','pub mod block_contact;\npub mod high_jump_bridge;\n'))
    def entry(t):
        t=once(t,'    horizon_threads::configure_allocator();','    horizon_client::high_jump_bridge::install(dispatch_high_jump);\n    horizon_threads::configure_allocator();')
        return t+'\n/// Synchronous callback from the actual local PlayerJumpEvent HEAD.\nfn dispatch_high_jump(motion:&mut f32){crate::mod_api::dispatch_client_high_jump(crate::mod_api::mods(),motion);}\n'
    edit('crates/horizon-client/src/entry.rs',entry)
    # A free entry hook needs no Host: it runs before physics with live runtime
    # settings and registers its wait immediately in the common FIFO ticker.
    edit('tools/native-hook-fragments/liquidbounce_high_jump.json',lambda _:json.dumps({'hooks':[dict(name='client_high_jump',target='horizon_client_lib::high_jump_bridge::native_client_high_jump',side='client',cancellable=False,mod_api={'contract':'listen'})],'helpers':[]},indent=2)+'\n')
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'high-jump-host.rs').read_text(encoding='utf-8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    def motion(t):
        t=once(t,'    pub no_jump_delay: i32,\n','    pub no_jump_delay: i32,\n    pub client_high_jump_owner: bool,\n')
        original=method(t,'    pub fn jump_from_ground(&mut self) -> Result<()> {')
        body=once(original,'        living_motion::jump_from_ground(self)?;','''        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){
            // Original mixin evaluates getJumpPower at HEAD; its event may
            // replace the second calculation before threshold/max/sprint.
            let mut power=living_motion::jump_power(self,1.0)?;
            crate::high_jump_bridge::jump(&mut power);
            let _original_second_power=living_motion::jump_power(self,1.0)?;
            living_motion::jump_from_ground_with_power(self,power)?;
        }else{living_motion::jump_from_ground(self)?;}''')
        return once(t,original,body)
    edit('crates/horizon-client/src/motion.rs',motion)
    edit('crates/horizon-client/src/motion/body.rs',lambda t:once(t,'            no_jump_delay: 0,\n','            no_jump_delay: 0,\n            client_high_jump_owner: false,\n'))
    def remote(t):
        t=once(t,'        self.sync_player_body();\n','        self.sync_player_body();\n        let high_jump_owner=self.native_window()&&self.lan_acting_uuid.is_none();\n')
        return once(t,'        let was_gliding = player.fall_flying;\n','        player.client_high_jump_owner=high_jump_owner;\n        let was_gliding = player.fall_flying;\n')
    edit('crates/horizon-client/src/game/remote.rs',remote)
    def living(t):
        old='''pub fn jump_from_ground<W: LivingMotionWorld>(world: &mut W) -> Result<(), W::Error> {
    let power = jump_power(world, 1.0)?;'''
        return once(t,old,'''pub fn jump_from_ground<W: LivingMotionWorld>(world: &mut W) -> Result<(), W::Error> {
    let power=jump_power(world,1.0)?;
    jump_from_ground_with_power(world,power)
}
/// Same LivingEntity.jumpFromGround body, supplied PlayerJumpEvent motion.
pub fn jump_from_ground_with_power<W:LivingMotionWorld>(world:&mut W,power:f32)->Result<(),W::Error>{''')
    edit('crates/horizon-world/src/living_motion.rs',living)
    return dict(hook_names=HOOK_NAMES,helper_names=HELPER_NAMES)
