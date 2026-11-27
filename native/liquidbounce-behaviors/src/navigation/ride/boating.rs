// SPDX-License-Identifier: GPL-3.0-or-later
//! Inventory boat placement, boarding, shore recovery and pickup.
use super::super::{
    aabb::Aabb,
    geometry::{BlockPos, Vec3},
    terrain::{TerrainLevel, TerrainState},
};
use super::{
    body::{RideBody, RideEntity, RideLook},
    moves::RideState,
    steering::sub,
    view_rotation::ViewRotation,
};
pub trait BoatState: RideState {
    fn is_air(&self) -> bool;
}
pub trait BoatInventory: Clone {
    fn item_is_boat(&self, slot: i32) -> bool;
    fn suitable_hotbar_slot(&self) -> i32;
}
pub trait BoatEntity: RideEntity {
    type Inventory: BoatInventory;
    fn on_ground(&self) -> bool;
    fn is_in_water(&self) -> bool;
    fn eye_position(&self) -> Vec3;
    fn block_interaction_range(&self) -> f64;
    fn entity_interaction_range(&self) -> f64;
    fn inventory(&self) -> Self::Inventory;
    fn main_hand_is_boat(&self) -> bool;
    fn passengers_empty(&self) -> bool;
    fn is_alive(&self) -> bool;
    fn distance_to(&self, other: &Self) -> f32;
    fn attack_strength_scale(&self, partial: f32) -> f32;
    fn item_is_boat(&self) -> bool;
}
pub trait BoatFluid: Clone {
    fn is_water(&self) -> bool;
    fn height<B: BoatState, L: BoatLevel<B>>(&self, level: &L, pos: BlockPos) -> f32;
}
pub trait BoatLevel<B: BoatState>: TerrainLevel<B> {
    type Entity: BoatEntity;
    type Fluid: BoatFluid;
    fn fluid_state(&self, pos: BlockPos) -> Self::Fluid;
    fn boats(&self, box_: Aabb) -> Vec<Self::Entity>;
    fn items(&self, box_: Aabb) -> Vec<Self::Entity>;
    /// OUTLINE / ANY fluid, with the supplied player's collision context.
    fn boat_clip(&self, eye: Vec3, end: Vec3, player: &Self::Entity) -> Option<BlockPos>;
}
pub trait BoatBody:
    RideBody<State: BoatState, Entity: BoatEntity, Level: BoatLevel<Self::State, Entity = Self::Entity>>
{
    fn can_boat(&self) -> bool;
    fn can_click_inventory(&self) -> bool;
    fn should_stand(&mut self);
    fn select_hotbar_slot(&mut self, slot: i32);
    fn release_hotbar_slot(&mut self);
    fn swap_with_hotbar(&mut self, inventory: i32, hotbar: i32);
    fn sent_view_vector(&self) -> Vec3;
    fn use_main_hand(&mut self) -> bool;
    fn sees(&self, entity: &Self::Entity) -> bool;
    fn interact_main_hand(&mut self, entity: &Self::Entity);
    fn attack(&mut self, entity: &Self::Entity);
}
pub trait SwimAhead {
    fn swim_ahead(&mut self) -> Vec<BlockPos>;
}
#[derive(Clone)]
enum Phase<E> {
    Idle,
    Placing { water: BlockPos, hotbar: i32 },
    Placed { water: BlockPos },
    Boarding(E),
    Riding(E),
    Breaking(E),
    Collecting(Vec3),
}
pub struct Boating<E> {
    phase: Phase<E>,
    ticks: i32,
    refused: Option<BlockPos>,
    walk_target: Option<Vec3>,
}
impl<E: BoatEntity> Default for Boating<E> {
    fn default() -> Self {
        Self {
            phase: Phase::Idle,
            ticks: 0,
            refused: None,
            walk_target: None,
        }
    }
}
impl<E: BoatEntity> Boating<E> {
    pub fn walk_target(&self) -> Option<Vec3> {
        self.walk_target
    }
    pub fn tick<B: BoatBody<Entity = E>>(
        &mut self,
        body: &mut B,
        follower: &mut impl SwimAhead,
    ) -> bool {
        let player = body.player();
        self.walk_target = None;
        self.ticks = self.ticks.wrapping_add(1);
        match self.phase.clone() {
            Phase::Idle => {
                let Some(water) = self.spot(body, follower) else {
                    return false;
                };
                let slot = self.boat_slot(body);
                if slot < 0 {
                    return false;
                }
                if !(0..9).contains(&slot) && !body.can_click_inventory() {
                    body.should_stand();
                    return true;
                }
                let hotbar = self.hold(body, slot);
                self.enter(body, Phase::Placing { water, hotbar });
                true
            }
            Phase::Placing { water, hotbar } => self.place(body, water, hotbar),
            Phase::Placed { water } => {
                let boat = body
                    .level()
                    .boats(Aabb::from_block(water).inflate(Vec3::new(1.5, 1.5, 1.5)))
                    .into_iter()
                    .find(|b| b.passengers_empty());
                if let Some(boat) = boat {
                    self.enter(body, Phase::Boarding(boat))
                } else if self.ticks > 10 {
                    self.refuse(body, water)
                }
                true
            }
            Phase::Boarding(boat) => self.board(body, boat),
            Phase::Riding(boat) => {
                if player.is_passenger() {
                    return false;
                }
                self.enter(body, Phase::Breaking(boat));
                true
            }
            Phase::Breaking(boat) => self.break_boat(body, boat),
            Phase::Collecting(at) => self.collect(body, at),
        }
    }
    pub fn halt(&mut self, body: &mut impl BoatBody<Entity = E>) {
        if !matches!(self.phase, Phase::Riding(_)) {
            self.enter(body, Phase::Idle)
        }
        self.walk_target = None;
    }
    fn enter(&mut self, body: &mut impl BoatBody<Entity = E>, next: Phase<E>) {
        if matches!(self.phase, Phase::Placing { .. }) && !matches!(next, Phase::Placing { .. }) {
            body.release_hotbar_slot()
        }
        self.phase = next;
        self.ticks = 0;
    }
    fn refuse(&mut self, body: &mut impl BoatBody<Entity = E>, water: BlockPos) {
        self.refused = Some(water);
        self.enter(body, Phase::Idle)
    }
    fn spot<B: BoatBody<Entity = E>>(
        &self,
        body: &B,
        follower: &mut impl SwimAhead,
    ) -> Option<BlockPos> {
        let player = body.player();
        if !body.can_boat() || player.is_passenger() || !player.on_ground() || player.is_in_water()
        {
            return None;
        }
        let swim = follower.swim_ahead();
        if swim.len() < 5 {
            return None;
        }
        let level = body.level();
        let eye = player.eye_position();
        for pos in swim.into_iter().take(3) {
            if Some(pos) == self.refused
                || !level.fluid_state(pos).is_water()
                || !level.block_state(pos.above()).is_air()
                || Vec3::new(
                    f64::from(pos.x) + 0.5,
                    f64::from(pos.y) + 0.5,
                    f64::from(pos.z) + 0.5,
                )
                .distance(eye)
                    > player.block_interaction_range() - 0.5
            {
                continue;
            }
            let mut clear = true;
            'scan: for z in pos.z.wrapping_sub(1)..=pos.z.wrapping_add(1) {
                for x in pos.x.wrapping_sub(1)..=pos.x.wrapping_add(1) {
                    let at = BlockPos::new(x, pos.y, z);
                    if !level.block_state(at).collision_shape(&level, at).is_empty() {
                        clear = false;
                        break 'scan;
                    }
                }
            }
            if clear {
                return Some(pos);
            }
        }
        None
    }
    fn boat_slot(&self, body: &impl BoatBody<Entity = E>) -> i32 {
        let inventory = body.player().inventory();
        (0..36)
            .find(|slot| inventory.item_is_boat(*slot))
            .unwrap_or(-1)
    }
    fn hold(&self, body: &mut impl BoatBody<Entity = E>, slot: i32) -> i32 {
        let hotbar = if (0..9).contains(&slot) {
            slot
        } else {
            let hotbar = body.player().inventory().suitable_hotbar_slot();
            body.swap_with_hotbar(slot, hotbar);
            hotbar
        };
        body.select_hotbar_slot(hotbar);
        hotbar
    }
    fn place<B: BoatBody<Entity = E>>(
        &mut self,
        body: &mut B,
        water: BlockPos,
        hotbar: i32,
    ) -> bool {
        let player = body.player();
        if self.ticks > 20 {
            self.refuse(body, water);
            return false;
        }
        body.select_hotbar_slot(hotbar);
        let level = body.level();
        let surface = Vec3::new(
            f64::from(water.x) + 0.5,
            f64::from(water.y) + f64::from(level.fluid_state(water).height(&level, water)),
            f64::from(water.z) + 0.5,
        );
        let view = ViewRotation::from_vector(sub(surface, player.eye_position()));
        body.look(view.y_rot, view.x_rot, RideLook::Aim);
        let eye = player.eye_position();
        let look = body.sent_view_vector();
        let scale = player.block_interaction_range();
        let end = Vec3::new(
            eye.x + look.x * scale,
            eye.y + look.y * scale,
            eye.z + look.z * scale,
        );
        if level.boat_clip(eye, end, &player) == Some(water)
            && player.main_hand_is_boat()
            && body.use_main_hand()
        {
            self.enter(body, Phase::Placed { water })
        }
        true
    }
    fn board<B: BoatBody<Entity = E>>(&mut self, body: &mut B, boat: E) -> bool {
        let player = body.player();
        if player.is_passenger() {
            self.enter(body, Phase::Riding(boat));
            return false;
        }
        if !boat.is_alive() || self.ticks > 40 {
            self.refused = Some(BlockPos::containing(boat.position()));
            self.enter(body, Phase::Breaking(boat));
            return true;
        }
        let bounds = boat.bounding_box();
        let center = bounds.center();
        let view = ViewRotation::from_vector(sub(center, player.eye_position()));
        body.look(view.y_rot, view.x_rot, RideLook::Aim);
        if self.ticks % 5 == 0 && body.sees(&boat) {
            body.interact_main_hand(&boat)
        }
        true
    }
    fn break_boat<B: BoatBody<Entity = E>>(&mut self, body: &mut B, boat: E) -> bool {
        let player = body.player();
        if !boat.is_alive() {
            self.enter(body, Phase::Collecting(boat.position()));
            return true;
        }
        let distance = boat.distance_to(&player);
        if self.ticks > 200 || f64::from(distance) > player.entity_interaction_range() + 3.0 {
            self.enter(body, Phase::Idle);
            return false;
        }
        let bounds = boat.bounding_box();
        let center = bounds.center();
        let view = ViewRotation::from_vector(sub(center, player.eye_position()));
        body.look(view.y_rot, view.x_rot, RideLook::Aim);
        if f64::from(distance) > player.entity_interaction_range() {
            self.walk_target = Some(boat.position())
        } else if player.attack_strength_scale(0.5) >= 0.9_f32 && body.sees(&boat) {
            body.attack(&boat)
        }
        true
    }
    fn collect<B: BoatBody<Entity = E>>(&mut self, body: &mut B, at: Vec3) -> bool {
        let item = body
            .level()
            .items(Aabb::new(
                Vec3::new(at.x - 3.0, at.y - 2.0, at.z - 3.0),
                Vec3::new(at.x + 3.0, at.y + 2.0, at.z + 3.0),
            ))
            .into_iter()
            .find(|item| item.item_is_boat());
        let Some(item) = item else {
            if self.ticks > 10 {
                self.enter(body, Phase::Idle)
            }
            return self.ticks <= 10;
        };
        if self.ticks > 100 {
            self.enter(body, Phase::Idle);
            return false;
        }
        self.walk_target = Some(item.position());
        true
    }
}
