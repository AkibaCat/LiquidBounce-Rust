#!/usr/bin/env python3
"""VehicleOneHit host operations; compose after the real pre-attack seam."""
from pathlib import Path
HERE = Path(__file__).resolve().parent
HELPER_NAMES = ['client_vehicle_one_hit_context', 'client_vehicle_one_hit_attack', 'client_vehicle_one_hit_swing']
HOOK_NAMES = []

def apply(edit, once):
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: once(text, anchor,
        (HERE/'vehicle-one-hit-host.rs').read_text(encoding='utf-8')+'\n'+anchor))
    def boats(text):
        return once(text, '''        self.strike_boat(i);
        self.swing_arm();
        true
''', '''        let target = self.boats[i].id;
        if self.client_weapon_before_attack(target, false) {
            // A native handler may remove a target. Re-resolve its id after
            // dispatch instead of retaining an index into the old collection.
            if let Some(index) = self.boats.iter().position(|boat| boat.id == target) {
                self.strike_boat(index);
            }
        }
        // Minecraft.startAttack swings even if game-mode attack was canceled.
        self.swing_arm();
        true
''')
    edit('crates/horizon-client/src/game/boats.rs', boats)
    def carts(text):
        return once(text, '''        let burning = self
            .held()
''', '''        let target = self.carts[i].id;
        if !self.client_weapon_before_attack(target, false) {
            self.swing_arm();
            return true;
        }
        let Some(i) = self.carts.iter().position(|cart| cart.id == target) else {
            self.swing_arm();
            return true;
        };
        let burning = self
            .held()
''')
    edit('crates/horizon-client/src/game/special_attacks.rs', carts)
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
