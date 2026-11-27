pub fn liquidbounce_forecast_initialize(&mut self) -> Result<()> {
    let water = self.player.fluids.was_touching_water;
    let eye = self.player.fluids.was_eye_in_water;
    self.player.replaying = true;
    let result = self.update_fluids();
    self.player.replaying = false;
    self.player.fluids.was_touching_water = water;
    self.player.fluids.was_eye_in_water = eye;
    result.map(|_| ())
}
/// Agent SimulatedPlayer.aiStep sprint phase, without double-tap/flight
/// toggles, native module queries, live block changes or entity pushes.
pub fn liquidbounce_forecast_tick(&mut self, keys: crate::sprint::Keys) -> Result<()> {
    self.player.replaying = true;
    let result = self.local_tick(keys, |p, keys| {
        let forward = keys.direction()[1] > 1.0e-5;
        let c = p.sprint_conditions();
        let possible = |shallow: bool| {
            !c.mobility_restricted && (c.enough_food || c.may_fly) && (shallow || !c.shallow_water)
        };
        if keys.sprint
            && !p.sprinting
            && forward
            && possible(c.flying)
            && !c.slow_item
            && (c.under_water || !c.fall_flying && !c.moving_slowly)
        {
            p.set_sprinting(true)?;
        }
        if p.sprinting
            && if c.swimming {
                !possible(true) || !c.in_water || !forward && !c.on_ground && !keys.shift
            } else {
                !possible(c.flying)
                    || !forward
                    || c.horizontal_collision && !c.minor_horizontal_collision
            }
        {
            p.set_sprinting(false)?;
        }
        Ok(())
    });
    self.player.replaying = false;
    result
}
