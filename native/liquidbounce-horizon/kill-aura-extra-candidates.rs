// SPDX-License-Identifier: GPL-3.0-or-later
//! Candidate categories that must never be interpreted as a `mobs` index.
const KILL_AURA_GUEST: u8 = 12;
const KILL_AURA_DRAGON_PARENT: u8 = 13;
const KILL_AURA_DRAGON_BODY: usize = 2;

impl Game<'_> {
    /// Extend the native pickable categories. The dragon parent is a tracked
    /// LivingEntity, but only its parts belong in a natural entity ray.
    fn kill_aura_extra_candidates(&self, out: &mut Vec<super::ClientReachCandidate>) {
        if let Some(dragon) = self.dragon.as_ref() {
            out.retain(|c| c.id as u32 != dragon.id);
        }
        let own = self
            .remote
            .as_ref()
            .map(|r| r.entity_id)
            .or_else(|| self.player.as_ref().map(|p| p.entity_id));
        let root = self.riding.map(|r| self.client_reach_root_vehicle(r));
        for guest in self
            .lan
            .iter()
            .flat_map(|host| host.hub.guests.iter())
            .filter(|guest| {
                guest.placed
                    && guest.dimension == self.dim.id()
                    && !guest.spectator()
                    && Some(guest.entity_id) != own
            })
        {
            let Some(body) = guest.motion.as_deref() else {
                continue;
            };
            // A host may also retain a render mirror in the mob list. The
            // authoritative guest category must replace that mirror's index.
            out.retain(|c| c.id != guest.entity_id);
            let guest_root = guest
                .vehicle
                .map(|vehicle| self.client_reach_root_vehicle(vehicle))
                .unwrap_or(guest.entity_id as u32);
            let [x, y, z] = guest.position;
            out.push(super::ClientReachCandidate {
                kind: KILL_AURA_GUEST,
                // This is deliberately not a mobs index. The action helper
                // resolves the guest's connection again from its entity id.
                index: 0,
                id: guest.entity_id,
                local_id: guest.entity_id as u32,
                position: guest.position,
                bounds: body.dimensions.bounds(Vec3::new(x, y, z)),
                pick_radius: 0.0,
                same_root_vehicle: root == Some(guest_root),
                can_pick_inside: true,
            });
        }
    }

    /// AttackEntity receives either an actual ray target or the target
    /// tracker's living entity. A dragon parent's network identity stays the
    /// parent; its authoritative hurt is EnderDragon.hurtServer(BODY).
    fn kill_aura_attack_candidate(&self, id: u32) -> Option<super::ClientReachCandidate> {
        if let Some(dragon) = self.dragon.as_ref().filter(|d| d.id == id) {
            return Some(super::ClientReachCandidate {
                kind: KILL_AURA_DRAGON_PARENT,
                index: KILL_AURA_DRAGON_BODY,
                id: id as i32,
                local_id: super::dragon::SPECIAL + KILL_AURA_DRAGON_BODY as u32,
                position: [dragon.position.x, dragon.position.y, dragon.position.z],
                bounds: dragon.whole_box(),
                pick_radius: 0.0,
                same_root_vehicle: false,
                can_pick_inside: true,
            });
        }
        self.kill_aura_candidates()
            .into_iter()
            .find(|c| c.id as u32 == id)
    }
}
