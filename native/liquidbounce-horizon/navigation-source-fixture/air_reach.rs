use crate::navigation::{
    destination::{BlockDestination, Destination, NearDestination},
    flight::{Acceleration, FlyAbilities, FlyMoves, FlyRoute, FlyRouteSearch, is_clear_line},
    reach::{ClipBlock, RayHit, ReachLevel, ReachOptions, reach_positions, sees},
    search::{Limits, Outcome, SearchProblem},
};
thread_local! {static RAY:RefCell<(i32,BlockPos,i32,i32)>=const{RefCell::new((0,BlockPos::new(0,0,0),0,0))};}
impl ReachLevel<S> for World {
    fn clip(&self, from: Vec3, to: Vec3, shapes: ClipBlock) -> RayHit {
        self.events.borrow_mut().push(format!(
            "R:{}:NONE:true:{}:{}",
            if shapes == ClipBlock::Outline {
                "OUTLINE"
            } else {
                "COLLIDER"
            },
            vector(from),
            vector(to)
        ));
        RAY.with_borrow_mut(|ray| {
            ray.2 += 1;
            let block = ray.1;
            match ray.0 {
                0 => RayHit::Miss,
                1 => RayHit::Block(block),
                2 => RayHit::Block(block.above()),
                3 => RayHit::Block(if to.x > 0.65 && to.y > 0.4 {
                    block
                } else {
                    block.above()
                }),
                4 => {
                    if ray.2 > ray.3 {
                        RayHit::Miss
                    } else {
                        RayHit::Block(block.above())
                    }
                }
                _ => panic!("ray mode"),
            }
        })
    }
}
#[derive(Clone)]
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
fn outcome(o: Outcome<FlyRoute>) -> String {
    match o {
        Outcome::Searching => "S".into(),
        Outcome::AlreadyThere => "A".into(),
        Outcome::Failed(f) => format!(
            "E:{}",
            match f {
                crate::navigation::search::Failure::InvalidDestination => "INVALID_DESTINATION",
                crate::navigation::search::Failure::TooFar => "TOO_FAR",
                crate::navigation::search::Failure::NoPath => "NO_PATH",
                crate::navigation::search::Failure::TooComplex => "TOO_COMPLEX",
                crate::navigation::search::Failure::TimedOut => "TIMED_OUT",
            }
        ),
        Outcome::Found(r) => format!(
            "F:{}:{}:{}:{}",
            vector(r.start),
            r.points
                .iter()
                .copied()
                .map(vector)
                .collect::<Vec<_>>()
                .join(";"),
            r.can_reach,
            vector(r.end())
        ),
    }
}
type AirSearch = FlyRouteSearch<S, World, Goal>;
type AirMoves = FlyMoves<S, World, Goal>;
fn extra(
    a: &[&str],
    t: &mut Terrain<S, World>,
    search: &mut Option<AirSearch>,
    moves: &mut Option<AirMoves>,
    expected: &str,
) -> String {
    match a[0] {
        "ray" => {
            RAY.with_borrow_mut(|r| {
                *r = (a[1].parse().unwrap(), p(a[2]), 0, a[3].parse().unwrap())
            });
            "-".into()
        }
        "sees" => sees(t, v(a[3]), b(a[1]), d(a[2]), (a[4] != "-").then(|| p(a[4]))).to_string(),
        "reach" => {
            let mut options = ReachOptions::new(d(a[2]), v(a[3]));
            options.block = (a[4] != "-").then(|| p(a[4]));
            if a[5] != "-" {
                options.eye_height = d(a[5]);
            }
            options.requires_support = a[6].parse().unwrap();
            reach_positions(t, b(a[1]), options)
                .into_iter()
                .map(position)
                .collect::<Vec<_>>()
                .join(";")
        }
        "creative" => FlyAbilities::of(
            a[2].parse().unwrap(),
            a[3].parse().unwrap(),
            f32::from_bits(a[1].parse().unwrap()),
        )
        .map_or_else(
            || "-".into(),
            |a| {
                format!(
                    "MOMENTUM,{},{},{},{},{}",
                    a.horizontal_speed().to_bits(),
                    a.vertical_speed().to_bits(),
                    a.is_flying(),
                    a.horizontal_cost().to_bits(),
                    a.vertical_cost().to_bits()
                )
            },
        ),
        "fly" => {
            let ability = FlyAbilities::new(
                if a[1] == "M" {
                    Acceleration::Momentum
                } else {
                    Acceleration::Direct
                },
                d(a[2]),
                d(a[3]),
                a[4].parse().unwrap(),
            );
            let dest = goal(a[6], a[7], a[8]);
            let fresh =
                || Terrain::new(t.level().clone(), t.width(), t.height(), t.can_open_doors());
            *moves = Some(FlyMoves::new(fresh(), ability, dest.clone()));
            let mut limits = Limits::new(d(a[9]));
            limits.max_visited_nodes = a[10].parse().unwrap();
            limits.visited_nodes_per_slice = a[11].parse().unwrap();
            limits.max_compute_nanos = a[12].parse().unwrap();
            *search = Some(FlyRouteSearch::new(fresh(), v(a[5]), dest, ability, limits));
            "-".into()
        }
        "moves" => {
            let m = moves.as_mut().unwrap();
            let q = p(a[1]);
            format!(
                "{}|{}|{}",
                m.is_open(q),
                vector(m.point(q)),
                m.neighbors(q)
                    .into_iter()
                    .map(|e| format!("{}:{}", position(e.node), e.cost.to_bits()))
                    .collect::<Vec<_>>()
                    .join(";")
            )
        }
        "line" => is_clear_line(t, v(a[1]), v(a[2])).to_string(),
        "h" => search
            .as_mut()
            .unwrap()
            .problem
            .heuristic(&p(a[1]))
            .to_bits()
            .to_string(),
        "advance" => {
            let s = search.as_mut().unwrap();
            let nanos = expected.split('~').nth(1).unwrap().parse::<i64>().unwrap();
            let elapsed = nanos.wrapping_sub(s.engine.compute_nanos());
            let mut times = [0, elapsed].into_iter();
            let returned = s
                .advance_with_clock(|| times.next().unwrap())
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
            format!(
                "{returned}~{}~{iterations}~{finished}~{}",
                s.engine.compute_nanos(),
                s.preview().map_or_else(
                    || "-".into(),
                    |p| p.into_iter().map(vector).collect::<Vec<_>>().join(";")
                )
            )
        }
        "preview" => search.as_mut().unwrap().preview().map_or_else(
            || "-".into(),
            |p| p.into_iter().map(vector).collect::<Vec<_>>().join(";"),
        ),
        _ => panic!("{}", a.join(" ")),
    }
}
