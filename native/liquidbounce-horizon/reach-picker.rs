// ProjectileUtil.getEntityHitResult(Entity, Vec3, Vec3, AABB, Predicate, double).
// Kept in the host so every pickable entity category shares one target.
fn client_rotation_direction(yaw: f32, pitch: f32) -> [f64; 3] {
    use horizon_core::geometry::trig;
    let y = -yaw * trig::DEG_TO_RAD - std::f32::consts::PI;
    let p = -pitch * trig::DEG_TO_RAD;
    let cy = trig::cos(f64::from(y));
    let sy = trig::sin(f64::from(y));
    let cp = -trig::cos(f64::from(p));
    let sp = trig::sin(f64::from(p));
    [f64::from(sy * cp), f64::from(sp), f64::from(cy * cp)]
}
#[derive(Clone, Copy, Debug)]
struct ClientReachCandidate {
    kind: u8,
    index: usize,
    id: i32,
    local_id: u32,
    position: [f64; 3],
    bounds: Aabb,
    pick_radius: f64,
    same_root_vehicle: bool,
    can_pick_inside: bool,
}
#[derive(Clone, Copy, Debug)]
struct ClientReachPick {
    kind: u8,
    index: usize,
    id: i32,
    local_id: u32,
    distance: f64,
    distance_squared: f64,
    point: [f64; 3],
    location: [f64; 3],
}
fn client_reach_pick(
    candidates: impl IntoIterator<Item = ClientReachCandidate>,
    eye: Vec3,
    direction: Vec3,
    player_bounds: Aabb,
    range: f64,
) -> Option<ClientReachPick> {
    let ray = direction.scale(range);
    let end = eye.add_vector(ray);
    let scan = player_bounds.expand_towards(ray).inflate(Vec3::new(1.0, 1.0, 1.0));
    let mut remaining = range * range;
    let mut best = None;
    for candidate in candidates {
        if !candidate.bounds.intersects(scan) { continue; }
        let radius = candidate.pick_radius;
        let bounds = candidate.bounds.inflate(Vec3::new(radius, radius, radius));
        let clipped = bounds.clip(eye, end);
        let at = if bounds.contains(eye) {
            if remaining < 0.0 || !candidate.can_pick_inside { continue; }
            remaining = 0.0;
            clipped.unwrap_or(eye)
        } else if let Some(at) = clipped {
            let d2 = eye.distance_squared(at);
            if d2 >= remaining && remaining != 0.0 { continue; }
            if candidate.same_root_vehicle {
                if remaining != 0.0 { continue; }
            } else {
                remaining = d2;
            }
            at
        } else { continue; };
        let point = [at.x, at.y, at.z];
        best = Some(ClientReachPick {
            kind: candidate.kind, index: candidate.index, id: candidate.id,
            local_id: candidate.local_id,
            distance: eye.distance_squared(at).sqrt(), distance_squared: eye.distance_squared(at), point,
            location: std::array::from_fn(|i| point[i] - candidate.position[i]),
        });
    }
    best
}
