// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleScaffold lifecycle, target height and placement tick decisions.
pub type BlockPos = [i32; 3];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Timing {
    #[default]
    Normal,
    OnTick,
    OnTickSnap,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SameY {
    #[default]
    Off,
    On,
    JumpKey,
    Falling,
    Hypixel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    Main,
    Off,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotAction {
    Select(usize),
    Reset,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub action: SlotAction,
    pub has_main: bool,
}

/// Source deliberately ignores selectSlotSilently's cancellation return when
/// deciding hasBlockInMainHand. The real held stack is still queried afterward.
pub fn select_block(
    auto_block: bool,
    main: bool,
    offhand: bool,
    best_slot: Option<usize>,
) -> Selection {
    if auto_block
        && !main
        && !offhand
        && let Some(slot) = best_slot
    {
        return Selection {
            action: SlotAction::Select(slot),
            has_main: true,
        };
    }
    Selection {
        action: SlotAction::Reset,
        has_main: main,
    }
}
pub fn hand(main: bool, offhand: bool) -> Option<Hand> {
    if main {
        Some(Hand::Main)
    } else if offhand {
        Some(Hand::Off)
    } else {
        None
    }
}
pub fn valid_crosshair(point: [f64; 3], eye: [f64; 3], face: [i32; 3], min_dist: f32) -> bool {
    if face[1] != 0 {
        return true;
    }
    let axis = if face[2] != 0 { 2 } else { 0 };
    (point[axis] - eye[axis]).abs() >= f64::from(min_dist)
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PositionFacts {
    pub block: BlockPos,
    pub velocity_y: f64,
    pub input_jump: bool,
    pub physical_jump: bool,
    pub moving: bool,
    pub horizontal_collision: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub placement_y: i32,
    pub start_y: i32,
    pub jumps: i32,
    pub was_towering: bool,
    pub force_sneak: i32,
}
impl State {
    pub fn enabled(&mut self, block_y: i32) {
        self.placement_y = block_y.wrapping_sub(1);
        self.start_y = block_y;
        self.jumps = 2;
    }
    /// Other reset effects (planner/history/slot/renderer/target/NoFallBlink)
    /// are performed by the runtime. The source retains its height/tower data.
    pub fn reset(&mut self) {
        self.force_sneak = 0;
    }
    pub fn tick_begin(&mut self, block_y: i32, on_ground: bool, physical_jump: bool) {
        if on_ground {
            self.placement_y = block_y.wrapping_sub(1);
            self.jumps = self.jumps.wrapping_add(1);
            self.was_towering = false;
        }
        if physical_jump {
            self.start_y = block_y;
            self.jumps = 2;
        }
    }
    pub fn is_towering(&mut self, tower_selected: bool, physical_jump: bool) -> bool {
        let active = tower_selected && physical_jump;
        if active {
            self.was_towering = true;
        }
        active
    }
    pub fn take_forced_sneak(&mut self) -> bool {
        if self.force_sneak <= 0 {
            return false;
        }
        self.force_sneak -= 1;
        true
    }
    pub fn ledge_sneak(&mut self, ticks: i32) -> bool {
        if ticks > self.force_sneak {
            self.force_sneak = ticks;
            true
        } else {
            false
        }
    }
    /// Called only after the tower/down/ceiling precedence is resolved.
    pub fn targeted_position(&mut self, facts: PositionFacts, same_y: SameY) -> BlockPos {
        let mut pos = facts.block;
        if facts.input_jump && (!facts.moving || facts.horizontal_collision) {
            pos[1] = pos[1].wrapping_sub(1);
            return pos;
        }
        let same_y = match same_y {
            SameY::Off => None,
            SameY::On => Some(self.placement_y),
            SameY::JumpKey => (!facts.physical_jump).then_some(self.placement_y),
            SameY::Falling => (facts.velocity_y < 0.2).then_some(self.placement_y),
            SameY::Hypixel => Some(
                if facts.velocity_y == -0.15233518685055708 && self.jumps >= 2 {
                    self.jumps = 0;
                    self.start_y
                } else {
                    self.start_y.wrapping_sub(1)
                },
            ),
        };
        pos[1] = same_y.unwrap_or_else(|| pos[1].wrapping_sub(1));
        pos
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attempt {
    pub enabled: bool,
    pub failed_only: bool,
    pub suitable_hand: bool,
    pub block_hit: bool,
    pub can_place_on_face: bool,
    pub same_y: SameY,
    pub physical_jump: bool,
    pub clicked_y: i32,
    pub clicked_up: bool,
    pub placement_y: i32,
    pub player_block_y: i32,
}
pub fn simulate_attempt(a: Attempt) -> bool {
    if !a.suitable_hand || !a.enabled || !a.block_hit {
        return false;
    }
    if a.failed_only {
        return !a.can_place_on_face;
    }
    if a.same_y != SameY::Off && (a.same_y != SameY::JumpKey || a.physical_jump) {
        return a.clicked_y == a.placement_y && (!a.clicked_up || !a.can_place_on_face);
    }
    let below = a.player_block_y.wrapping_sub(1);
    let under = a.clicked_y <= below;
    let towering = a.clicked_y == below && a.can_place_on_face && a.clicked_up;
    under && !towering
}
