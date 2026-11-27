#!/usr/bin/env python3
"""Filter only LocalPlayer.pick's ProjectileUtil result; compose after Reach."""
HOOK_NAMES = ['client_entity_pick']
HELPER_NAMES = ['client_default_no_entity_holding_items', 'client_no_interact_registries']
HOOKS = '''    pub(super) fn native_client_entity_pick(&self, main_hand_item: &str, allowed: &mut bool) {
        if !self.client_reach_player() { return; }
        mod_api::dispatch_client_entity_pick(self, mod_api::mods(), main_hand_item, allowed);
    }
    pub(super) fn client_entity_pick_allowed(&self) -> bool {
        // raycastHitResult returns the active component's non-MISS before pick.
        if self.active_attack_range().is_some_and(|r| self.attack_range_pick(&r).is_some()) { return true; }
        let item = self.stack_in(self.client_hotbar_slot()).map_or("minecraft:air", |s| &*s.item);
        let mut allowed = true;
        self.native_client_entity_pick(item, &mut allowed);
        allowed
    }
    pub(super) fn native_client_default_no_entity_holding_items(&self) -> Vec<String> {
        let items = self.items();
        let mut defaults: std::collections::BTreeSet<String> = ["minecraft:air", "minecraft:shears", "minecraft:tnt", "minecraft:water_bucket", "minecraft:lava_bucket", "minecraft:cobweb"]
            .into_iter().map(str::to_owned).collect();
        for item in &items.order {
            if ["minecraft:axes", "minecraft:pickaxes", "minecraft:shovels", "minecraft:hoes"]
                .iter().any(|tag| items.item_in_tag(tag, item)) {
                defaults.insert(item.to_string());
            }
        }
        defaults.into_iter().collect()
    }
    pub(super) fn native_client_no_interact_registries(&self) -> (Vec<String>, Vec<String>) {
        let mut entities = Vec::new();
        for id in 0..i32::MAX {
            match self.native_registry_name("minecraft:entity_type", id) {
                Some(name) => entities.push(name), None => break,
            }
        }
        // No connection on the title screen: read the same actual resource
        // table that MobTypes and the command completions use (native mods included).
        if entities.is_empty() {
            if let Ok(bytes) = std::fs::read(self.options.resources.join("entity-types.json")) {
                if let Ok(serde_json::Value::Object(values)) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    entities.extend(values.keys().cloned());
                }
            }
        }
        (entities, self.items().order.iter().map(|name| name.to_string()).collect())
    }
'''


def apply(edit, replace_once):
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: replace_once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HOOKS + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    methods = [('game.rs', 'mob_hit'), ('game/paintings.rs', 'painting_hit'), ('game/frames.rs', 'frame_hit'),
               ('game/armor_stands.rs', 'armor_stand_hit'), ('game/cushions.rs', 'cushion_hit'), ('game/boats.rs', 'boat_hit'),
               ('game/minecarts.rs', 'cart_hit'), ('game/interactions.rs', 'interaction_hit'), ('game/deflect.rs', 'redirectable_hit'),
               ('game/remote_vehicles.rs', 'remote_knot_hit'), ('game/remote_vehicles.rs', 'remote_object_hit'), ('game/dragon.rs', 'dragon_hit')]
    for path, name in methods:
        def add(text, name=name):
            start = text.index(f'fn {name}(')
            # Reach's canonical through-wall EntityHitResult replaces Entity.pick,
            # not ProjectileUtil, so its early return deliberately precedes this.
            forced = text.index('        if let Some(hit) = self.client_reach_target() {\n', start)
            end = text.index('\n        }\n', forced) + len('\n        }\n')
            return text[:end] + '        if !self.client_entity_pick_allowed() { return None; }\n' + text[end:]
        edit('crates/horizon-client/src/' + path, add)
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
