from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/mob_owners.rs',lambda _:(HERE/'mob-owners-cache.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/mob_owners_types.rs',lambda _:(HERE/'mob-owners-types.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/mob_owners.rs',lambda _:(HERE/'mob-owners-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod block_contact;\n','pub mod block_contact;\npub mod mob_owners;\n'))
 edit('crates/horizon-client/src/game.rs',lambda t:once(once(t,'mod native_projectiles;','mod native_projectiles;\nmod mob_owners;'),'        let mut entities = std::mem::take(&mut self.entity_vertices);','        self.client_mob_owners_begin();\n        let mut entities = std::mem::take(&mut self.entity_vertices);'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'mob-owners-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game/names.rs',lambda t:once(t,'        const FADED: u32 = 0x7F;','        self.mob_owner_geometry(camera,out);\n        const FADED: u32 = 0x7F;'))
 def geometry(t):
  t=once(t,'            let outline = self.mob_outline(mob);','            self.client_record_mob_owner(mob,alpha);\n            let outline = self.mob_outline(mob);')
  return once(t,'        for (a, at) in live.chain(taken) {','        for (a, at) in live.chain(taken) {\n            self.client_record_projectile_owner(a.id,a.owner,a.guest,at,a.kind.size() as f32);')
 edit('crates/horizon-client/src/game/creatures/mob_geometry.rs',geometry)
 def remote(t):
  anchor='''    ) -> bool {
        let id = id as u32;
        let at = Vec3::new(position[0], position[1], position[2]);'''
  t=once(t,anchor,'''    ) -> bool {
        horizon_client::mob_owners::remember(id as u32,data as u32);
        let id = id as u32;
        let at = Vec3::new(position[0], position[1], position[2]);''')
  t=once(t,'    pub(super) fn remove_remote_small_projectiles(&mut self, id: u32) {','    pub(super) fn remove_remote_small_projectiles(&mut self, id: u32) {\n        horizon_client::mob_owners::forget(id);')
  return once(t,'    pub(super) fn clear_remote_projectiles(&mut self) {','    pub(super) fn clear_remote_projectiles(&mut self) {\n        horizon_client::mob_owners::clear_remote();')
 edit('crates/horizon-client/src/game/remote_projectiles.rs',remote)
 edit('crates/horizon-client/src/game/trident.rs',lambda t:once(t,'        for (t, at) in live.chain(taken) {','        for (t, at) in live.chain(taken) {\n            self.client_record_projectile_owner(t.id,if self.remote.is_some(){Some(Who::Mob(t.owner))}else{Some(Who::Player)},t.guest,at,(2.0*HALF) as f32);'))
 edit('crates/horizon-client/src/game/fireworks.rs',lambda t:once(t,'            // `updateForNonLiving`: GROUND, owned by the rocket.','            self.client_record_projectile_owner(r.id,r.owner,r.guest,p.to_array(),0.25);\n            // `updateForNonLiving`: GROUND, owned by the rocket.'))
 edit('crates/horizon-client/src/game/potions.rs',lambda t:once(t,'            let p = previous.lerp(position, alpha);','            let p = previous.lerp(position, alpha);\n            self.client_record_projectile_owner(t.id,t.thrower,t.guest,p.to_array(),0.25);'))
 def fishing(t):
  a='''    pub(super) fn fishing_geometry(&self, camera: glam::DVec3, alpha: f64, out: &mut Vec<Vertex>) {
'''
  n=a+'''        if let Some(h)=&self.hook {if let Some(at)=self.hook_position(alpha){self.client_record_projectile_owner(h.id,Some(horizon_client::mobs::Who::Player),None,at.to_array(),0.25);}}
        for h in &self.remote_hooks {let at=[h.previous.x+(h.position.x-h.previous.x)*alpha,h.previous.y+(h.position.y-h.previous.y)*alpha,h.previous.z+(h.position.z-h.previous.z)*alpha];self.client_record_projectile_owner(h.id,Some(horizon_client::mobs::Who::Mob(h.owner)),None,at,0.25);}
'''
  return once(t,a,n)
 edit('crates/horizon-client/src/game/fishing.rs',fishing)
 return {'hook_names':['client_mob_owners'],'helper_names':[]}
