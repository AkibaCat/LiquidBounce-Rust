#!/usr/bin/env python3
"""Actual death-screen opening events and LocalPlayer.respawn helpers."""
import json


HELPERS = '''    /// ScreenEvent at setScreen(DeathScreen) HEAD; it is not cancellation.
    pub(super) fn native_client_death_screen_opening(&mut self) {
        if self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed()) {
            mod_api::dispatch_client_death_screen_opening(self, mod_api::mods());
        }
    }
    pub(super) fn native_client_is_death_screen(&self) -> bool {
        self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed())
            && self.screen == Screen::Dead
    }
    /// AutoRespawn calls LocalPlayer.respawn(), then Gui.setScreen(null).
    /// Zero-delay ScreenEvent runs may call this before DeathScreen is assigned.
    pub(super) fn native_client_respawn(&mut self) -> bool {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed()) {
            return false;
        }
        if self.respawn_player().is_err() { return false; }
        self.screen = Screen::Playing;
        self.capture(true);
        true
    }

'''


def apply(edit, replace_once):
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: replace_once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HELPERS + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))

    def remote(text):
        anchor = '                    self.screen = Screen::Dead;\n'
        if text.count(anchor) != 2:
            raise ValueError('Expected both SetHealth and PlayerCombatKill death-screen opens')
        return text.replace(anchor, '                    self.native_client_death_screen_opening();\n' + anchor)
    edit('crates/horizon-client/src/game/remote.rs', remote)

    def spawn(text):
        # Vanilla's ScreenEvent already has the actual local player installed.
        # Horizon originally opened a saved-death screen before assigning it.
        block = '''        // A saved death already dropped inventory/XP and counted the death.
        if player.health.dead_or_dying() {
            self.screen = Screen::Dead;
            self.death_focus = None;
            self.ui.died_at = Some(Instant::now());
            self.ui.death_text = None;
            self.ui.death_confirm = None;
            self.capture(false);
        }
'''
        text = replace_once(text, block, '        let saved_death = player.health.dead_or_dying();\n')
        opening = block.replace('if player.health.dead_or_dying()', 'if saved_death').replace('            self.screen = Screen::Dead;', '            self.native_client_death_screen_opening();\n            self.screen = Screen::Dead;')
        return replace_once(text, '        self.player = Some(player);\n', '        self.player = Some(player);\n' + opening)
    edit('crates/horizon-client/src/game/player_spawn.rs', spawn)
    helpers = ['client_is_death_screen', 'client_respawn']
    fragment = {
        'hooks': [{'name':'client_death_screen_opening','target':"horizon_client::game::native_hooks::Game<'_>::native_client_death_screen_opening",'side':'client','cancellable':False,'mod_api':{'contract':'listen'}}],
        'helpers': [{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in helpers],
        'rust_summary':'AutoRespawn ScreenEvent on actual death-screen opens, current screen query and ordinary LocalPlayer.respawn.',
        'doc_summary':'The opening callback is synchronous and does not cancel the enclosing screen assignment. Delayed continuations recheck DeathScreen at expiry. No health/death polling substitute.',
    }
    return {'tools/native-hook-fragments/liquidbounce_respawn.json':json.dumps(fragment,indent=2)+'\n'}
