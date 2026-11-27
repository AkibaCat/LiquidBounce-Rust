// SPDX-License-Identifier: GPL-3.0-or-later
//! Frozen Scaffold child handlers. World and random observations are supplied by
//! the native caller, retaining the original event order and Float boundaries.
pub type Input = [bool; 4];
pub fn moving(input: Input) -> bool {
    input[0] != input[1] || input[2] != input[3]
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LedgeAction {
    pub jump: bool,
    pub sneak: i32,
    pub stop: bool,
    pub backwards: bool,
}
pub fn ledge(close: bool, blocks: i32, ticks: i32) -> Option<LedgeAction> {
    (close && (blocks <= 0 || ticks >= 1)).then_some(LedgeAction {
        sneak: ticks.max(1),
        ..Default::default()
    })
}
pub fn apply_ledge(action: LedgeAction, input: &mut Input, jump: &mut bool, force_sneak: &mut i32) {
    *jump |= action.jump;
    *force_sneak = (*force_sneak).max(action.sneak);
    if action.stop {
        *input = [false; 4];
    }
    if action.backwards {
        input[0] = false;
        input[1] = true;
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Telly {
    pub ticks_until_jump: i32,
    pub jump_ticks: i32,
}
impl Telly {
    pub fn tick(&mut self, ground: bool) {
        if ground {
            self.ticks_until_jump = self.ticks_until_jump.wrapping_add(1);
        }
    }
    pub fn after_jump(&mut self, next_jump_ticks: i32) {
        self.ticks_until_jump = 0;
        self.jump_ticks = next_jump_ticks;
    }
    pub fn do_not_aim(
        self,
        air_ticks: i32,
        straight: i32,
        towering: bool,
        aim_on_tower: bool,
    ) -> bool {
        air_ticks <= straight
            && self.ticks_until_jump >= self.jump_ticks
            && !(towering && aim_on_tower)
    }
    pub fn input(
        self,
        moving: bool,
        blocks: i32,
        ground: bool,
        reverse: bool,
        rotation_none: bool,
        straight: i32,
    ) -> bool {
        moving
            && blocks > 0
            && ground
            && (reverse
                || ((rotation_none || straight == 0) && self.ticks_until_jump >= self.jump_ticks))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Eagle {
    pub placed: i32,
    pub blocks: i32,
    pub edge: f32,
}
impl Default for Eagle {
    fn default() -> Self {
        Self {
            placed: 0,
            blocks: 0,
            edge: 0.01,
        }
    }
}
impl Eagle {
    pub fn eligible(
        self,
        down_falloff: bool,
        ground: bool,
        ground_only: bool,
        flying: bool,
    ) -> bool {
        !down_falloff && (ground || !ground_only) && !flying && self.placed == 0
    }
    /// Refresh both refreshable ranges only after exceeding the current count.
    pub fn placement(&mut self, mut refresh: impl FnMut() -> (i32, f32)) {
        self.placed = self.placed.wrapping_add(1);
        if self.placed > self.blocks {
            self.placed = 0;
            (self.blocks, self.edge) = refresh();
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SprintMode {
    Keep,
    Force,
    Never,
    NoPlace,
    NoGround,
}
pub fn sprint(mode: SprintMode, original: bool, moving: bool, ground: bool, placed: bool) -> bool {
    match mode {
        SprintMode::Keep => original,
        SprintMode::Force => moving || original,
        SprintMode::Never => false,
        SprintMode::NoPlace => original && !placed,
        SprintMode::NoGround => !ground,
    }
}
pub fn strafe_speed(hypixel: bool, speed: f32, has_speed: bool, tick: i32, move_ticks: i32) -> f64 {
    if !hypixel {
        f64::from(speed)
    } else if tick % 20 == 0 || move_ticks <= 7 {
        0.09800000190734863
    } else if has_speed {
        0.295
    } else {
        0.207
    }
}
pub fn stabilize(
    input: Input,
    toward_line: [f64; 3],
    horizontal_velocity: [f64; 3],
    correction: Input,
) -> Input {
    let towards =
        toward_line[0] * horizontal_velocity[0] + toward_line[2] * horizontal_velocity[2] > 0.0;
    let deviation = if towards { 0.075 } else { 0.2 };
    if toward_line.iter().map(|v| v * v).sum::<f64>() < deviation * deviation {
        return input;
    }
    let front = input[0] || input[1];
    let side = input[2] || input[3];
    [
        if front { input[0] } else { correction[0] },
        if front { input[1] } else { correction[1] },
        if side { input[2] } else { correction[2] },
        if side { input[3] } else { correction[3] },
    ]
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TowerMode {
    None,
    Motion,
    Pulldown,
    Karhu,
    Vulcan,
    Hypixel,
}
#[derive(Clone, Copy, Debug)]
pub struct TowerSettings {
    pub mode: TowerMode,
    pub motion: f32,
    pub height: f32,
    pub slow: f32,
    pub trigger: f32,
    pub timer: f32,
    pub pulldown: bool,
}
impl Default for TowerSettings {
    fn default() -> Self {
        Self {
            mode: TowerMode::None,
            motion: 0.42,
            height: 0.78,
            slow: 1.0,
            trigger: 0.1,
            timer: 5.0,
            pulldown: true,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct TowerFacts {
    pub y: f64,
    pub velocity_y: f64,
    pub physical_jump: bool,
    pub blocks: i32,
    pub block_below: bool,
    pub ground: bool,
    pub moving: bool,
    pub horizontal_speed: f64,
    pub tick: i32,
    pub protocol_1_8: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct TowerEffect {
    pub y: Option<f64>,
    pub velocity_y: Option<f64>,
    pub horizontal_multiplier: Option<f64>,
    pub award_jump: bool,
    pub timer: Option<f32>,
    pub wrong_protocol: bool,
    pub stationary: bool,
}
#[derive(Clone, Debug)]
pub struct Tower {
    pub jump_off: f64,
    // Independent source sequenceHandlers may coexist after separate jumps.
    pub pending: Vec<(TowerMode, u8)>,
    pub wrong_protocol: bool,
    pub stationary: bool,
}
impl Default for Tower {
    fn default() -> Self {
        Self {
            jump_off: f64::NAN,
            pending: Vec::new(),
            wrong_protocol: false,
            stationary: false,
        }
    }
}
impl Tower {
    pub fn jump(&mut self, mode: TowerMode, y: f64, motion: f32, cancelled: bool) {
        if mode == TowerMode::Motion {
            self.jump_off = y;
        }
        if motion != 0.0 && !cancelled && matches!(mode, TowerMode::Pulldown | TowerMode::Karhu) {
            self.pending.push((mode, 0));
        }
    }
    pub fn tick(&mut self, settings: TowerSettings, facts: TowerFacts) -> TowerEffect {
        let mut effect = TowerEffect::default();
        let active = facts.physical_jump && facts.blocks > 0;
        match settings.mode {
            TowerMode::Motion => {
                if !active || !facts.block_below {
                    self.jump_off = f64::NAN;
                } else if facts.y > self.jump_off + f64::from(settings.height) {
                    let y = facts.y.trunc();
                    effect.y = Some(y);
                    effect.velocity_y = Some(f64::from(settings.motion));
                    effect.horizontal_multiplier = Some(f64::from(settings.slow));
                    effect.award_jump = true;
                    self.jump_off = y;
                }
            }
            TowerMode::Vulcan if active && facts.block_below => {
                effect.velocity_y = Some(if facts.tick % 2 == 0 {
                    0.7
                } else if facts.moving {
                    f64::from(0.42_f32)
                } else {
                    0.6
                });
                effect.award_jump = facts.tick % 2 != 0;
            }
            TowerMode::Hypixel if active => {
                if !facts.protocol_1_8 {
                    effect.wrong_protocol = !self.wrong_protocol;
                    self.wrong_protocol = true;
                } else {
                    self.wrong_protocol = false;
                }
                if facts.horizontal_speed > 0.01 {
                    effect.stationary = !self.stationary;
                    self.stationary = true;
                } else if facts.ground {
                    effect.velocity_y = Some(0.42);
                } else if facts.velocity_y <= 0.0 && facts.velocity_y >= -0.09 {
                    effect.velocity_y = Some(-0.38);
                }
            }
            _ => {}
        }
        effect
    }
    /// CoroutineTicker resumes sequenceHandlers before ordinary GameTick handlers.
    pub fn advance_sequences(&mut self, settings: TowerSettings, facts: TowerFacts) -> TowerEffect {
        let mut effect = TowerEffect::default();
        let mut pending = Vec::new();
        for (mode, mut phase) in self.pending.drain(..) {
            if mode != settings.mode {
                continue;
            }
            if mode == TowerMode::Pulldown {
                if !facts.ground && facts.velocity_y < f64::from(settings.trigger) {
                    if facts.block_below {
                        effect.velocity_y = Some(-1.0);
                    }
                } else {
                    pending.push((mode, phase));
                }
            } else if mode == TowerMode::Karhu {
                if phase == 0 && !facts.ground {
                    effect.timer = Some(settings.timer);
                    phase = 1;
                }
                if phase == 0 {
                    pending.push((mode, phase));
                } else if settings.pulldown {
                    // tickUntil first tests immediately, as the source sequence does.
                    if !facts.ground && facts.velocity_y < f64::from(settings.trigger) {
                        if facts.block_below {
                            effect.velocity_y = Some(facts.velocity_y - 1.0);
                        }
                    } else {
                        pending.push((mode, phase));
                    }
                }
            }
        }
        self.pending = pending;
        effect
    }
}
