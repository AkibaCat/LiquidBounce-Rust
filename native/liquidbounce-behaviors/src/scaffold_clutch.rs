// SPDX-License-Identifier: GPL-3.0-or-later
//! ScaffoldClutchTechnique's fall-layer search and Normal aim-ahead decision.
pub type Position = [f64; 3];
pub type BlockPos = [i32; 3];

pub struct Search<'a> {
    pub below: BlockPos,
    pub offsets: &'a [BlockPos],
    pub from: Position,
    pub priority: Position,
}
pub trait Host {
    type Target;
    /// Real BlockState.isFaceSturdy(UP, CENTER), including unloaded rejection.
    fn can_stand_on(&mut self, position: BlockPos) -> bool;
    /// CenterTargetPositionFactory and the shared vanilla placement context.
    fn find_target(&mut self, search: Search<'_>) -> Option<Self::Target>;
    /// The server rotation can click it from the actual current player eye.
    fn can_click(&mut self, target: &Self::Target) -> bool;
}

pub fn target<H: Host>(
    host: &mut H,
    from: Position,
    fall: &[Position],
    eye_height: f32,
    range: f64,
    full_offsets: &[BlockPos],
) -> Option<H::Target> {
    let mut feet_y = from[1];
    for position in fall {
        let top = feet_y.floor();
        feet_y = position[1];
        if position[1] >= top || top > from[1] {
            continue;
        }
        if from[1] + f64::from(eye_height) - top > range {
            break;
        }
        let below = [
            from[0].floor() as i32,
            (top - 1.0).floor() as i32,
            from[2].floor() as i32,
        ];
        // Ground under this row cancels this search instead of skipping a layer.
        if host.can_stand_on(below) {
            return None;
        }
        let offsets = full_offsets
            .iter()
            .copied()
            .filter(|offset| {
                let under = [
                    below[0].wrapping_add(offset[0]),
                    below[1].wrapping_add(offset[1]).wrapping_sub(1),
                    below[2].wrapping_add(offset[2]),
                ];
                !host.can_stand_on(under)
            })
            .collect::<Vec<_>>();
        if let Some(target) = host.find_target(Search {
            below,
            offsets: &offsets,
            from,
            priority: [from[0], top - 0.5, from[2]],
        }) {
            return Some(target);
        }
    }
    None
}

pub fn find_placement_target<H: Host>(
    host: &mut H,
    current: Position,
    fall: &[Position],
    eye_height: f32,
    range: f64,
    full_offsets: &[BlockPos],
    aims_ahead: bool,
) -> Option<H::Target> {
    let now = target(host, current, fall, eye_height, range, full_offsets);
    if !aims_ahead {
        return now;
    }
    if let Some(now) = now
        && host.can_click(&now)
    {
        return Some(now);
    }
    let (&next, rest) = fall.split_first()?;
    target(host, next, rest, eye_height, range, full_offsets)
}
