#!/usr/bin/env python3
"""Real click queues, isDown composition and source-hook local FallingPlayer."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'crosshair-clicker-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    edit('crates/horizon-client/src/game/crosshair_falling.rs',lambda _:(HERE/'crosshair-falling.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/crosshair_falling_bridge.rs',lambda _:(HERE/'crosshair-falling-bridge.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod spear_kill_forecast;\n','mod spear_kill_forecast;\nmod crosshair_falling;\n'))
    edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod keymapping_clicks;\n','pub mod keymapping_clicks;\npub mod crosshair_falling_bridge;\npub use horizon_core::jmath as clicker_math;\n'))
    edit('crates/horizon-client/src/motion.rs',lambda s:once(s,'    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {\n','    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {\n        let movement=crate::crosshair_falling_bridge::movement(self,movement)?;\n'))
    # AutoClicker and TriggerBot are constructed before AutoBreak. Their isDown
    # listeners therefore run first, then the genuine AutoBreak listener.
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'        self.native_client_auto_break_key(true, miss_time, block_hit, block_hit, air, destroying, &mut pressed, &mut click);\n','        self.native_client_crosshair_clicker_attack_key(&mut pressed);\n        self.native_client_auto_break_key(true, miss_time, block_hit, block_hit, air, destroying, &mut pressed, &mut click);\n'))
    edit('crates/horizon-client/src/navigation_physics.rs',lambda s:once(s,'pub fn set_module_safe_walk(value:bool){MODULE_SAFE_WALK.set(value);}','pub fn set_module_safe_walk(value:bool){MODULE_SAFE_WALK.set(value);}\npub fn module_safe_walk()->bool{MODULE_SAFE_WALK.get()}'))
    return {'hook_names':['client_crosshair_clicker_attack_key'],'helper_names':['client_crosshair_clicker_context','client_crosshair_clicker_blocks','client_crosshair_clicker_critical','client_crosshair_clicker_target','client_crosshair_clicker_click','client_crosshair_clicker_release','client_crosshair_clicker_falling']}
