#!/usr/bin/env python3
"""KeepSprint at Player.attack queries and the causeExtraKnockback tail."""

HOOK_NAMES = ["client_keep_sprint_attack", "client_keep_sprint_motion", "client_keep_sprint_set"]
HELPER_NAMES = ["client_combat_state"]
HOOKS = '''    pub(super) fn native_client_keep_sprint_attack(&self, sprinting: &mut bool) {
        if !self.client_combat_player() { return; }
        mod_api::dispatch_client_keep_sprint_attack(self, mod_api::mods(), sprinting);
    }
    pub(super) fn native_client_keep_sprint_motion(&self, hurt_time: i32, multiplier: &mut f64) {
        if !self.client_combat_player() { return; }
        mod_api::dispatch_client_keep_sprint_motion(self, mod_api::mods(), hurt_time, multiplier);
    }
    pub(super) fn native_client_keep_sprint_set(&self, requested: bool, apply: &mut bool) {
        if !self.client_combat_player() { return; }
        mod_api::dispatch_client_keep_sprint_set(self, mod_api::mods(), requested, apply);
    }
    pub(super) fn native_client_combat_state(&self) -> [i32; 2] {
        self.player.as_ref().map_or([0; 2], |p| [i32::from(p.sprinting), p.health.hurt.hurt_time])
    }
    fn client_combat_player(&self) -> bool {
        self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_combat_queries_allowed())
    }
    pub(super) fn client_attack_sprinting(&self, original: bool) -> bool {
        let mut sprinting = original;
        self.native_client_keep_sprint_attack(&mut sprinting);
        sprinting
    }

'''


def apply(edit, replace_once):
    anchor = "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n"
    edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: replace_once(text, anchor, HOOKS + anchor))
    edit("crates/horizon-client/src/motion.rs", lambda text: replace_once(text, "impl PlayerMotion {\n", '''impl PlayerMotion {
    pub fn native_client_combat_queries_allowed(&self) -> bool { !self.server_copy && !self.replaying }
'''))

    def effects(text):
        text = replace_once(text, "        let sprinting = self.player.as_ref().is_some_and(|p| p.sprinting);\n", "        let sprinting = self.client_attack_sprinting(self.player.as_ref().is_some_and(|p| p.sprinting));\n")
        return replace_once(text, '''    pub(super) fn stop_sprint_after_extra_knockback(&mut self) {
        if let Some(p) = self.player.as_mut() {
            let _ = p.set_sprinting(false);
        }
    }
''', '''    pub(super) fn stop_sprint_after_extra_knockback(&mut self) {
        let mut multiplier = horizon_client::attack_strength::ATTACKER_SLOWDOWN;
        let hurt_time = self.player.as_ref().map_or(0, |p| p.health.hurt.hurt_time);
        self.native_client_keep_sprint_motion(hurt_time, &mut multiplier);
        if let Some(p) = self.player.as_mut() {
            p.velocity.x *= multiplier;
            p.velocity.z *= multiplier;
        }
        let mut apply = true;
        self.native_client_keep_sprint_set(false, &mut apply);
        if apply && let Some(p) = self.player.as_mut() {
            let _ = p.set_sprinting(false);
        }
    }
''')
    edit("crates/horizon-client/src/game/enchant_effects.rs", effects)
    edit("crates/horizon-client/src/game/melee.rs", lambda text: replace_once(text,
        "        if full && player.is_some_and(|p| p.sprinting) {\n",
        "        if full && self.client_attack_sprinting(player.is_some_and(|p| p.sprinting)) {\n"))

    def remote(text):
        text = replace_once(text, "        let sprinting = self.player.as_ref().is_some_and(|p| p.sprinting);\n", "        let sprinting = self.client_attack_sprinting(self.player.as_ref().is_some_and(|p| p.sprinting));\n")
        return replace_once(text, '''        if let Some(p) = self.player.as_mut() {
            let slowdown = horizon_client::attack_strength::ATTACKER_SLOWDOWN;
            p.velocity = Vec3::new(
                p.velocity.x * slowdown,
                p.velocity.y,
                p.velocity.z * slowdown,
            );
        }
        self.stop_sprint_after_extra_knockback();
''', '''        self.stop_sprint_after_extra_knockback();
''')
    edit("crates/horizon-client/src/game/remote.rs", remote)
    edit("crates/horizon-client/src/game/special_attacks.rs", lambda text: replace_once(text, '''            if let Some(player) = self.player.as_mut() {
                player.velocity.x *= horizon_client::attack_strength::ATTACKER_SLOWDOWN;
                player.velocity.z *= horizon_client::attack_strength::ATTACKER_SLOWDOWN;
            }
''', ""))
    return {"hook_names": HOOK_NAMES, "helper_names": HELPER_NAMES}
