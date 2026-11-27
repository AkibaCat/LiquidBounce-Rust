#!/usr/bin/env python3
"""AutoWeapon actual pre-attack, item-component and attack-delay seams.

Compose after slot-patch and item-identity-patch. The post-attack native event
remains available to existing integrations; AutoWeapon uses the new HEAD seam.
"""
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
HELPER_NAMES=['client_weapon_context','client_ensure_carried']
HOOK_NAMES=['client_attack_entity','client_attack_speed']
HOOKS=(HERE/'auto-weapon-host.rs').read_text(encoding='utf-8')
FACTS=(ROOT/'native/liquidbounce-behaviors/src/no_interact_items.rs').read_text(encoding='utf-8')

def apply(edit,replace_once):
    # A read-only AttackSpeed hook must be able to query the attribute base.
    # Read an existing instance or its supplier directly, without materializing
    # a lazy instance or recomputing its effective value through this hook.
    edit('crates/horizon-world/src/attributes/live.rs',lambda text:replace_once(text,
        '    /// The computed value of the attribute, calculating current dirty modifiers if needed.\n',
        '''    /// Raw AttributeInstance.getBaseValue, or the supplier base when
    /// the lazy instance has not been materialized. Does not apply modifiers.
    pub fn base_value_of(&self, name: &str) -> Result<f64, Error> {
        let key = self.key(name).ok_or(Error::MissingAttribute)?;
        if let Some(handle) = self.map.instances.get(Some(&key)) {
            return Ok(self.data.node(&handle)?.value.base_value());
        }
        self.data.base(&key).ok_or(Error::MissingAttribute)
    }
    /// The computed value of the attribute, calculating current dirty modifiers if needed.
'''))
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:replace_once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HOOKS+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')+'\n#[allow(dead_code)]\nmod client_weapon_item_facts {\n'+FACTS+'\n}\n')
    def remote(text):
        text=replace_once(text,'    fn remote_send_carried(&self) {\n','    pub(super) fn remote_send_carried(&self) {\n')
        # The original remote receiver retained only the offhand id. Preserve
        # its real server components in the existing gear stack store before
        # the ordinary field update, so a custom shield is queried faithfully.
        text=replace_once(text,'                    for (slot, item, stack) in items {\n',
            '                    for (slot, item, stack) in items {\n'
            '                        if slot == 1 && let Some(stack) = stack.as_ref() { horizon_client::mobs::finalize::set_item_stack(m, 1, stack.clone()); }\n')
        text=replace_once(text,'''            self.send(S::Attack(play::Attack {
                id: self.creatures.projectiles[i].id as i32,
            }));
''','''            let target = self.creatures.projectiles[i].id;
            if !self.client_weapon_before_attack(target, false) { return false; }
            self.remote_send_carried();
            self.send(S::Attack(play::Attack { id: target as i32 }));
''')
        text=replace_once(text,'''            self.send(S::Attack(play::Attack { id: hit.id }));
''','''            let target = hit.id as u32;
            let living = self.native_entity(target).and_then(|entity| self.native_living(entity)).is_some();
            if !self.client_weapon_before_attack(target, living) { return false; }
            self.remote_send_carried();
            self.send(S::Attack(play::Attack { id: hit.id }));
''')
        text=replace_once(text,'''            if self.held_range_allows(distance) {
                self.send(S::Attack(play::Attack {
                    id: self.mobs[i].id as i32,
                }));
''','''            if self.held_range_allows(distance) {
                let target = self.mobs[i].id;
                if !self.client_weapon_before_attack(target, true) { return false; }
                self.remote_send_carried();
                self.send(S::Attack(play::Attack { id: target as i32 }));
''')
        # This path bypasses main_hand_attack_speed with a synchronized
        # attribute; place the expression event after the chosen raw value.
        # Use the raw fallback so the expression event runs exactly once.
        text=replace_once(text,'            .unwrap_or_else(|| self.main_hand_attack_speed());\n',
            '            .unwrap_or_else(|| self.client_raw_main_hand_attack_speed());\n')
        text=replace_once(text,'''        let strength = self.hand_strength.scale(0.5, speed);
''','''        let mut attack_speed = speed;
        self.native_client_attack_speed(speed, &mut attack_speed);
        let strength = self.hand_strength.scale(0.5, attack_speed);
''')
        return text
    edit('crates/horizon-client/src/game/remote.rs',remote)
    edit('crates/horizon-client/src/game/melee.rs',lambda text:replace_once(text,
        '    pub(super) fn attack_mob(&mut self, index: usize) {\n',
        '    pub(super) fn attack_mob(&mut self, index: usize) {\n        if !self.client_weapon_before_attack(self.mobs[index].id, true) { return; }\n'))
    edit('crates/horizon-client/src/game/armor_stands.rs',lambda text:replace_once(text,
        '        let ((_, scale, crit, magic), admitted) =\n',
        '        if !self.client_weapon_before_attack(self.armor_stands[i].id, true) { return false; }\n        let ((_, scale, crit, magic), admitted) =\n'))
    def hand(text):
        return replace_once(text,'    pub(super) fn main_hand_attack_speed(&self) -> f64 {\n',
            '''    pub(super) fn main_hand_attack_speed(&self) -> f64 {
        let original = self.client_raw_main_hand_attack_speed();
        let mut speed = original;
        self.native_client_attack_speed(original, &mut speed);
        speed
    }
    pub(super) fn client_raw_main_hand_attack_speed(&self) -> f64 {
''')
    edit('crates/horizon-client/src/game/hand.rs',hand)
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
