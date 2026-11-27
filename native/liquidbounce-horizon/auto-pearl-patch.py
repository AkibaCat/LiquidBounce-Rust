"""AutoPearl: packet snapshots, integrated-server spawn queue and actual world rays."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def source():
 # Reuse the audited native pickable universe without changing AutoBow's owned source.
 text=(HERE/'auto-bow-native.rs').read_text(encoding='utf8')
 start=text.index('    fn auto_bow_collision_boxes(&self) -> Vec<Aabb> {')
 body=text[start:text.rfind('\n}')]
 body=body.replace('fn auto_bow_collision_boxes(&self)','fn auto_pearl_collision_boxes(&self, owner:i32)')
 body=body.replace('let root = self.riding.map(|r| self.client_reach_root_vehicle(r));','let root = self.client_reach_root_vehicle(owner as u32);')
 body=body.replace('if id != local && root.is_none_or(|r| r != self.client_reach_root_vehicle(id)) {','if id != owner as u32 && root != self.client_reach_root_vehicle(id) {')
 body=body.replace('        candidates\n','''        drop(offer);
        if owner as u32!=local && root!=self.client_reach_root_vehicle(local) {
            if let Some(p)=self.player.as_ref().filter(|p|!p.spectator&&!p.health.dead_or_dying()){
                candidates.push(p.dimensions.bounds(p.position));
            }
        }
        candidates
''')
 return (HERE/'auto-pearl-native.rs').read_text(encoding='utf8').replace(' // AUTO_PEARL_COLLISION_BOXES',body)
def apply(edit,once):
 edit('crates/horizon-client/src/game/auto_pearl.rs',lambda _:source())
 edit('crates/horizon-client/src/game/auto_pearl_ray.rs',lambda _:(HERE/'auto-pearl-ray.rs').read_text(encoding='utf8'))
 model=(HERE.parent/'liquidbounce-behaviors/src/auto_pearl.rs').read_text(encoding='utf8')
 edit('crates/horizon-client/src/game/auto_pearl_model.rs',lambda _:'use super::auto_bow_math::{B,V};\n'+model[model.index('pub fn simulate('):])
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_bow;','mod auto_bow;\nmod auto_pearl;\nmod auto_pearl_ray;\nmod auto_pearl_model;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'auto-pearl-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 anchor='                let replace = self.tracks_remote_entity(p.id);\n'
 edit('crates/horizon-client/src/game/remote.rs',lambda s:once(s,anchor,'                if kind=="minecraft:ender_pearl"{self.auto_pearl_spawn((p.data!=0).then_some(p.data),Vec3::new(p.position[0],p.position[1],p.position[2]),Vec3::new(p.velocity[0],p.velocity[1],p.velocity[2]));}\n'+anchor))
 anchor='        self.creatures.projectiles.put_back(p);\n        self.native_projectile_launch(player, &name, id, kind);'
 edit('crates/horizon-client/src/game/creatures.rs',lambda s:once(s,anchor,'''        if self.remote.is_none() && matches!(p.kind,horizon_client::projectiles::ProjectileKind::EnderPearl){
            let owner=match p.owner{Some(horizon_client::mobs::Who::Player)=>Some(self.lan_acting_entity.unwrap_or(0) as i32),Some(horizon_client::mobs::Who::Mob(id))=>Some(id as i32),None=>None};
            self.auto_pearl_spawn(owner,p.position,p.velocity);
        }
'''+anchor))
 return {'hook_names':[],'helper_names':['client_auto_pearl_query']}
