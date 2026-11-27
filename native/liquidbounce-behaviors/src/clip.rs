// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleClip at ae0eb396: wall-first search, input priority and coroutine wait.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    pub mode: u8,
    pub horizontal: i32,
    pub vertical: i32,
    pub liquids: u32,
    pub requires_stand_on: bool,
    pub old_horizontal: f32,
    pub old_vertical: f32,
    pub reset_velocity: bool,
    pub possible: [bool; 2],
    ticket: u64,
    wait: Option<(u64, i32)>,
}
impl Default for Clip {
    fn default() -> Self {
        Self {
            mode: 0,
            horizontal: 0,
            vertical: 5,
            liquids: 0,
            requires_stand_on: true,
            old_horizontal: 0.0,
            old_vertical: 5.0,
            reset_velocity: true,
            possible: [false; 2],
            ticket: 0,
            wait: None,
        }
    }
}
pub const DIRECTIONS: [[i32; 3]; 6] = [
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [-1, 0, 0],
    [0, 0, -1],
    [1, 0, 0],
];
pub fn find_target(
    start: [i32; 3],
    direction: usize,
    length: i32,
    mut valid: impl FnMut([i32; 3], bool) -> bool,
) -> Option<[i32; 3]> {
    let offset = DIRECTIONS[direction];
    let mut pos = start;
    let mut wall = false;
    for _ in 0..length.max(0) {
        for i in 0..3 {
            pos[i] = pos[i].wrapping_add(offset[i]);
        }
        if valid(pos, direction != 0) {
            if wall {
                return Some(pos);
            }
        } else {
            wall = true;
        }
    }
    None
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub horizontal_collision: bool,
    pub keys: [bool; 6], // forward, backward, left, right, sneak, jump
    pub yaw: f32,
}
pub fn movement_direction(c: Input) -> Option<usize> {
    let facing = ((c.yaw / 90.0 + 0.5).floor() as i32).rem_euclid(4) as usize;
    if c.horizontal_collision {
        let offset = c.keys[..4]
            .iter()
            .position(|v| *v)
            .map(|i| [0, 2, 3, 1][i])?;
        Some(2 + (facing + offset) % 4)
    } else if c.keys[4] {
        Some(1)
    } else if c.keys[5] {
        Some(0)
    } else {
        None
    }
}
impl Clip {
    pub fn cancel(&mut self) {
        self.wait = None;
        self.possible = [false; 2];
    }
    pub fn busy(&self) -> bool {
        self.wait.is_some()
    }
    pub fn register_wait(&mut self) -> u64 {
        self.ticket = self.ticket.wrapping_add(1);
        self.wait = Some((self.ticket, 5));
        self.ticket
    }
    /// false finishes this callback; ready allows its body to resume this tick.
    pub fn advance(&mut self, id: u64, running: bool) -> (bool, bool) {
        let Some((ticket, remaining)) = self.wait.as_mut() else {
            return (false, false);
        };
        if *ticket != id {
            return (false, false);
        }
        if !running || self.mode != 0 {
            self.cancel();
            return (false, false);
        }
        *remaining -= 1;
        if *remaining > 0 {
            return (true, false);
        }
        self.wait = None;
        (false, true)
    }
    pub fn old_offset(&self, yaw: f32) -> [f64; 3] {
        // Kotlin Float.toRadians performs Float arithmetic before widening.
        let yaw = (yaw * (std::f32::consts::PI / 180.0)) as f64;
        [
            -yaw.sin() * f64::from(self.old_horizontal),
            f64::from(self.old_vertical),
            yaw.cos() * f64::from(self.old_horizontal),
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wall_input_and_five_tick_smoke() {
        assert_eq!(
            find_target([0, 0, 0], 0, 5, |p, stand| {
                assert!(!stand);
                p[1] != 2
            }),
            Some([0, 3, 0])
        );
        assert_eq!(find_target([0, 0, 0], 1, 5, |_, _| true), None);
        assert_eq!(
            movement_direction(Input {
                horizontal_collision: true,
                keys: [false, false, true, false, true, true],
                yaw: 0.
            }),
            Some(5)
        );
        assert_eq!(
            movement_direction(Input {
                keys: [false, false, false, false, true, true],
                ..Input::default()
            }),
            Some(1)
        );
        let mut s = Clip::default();
        let id = s.register_wait();
        for _ in 0..4 {
            assert_eq!(s.advance(id, true), (true, false));
        }
        assert_eq!(s.advance(id, true), (false, true));
        assert!(!s.busy());
        let id = s.register_wait();
        assert_eq!(s.advance(id, false), (false, false));
        assert!(!s.busy());
        assert_eq!(s.old_offset(0.), [0., 5., 0.]);
    }
}
