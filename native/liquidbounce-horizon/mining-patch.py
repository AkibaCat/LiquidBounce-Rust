#!/usr/bin/env python3
"""Local Player.getDestroySpeed condition queries, before BreakSpeed events."""


def apply(edit, replace_once):
    def native(text):
        anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        return replace_once(text, anchor, '''    /// Only the window's player, never a swapped LAN guest or server copy.
    pub(super) fn native_client_mining_conditions(&self, fatigue: &mut bool, water: &mut bool, on_ground: &mut bool) {
        if self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed()) {
            mod_api::dispatch_client_mining_conditions(self, mod_api::mods(), fatigue, water, on_ground);
        }
    }

''' + anchor)

    def effects(text):
        return replace_once(text, '''    pub fn mining_speed_factor(&self) -> f32 {
        let mut f = 1.0;
''', '''    pub fn mining_speed_factor(&self) -> f32 {
        self.mining_speed_factor_with_fatigue(true)
    }
    /// Same effect arithmetic, with only the getDestroySpeed fatigue query changed.
    pub fn mining_speed_factor_with_fatigue(&self, fatigue: bool) -> f32 {
        let mut f = 1.0;
''').replace('''        if let Some(a) = self.amplifier("mining_fatigue") {
''', '''        if let Some(a) = self.amplifier("mining_fatigue").filter(|_| fatigue) {
''', 1)

    def hand(text):
        text = replace_once(text, '''        let player = DestroySpeedModifiers {
''', '''        let (mut fatigue, mut water, mut on_ground) = (true, self.mining_eyes_in_water(), self.player.as_ref().is_none_or(|p| p.ground.on_ground));
        self.native_client_mining_conditions(&mut fatigue, &mut water, &mut on_ground);
        let player = DestroySpeedModifiers {
''')
        text = replace_once(text, '''            effects: self.potions.effects.mining_speed_factor(),
''', '''            effects: self.potions.effects.mining_speed_factor_with_fatigue(fatigue),
''')
        return replace_once(text, '''            submerged_mining_speed: self
                .mining_eyes_in_water()
                .then(|| self.submerged_mining_speed()),
            on_ground: self.player.as_ref().is_none_or(|p| p.ground.on_ground),
''', '''            submerged_mining_speed: water.then(|| self.submerged_mining_speed()),
            on_ground,
''')

    edit('crates/horizon-client/src/game/native_hooks.rs', native)
    edit('crates/horizon-client/src/motion.rs', lambda text: replace_once(text,
        '    fn run_sprint_tick(\n',
        '    /// Local native queries exclude server copies and speculative replay.\n    pub fn native_client_queries_allowed(&self) -> bool { !self.server_copy && !self.replaying }\n    fn run_sprint_tick(\n'))
    edit('crates/horizon-client/src/game/hand.rs', hand)
    edit('crates/horizon-client-render/src/effects.rs', effects)
    return {'hook_names':['client_mining_conditions'], 'helper_names':[]}
