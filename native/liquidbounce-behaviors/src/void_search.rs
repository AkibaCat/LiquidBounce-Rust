// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleVoidESP search(), LiquidBounce ae0eb396. Shared verbatim with the native Game patch.
use std::collections::BTreeSet;
pub fn search(
    player: [i32; 3],
    facing: [i32; 2],
    side_range: i32,
    facing_range: i32,
    threshold: i32,
    min_y: i32,
    mut sturdy: impl FnMut([i32; 3]) -> bool,
) -> BTreeSet<[i32; 3]> {
    let mut start = [player[0], player[1].wrapping_sub(1), player[2]];
    let mut found = false;
    for _ in 0..threshold {
        if sturdy(start) {
            found = true;
            break;
        }
        start[1] = start[1].wrapping_sub(1);
    }
    if !found {
        return BTreeSet::new();
    }
    let side = [-facing[1], facing[0]];
    let a = [
        start[0]
            .wrapping_add(facing[0].wrapping_mul(facing_range))
            .wrapping_sub(side[0].wrapping_mul(side_range)),
        start[2]
            .wrapping_add(facing[1].wrapping_mul(facing_range))
            .wrapping_sub(side[1].wrapping_mul(side_range)),
    ];
    let b = [
        start[0]
            .wrapping_sub(facing[0].wrapping_mul(facing_range))
            .wrapping_add(side[0].wrapping_mul(side_range)),
        start[2]
            .wrapping_sub(facing[1].wrapping_mul(facing_range))
            .wrapping_add(side[1].wrapping_mul(side_range)),
    ];
    let mut out = BTreeSet::new();
    for z in a[1].min(b[1])..=a[1].max(b[1]) {
        for x in a[0].min(b[0])..=a[0].max(b[0]) {
            let p = [x, start[1], z];
            if sturdy(p) {
                continue;
            }
            let mut scan = p;
            let mut covered = false;
            for _ in 0..threshold {
                scan[1] = scan[1].wrapping_sub(1);
                if sturdy(scan) {
                    covered = true;
                    break;
                }
                if scan[1] <= min_y {
                    break;
                }
            }
            if !covered {
                out.insert(p);
            }
        }
    }
    out
}
