from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'combine-mobs-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game.rs',lambda t:once(t,'        let mut entities = std::mem::take(&mut self.entity_vertices);','        self.native_client_combine_mobs_frame();\n        let mut entities = std::mem::take(&mut self.entity_vertices);'))
 edit('crates/horizon-client/src/game/creatures/mob_geometry.rs',lambda t:once(t,'            let is_player = &*mob.kind.id == "minecraft:player";','            let combine_group=if matches!(&*mob.kind.id,"minecraft:player"|"minecraft:mannequin"){3}else{0};\n            if self.client_combine_mobs_skip(&mob.kind.id,combine_group,mob.is_baby(),mob.position){continue;}\n            let is_player = &*mob.kind.id == "minecraft:player";'))
 edit('crates/horizon-client/src/game/armor_stands.rs',lambda t:once(t,'            // ArmorStand.shouldRenderAtSqrDistance: 256 times the box\'s mean','            if self.client_combine_mobs_skip("minecraft:armor_stand",1,a.small,a.position){continue;}\n            // ArmorStand.shouldRenderAtSqrDistance: 256 times the box\'s mean'))
 edit('crates/horizon-client/src/game/minecarts.rs',lambda t:once(t,'        for c in &self.carts {\n            let Some(model) = self.cart_models.models.get(&c.kind) else {','        for c in &self.carts {\n            if self.client_combine_mobs_skip(c.kind.id(),2,false,c.position){continue;}\n            let Some(model) = self.cart_models.models.get(&c.kind) else {'))
 return {'hook_names':['client_combine_mobs_frame','client_combine_mobs'],'helper_names':[]}
