#!/usr/bin/env python3
"""SuperKnockback's real avatar/network sprint state and attack/critical facts."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 def native(t):
  t=once(t,anchor,(HERE/'super-knockback-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  return once(t,'        mod_api::dispatch_client_player_tick_post(self, mod_api::mods());','        mod_api::dispatch_client_player_tick_post(self, mod_api::mods());\n        self.sync_native_super_knockback_sprint();')
 edit('crates/horizon-client/src/game/native_hooks.rs',native)
 edit('crates/horizon-client/src/game/remote.rs',lambda t:once(t,'    was_sprinting: bool,\n','    pub(super) was_sprinting: bool,\n'))
 edit('crates/horizon-client/src/motion.rs',lambda t:once(t,'    pub sprinting: bool,\n','    pub sprinting: bool,\n    pub client_super_knockback_was_sprinting: bool,\n'))
 edit('crates/horizon-client/src/motion/body.rs',lambda t:once(t,'            sprinting: false,\n','            sprinting: false,\n            client_super_knockback_was_sprinting: false,\n'))
 return {'hook_names':[],'helper_names':['client_super_knockback_context','client_super_knockback_packet','client_super_knockback_mark_sprint']}
