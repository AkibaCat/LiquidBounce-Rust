use crate::navigation::{
    destination::{BlockDestination, Destination, NearDestination},
    ride::{
        moves::{Moves, RideState, Spot},
        route_search::{Route, RouteSearch},
        vehicle::{EntityKind, Vehicle, VehicleEntity, VehicleKind},
    },
    search::{Failure, Limits, Outcome},
};
impl RideState for S {
    fn friction(&self) -> f32 {
        self.registry.borrow().ride_friction[&self.def().block]
    }
}
struct Actor {
    kind: EntityKind,
    data: Vec<f64>,
    jump: bool,
    prefix: &'static str,
    events: Events,
}
impl Actor {
    fn log(&self, m: &str) {
        self.events
            .borrow_mut()
            .push(format!("{}:{m}", self.prefix))
    }
}
impl VehicleEntity for Actor {
    fn entity_kind(&self) -> EntityKind {
        self.kind
    }
    fn passenger_riding_position(&self, _rider: &Self) -> Vec3 {
        self.log("getPassengerRidingPosition");
        Vec3::new(0.0, self.data[0], 0.0)
    }
    fn y(&self) -> f64 {
        self.log("getY");
        self.data[1]
    }
    fn bounding_height(&self) -> f32 {
        self.log("getBbHeight");
        self.data[2] as f32
    }
    fn eye_height(&self) -> f32 {
        self.log("getEyeHeight");
        self.data[3] as f32
    }
    fn bounding_width(&self) -> f32 {
        self.log("getBbWidth");
        self.data[4] as f32
    }
    fn max_up_step(&self) -> f32 {
        self.log("maxUpStep");
        self.data[5] as f32
    }
    fn can_jump(&self) -> bool {
        self.log("canJump");
        self.jump
    }
    fn jump_strength(&self) -> f64 {
        self.log("getAttributeValue");
        self.data[6]
    }
    fn jump_boost_power(&self) -> f32 {
        self.log("getJumpBoostPower");
        self.data[7] as f32
    }
    fn safe_fall_distance(&self) -> f64 {
        self.log("getAttributeValue");
        self.data[8]
    }
    fn movement_speed(&self) -> f64 {
        self.log("getAttributeValue");
        self.data[9]
    }
}
enum Goal {
    Blocks(BlockDestination),
    Near(NearDestination),
    NoEnds(BlockPos),
}
impl Destination for Goal {
    fn is_goal(&self, p: BlockPos) -> bool {
        match self {
            Self::Blocks(g) => g.is_goal(p),
            Self::Near(g) => g.is_goal(p),
            Self::NoEnds(t) => p == *t,
        }
    }
    fn distance(&self, p: BlockPos) -> f64 {
        match self {
            Self::Blocks(g) => g.distance(p),
            Self::Near(g) => g.distance(p),
            Self::NoEnds(t) => (p.distance_sqr_long(*t) as f64).sqrt(),
        }
    }
    fn ends(&self) -> Vec<Vec3> {
        match self {
            Self::Blocks(g) => g.ends(),
            Self::Near(g) => g.ends(),
            Self::NoEnds(_) => Vec::new(),
        }
    }
}
fn goal(kind: &str, raw: &str, radius: &str) -> Goal {
    match kind {
        "B" => Goal::Blocks(BlockDestination::new(raw.split(';').map(p))),
        "N" => Goal::Near(NearDestination::new(v(raw), d(radius))),
        "G" => Goal::NoEnds(p(raw)),
        _ => panic!("goal kind"),
    }
}
#[derive(Default)]
struct RideFixture {
    vehicle: Option<Vehicle>,
    moves: Option<Moves<S, World>>,
    search: Option<RouteSearch<S, World, Goal>>,
}
fn vehicle_raw(v: Vehicle) -> String {
    format!(
        "{},{},{},{},{},{},{},{}",
        if v.kind == VehicleKind::Boat {
            "BOAT"
        } else {
            "MOUNT"
        },
        v.width.to_bits(),
        v.height.to_bits(),
        v.step_height.to_bits(),
        v.jump_height.to_bits(),
        v.safe_fall,
        v.speed.to_bits(),
        v.ice_speed.to_bits()
    )
}
fn outcome(o: Outcome<Route>) -> String {
    match o {
        Outcome::Searching => "S".into(),
        Outcome::AlreadyThere => "A".into(),
        Outcome::Failed(f) => format!(
            "E:{}",
            match f {
                Failure::InvalidDestination => "INVALID_DESTINATION",
                Failure::TooFar => "TOO_FAR",
                Failure::NoPath => "NO_PATH",
                Failure::TooComplex => "TOO_COMPLEX",
                Failure::TimedOut => "TIMED_OUT",
            }
        ),
        Outcome::Found(r) => format!(
            "F${}${}${}${}${}${}",
            vector(r.start),
            r.points
                .iter()
                .map(|p| format!("{}:{}:{}", position(p.node), vector(p.position), p.jump))
                .collect::<Vec<_>>()
                .join("@"),
            r.can_reach,
            r.exhausted,
            r.positions
                .iter()
                .copied()
                .map(vector)
                .collect::<Vec<_>>()
                .join(";"),
            vector(r.end())
        ),
    }
}
fn extra(a: &[&str], t: &mut Terrain<S, World>, r: &mut RideFixture, expected: &str) -> String {
    match a[0] {
        "capture" => {
            let data = values(a[2]);
            let jump = a[3].parse().unwrap();
            let rider = Actor {
                kind: EntityKind::Other,
                data: data.clone(),
                jump,
                prefix: "R",
                events: t.level().events.clone(),
            };
            let actor = Actor {
                kind: match a[1] {
                    "B" => EntityKind::Boat,
                    "H" => EntityKind::Horse,
                    "C" => EntityKind::Camel,
                    _ => EntityKind::Other,
                },
                data,
                jump,
                prefix: "V",
                events: t.level().events.clone(),
            };
            Vehicle::of(&rider, &actor).map_or_else(|| "-".into(), vehicle_raw)
        }
        "vehicle" => {
            let q = values(a[2]);
            let vehicle = Vehicle {
                kind: if a[1] == "B" {
                    VehicleKind::Boat
                } else {
                    VehicleKind::Mount
                },
                width: q[0],
                height: q[1],
                step_height: q[2],
                jump_height: q[3],
                safe_fall: a[3].parse().unwrap(),
                speed: q[4],
                ice_speed: q[5],
            };
            r.moves = Some(Moves::new(
                Terrain::new(t.level().clone(), vehicle.width, vehicle.height, false),
                vehicle,
            ));
            r.vehicle = Some(vehicle);
            "-".into()
        }
        "rideSpot" => r.moves.as_mut().unwrap().spot(p(a[1])).map_or_else(
            || "-".into(),
            |s| format!("{}:{}", vector(s.position), s.speed.to_bits()),
        ),
        "rideMoves" => r
            .moves
            .as_mut()
            .unwrap()
            .neighbors(p(a[1]))
            .iter()
            .map(|e| format!("{}:{}", position(e.node), e.cost.to_bits()))
            .collect::<Vec<_>>()
            .join(";"),
        "straight" => r
            .moves
            .as_mut()
            .unwrap()
            .is_straight(v(a[1]), v(a[2]))
            .to_string(),
        "passes" => r
            .moves
            .as_mut()
            .unwrap()
            .passes(v(a[1]), v(a[2]))
            .to_string(),
        "cost" => {
            let f = Spot {
                position: v(a[1]),
                speed: d(a[2]),
            };
            let to = Spot {
                position: v(a[3]),
                speed: d(a[4]),
            };
            let m = r.moves.as_ref().unwrap();
            format!("{}:{}", m.is_jump(f, to), m.cost(f, to).to_bits())
        }
        "path" => {
            let vehicle = r.vehicle.unwrap();
            let terrain = Terrain::new(t.level().clone(), vehicle.width, vehicle.height, false);
            let mut limits = Limits::new(d(a[5]));
            limits.max_visited_nodes = a[6].parse().unwrap();
            limits.visited_nodes_per_slice = a[7].parse().unwrap();
            limits.max_compute_nanos = a[8].parse().unwrap();
            r.search = Some(RouteSearch::new(
                terrain,
                v(a[1]),
                goal(a[2], a[3], a[4]),
                vehicle,
                limits,
            ));
            "-".into()
        }
        "advance" => {
            let s = r.search.as_mut().unwrap();
            let nanos: i64 = expected.split('~').nth(1).unwrap().parse().unwrap();
            let elapsed = nanos.wrapping_sub(s.engine.compute_nanos());
            let mut second = false;
            let returned = s
                .advance_with_clock(|| {
                    if second {
                        elapsed
                    } else {
                        second = true;
                        0
                    }
                })
                .map_or_else(|_| "invalid".into(), outcome);
            let iterations = s
                .engine
                .search
                .as_ref()
                .map_or_else(|| "-".into(), |s| s.iterations.to_string());
            let finished = s
                .engine
                .search
                .as_ref()
                .map_or_else(|| "-".into(), |s| s.is_finished().to_string());
            let preview = s.preview().map_or_else(
                || "-".into(),
                |points| points.into_iter().map(vector).collect::<Vec<_>>().join(";"),
            );
            format!(
                "{returned}~{}~{iterations}~{finished}~{preview}",
                s.engine.compute_nanos()
            )
        }
        "preview" => r.search.as_mut().unwrap().preview().map_or_else(
            || "-".into(),
            |points| points.into_iter().map(vector).collect::<Vec<_>>().join(";"),
        ),
        _ => panic!("{}", a[0]),
    }
}
