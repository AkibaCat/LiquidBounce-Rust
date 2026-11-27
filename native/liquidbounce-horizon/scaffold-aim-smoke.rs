// SPDX-License-Identifier: GPL-3.0-or-later
// Prepared for the coordinator's centralized Rust batch; not run in isolation.
#[path = "../liquidbounce-behaviors/src/scaffold_aim.rs"]
mod aim;
use aim::*;

struct Arithmetic {
    direction: Point,
    random: Vec<(f64, f64)>,
}
impl Math for Arithmetic {
    fn rotation_direction(&mut self, _: [f32; 2]) -> Point {
        self.direction
    }
    fn yaw_axis(&mut self, yaw: f32) -> Point {
        let r = f64::from(yaw.to_radians());
        [r.sin(), 0.0, r.cos()]
    }
    fn point_yaw(&mut self, p: Point) -> f32 {
        (-p[0]).atan2(p[2]).to_degrees() as f32
    }
    fn random_between(&mut self, from: f64, to: f64) -> f64 {
        self.random.push((from, to));
        from + (to - from) * 0.25
    }
}
fn math() -> Arithmetic {
    Arithmetic {
        direction: [0.0, -1.0, 0.0],
        random: Vec::new(),
    }
}
fn close(a: Point, b: Point) {
    for i in 0..3 {
        assert!((a[i] - b[i]).abs() < 1e-8, "{a:?} != {b:?}");
    }
}
fn main() {
    let top = Face {
        from: [0.0, 1.0, 0.0],
        to: [1.0, 1.0, 1.0],
    };
    let mut loc = Location {
        position: [0.5, 0.0, 0.5],
        eye: [0.5, 2.0, 0.5],
        rotation: [0.0, 90.0],
        moving: false,
    };
    close(
        sample(Mode::Center, top, [0.0; 3], loc, None, &mut math()),
        [0.5, 1.0, 0.5],
    );
    let mut random = math();
    close(
        sample(Mode::Random, top, [0.0; 3], loc, None, &mut random),
        [0.325, 1.0, 0.325],
    );
    assert_eq!(random.random, vec![(0.15, 0.85), (0.15, 0.85)]);
    for mode in [
        Mode::Stabilized,
        Mode::NearestRotation,
        Mode::ReverseYaw,
        Mode::DiagonalYaw,
        Mode::AngleYaw,
        Mode::EdgePoint,
    ] {
        close(
            sample(mode, top, [0.0; 3], loc, None, &mut math()),
            [0.5, 1.0, 0.5],
        );
    }
    // Lines missing the trimmed face clamp to its closest edge, preserving Y.
    loc.eye = [-1.0, 2.0, 0.5];
    close(
        sample(Mode::NearestRotation, top, [0.0; 3], loc, None, &mut math()),
        [0.15, 1.0, 0.5],
    );
    // Equal-distance vertices keep the first x/y/z iteration candidate.
    loc.position = [0.5, 0.0, 0.5];
    loc.moving = true;
    close(
        sample(Mode::EdgePoint, top, [0.0; 3], loc, None, &mut math()),
        [0.15, 1.0, 0.15],
    );
    loc.position = [0.0, 0.0, 0.0];
    close(
        sample(Mode::EdgePoint, top, [0.0; 3], loc, None, &mut math()),
        [0.85, 1.0, 0.85],
    );
    // Yaw factories operate in block-local space even at distant coordinates.
    let target = [320.0, 80.0, -570.0];
    let local = Location {
        position: [0.2, 1.2, -0.7],
        eye: [0.2, 2.82, -0.7],
        rotation: [35.0, 70.0],
        moving: true,
    };
    let global = Location {
        position: add(local.position, target),
        eye: add(local.eye, target),
        ..local
    };
    let optimal = Linear {
        anchor: [0.0, 0.0, 0.0],
        direction: [1.0, 0.0, 0.0],
        segment: false,
    };
    for mode in [
        Mode::Center,
        Mode::Random,
        Mode::Stabilized,
        Mode::NearestRotation,
        Mode::ReverseYaw,
        Mode::DiagonalYaw,
        Mode::AngleYaw,
        Mode::EdgePoint,
    ] {
        let a = sample(mode, top, [0.0; 3], local, Some(optimal), &mut math());
        let b = sample(
            mode,
            top,
            target,
            global,
            Some(Linear {
                anchor: add(optimal.anchor, target),
                ..optimal
            }),
            &mut math(),
        );
        close(a, b);
    }
    let collapsed = Face {
        from: [0.2, 0.3, 0.4],
        to: [0.2, 0.3, 0.4],
    };
    for mode in [
        Mode::NearestRotation,
        Mode::ReverseYaw,
        Mode::DiagonalYaw,
        Mode::AngleYaw,
    ] {
        close(
            sample(mode, collapsed, [0.0; 3], loc, None, &mut math()),
            collapsed.from,
        );
    }
    // Stabilization crops to a lateral corridor before the nearest rotation.
    let side = Face {
        from: [1.0, 0.0, 0.0],
        to: [1.0, 1.0, 1.0],
    };
    let location = Location {
        position: [0.0, 0.0, 0.5],
        eye: [0.0, 0.5, 0.5],
        rotation: [0.0; 2],
        moving: true,
    };
    let mut m = math();
    m.direction = [1.0, 0.0, 0.0];
    close(
        sample(
            Mode::Stabilized,
            side,
            [0.0; 3],
            location,
            Some(optimal),
            &mut m,
        ),
        [1.0, 0.5, 0.5],
    );
    let segment = Linear {
        anchor: [0.0; 3],
        direction: [1.0, 0.0, 0.0],
        segment: true,
    };
    close(segment.nearest([2.0, 4.0, 0.0]), [1.0, 0.0, 0.0]);
}
