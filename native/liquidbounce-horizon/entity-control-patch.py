#!/usr/bin/env python3
"""Exact Mob.isSaddled and LocalPlayer.getJumpRidingScale consumer boundaries.

The Mob getter is process-wide like the original Kotlin singleton. Equipment,
saddle models, save data and packets continue using the actual worn stack.
Apply after mining, which supplies native_client_queries_allowed identity.
"""
from pathlib import Path
HERE = Path(__file__).resolve().parent
HOOKS = '''    /// Snapshot the original process-wide Mob getter's two running flags.
    pub(super) fn native_client_entity_control(&self, saddled: &mut bool, jump_strength: &mut bool) {
        if !self.native_window() || self.lan_acting_uuid.is_some() { return; }
        mod_api::dispatch_client_entity_control(self, mod_api::mods(), saddled, jump_strength);
    }
    pub(super) fn sync_native_entity_control(&self) {
        if !self.native_window() || self.lan_acting_uuid.is_some() { return; }
        let (mut saddled, mut jump_strength) = (false, false);
        self.native_client_entity_control(&mut saddled, &mut jump_strength);
        horizon_client::entity_control::sync(saddled, jump_strength);
    }
    /// getJumpRidingScale belongs to the window's LocalPlayer, never a guest.
    pub(super) fn native_client_entity_control_jump_strength(&self) -> bool {
        self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed())
            && horizon_client::entity_control::force_jump_strength()
    }
    pub(super) fn native_client_riding_jump_scale(&self, original: f32) -> f32 {
        horizon_client::entity_control::jump_scale(original, self.native_client_entity_control_jump_strength())
    }

'''
MOB_GETTER = '''    /// Mob.isSaddled's return modifier; actual equipment remains independent.
    pub fn is_saddled(&self) -> bool {
        crate::entity_control::is_saddled(self.extra.saddled)
    }
'''


def apply(edit, once):
    edit('crates/horizon-client/src/entity_control.rs', lambda _: (HERE / 'entity-control-policy.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs', lambda text: once(text, 'pub mod block_contact;\n', 'pub mod block_contact;\npub mod entity_control;\n'))
    def native(text):
        anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        text = once(text, anchor, HOOKS + anchor)
        return once(text, '        mod_api::dispatch_client_tick_pre(self, mod_api::mods());\n',
            '        mod_api::dispatch_client_tick_pre(self, mod_api::mods());\n        self.sync_native_entity_control();\n')
    edit('crates/horizon-client/src/game/native_hooks.rs', native)
    edit('crates/horizon-client/src/mobs.rs', lambda text: once(text, '    /// The saddle worn, with its components.\n', MOB_GETTER + '    /// The saddle worn, with its components.\n'))
    def equine(text):
        text = once(text, 'Species::Pig | Species::Strider => m.extra.saddled && r.steer,',
            'Species::Pig | Species::Strider => m.is_saddled() && r.steer,')
        text = once(text, '        _ => m.extra.saddled,', '        _ => m.is_saddled(),')
        # Vanilla llamas cannot wear a saddle, which let Horizon omit this
        # otherwise inherited AbstractHorse controller. The mixin removes that
        # assumption, so a forced-saddled llama also runs its inherited ride.
        return once(text,
            '    matches!(s, Species::Pig | Species::Strider) || (is_equine(s) && !super::llama::is_llama(s))\n',
            '    matches!(s, Species::Pig | Species::Strider) || (is_equine(s) && (!super::llama::is_llama(s) || crate::entity_control::is_saddled(false)))\n')
    edit('crates/horizon-client/src/mobs/equine.rs', equine)
    def nautilus(text):
        text = once(text, '    m.rider.is_some() && m.extra.saddled\n', '    m.rider.is_some() && m.is_saddled()\n')
        return once(text, '    if !m.extra.saddled || m.extra.anim_ticks > 0 {\n', '    if !m.is_saddled() || m.extra.anim_ticks > 0 {\n')
    edit('crates/horizon-client/src/mobs/nautilus.rs', nautilus)
    def mounts(text):
        text = once(text, '        if !m.extra.saddled || food || self.keys.held(Action::Sneak) {\n',
            '        if !m.is_saddled() || food || self.keys.held(Action::Sneak) {\n')
        text = once(text, '        let saddled = self.mobs[i].extra.saddled && Self::jumps(self.mobs[i].species());\n',
            '        let saddled = self.mobs[i].is_saddled() && Self::jumps(self.mobs[i].species());\n        let force_jump = self.native_client_entity_control_jump_strength();\n')
        text = once(text, '                let amount = (st.jump_scale * 100.0).floor() as i32;\n',
            '                let amount = (horizon_client::entity_control::jump_scale(st.jump_scale, force_jump) * 100.0).floor() as i32;\n')
        text = once(text, '            .is_some_and(|i| self.mobs[i].extra.saddled && Self::jumps(self.mobs[i].species()))\n',
            '            .is_some_and(|i| self.mobs[i].is_saddled() && Self::jumps(self.mobs[i].species()))\n')
        text = once(text, '        self.mount.jump_scale > 0.0 || self.mount.jump_ticks < 0\n',
            '        self.native_client_riding_jump_scale(self.mount.jump_scale) > 0.0 || self.mount.jump_ticks < 0\n')
        text = once(text, '        let draws = horizon_client::hud_overlays::jump_bar(w, h, cooldown, self.mount.jump_scale);\n',
            '        let draws = horizon_client::hud_overlays::jump_bar(w, h, cooldown, self.native_client_riding_jump_scale(self.mount.jump_scale));\n')
        return once(text, '    if data <= 0 || !m.extra.saddled {\n', '    if data <= 0 || !m.is_saddled() {\n')
    edit('crates/horizon-client/src/game/mounts.rs', mounts)
    def remote(text):
        text = once(text, '            self.send(riding_jump(id, self.mount.jump_scale));\n',
            '            self.send(riding_jump(id, self.native_client_riding_jump_scale(self.mount.jump_scale)));\n')
        return once(text, '    if equine::moved_by_rider(m.species()) && m.extra.saddled {\n',
            '''    if (equine::moved_by_rider(m.species())
            || (horizon_client::entity_control::is_saddled(false)
                && horizon_client::mobs::nautilus::is_nautilus(m.species()))) && m.is_saddled() {
''')
    edit('crates/horizon-client/src/game/remote_vehicles.rs', remote)
    edit('crates/horizon-client/src/game/remote_mob_use.rs', lambda text: once(text,
        '                m.extra.saddled\n                    && !sneaking\n',
        '                m.is_saddled()\n                    && !sneaking\n'))
    return {'hook_names': ['client_entity_control'], 'helper_names': []}
