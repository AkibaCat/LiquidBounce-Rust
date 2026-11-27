// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::{
    background::Executor,
    destination::NearDestination,
    geometry::{BlockPos, Vec3},
    owned_search::{OwnedTerrainSearch, SearchSlot, StoredLevel, StoredWorker},
    progress::ProgressWatch,
    replanner::{Replanner, ReplannerLevel, RouteFactory, SearchGoal},
    search::{Limits, Outcome},
    terrain::Terrain,
    walk_jump::vector_yaw,
};
use super::{
    body::{RideBody, RideEntity},
    moves::Moves,
    route_search::Route,
    steering::{Steering, horizontal_distance, sub},
    vehicle::{Vehicle, VehicleKind},
};
use std::sync::Arc;
pub trait RideReport {
    /// The host ties the supplied index to this mover's live progress handle.
    fn following(&mut self, points: &[Vec3], index: usize);
    fn searching(&mut self, position: Vec3, goal: Vec3, radius: f64, preview: Option<&[Vec3]>);
    fn done(&mut self);
}
pub type RideSearchFactory<T, S> =
    Arc<dyn Fn(T, Vec3, SearchGoal, Vehicle, Limits) -> S + Send + Sync>;
type Routes<T, S> = Replanner<VehicleKind, SearchSlot<T, S>, StoredWorker, Route>;
pub struct Riding<T, S, E> {
    routes: Routes<T, S>,
    create: RideSearchFactory<T, S>,
    steering: Steering,
    mount: Option<E>,
    path: Option<Route>,
    index: usize,
    stuck: bool,
    stuck_count: i32,
    watch: ProgressWatch,
    at_goal: bool,
    approach_ticks: i32,
    dismount_ticks: i32,
}
impl<T: Send + 'static, S: OwnedTerrainSearch<T, Route = Route> + Send + 'static, E: RideEntity>
    Riding<T, S, E>
{
    pub fn new(executor: Arc<dyn Executor>, create: RideSearchFactory<T, S>) -> Self {
        Self {
            routes: Replanner::new(16, executor),
            create,
            steering: Steering::default(),
            mount: None,
            path: None,
            index: 0,
            stuck: false,
            stuck_count: 0,
            watch: ProgressWatch::new(0.3, 60),
            at_goal: false,
            approach_ticks: 0,
            dismount_ticks: 0,
        }
    }
    pub fn navigate<B: RideBody<Entity = E>>(
        &mut self,
        body: &mut B,
        level: &mut impl ReplannerLevel<T>,
        report: &mut impl RideReport,
        goal: Vec3,
        radius: f64,
        mut max_length: impl FnMut() -> f64,
    ) -> bool {
        let player = body.player();
        if !player.is_passenger() {
            if self.mount.is_some() || self.dismount_ticks > 0 {
                self.halt()
            }
            return false;
        }
        self.routes.tick();
        let mount = player.controlled_vehicle();
        let vehicle = mount.as_ref().and_then(|m| Vehicle::of(&player, m));
        let (Some(mount), Some(vehicle)) = (mount, vehicle) else {
            self.dismount(body, report, goal);
            return true;
        };
        if self.dismount_ticks > 0 {
            self.dismount(body, report, goal);
            return true;
        }
        self.mount = Some(mount.clone());
        let position = mount.position();
        self.collect_route(level);
        if self
            .path
            .as_ref()
            .is_some_and(|p| self.index < p.points.len())
        {
            let path = self
                .path
                .as_ref()
                .expect("route exists while following")
                .clone();
            self.follow(body, report, &mount, vehicle, &path);
        } else if self.at_goal || self.path.as_ref().is_some_and(|p| p.can_reach) {
            self.approach(body, report, &mount, vehicle, goal)
        } else if self.stuck_count > 4
            || self.routes.failed()
            || self.path.as_ref().is_some_and(|p| p.exhausted)
        {
            self.dismount(body, report, goal)
        } else {
            self.steering.stop(body, false);
            report.searching(position, goal, radius, self.routes.preview());
        }
        if self.dismount_ticks == 0 {
            self.plan_if_needed(level, goal, radius, position, vehicle, &mut max_length)
        }
        true
    }
    pub fn halt(&mut self) {
        self.routes.reset();
        self.steering.reset();
        self.mount = None;
        self.path = None;
        self.index = 0;
        self.stuck = false;
        self.stuck_count = 0;
        self.at_goal = false;
        self.approach_ticks = 0;
        self.dismount_ticks = 0;
    }
    pub fn block_changed(&mut self, pos: BlockPos) {
        self.routes.block_changed(pos)
    }
    pub fn level_changed(&mut self) {
        self.routes.level_changed()
    }
    pub fn is_moving(&self) -> bool {
        self.path
            .as_ref()
            .is_some_and(|p| self.index < p.points.len())
    }
    pub fn head<B: RideBody<Entity = E>>(&self, body: &B, partial_ticks: f32) -> Vec3 {
        self.mount
            .as_ref()
            .cloned()
            .unwrap_or_else(|| body.player())
            .interpolated_position(partial_ticks)
    }
    pub fn ahead(&self) -> &[Vec3] {
        self.path.as_ref().map_or(&[], |p| {
            &p.positions[(self.index + 1).min(p.positions.len())..]
        })
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn path(&self) -> Option<&Route> {
        self.path.as_ref()
    }
    fn follow<B: RideBody<Entity = E>>(
        &mut self,
        body: &mut B,
        report: &mut impl RideReport,
        mount: &E,
        vehicle: Vehicle,
        path: &Route,
    ) {
        let position = mount.position();
        self.advance(position, vehicle, path);
        if self.index >= path.points.len() {
            self.steering.stop(body, false);
            return;
        }
        let point = &path.points[self.index];
        self.track_progress(position, point.position, path);
        let (target, is_end) = if point.jump {
            (point.position, self.index == path.points.len() - 1)
        } else {
            self.straight_ahead(body, position, vehicle, path)
        };
        match vehicle.kind {
            VehicleKind::Boat => {
                let brake = is_end
                    && horizontal_distance(position, target)
                        < super::steering::horizontal_length(mount.delta_movement()) * 10.0 + 1.0;
                self.steering.boat(body, mount, target, brake)
            }
            VehicleKind::Mount => self.steering.mount(
                body,
                mount,
                target,
                point.jump.then_some(point.position),
                vehicle.width / 2.0 + 1.0,
            ),
        }
        report.following(&path.positions, self.index);
    }
    fn straight_ahead<B: RideBody<Entity = E>>(
        &self,
        body: &B,
        position: Vec3,
        vehicle: Vehicle,
        path: &Route,
    ) -> (Vec3, bool) {
        let mut moves = Moves::new(Terrain::player(body.level()), vehicle);
        let mut target = path.points[self.index].position;
        let mut target_index = self.index;
        for i in self.index + 1..path.points.len().min(self.index + 6) {
            let next = &path.points[i];
            if next.jump || !moves.is_straight(position, next.position) {
                break;
            }
            target = next.position;
            target_index = i;
        }
        (target, target_index == path.points.len() - 1)
    }
    fn advance(&mut self, position: Vec3, vehicle: Vehicle, path: &Route) {
        let arrive = if vehicle.kind == VehicleKind::Boat {
            1.0
        } else {
            0.6
        };
        let mut reached = None;
        for i in self.index..path.points.len().min(self.index + 4) {
            let point = &path.points[i];
            if (position.y - point.position.y).abs() > 1.2 {
                continue;
            }
            let distance = horizontal_distance(position, point.position);
            let passed = path.points.get(i + 1).is_some_and(|next| {
                !point.jump
                    && !next.jump
                    && distance <= arrive * 2.0
                    && horizontal_distance(next.position, position)
                        < horizontal_distance(next.position, point.position)
            });
            if passed || distance <= arrive {
                reached = Some(i)
            }
        }
        if let Some(i) = reached {
            self.index = i + 1;
            self.watch.reset();
        }
    }
    fn track_progress(&mut self, position: Vec3, point: Vec3, path: &Route) {
        let distance = horizontal_distance(position, point);
        if !self.watch.stalls(distance) {
            return;
        }
        self.watch.reset();
        if self.index == path.points.len() - 1 && distance < 2.5 {
            self.index = path.points.len()
        } else {
            self.stuck = true;
            self.stuck_count = self.stuck_count.wrapping_add(1);
        }
    }
    fn approach<B: RideBody<Entity = E>>(
        &mut self,
        body: &mut B,
        report: &mut impl RideReport,
        mount: &E,
        vehicle: Vehicle,
        goal: Vec3,
    ) {
        self.approach_ticks = self.approach_ticks.wrapping_add(1);
        if self.approach_ticks > 60 {
            self.dismount(body, report, goal);
            return;
        }
        match vehicle.kind {
            VehicleKind::Boat => self.steering.boat(body, mount, goal, true),
            VehicleKind::Mount => self.steering.mount(body, mount, goal, None, 0.0),
        }
        report.done();
    }
    fn dismount<B: RideBody<Entity = E>>(
        &mut self,
        body: &mut B,
        report: &mut impl RideReport,
        goal: Vec3,
    ) {
        if self.dismount_ticks == 0 {
            self.routes.reset();
            self.path = None;
        }
        self.dismount_ticks = self.dismount_ticks.wrapping_add(1);
        let offset=sub(goal,body.player().position());
        self.steering.face(body,vector_yaw(offset.x,offset.z));
        self.steering.stop(body, self.dismount_ticks > 2);
        report.done();
    }
    fn collect_route(&mut self, level: &mut impl ReplannerLevel<T>) {
        match self.routes.poll(&mut StoredLevel::new(level)) {
            Outcome::Searching | Outcome::Failed(_) => {}
            Outcome::AlreadyThere => {
                self.path = None;
                self.at_goal = true
            }
            Outcome::Found(path) => {
                self.index = 0;
                self.stuck = false;
                self.at_goal = false;
                self.approach_ticks = 0;
                self.watch.reset();
                self.routes.route_ends(
                    path.points
                        .last()
                        .expect("ride result contains a point")
                        .node,
                );
                self.path = Some(path);
            }
        }
    }
    fn plan_if_needed(
        &mut self,
        level: &mut impl ReplannerLevel<T>,
        goal: Vec3,
        radius: f64,
        position: Vec3,
        vehicle: Vehicle,
        max_length: &mut impl FnMut() -> f64,
    ) {
        if self.stuck_count > 4 {
            self.path = None;
            return;
        }
        let path = self.path.as_ref();
        let riding = path.is_some_and(|p| self.index < p.points.len());
        let off_path = path.is_some_and(|p| {
            riding
                && horizontal_segment_distance(
                    position,
                    p.positions[self.index],
                    p.positions[self.index + 1],
                ) > 3.0
        });
        let finished_short = path.is_some_and(|p| !riding && !p.can_reach && !p.exhausted);
        let near_stage = path.is_some_and(|p| {
            self.routes.ends_at_stage()
                && horizontal_distance(position, p.end()) < Routes::<T, S>::STAGE_LEAD
        });
        let urgent = self.stuck || off_path;
        let needed = self.routes.has_goal_moved(goal)
            || off_path
            || finished_short
            || self.stuck
            || near_stage;
        if !needed || !self.routes.is_due(urgent) {
            return;
        }
        let max_length = max_length();
        let limits = Limits::new(max_length);
        let create = self.create.clone();
        let search: RouteFactory<SearchSlot<T, S>, StoredWorker> =
            Arc::new(move |slot, descriptor| {
                slot.initialize(|terrain| create(terrain, position, descriptor, vehicle, limits))
            });
        self.routes.request(
            &mut StoredLevel::new(level),
            position,
            NearDestination::new(goal, radius),
            max_length,
            vehicle.kind,
            max_length / 2.0,
            search,
        );
        self.stuck = false;
    }
}
fn horizontal_segment_distance(p: Vec3, from: Vec3, to: Vec3) -> f64 {
    let a = Vec3::new(from.x, 0.0, from.z);
    let b = Vec3::new(to.x, 0.0, to.z);
    let p = Vec3::new(p.x, 0.0, p.z);
    let ab = sub(b, a);
    let length = ab.distance_sqr(Vec3::ZERO);
    if length.abs() < f64::from(1.0e-5_f32) {
        return p.distance(a);
    }
    let ap = sub(p, a);
    let projected = (ab.x * ap.x + ab.y * ap.y + ab.z * ap.z) / length;
    assert!(projected.is_finite(), "cannot project onto ride segment");
    let t = projected.clamp(0.0, 1.0);
    p.distance(Vec3::new(
        t.mul_add(ab.x, a.x),
        t.mul_add(ab.y, a.y),
        t.mul_add(ab.z, a.z),
    ))
}
