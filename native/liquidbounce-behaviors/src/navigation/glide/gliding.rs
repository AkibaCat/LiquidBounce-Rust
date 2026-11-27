// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete navigation/glide/Gliding.kt orchestration.
use super::super::search::Outcome;
use super::super::{
    background::{BackgroundSearch, Executor, SearchFactory},
    geometry::{BlockPos, Vec3},
    input::Input,
    owned_search::{OwnedTerrainSearch, SearchSlot, StoredLevel, StoredWorker},
    replanner::ReplannerLevel,
    terrain::{Terrain, TerrainLevel, TerrainState},
    walk_jump::vector_yaw,
};
use super::{
    Controls, Steering,
    abilities::{GlideAbilities, GlideStack},
    airways::{AirwayFactory, Airways},
    body::{GlideBody, GlideHand, GlideLevel, GlideLook, GlidePlayer},
    capture::{EquippedGlidePlayer, GlideInventory},
    contacts::{ContactBody, ContactListener, ContactPlayer, Contacts},
    cruise::{ColumnHeights, Cruise, cruise_altitude, rockets_needed},
    landing::Landing,
    passage::Passage,
    rockets::Rockets,
    roof_bounce::{directional_input, horizontal_distance, sub},
    route_search::{Route, region},
};
use std::{marker::PhantomData, sync::Arc};

pub trait GlideReport: ContactListener {
    fn blocks_empty(&mut self);
    /// TargetProgress remains tied to this mover in the host's progress registry.
    fn following(&mut self, points: &[Vec3], landing: Vec3);
}
pub type GlideSearchFactory<T, S> =
    Arc<dyn Fn(T, Vec3, Vec3, f64, GlideAbilities) -> S + Send + Sync>;
type Search<T, S> = BackgroundSearch<(BlockPos, Vec3), SearchSlot<T, S>, StoredWorker, Route>;
type WorldProvider<L> = Arc<dyn Fn() -> L>;
type Fallback = Box<dyn FnMut(Vec3) -> Option<Vec3>>;
struct LiveHeights<B, L> {
    world: WorldProvider<L>,
    marker: PhantomData<fn() -> B>,
}
impl<B: TerrainState, L: GlideLevel<B>> ColumnHeights for LiveHeights<B, L> {
    fn top(&mut self, x: i32, z: i32) -> Option<i32> {
        if (self.world)().has_column(x, z) {
            Some((self.world)().column_height(x, z))
        } else {
            None
        }
    }
}
enum Control<B, L> {
    Landing(Landing),
    Cruise(Box<Cruise<LiveHeights<B, L>, Fallback>>),
    Passage(Passage),
}
impl<B: TerrainState, L: GlideLevel<B>> Control<B, L> {
    fn target(&self) -> Vec3 {
        match self {
            Self::Landing(x) => x.target(),
            Self::Cruise(x) => x.target(),
            Self::Passage(x) => x.target(),
        }
    }
    fn steer(&mut self, position: Vec3, motion: Vec3, world: WorldProvider<L>) -> Controls {
        match self {
            Self::Landing(x) => x.steer(position, motion),
            Self::Cruise(x) => x.steer(position, motion),
            Self::Passage(x) => x.steer_with(|| Terrain::player(world()), position, motion),
        }
    }
}
struct Glide<B, L> {
    goal: Vec3,
    steering: Control<B, L>,
    path: Option<Route>,
    ticks: i32,
    has_opened: bool,
    pressed_at: i32,
    jumped: bool,
}
impl<B: TerrainState, L: GlideLevel<B>> Glide<B, L> {
    fn new(goal: Vec3, steering: Control<B, L>, path: Option<Route>) -> Self {
        Self {
            goal,
            steering,
            path,
            ticks: 0,
            has_opened: false,
            pressed_at: -10,
            jumped: false,
        }
    }
    fn points(&self, position: Vec3) -> Vec<Vec3> {
        if let Some(path) = &self.path {
            return path.points.clone();
        }
        if let Control::Passage(p) = &self.steering
            && p.route().len() > 1
        {
            return p.route().to_vec();
        }
        vec![position, self.steering.target()]
    }
    fn landing(&self) -> Vec3 {
        self.path
            .as_ref()
            .map_or_else(|| self.steering.target(), |p| p.landing)
    }
}
pub struct Gliding<T, S, AS, B, L> {
    search: Search<T, S>,
    create: GlideSearchFactory<T, S>,
    airways: Airways<T, AS>,
    contacts: Contacts,
    glide: Option<Glide<B, L>>,
    ticks: i32,
    retry_at: i32,
    should_fire_rocket: bool,
    holds_rocket_slot: bool,
    sprint: bool,
    world: WorldProvider<L>,
}
impl<
    T: Send + 'static,
    S: OwnedTerrainSearch<T, Route = Route> + Send + 'static,
    AS: OwnedTerrainSearch<T, Route = super::super::flight::FlyRoute> + Send + 'static,
    B: TerrainState + 'static,
    L: GlideLevel<B> + 'static,
> Gliding<T, S, AS, B, L>
{
    pub fn new(
        executor: Arc<dyn Executor>,
        create: GlideSearchFactory<T, S>,
        airway: AirwayFactory<T, AS>,
        world: WorldProvider<L>,
    ) -> Self {
        Self {
            search: BackgroundSearch::new(executor.clone()),
            create,
            airways: Airways::new(executor, airway),
            contacts: Contacts::default(),
            glide: None,
            ticks: 0,
            retry_at: 0,
            should_fire_rocket: false,
            holds_rocket_slot: false,
            sprint: false,
            world,
        }
    }
    pub fn is_active(&self) -> bool {
        self.glide.is_some()
    }
    pub fn sprint(&self) -> bool {
        self.sprint
    }
    pub fn target(&self) -> Option<Vec3> {
        self.glide.as_ref().map(Glide::landing)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn navigate(
        &mut self,
        body: &mut impl GlideBody<State = B, Level = L>,
        level: &mut impl ReplannerLevel<T>,
        report: &mut impl GlideReport,
        goal: Vec3,
        radius: f64,
        abilities: Option<GlideAbilities>,
        direct: bool,
        frame_y_rot: f32,
    ) -> bool {
        self.ticks = self.ticks.wrapping_add(1);
        self.release_rocket_slot(body);
        self.contacts.tick(&ContactAdapter(body), report);
        if self.glide.is_some() {
            self.retarget(body, goal, abilities)
        } else {
            self.glide = self.start(body, level, goal, radius, abilities, direct)
        }
        let Some(mut glide) = self.glide.take() else {
            return false;
        };
        if !self.fly(body, report, &mut glide, frame_y_rot) {
            return false;
        }
        if let Control::Passage(passage) = &mut glide.steering {
            passage.lands_at_target = has_ground(body, goal);
            self.airways.tick(
                passage,
                level,
                goal,
                radius,
                || body.player().position(),
                || body.glide_clearance(),
            );
        }
        self.steer(body, &mut glide);
        self.use_rocket(body);
        self.glide = Some(glide);
        true
    }
    pub fn halt(&mut self, body: &mut impl GlideBody) {
        self.search.cancel();
        self.airways.reset();
        self.glide = None;
        self.should_fire_rocket = false;
        self.sprint = false;
        self.release_rocket_slot(body);
    }
    pub fn block_changed(&mut self, pos: BlockPos) {
        self.search.block_changed(pos);
        self.airways.block_changed(pos)
    }
    pub fn level_changed(&mut self) {
        self.search.level_changed();
        self.airways.level_changed()
    }
    fn retarget(
        &mut self,
        body: &impl GlideBody<State = B, Level = L>,
        goal: Vec3,
        abilities: Option<GlideAbilities>,
    ) {
        let glide = self.glide.as_ref().expect("active glide");
        if abilities.is_none()
            || body.player().on_ground()
            || glide.goal.distance_sqr(goal) <= 1.5 * 1.5
        {
            return;
        }
        let opened = glide.has_opened;
        self.glide = Some(self.passage(
            body,
            goal,
            abilities.expect("checked glide equipment"),
            opened,
        ));
    }
    fn start(
        &mut self,
        body: &impl GlideBody<State = B, Level = L>,
        level: &mut impl ReplannerLevel<T>,
        goal: Vec3,
        radius: f64,
        abilities: Option<GlideAbilities>,
        direct: bool,
    ) -> Option<Glide<B, L>> {
        let Some(abilities) = abilities else {
            self.search.cancel();
            return None;
        };
        if !body.player().on_ground()
            && self.ticks >= self.retry_at
            && (body.player().is_fall_flying() || is_falling_far(body))
        {
            return Some(self.passage(body, goal, abilities, body.player().is_fall_flying()));
        }
        if body.player().is_fall_flying() {
            return None;
        }
        let glide = self
            .cruise(body, goal, abilities, direct)
            .or_else(|| self.planned(body, level, goal, abilities));
        if glide.is_none() {
            self.plan_if_useful(body, level, goal, radius, abilities, direct)
        }
        glide
    }
    fn cruise(
        &mut self,
        body: &impl GlideBody<State = B, Level = L>,
        goal: Vec3,
        abilities: GlideAbilities,
        direct: bool,
    ) -> Option<Glide<B, L>> {
        if direct
            || self.ticks < self.retry_at
            || !body.player().on_ground()
            || body.level().has_ceiling()
        {
            return None;
        }
        let start = body.player().position();
        let distance = horizontal_distance(start, goal);
        let loaded = body.level().has_chunk_at(BlockPos::containing(goal));
        if distance < 16.0 || distance <= 192.0 && loaded {
            return None;
        }
        let mut heights = LiveHeights::<B, L> {
            world: self.world.clone(),
            marker: PhantomData,
        };
        let climb = cruise_altitude(start, goal, &mut heights) - start.y;
        if abilities.rockets < rockets_needed(distance, climb, abilities.rocket_ticks) {
            return None;
        }
        self.search.cancel();
        let rockets = Rockets::new(abilities.rockets, abilities.rocket_ticks);
        let ceiling = body.level().max_y();
        let world = self.world.clone();
        let fallback: Fallback = Box::new(move |position| ground_near::<B, L>(&world, position));
        Some(Glide::new(
            goal,
            Control::Cruise(Box::new(Cruise::new(
                goal, heights, rockets, ceiling, fallback,
            ))),
            None,
        ))
    }
    fn planned(
        &mut self,
        body: &impl GlideBody<State = B, Level = L>,
        level: &mut impl ReplannerLevel<T>,
        goal: Vec3,
        abilities: GlideAbilities,
    ) -> Option<Glide<B, L>> {
        match self.search.poll(&mut StoredLevel::new(level)) {
            Outcome::Searching => None,
            Outcome::Found(path) => {
                self.search.cancel();
                if body.player().on_ground()
                    && path.start.distance_sqr(body.player().position()) < 1.0
                {
                    let rockets = Rockets::new(path.rockets, abilities.rocket_ticks);
                    Some(Glide::new(
                        goal,
                        Control::Landing(Landing::new(goal, rockets, path.approach)),
                        Some(path),
                    ))
                } else {
                    None
                }
            }
            Outcome::AlreadyThere | Outcome::Failed(_) => {
                self.search.cancel();
                self.retry_at = self.ticks.wrapping_add(40);
                None
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn plan_if_useful(
        &mut self,
        body: &impl GlideBody<State = B, Level = L>,
        level: &mut impl ReplannerLevel<T>,
        goal: Vec3,
        radius: f64,
        abilities: GlideAbilities,
        direct: bool,
    ) {
        if !self.search.is_idle()
            || self.ticks < self.retry_at
            || direct
            || !body.player().on_ground()
        {
            return;
        }
        let start = body.player().position();
        let distance = horizontal_distance(start, goal);
        let drop = start.y - goal.y;
        if distance < 16.0 || distance > 192.0 || drop < 6.0 && abilities.rockets == 0 {
            return;
        }
        let create = self.create.clone();
        let factory: SearchFactory<SearchSlot<T, S>, StoredWorker> = Arc::new(move |slot| {
            slot.initialize(|terrain| create(terrain, start, goal, radius, abilities))
        });
        self.search.request(
            &mut StoredLevel::new(level),
            (BlockPos::containing(start), goal),
            region(start, goal),
            factory,
        );
    }
    fn fly(
        &mut self,
        body: &mut impl GlideBody,
        report: &mut impl GlideReport,
        glide: &mut Glide<B, L>,
        frame_y_rot: f32,
    ) -> bool {
        glide.ticks = glide.ticks.wrapping_add(1);
        if body.player().is_fall_flying() && !body.player().on_ground() {
            glide.has_opened = true
        } else if glide.has_opened && body.player().on_ground()
            || !glide.has_opened && glide.ticks > 24
        {
            if !glide.has_opened {
                self.retry_at = self.ticks.wrapping_add(40)
            }
            self.halt(body);
            return false;
        }
        let jump = if body.player().is_fall_flying() {
            false
        } else if body.player().on_ground() {
            true
        } else {
            press_to_open(glide)
        };
        glide.jumped = jump;
        let direction = if glide.has_opened {
            Input::EMPTY
        } else {
            directional_input(sub(glide.goal, body.player().position()), frame_y_rot)
        };
        body.press(direction.with_keys(jump, false, false), true);
        self.sprint = !glide.has_opened;
        report.blocks_empty();
        let points = glide.points(body.player().position());
        report.following(&points, glide.landing());
        true
    }
    fn steer(&mut self, body: &mut impl GlideBody, glide: &mut Glide<B, L>) {
        if body.player().is_fall_flying() {
            let input = glide.steering.steer(
                body.player().position(),
                body.player().delta_movement(),
                self.world.clone(),
            );
            self.should_fire_rocket = input.should_fire_rocket;
            body.look(input.y_rot, input.x_rot, GlideLook::Steer);
        } else {
            let offset = sub(glide.goal, body.player().position());
            body.look(vector_yaw(offset.x, offset.z), 0.0, GlideLook::Movement);
        }
    }
    fn use_rocket(&mut self, body: &mut impl GlideBody) {
        if !self.should_fire_rocket {
            return;
        }
        self.should_fire_rocket = false;
        if body.player().offhand_item().is_firework_rocket() {
            body.use_item(GlideHand::Off);
            return;
        }
        let inventory = body.player().inventory();
        let Some(slot) = (0..9).find(|&i| inventory.item(i).is_firework_rocket()) else {
            return;
        };
        body.select_hotbar_slot(slot);
        self.holds_rocket_slot = true;
        body.use_item(GlideHand::Main);
    }
    fn release_rocket_slot(&mut self, body: &mut impl GlideBody) {
        if self.holds_rocket_slot {
            self.holds_rocket_slot = false;
            body.release_hotbar_slot()
        }
    }
    fn passage(
        &mut self,
        body: &impl GlideBody<State = B, Level = L>,
        goal: Vec3,
        abilities: GlideAbilities,
        opened: bool,
    ) -> Glide<B, L> {
        self.search.cancel();
        self.airways.reset();
        let rockets = Rockets::new(abilities.rockets, abilities.rocket_ticks);
        let clearance = body.glide_clearance();
        let steering = Passage::new(goal, rockets, clearance, has_ground(body, goal));
        let mut glide = Glide::new(goal, Control::Passage(steering), None);
        glide.has_opened = opened;
        glide
    }
}
fn press_to_open<B: TerrainState, L: GlideLevel<B>>(glide: &mut Glide<B, L>) -> bool {
    if glide.jumped || glide.ticks.wrapping_sub(glide.pressed_at) < 10 {
        return false;
    }
    glide.pressed_at = glide.ticks;
    true
}
fn has_ground<B: TerrainState, L: GlideLevel<B>>(
    body: &impl GlideBody<State = B, Level = L>,
    goal: Vec3,
) -> bool {
    Terrain::player(body.level()).is_standable(BlockPos::containing(goal))
}
fn is_falling_far(body: &impl GlideBody) -> bool {
    if body.player().delta_movement().y >= 0.0 {
        return false;
    }
    let feet = body.player().block_position();
    for depth in 0..=body.safe_fall().wrapping_add(1) {
        let pos = feet.offset(0, depth.wrapping_neg(), 0);
        let level = body.level();
        let state = level.block_state(pos);
        if !state.collision_shape(&body.level(), pos).is_empty() || state.has_liquid() {
            return false;
        }
    }
    true
}
fn ground_near<B: TerrainState, L: GlideLevel<B>>(
    world: &WorldProvider<L>,
    position: Vec3,
) -> Option<Vec3> {
    let mut best = None;
    let mut distance = f64::MAX;
    let x0 = position.x.floor() as i32;
    let z0 = position.z.floor() as i32;
    let mut heights = LiveHeights::<B, L> {
        world: world.clone(),
        marker: PhantomData,
    };
    for dx in (-96_i32..=96).step_by(8) {
        for dz in (-96_i32..=96).step_by(8) {
            let Some(top) = heights.top(x0.wrapping_add(dx), z0.wrapping_add(dz)) else {
                continue;
            };
            let squared = f64::from(dx * dx + dz * dz);
            if top > world().min_y() && f64::from(top) < position.y && squared < distance {
                best = Some(Vec3::new(
                    f64::from(x0.wrapping_add(dx)) + 0.5,
                    f64::from(top),
                    f64::from(z0.wrapping_add(dz)) + 0.5,
                ));
                distance = squared
            }
        }
    }
    best
}
struct ContactAdapter<'a, B>(&'a B);
impl<B: GlideBody> ContactBody for ContactAdapter<'_, B> {
    type Player = B::Player;
    type State = B::State;
    type Level = B::Level;
    fn player(&self) -> &Self::Player {
        self.0.player()
    }
    fn live_level(&self) -> Self::Level {
        self.0.level()
    }
    fn direction_hash(&self, d: super::super::aabb::Direction) -> i32 {
        self.0.direction_hash(d)
    }
}
