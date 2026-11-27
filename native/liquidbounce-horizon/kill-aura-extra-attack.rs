// SPDX-License-Identifier: GPL-3.0-or-later
//! Local authoritative tails for targets kept outside Horizon's mob list.
impl Game<'_> {
    /// Called only after the caller's single AttackEntityEvent, carried-item
    /// synchronization and scoped KeepSprint/implicit-swing setup. None means
    /// the ordinary exact-category native attack path still owns this target.
    fn kill_aura_extra_local_attack(&mut self, c: super::ClientReachCandidate) -> Option<bool> {
        match c.kind {
            KILL_AURA_GUEST => Some(self.kill_aura_attack_guest(c.id)),
            KILL_AURA_DRAGON_PARENT => Some(self.kill_aura_attack_dragon_parent(c)),
            _ => None,
        }
    }

    fn kill_aura_attack_guest(&mut self, id: i32) -> bool {
        let Some((conn, position)) = self
            .lan
            .iter()
            .flat_map(|host| host.hub.guests.iter())
            .find(|guest| guest.entity_id == id && guest.placed && guest.dimension == self.dim.id())
            .map(|guest| (guest.conn, guest.position))
        else {
            return false;
        };
        self.native_client_attack(id as u32, true);
        // This is the server hook, distinct from the client's already-fired
        // AttackEntityEvent. A canceled authoritative hurt still consumes the
        // attack packet, as guest_attack does.
        let mut canceled = false;
        self.native_attack_entity(id, "minecraft:player", position, &mut canceled);
        if canceled {
            return true;
        }
        // Keep the hub available while melee_damage / Criticals consult the
        // actual target. Strike owns values, so no guest borrow crosses hooks.
        let Some(strike) = self.pvp_strike() else {
            return false;
        };
        let Some(mut host) = self.lan.take() else {
            return false;
        };
        let victim = super::lan_pvp::Victim::Guest(conn);
        let here = host
            .hub
            .guest(conn)
            .is_some_and(|guest| guest.entity_id == id && host.hub.here(guest));
        let landed = here && self.pvp_land(&mut host, victim, &strike);
        let grounded = Self::pvp_victim_on_ground(&host, victim);
        self.lan = Some(host);
        if landed {
            // The actual victim blocking, armor, teams/PVP, Thorns, sweep,
            // criticals and kill credit were handled by pvp_land. This is the
            // weapon/mace/exhaustion tail, with the normal KeepSprint gate.
            let _ = self.pvp_hit_after(grounded, strike.stops_sprint());
        }
        true
    }

    fn kill_aura_attack_dragon_parent(&mut self, c: super::ClientReachCandidate) -> bool {
        self.native_client_attack(c.id as u32, false);
        let mut canceled = false;
        self.native_attack_entity(c.id, "minecraft:ender_dragon", c.position, &mut canceled);
        if canceled {
            return true;
        }
        let Some((area, old_health)) = self
            .dragon
            .as_ref()
            .filter(|dragon| dragon.id as i32 == c.id)
            .map(|dragon| (dragon.whole_box(), dragon.health))
        else {
            return false;
        };
        let ((amount, scale, crit, magic), admitted) =
            self.melee_damage_with_admission("minecraft:ender_dragon", true);
        if !admitted {
            self.attack_ticks = 0;
            return true;
        }
        let (knockback, _) = self.extra_knockback("minecraft:ender_dragon", scale);
        // EnderDragon.hurtServer delegates to BODY (2). Keep the network id
        // and living critical-hit predicate of the parent; EnderDragonPart's
        // non-living damage path would lose that predicate.
        let hurt = self.hit_special(
            c.local_id,
            amount,
            true,
            horizon_client::mobs::DamageKind::PlayerAttack,
            None,
        );
        let landed = matches!(hurt, super::dragon::Hurt::Landed { .. });
        let new_health = self
            .dragon
            .as_ref()
            .map_or(old_health, |dragon| dragon.health);
        if landed {
            if knockback > 0.0 {
                let (x, z) = self.extra_knockback_direction();
                let attacker = self.player.as_ref().map_or(0, |player| player.entity_id);
                let _attacker = self.knockback_scope(attacker);
                if let Some(dragon) = self.dragon.as_mut() {
                    dragon.knockback(
                        f64::from(knockback),
                        x,
                        z,
                        horizon_client::health::HurtSource {
                            damage_type: "minecraft:player_attack",
                            causing: attacker,
                            direct: attacker,
                        },
                    );
                }
            }
            self.attack_effects(
                area,
                Some(c.id),
                crit,
                super::mace::Spared::Nothing,
                magic.then_some(c.id),
            );
            if knockback > 0.0 {
                self.stop_sprint_after_extra_knockback();
            }
            let actual = old_health - new_health;
            self.award_custom_stat("damage_dealt", (actual * 10.0).round() as i32);
            if actual > 2.0 {
                let feet = area.bottom_center();
                let at = [feet.x, feet.y + area.size().y * 0.5, feet.z];
                let indicator = horizon_client::particles::ParticleOptions::simple(
                    horizon_client::particles::ParticleType::DamageIndicator,
                );
                self.particles.send_particles(
                    &indicator,
                    at,
                    [0.1, 0.0, 0.1],
                    0.2,
                    (actual * 0.5) as u32,
                );
                self.lan_send_particles(super::lan_particles::QueuedParticle {
                    dimension: self.dim.id(),
                    options: indicator,
                    state: None,
                    at,
                    spread: [0.1, 0.0, 0.1],
                    speed: [0.2; 3],
                    count: (actual * 0.5) as u32 as i32,
                    randomization: 0,
                });
            }
            if let Some(stack) = self.held().cloned() {
                self.native_hurt_enemy(&stack, c.id as u32, "minecraft:ender_dragon");
                let smash = self.smash_attack();
                if smash && let Some(fall) = self.mace_smash_self(false) {
                    // The dragon never sets onGround (noPhysics). Horizon's
                    // normal smash helper pushes nearby mobs, stands/players.
                    self.smash_knockback(area.bottom_center(), super::mace::Spared::Nothing, fall);
                }
                if self.res.items().weapon_damage(&stack).is_some() {
                    self.award_stat("used", &stack.item, 1);
                }
                self.post_attack_dragon_burst();
                if smash && let Some(player) = self.player.as_mut() {
                    player.fall_distance = 0.0;
                }
                self.weapon_wear(&stack);
            }
            self.pvp_exhaustion();
        } else {
            self.play_at_feet("minecraft:entity.player.attack.nodamage");
        }
        if let Some(player) = self.player.as_mut() {
            if player.attack_log.len() >= 64 {
                player.attack_log.remove(0);
            }
            player
                .attack_log
                .push(horizon_client::health::AttackRecord {
                    game_time: player.health.game_time(),
                    target: "minecraft:ender_dragon".into(),
                    id: c.id as u32,
                    killed: landed && new_health <= 0.0,
                    amount,
                    scale,
                    crit,
                    landed,
                    dealt: old_health - new_health,
                });
        }
        self.attack_ticks = 0;
        true
    }
}
