// SPDX-License-Identifier: GPL-3.0-or-later
//! Live facts and actions the complete flight controller reads from Body/Player.
use super::{
    aabb::Aabb,
    flight::FlyAbilities,
    flight_handling::{FlightPlayer, Handling},
    geometry::Vec3,
    input::Input,
};
pub trait PlayerFlightAbilities {
    fn flying(&self) -> bool;
    fn may_fly(&self) -> bool;
    fn flying_speed(&self) -> f32;
}
pub trait FlightActor: FlightPlayer {
    type Abilities: PlayerFlightAbilities;
    fn abilities(&self) -> &Self::Abilities;
    fn y(&self) -> f64;
    fn on_ground(&self) -> bool;
    fn bounding_box(&self) -> Aabb;
    fn interpolated_position(&self, partial_ticks: f32) -> Vec3;
}
pub trait FlightBody {
    type Player: FlightActor;
    fn player(&self) -> &Self::Player;
    fn is_flying(&self) -> bool;
    fn press(&mut self, input: Input, exact: bool);
    fn no_block_collision(&mut self, bounding_box: Aabb) -> bool;
}
pub fn capture_flight(
    body: &impl FlightBody,
    can_fly: bool,
    handling: &mut Handling,
) -> Option<FlyAbilities> {
    if !can_fly {
        return None;
    }
    let abilities = body.player().abilities();
    let base = if body.is_flying() {
        FlyAbilities::creative(abilities.flying_speed(), true)
    } else {
        if !abilities.may_fly() && !abilities.flying() {
            return None;
        }
        FlyAbilities::creative(abilities.flying_speed(), abilities.flying())
    };
    Some(handling.abilities(base))
}
