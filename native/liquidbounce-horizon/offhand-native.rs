// SPDX-License-Identifier: GPL-3.0-or-later
//! Live Offhand hazard facts. Damage prediction never hurts the actual player.
use super::Game;
use horizon_core::{
    geometry::{Aabb, Vec3, VoxelShape},
    position::BlockPos,
};
use horizon_world::{
    registry::BlockStateId,
    world_ray::{RayBlockMode, RayFluidMode, RayWorld, clip_world},
};
use serde_json::{Value, json};
use std::io;
struct Rays<'a, 'r> {
    game: &'a Game<'r>,
    exclude: &'a [[i32; 3]],
}
impl RayWorld for Rays<'_, '_> {
    type Error = io::Error;
    type Block = BlockStateId;
    type Fluid = ();
    fn block_state(&mut self, p: BlockPos) -> io::Result<BlockStateId> {
        Ok(self.game.world.block(p.x, p.y, p.z))
    }
    fn fluid_state(&mut self, _: BlockPos) -> io::Result<()> {
        Ok(())
    }
    fn block_shape(
        &mut self,
        s: BlockStateId,
        p: BlockPos,
        _: RayBlockMode,
    ) -> io::Result<VoxelShape> {
        if self.exclude.contains(&[p.x, p.y, p.z]) {
            return Ok(VoxelShape::empty());
        }
        use horizon_world::{
            collision_context::{
                CollisionContext, CollisionEntity, CollisionItem, EntityContextKind,
            },
            entity_obstruction::EntityIdentity,
        };
        let player = self.game.player.as_ref().expect("Offhand live player");
        let main = self
            .game
            .inventory
            .get(self.game.selected)
            .and_then(Option::as_ref);
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(0),
                kind: EntityContextKind::Player,
                y: player.bounds().min_position().y,
                descending: player.shift_key_down,
                main_hand: Some(main.map_or(CollisionItem::EMPTY, |s| CollisionItem {
                    item: &s.item,
                    count: s.count as i32,
                })),
            },
            false,
        );
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &self.game.world,
            binding: self.game.collision,
            subject: Some(player.powder_snow_subject()),
        };
        self.game
            .collision
            .collision_shape_with_context(&mut scene, s, p, &context)
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
    fn interaction_shape(&mut self, _: BlockStateId, _: BlockPos) -> io::Result<VoxelShape> {
        Ok(VoxelShape::empty())
    }
    fn fluid_is_source(&mut self, _: ()) -> io::Result<bool> {
        Ok(false)
    }
    fn fluid_is_empty(&mut self, _: ()) -> io::Result<bool> {
        Ok(true)
    }
    fn fluid_is_water(&mut self, _: ()) -> io::Result<bool> {
        Ok(false)
    }
    fn fluid_shape(&mut self, _: (), _: BlockPos) -> io::Result<VoxelShape> {
        Ok(VoxelShape::empty())
    }
}
fn exposure(g: &Game<'_>, at: [f64; 3], exclude: &[[i32; 3]]) -> f32 {
    let b = g.player.as_ref().expect("Offhand live player").bounds();
    let lo = b.min_position();
    let hi = b.max_position();
    let step = [
        1. / ((hi.x - lo.x) * 2. + 1.),
        1. / ((hi.y - lo.y) * 2. + 1.),
        1. / ((hi.z - lo.z) * 2. + 1.),
    ];
    if step.iter().any(|&s| s < 0.) {
        return 0.;
    }
    let offset = [
        (1. - (1. / step[0]).floor() * step[0]) / 2.,
        (1. - (1. / step[2]).floor() * step[2]) / 2.,
    ];
    let mut rays = Rays { game: g, exclude };
    let (mut hits, mut total) = (0, 0);
    let mut x = 0.;
    while x <= 1. {
        let mut y = 0.;
        while y <= 1. {
            let mut z = 0.;
            while z <= 1. {
                let from = Vec3::new(
                    lo.x + (hi.x - lo.x) * x + offset[0],
                    lo.y + (hi.y - lo.y) * y,
                    lo.z + (hi.z - lo.z) * z + offset[1],
                );
                if clip_world(
                    &mut rays,
                    from,
                    Vec3::new(at[0], at[1], at[2]),
                    RayBlockMode::Collider,
                    RayFluidMode::None,
                    4096,
                )
                .is_ok_and(|r| r.miss)
                {
                    hits += 1;
                }
                total += 1;
                z += step[2];
            }
            y += step[1];
        }
        x += step[0];
    }
    hits as f32 / total as f32
}
fn effective(
    g: &Game<'_>,
    kind: &str,
    mut damage: f32,
    at: Option<[f64; 3]>,
    living_nonplayer: bool,
) -> f32 {
    let Some(p) = g.player.as_ref() else {
        return 0.;
    };
    let catalog = &g.health_resources.damage;
    let Some(id) = catalog.id(kind) else {
        return 0.;
    };
    let tag = |name: &str| catalog.in_tag(id, name);
    if p.health.dead_or_dying() || p.invulnerable && !tag("minecraft:bypasses_invulnerability") {
        return 0.;
    }
    use horizon_world::damage_source::Scaling;
    let scales = catalog.definition(id).is_some_and(|d| {
        d.scaling == Scaling::Always || d.scaling == Scaling::LivingNonPlayer && living_nonplayer
    });
    if scales {
        damage = match g.world.health_rules.difficulty {
            horizon_world::food::Difficulty::Peaceful => 0.,
            horizon_world::food::Difficulty::Easy => (damage / 2. + 1.).min(damage),
            horizon_world::food::Difficulty::Hard => damage * 3. / 2.,
            _ => damage,
        };
    }
    if damage == 0. || tag("minecraft:is_fire") && p.status.fire_resistance {
        return 0.;
    }
    if let Some(blocking) = g.player_blocking(at, kind, false) {
        damage -= blocking.blocked(damage);
        if damage == 0. {
            return 0.;
        }
    }
    if !tag("minecraft:bypasses_armor") {
        let armor = p
            .attributes
            .computed_value_of("minecraft:armor")
            .unwrap_or(0.)
            .floor() as i32 as f32;
        let tough = p
            .attributes
            .computed_value_of("minecraft:armor_toughness")
            .unwrap_or(0.) as f32;
        let protect = (armor - damage / (2. + tough / 4.)).clamp(armor * 0.2, 20.);
        damage *= 1. - protect / 25.;
    }
    if !tag("minecraft:bypasses_effects") {
        if let Some(a) = p
            .status
            .resistance
            .filter(|_| !tag("minecraft:bypasses_resistance"))
        {
            damage = (damage * (25i32.wrapping_sub(a.wrapping_add(1).wrapping_mul(5))) as f32
                / 25.)
                .max(0.);
        }
        if damage <= 0. {
            return 0.;
        }
        // Client-level EnchantmentHelper is skipped by getDamageAfterMagicAbsorb,
        // as the original helper only resolves equipment protection on ServerLevel.
        // javap MC26.3 LivingEntity confirms ClientLevel protection is zero.
    }
    damage
}
fn blast(
    g: &Game<'_>,
    at: [f64; 3],
    power: f32,
    exclude: &[[i32; 3]],
    kind: &str,
    living_nonplayer: bool,
    source_at: Option<[f64; 3]>,
) -> f32 {
    let p = g.player.as_ref().expect("Offhand live player");
    let range = power * 2.;
    let dist = (p.position.x - at[0]).powi(2)
        + (p.position.y - at[1]).powi(2)
        + (p.position.z - at[2]).powi(2);
    if dist > f64::from(range * range) {
        return 0.;
    }
    let before = f64::from(exposure(g, at, exclude)) * (1. - dist.sqrt() / f64::from(range));
    effective(
        g,
        kind,
        ((before * before + before) / 2. * 7. * f64::from(range) + 1.) as f32,
        Some(source_at.unwrap_or(at)),
        living_nonplayer,
    )
}
fn boxed_distance(b: Aabb, eye: [f64; 3]) -> f64 {
    let lo = b.min_position();
    let hi = b.max_position();
    let lo = [lo.x, lo.y, lo.z];
    let hi = [hi.x, hi.y, hi.z];
    (0..3)
        .map(|i| (eye[i] - eye[i].clamp(lo[i], hi[i])).powi(2))
        .sum()
}
pub(super) fn snapshot(g: &mut Game<'_>, policy: &str) -> Option<String> {
    if !g.client_auto_tool_player() {
        return None;
    }
    let query = serde_json::from_str::<Value>(policy).unwrap_or(Value::Null);
    let landing = if query["fall"] == true && g.player.as_ref()?.fall_distance > 3. {
        super::offhand_falling::collision(g, 20, 0.).ok().flatten()
    } else {
        None
    };
    let inventory_items = g
        .native_client_inventory_cleaner_context(&query["policy"].to_string())
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .map(|v| v["items"].clone());
    let p = g.player.as_ref()?;
    let pos = p.position;
    let bb = p.bounds();
    let lo = bb.min_position();
    let hi = bb.max_position();
    let feet = [
        ((lo.x + hi.x) * 0.5).floor() as i32,
        lo.y.ceil() as i32,
        ((lo.z + hi.z) * 0.5).floor() as i32,
    ];
    let resistant = |at: [i32; 3]| {
        g.res
            .facts()
            .get(g.world.block(at[0], at[1], at[2]))
            .explosion_resistance
            >= 600.
    };
    let burrowed = resistant(feet);
    let hole = [[0, -1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]]
        .into_iter()
        .all(|d| resistant(std::array::from_fn(|i| feet[i] + d[i])));
    let mut entity_damage = Vec::new();
    if query["entities"] == true {
        for c in &g.crystals {
            entity_damage.push(blast(
                g,
                [c.position.x, c.position.y, c.position.z],
                6.,
                &[],
                "minecraft:explosion",
                false,
                None,
            ));
        }
        for t in &g.tnts {
            let owner = t.owner.as_ref();
            entity_damage.push(blast(
                g,
                [t.position.x, t.position.y + 0.0625, t.position.z],
                4.,
                &[],
                if owner.is_some() {
                    "minecraft:player_explosion"
                } else {
                    "minecraft:explosion"
                },
                owner.is_some_and(|o| matches!(o, horizon_client::mobs::Who::Mob(_))),
                Some([t.position.x, t.position.y, t.position.z]),
            ));
        }
        for c in g
            .carts
            .iter()
            .filter(|c| c.kind == super::minecarts::CartKind::Tnt)
        {
            let power = 4.
                + (c.velocity.x * c.velocity.x + c.velocity.z * c.velocity.z)
                    .sqrt()
                    .min(5.) as f32
                    * 1.5;
            entity_damage.push(blast(
                g,
                [c.position.x, c.position.y, c.position.z],
                power,
                &[],
                "minecraft:explosion",
                false,
                None,
            ));
        }
        for m in g
            .mobs
            .iter()
            .filter(|m| !m.removed && m.kind.id.rsplit(':').next() == Some("creeper"))
        {
            let power = f32::from(m.extra.explosion_radius) * if m.extra.charged { 2. } else { 1. };
            entity_damage.push(blast(
                g,
                [m.position.x, m.position.y, m.position.z],
                power,
                &[],
                "minecraft:player_explosion",
                true,
                None,
            ));
        }
    }
    let mut block_damage = Vec::new();
    if query["blocks"] == true {
        let mut sphere = Vec::new();
        for x in -10i32..=10 {
            for y in -10i32..=10 {
                for z in -10i32..=10 {
                    let d = x * x + y * y + z * z;
                    if d <= 100 {
                        sphere.push((d, [x, y, z]));
                    }
                }
            }
        }
        sphere.sort_by_key(|p| p.0);
        let origin = [
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        ];
        for (_, d) in sphere {
            let at = std::array::from_fn::<_, 3, _>(|i| origin[i] + d[i]);
            let state = g.world.block(at[0], at[1], at[2]);
            let f = g.res.facts().get(state);
            let bed = g.dimension_type().bed_rule.destroy_on_use && f.has_class("BedBlock");
            let state = g.pack.assets.registry.state(state);
            let anchor = !g.anchor_works()
                && &*f.name == "minecraft:respawn_anchor"
                && state
                    .and_then(|s| s.properties.get("charges"))
                    .and_then(|s| s.parse::<i32>().ok())
                    .is_some_and(|n| n > 0);
            if !bed && !anchor {
                continue;
            }
            let mut exclude = vec![at];
            if bed {
                let facing = state
                    .and_then(|s| s.properties.get("facing"))
                    .map(String::as_str)
                    .unwrap_or("north");
                let delta = match facing {
                    "north" => [0, 0, 1],
                    "south" => [0, 0, -1],
                    "west" => [1, 0, 0],
                    _ => [-1, 0, 0],
                };
                exclude.push(std::array::from_fn(|i| at[i] + delta[i]));
            }
            block_damage.push(blast(
                g,
                at.map(|v| f64::from(v) + 0.5),
                5.,
                &exclude,
                "minecraft:explosion",
                false,
                None,
            ));
        }
    }
    let multiplier = landing.map_or(1., |at| {
        let f = g.res.facts().get(g.world.block(at.x, at.y, at.z));
        match &*f.name {
            "minecraft:water"
            | "minecraft:cobweb"
            | "minecraft:powder_snow"
            | "minecraft:slime_block" => 0.,
            "minecraft:hay_block" | "minecraft:honey_block" => 0.2,
            _ => {
                if f.has_class("AbstractBedBlock") || f.has_class("ShelfMushroomBlock") {
                    0.5
                } else {
                    1.
                }
            }
        }
    });
    let fall_raw = ((p.fall_distance + 1e-6
        - p.attributes
            .computed_value_of("minecraft:safe_fall_distance")
            .unwrap_or(3.))
        * f64::from(multiplier)
        * p.attributes
            .computed_value_of("minecraft:fall_damage_multiplier")
            .unwrap_or(1.))
    .floor() as i32 as f32;
    let fall_damage = if multiplier > 0. {
        effective(g, "minecraft:fall", fall_raw, None, false)
    } else {
        0.
    };
    let eye = [pos.x, pos.y + f64::from(p.dimensions.eye_height), pos.z];
    let mut candidates = Vec::new();
    for m in g.mobs.iter().filter(|m| !m.removed) {
        let b = m.bounds();
        let bounds = [b.min_position(), b.max_position()];
        candidates.push(json!({"id":m.id,"distance":boxed_distance(b,eye),"min":[bounds[0].x,bounds[0].y,bounds[0].z],"max":[bounds[1].x,bounds[1].y,bounds[1].z],"mace":m.extra.main_hand.as_deref()==Some("minecraft:mace"),"above":m.position.y-pos.y,"drop":m.previous.y-m.position.y,"horizontal":((m.position.x-pos.x).powi(2)+(m.position.z-pos.z).powi(2)).sqrt()}));
    }
    for a in &g.armor_stands {
        let b = super::entity::Entity::bounds(a);
        let v = b.min_position();
        let w = b.max_position();
        candidates.push(json!({"id":a.id,"distance":boxed_distance(b,eye),"min":[v.x,v.y,v.z],"max":[w.x,w.y,w.z],"mace":a.equipment[0].as_ref().is_some_and(|s|s.count>0&&&*s.item=="minecraft:mace"),"above":a.position.y-pos.y,"drop":a.previous.y-a.position.y,"horizontal":((a.position.x-pos.x).powi(2)+(a.position.z-pos.z).powi(2)).sqrt()}));
    }
    // Use the same physical boxes and client target identities as combat queries.
    if let Some(d) = g.dragon.as_ref() {
        let at = d.position;
        let b = Aabb::new(
            Vec3::new(at.x - 8., at.y, at.z - 8.),
            Vec3::new(at.x + 8., at.y + 8., at.z + 8.),
        );
        let lo = b.min_position();
        let hi = b.max_position();
        candidates.push(json!({"id":g.client_targeting_dragon_id(),"distance":boxed_distance(b,eye),"min":[lo.x,lo.y,lo.z],"max":[hi.x,hi.y,hi.z],"mace":false,"above":at.y-pos.y,"drop":d.previous.y-at.y,"horizontal":((at.x-pos.x).powi(2)+(at.z-pos.z).powi(2)).sqrt()}));
    }
    for guest in g
        .lan
        .iter()
        .flat_map(|h| h.hub.guests.iter())
        .filter(|p| p.placed && p.dimension == g.dim.id())
    {
        if candidates
            .iter()
            .any(|p| p["id"].as_i64() == Some(i64::from(guest.entity_id)))
        {
            continue;
        }
        let Some(body) = guest.motion.as_ref() else {
            continue;
        };
        let at = guest.position;
        let b = guest.bounds();
        let lo = b.min_position();
        let hi = b.max_position();
        let within = g
            .border
            .geometry()
            .is_within_bounds(at[0].floor(), at[2].floor(), 0.);
        let target = json!({"kind":"minecraft:player","alive":!body.health.dead_or_dying(),"invisible":guest.effects.get("invisibility").is_some(),"sleeping":guest.sleeping,"is_local":false,"has_local_passenger":g.riding==Some(guest.entity_id as u32),"within_border":within,"player_name":guest.name,"abstract_client_player":true,"teams":g.client_teams_facts(guest.entity_id),"murderMysteryUuid":format!("{:032x}",guest.uuid)});
        let mace = guest
            .inventory
            .get(guest.selected)
            .and_then(Option::as_ref)
            .is_some_and(|s| s.count > 0 && &*s.item == "minecraft:mace");
        candidates.push(json!({"id":guest.entity_id,"target":target,"distance":boxed_distance(b,eye),"min":[lo.x,lo.y,lo.z],"max":[hi.x,hi.y,hi.z],"mace":mace,"above":at[1]-pos.y,"drop":guest.previous_position[1]-at[1],"horizontal":((at[0]-pos.x).powi(2)+(at[2]-pos.z).powi(2)).sqrt()}));
    }
    let main = g
        .inventory
        .get(g.selected)
        .and_then(Option::as_ref)
        .filter(|s| s.count > 0)
        .map_or("minecraft:air", |s| &*s.item);
    Some(json!({"creative":g.creative,"spectator":g.spectator(),"dead":p.health.dead_or_dying(),"health":p.health.health()+p.health.absorption(),"maxHealth":p.attributes.computed_value_of("minecraft:max_health").unwrap_or(20.),"missingArmor":g.armor.iter().any(|s|s.as_ref().is_none_or(|s|s.count==0||&*s.item=="minecraft:air")),"burrowed":burrowed,"hole":hole,"fallDistance":p.fall_distance,"fallFlying":p.fall_flying&&p.pose==horizon_client::pose::Pose::FallFlying,"fallDamage":fall_damage,"entityDamage":entity_damage,"blockDamage":block_damage,"eye":eye,"candidates":candidates,"sword":g.items().item_in_tag("minecraft:swords",main),"strengthEffect":g.potions.effects.get("strength").is_some(),"modern":true,"selected":g.selected,"items":inventory_items}).to_string())
}
