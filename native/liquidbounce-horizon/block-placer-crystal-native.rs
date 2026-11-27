// SPDX-License-Identifier: GPL-3.0-or-later
//! Read-only getExplosionDamageFromEntity(EndCrystal), without health events or equipment wear.
use super::Game;
use horizon_world::{combat_rules::{self,ArmorWorld},damage_source::Scaling,food::Difficulty};
struct NoWeapon;
impl ArmorWorld for NoWeapon{
    type Error=std::convert::Infallible;type Weapon=();type ServerLevel=();
    fn weapon(&mut self)->Result<Option<()>,Self::Error>{Ok(None)}
    fn server_level(&mut self)->Result<Option<()>,Self::Error>{Ok(None)}
    fn modify_armor_effectiveness(&mut self,_:(),_:(),v:f32)->Result<f32,Self::Error>{Ok(v)}
}
impl Game<'_>{
    pub(super) fn block_placer_crystal_would_kill(&mut self,id:i32)->Option<bool>{
        let c=self.crystals.iter().find(|c|c.id as i32==id)?;let center=[c.position.x,c.position.y,c.position.z];
        let p=self.player.as_ref()?;let health=p.health.health()+p.health.absorption();
        if p.health.dead_or_dying(){return Some(health<=0.);}
        let damage=&self.health_resources.damage;let source=damage.id("minecraft:explosion")?;
        let tag=|name:&str|damage.in_tag(source,name);
        if p.invulnerable&&!tag("minecraft:bypasses_invulnerability"){return Some(health<=0.);}
        let distance=(p.position.x-center[0]).powi(2)+(p.position.y-center[1]).powi(2)+(p.position.z-center[2]).powi(2);
        if distance>144.{return Some(health<=0.);}
        let bounds=p.bounds();let lo=bounds.min_position();let hi=bounds.max_position();
        let exposure=self.seen_percent(center,[lo.x,lo.y,lo.z],[hi.x,hi.y,hi.z]);
        let decay=1.-distance.sqrt()/12.;let power=f64::from(exposure)*decay;
        let mut amount=((power*power+power)/2.*7.*12.+1.)as f32;
        // An EndCrystal is not a living causing entity. Only ALWAYS damage
        // types scale here; LivingNonPlayer has no living cause for this source.
        if damage.definition(source)?.scaling==Scaling::Always{amount=match self.world.health_rules.difficulty{Difficulty::Peaceful=>0.,Difficulty::Easy=>(amount/2.+1.).min(amount),Difficulty::Hard=>amount*3./2.,_=>amount};}
        if tag("minecraft:is_fire")&&p.status.fire_resistance{amount=0.;}
        if amount!=0.{amount-=self.player_blocking(Some(center),"minecraft:explosion",false).map_or(0.,|b|b.blocked(amount));}
        let armor_bypass=tag("minecraft:bypasses_armor");let effects_bypass=tag("minecraft:bypasses_effects");let resistance_bypass=tag("minecraft:bypasses_resistance");
        let resistance=p.status.resistance;
        if amount!=0.&&!armor_bypass{
            let p=self.player.as_mut()?;let armor=p.attribute("minecraft:armor").ok()?.floor()as i32 as f32;let toughness=p.attribute("minecraft:armor_toughness").ok()?as f32;
            amount=combat_rules::after_armor(&mut NoWeapon,amount,armor,toughness).expect("infallible armor arithmetic");
        }
        if amount!=0.&&!effects_bypass{
            if let Some(amplifier)=resistance.filter(|_|!resistance_bypass){let remainder=25i32.wrapping_sub(amplifier.wrapping_add(1).wrapping_mul(5));amount=(amount*remainder as f32/25.).max(0.);}
            // ModuleLiquidFiller runs on ClientLevel. Vanilla 26.3 queries
            // enchantment damage protection only for ServerLevel, so its client
            // getDamageAfterMagicAbsorb uses zero enchantment protection.
        }
        Some(health-amount<=0.)
    }
}
