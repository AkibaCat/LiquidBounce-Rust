// SPDX-License-Identifier: GPL-3.0-or-later
// Prepared for the coordinator's centralized Rust batch; not run in isolation.
#[path = "../liquidbounce-behaviors/src/scaffold_lifecycle.rs"]
mod lifecycle;
#[path = "../liquidbounce-behaviors/src/scaffold_techniques.rs"]
mod techniques;
use techniques::*;
fn main() {
    assert_eq!(normal_radius(true, true), 4);
    assert_eq!(normal_radius(true, false), 4);
    assert_eq!(normal_radius(false, true), 2);
    assert_eq!(normal_radius(false, false), 1);
    assert_eq!(reverse_rotation(22.5, 0.0), [0.0, 45.0]);
    assert_eq!(reverse_rotation(67.5, 80.0), [90.0, 80.0]);
    assert_eq!(reverse_rotation(-22.5, 50.0), [-0.0, 50.0]);
    assert_eq!(reverse_rotation(-67.5, 50.0), [-90.0, 50.0]);
    assert_eq!(
        expand_position([0.7, 64.8, -0.1], [-0.70710677, 0.70710677], 1),
        [0, 64, -1]
    );
    assert_eq!(
        expand_position([0.7, 64.8, -0.1], [-0.70710677, 0.70710677], 2),
        [-1, 64, 0]
    );
    let mut visited = Vec::new();
    let target = find_expand([0.7, 64.8, -0.1], [-1.0, 0.0], 4, |p| {
        visited.push(p);
        (p[0] == -2).then_some(p)
    });
    assert_eq!(target, Some([-2, 64, -1]));
    assert_eq!(visited, vec![[0, 64, -1], [-1, 64, -1], [-2, 64, -1]]);
    let mut count = 0;
    assert_eq!(
        find_expand([0.0; 3], [0.0, 1.0], 4, |_| {
            count += 1;
            None::<()>
        }),
        None
    );
    assert_eq!(count, 5); // inclusive 0..Length
    // A successful first expansion mutates Hypixel jumps only once.
    let mut height = lifecycle::State {
        start_y: 64,
        jumps: 2,
        ..Default::default()
    };
    let target = find_expand([0.0, 65.0, 0.0], [0.0, 1.0], 4, |block| {
        Some(height.targeted_position(
            lifecycle::PositionFacts {
                block,
                velocity_y: -0.15233518685055708,
                ..Default::default()
            },
            lifecycle::SameY::Hypixel,
        ))
    });
    assert_eq!(target, Some([0, 64, 0]));
    assert_eq!(height.jumps, 0);
}
