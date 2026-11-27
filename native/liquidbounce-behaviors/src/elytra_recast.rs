// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleElytraRecast and MixinLivingEntity.recastIfLanded, ae0eb396.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ElytraRecast {
    /// Entity-owned mixin field; it is sampled even while the module is off.
    pub previous_elytra: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Chest {
    pub glider: bool,
    pub next_damage_will_break: bool,
}

/// Calls stay ordered: sound stop, local flag, ordinary packet, cooldown reset.
pub trait RecastWorld {
    type Error;
    fn chest(&mut self) -> Chest;
    fn flying(&self) -> bool;
    fn passenger(&self) -> bool;
    fn climbable(&mut self) -> Result<bool, Self::Error>;
    fn in_water(&self) -> bool;
    fn levitating(&self) -> bool;
    fn fall_flying(&self) -> bool;
    fn jump_key(&self) -> bool;
    fn input_jump(&self) -> bool;
    fn stop_elytra_sound(&mut self);
    fn start_fall_flying(&mut self);
    fn send_start_fall_flying(&mut self);
    fn reset_jump_delay(&mut self);
}

impl ElytraRecast {
    /// Module's public function does not itself test module.running.
    pub fn recast_elytra<W: RecastWorld>(world: &mut W) -> Result<bool, W::Error> {
        // Kotlin reads the chest stack before evaluating the short-circuit chain.
        let chest = world.chest();
        if !world.flying()
            && !world.passenger()
            && !world.climbable()?
            && !world.in_water()
            && !world.levitating()
            && !world.fall_flying()
            && chest.glider
            && !chest.next_damage_will_break
            && (world.jump_key() || world.input_jump())
        {
            world.start_fall_flying();
            world.send_start_fall_flying();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Called only for the actual local entity, at LivingEntity.aiStep TAIL.
    pub fn ai_step_tail<W: RecastWorld>(
        &mut self,
        running: bool,
        world: &mut W,
    ) -> Result<(), W::Error> {
        let elytra = world.fall_flying();
        if running && self.previous_elytra && !elytra {
            world.stop_elytra_sound();
            Self::recast_elytra(world)?;
            // Reset even if shouldRecast rejected; never gated by its result.
            world.reset_jump_delay();
        }
        // Use the value before the recast, not the flag startFallFlying set.
        self.previous_elytra = elytra;
        Ok(())
    }
}
