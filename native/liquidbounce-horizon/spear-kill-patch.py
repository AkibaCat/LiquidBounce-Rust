#!/usr/bin/env python3
"""SpearKill native used-stack, Collider rays, player forecast and preview."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,anchor,(HERE/'spear-kill-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    for name in ('target','ray','forecast'):
        edit(f'crates/horizon-client/src/game/spear_kill_{name}.rs',lambda _,name=name:(HERE/f'spear-kill-{name}.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda text:once(text,'mod native_hooks;\n','mod native_hooks;\nmod spear_kill_target;\nmod spear_kill_ray;\nmod spear_kill_forecast;\n'))
    edit('crates/horizon-client/src/game/hand.rs',lambda text:once(text,'    fn stack_attack_range(&self, stack: &ItemStack)', '    pub(super) fn stack_attack_range(&self, stack: &ItemStack)'))
    edit('crates/horizon-client/src/game/remote.rs',lambda text:once(text,'    fn release_remote_use(&mut self) {','    pub(super) fn release_remote_use(&mut self) {'))
    anchor='    pub fn replayed_tick(&mut self, keys: crate::sprint::Keys, sprinting: bool) -> Result<()> {\n'
    edit('crates/horizon-client/src/motion.rs',lambda text:once(text,anchor,(HERE/'spear-kill-motion.rs').read_text(encoding='utf8')+'\n'+anchor))
    def mobs(text):
        text=once(text,'    pub fall_flying: bool,\n','    pub liquidbounce_swimming: bool,\n    pub liquidbounce_vertical_collision: bool,\n    pub liquidbounce_was_in_powder_snow: bool,\n    pub fall_flying: bool,\n')
        return once(text,'    pub(crate) fn effect_instance(&self, index: usize)', '    pub fn effect_instance(&self, index: usize)')
    edit('crates/horizon-client/src/mobs.rs',mobs)
    anchor='        let vertical = wanted.y != moved.y;\n        self.vertical_collision_below = vertical && wanted.y < 0.0;\n'
    edit('crates/horizon-client/src/mobs/living.rs',lambda text:once(text,anchor,'        let vertical = wanted.y != moved.y;\n        self.extra.liquidbounce_vertical_collision = vertical;\n        self.vertical_collision_below = vertical && wanted.y < 0.0;\n'))
    edit('crates/horizon-client/src/mobs/ai.rs',lambda text:once(text,'    m.in_powder_snow = in_snow;\n','    m.extra.liquidbounce_was_in_powder_snow = m.in_powder_snow;\n    m.in_powder_snow = in_snow;\n'))
    anchor='                m.extra.sprinting = flags & 8 != 0;\n'
    edit('crates/horizon-client/src/game/remote_mob_data.rs',lambda text:once(text,anchor,anchor+'                m.extra.liquidbounce_swimming = flags & 16 != 0;\n'))
    return {'hook_names':[],'helper_names':['client_spear_kill_context','client_spear_kill_restart','client_spear_kill_target','client_spear_kill_predict','client_spear_kill_eye','client_spear_kill_look','client_spear_kill_velocity','client_spear_kill_boxes']}
