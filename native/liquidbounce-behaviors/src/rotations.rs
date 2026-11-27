// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce Rotation, RotationManager, RequestHandler,
// RotationsValueGroup and MixinKeyboardInput at ae0eb396.
// Copyright (c) 2015-2026 CCBlueX.

//! Shared rotation primitives and the original default linear rotation plan.
//! No module is registered here. Runtime callers provide actual input flags,
//! vanilla sine/cosine values and packet final states at their real phases.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
    pub is_normalized: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationError {
    /// Kotlin Double.roundToInt throws IllegalArgumentException for NaN.
    NanRounding,
    /// A live rotation processor rejected its current imported settings.
    Processor(&'static str),
}

pub fn wrap_degrees(value: f32) -> f32 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

/// Current-protocol MouseHandler path. The option is a double, the 0.6f and
/// 0.2f constants are widened floats; only the sensitivity product then
/// becomes float before Entity.turn's float 0.15f multiplication.
pub fn modern_mouse_gcd(sensitivity: f64) -> f64 {
    let factor = sensitivity * f64::from(0.6f32) + f64::from(0.2f32);
    f64::from((factor * factor * factor * 8.0) as f32 * 0.15f32)
}

fn kotlin_round_to_int(value: f64) -> Result<i32, RotationError> {
    if value.is_nan() {
        return Err(RotationError::NanRounding);
    }
    // Math.round(double), then saturation to Int. floor(value + 0.5)
    // incorrectly rounds the representable value just below +0.5 to 1.
    let floor = value.floor();
    let rounded = if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    };
    Ok(rounded as i32)
}

/// Math.round(float): ties go towards positive infinity, including -0.5 -> 0.
fn java_round_float(value: f32) -> i32 {
    if value.is_nan() {
        return 0;
    }
    let value = f64::from(value);
    let floor = value.floor();
    if value - floor >= 0.5 {
        (floor + 1.0) as i32
    } else {
        floor as i32
    }
}

impl Rotation {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(yaw: f32, pitch: f32) -> Self {
        Self {
            yaw,
            pitch,
            is_normalized: false,
        }
    }

    pub fn delta_to(self, other: Self) -> [f32; 2] {
        [wrap_degrees(other.yaw - self.yaw), other.pitch - self.pitch]
    }

    pub fn delta_length_squared_to(self, other: Self) -> f32 {
        let [yaw, pitch] = self.delta_to(other);
        yaw * yaw + pitch * pitch
    }

    pub fn normalize(self, current: Self, gcd: f64) -> Result<Self, RotationError> {
        if self.is_normalized {
            return Ok(self);
        }
        let [delta_yaw, delta_pitch] = current.delta_to(self);
        let yaw_steps = kotlin_round_to_int(f64::from(delta_yaw) / gcd)?;
        let pitch_steps = kotlin_round_to_int(f64::from(delta_pitch) / gcd)?;
        Ok(Self {
            yaw: current.yaw + (f64::from(yaw_steps) * gcd) as f32,
            pitch: (current.pitch + (f64::from(pitch_steps) * gcd) as f32).clamp(-90.0, 90.0),
            is_normalized: true,
        })
    }

    /// FactorAngleSmooth's exact component limits, including the original
    /// zero-length behavior: NaN limits leave the zero component unchanged.
    pub fn towards_linear(self, target: Self, horizontal: f32, vertical: f32) -> Self {
        let [yaw, pitch] = self.delta_to(target);
        let length = f64::from(yaw * yaw + pitch * pitch).sqrt() as f32;
        let yaw_limit = (yaw / length).abs() * horizontal;
        let pitch_limit = (pitch / length).abs() * vertical;
        fn coerce(value: f32, limit: f32) -> f32 {
            if value < -limit {
                -limit
            } else if value > limit {
                limit
            } else {
                value
            }
        }
        Self::new(
            self.yaw + coerce(yaw, yaw_limit),
            self.pitch + coerce(pitch, pitch_limit),
        )
    }

    /// Rotation.interpolateTo uses Math.fma without wrapping the yaw delta.
    pub fn interpolate_to(self, other: Self, partial: f32) -> Self {
        Self::new(
            partial.mul_add(other.yaw - self.yaw, self.yaw),
            partial.mul_add(other.pitch - self.pitch, self.pitch),
        )
    }

    /// RotationUtil's modern Entity.turn arithmetic, before vanilla turn.
    pub fn mouse_turn(self, cursor_x: f64, cursor_y: f64) -> Self {
        Self::new(
            self.yaw + cursor_x as f32 * 0.15f32,
            (self.pitch + cursor_y as f32 * 0.15f32).clamp(-90.0, 90.0),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MovementCorrection {
    Off,
    Strict,
    #[default]
    Silent,
    ChangeLook,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectionalInput {
    pub forwards: bool,
    pub backwards: bool,
    pub left: bool,
    pub right: bool,
}

impl DirectionalInput {
    pub const FORWARDS: Self = Self {
        forwards: true,
        backwards: false,
        left: false,
        right: false,
    };

    pub fn is_moving(self) -> bool {
        self.forwards != self.backwards || self.left != self.right
    }

    /// KeyboardInput.calculateImpulse uses opposite key cancellation. Keep
    /// all four original flags until this phase, including W+S and A+D.
    pub fn axes(self) -> [f32; 2] {
        [
            f32::from(self.left) - f32::from(self.right),
            f32::from(self.forwards) - f32::from(self.backwards),
        ]
    }

    /// The runtime supplies vanilla Mth.sin/cos of the *float* angle
    /// `(playerYaw - managedYaw) * Mth.DEG_TO_RAD`; native libm sin differs
    /// at direction boundaries and cannot replace the quantized table.
    pub fn silent_correct_with_sin_cos(self, sin: f32, cos: f32) -> Self {
        let [x, z] = self.axes();
        let sideways = java_round_float(x * cos - z * sin);
        let forward = java_round_float(z * cos + x * sin);
        Self {
            forwards: forward > 0,
            backwards: forward < 0,
            left: sideways > 0,
            right: sideways < 0,
        }
    }

    /// MixinKeyboardInput's guards precede calculate/rebuild, so without a
    /// SILENT managed target opposite held flags remain independently true.
    pub fn transform(
        self,
        player_yaw: Option<f32>,
        managed: Option<Rotation>,
        correction: Option<MovementCorrection>,
        mut vanilla_sin_cos: impl FnMut(f32) -> (f32, f32),
    ) -> Self {
        match (player_yaw, managed, correction) {
            (Some(player_yaw), Some(managed), Some(MovementCorrection::Silent)) => {
                const DEG_TO_RAD: f32 = f32::from_bits(0x3c8e_fa35);
                let angle = (player_yaw - managed.yaw) * DEG_TO_RAD;
                let (sin, cos) = vanilla_sin_cos(angle);
                self.silent_correct_with_sin_cos(sin, cos)
            }
            _ => self,
        }
    }
}

pub fn movement_yaw(
    player: f32,
    managed: Option<Rotation>,
    correction: Option<MovementCorrection>,
) -> f32 {
    match (managed, correction) {
        (Some(rotation), Some(correction))
            if rotation.yaw.is_finite() && correction != MovementCorrection::Off =>
        {
            rotation.yaw
        }
        _ => player,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ActiveRequest<T> {
    provider: u64,
    priority: i32,
    expires_at: i64,
    value: T,
}

/// Main-thread RequestHandler semantics, with Java's heap tie behavior.
/// Expired/disabled requests are removed when they reach the top, so a
/// disabled lower-priority provider may replace its request before then.
#[derive(Clone, Debug, PartialEq)]
pub struct RequestHandler<T> {
    current_tick: i64,
    heap: Vec<ActiveRequest<T>>,
}

impl<T> Default for RequestHandler<T> {
    fn default() -> Self {
        Self {
            current_tick: 0,
            heap: Vec::new(),
        }
    }
}

impl<T> RequestHandler<T> {
    pub fn current_tick(&self) -> i64 {
        self.current_tick
    }

    pub fn tick(&mut self, delta: i32) {
        self.current_tick = self.current_tick.wrapping_add(i64::from(delta));
    }

    pub fn clear(&mut self) {
        self.heap.clear();
        self.current_tick = 0;
    }

    fn sift_down(&mut self, mut at: usize) {
        loop {
            let left = at * 2 + 1;
            if left >= self.heap.len() {
                return;
            }
            let right = left + 1;
            let child = if right < self.heap.len()
                && self.heap[right].priority > self.heap[left].priority
            {
                right
            } else {
                left
            };
            if self.heap[at].priority >= self.heap[child].priority {
                return;
            }
            self.heap.swap(at, child);
            at = child;
        }
    }

    pub fn request(&mut self, provider: u64, priority: i32, expires_in: i32, value: T) {
        // PriorityBlockingQueue.removeIf compacts its heap array and heapifies.
        self.heap.retain(|active| active.provider != provider);
        for at in (0..self.heap.len() / 2).rev() {
            self.sift_down(at);
        }
        self.heap.push(ActiveRequest {
            provider,
            priority,
            expires_at: self.current_tick.wrapping_add(i64::from(expires_in)),
            value,
        });
        let mut at = self.heap.len() - 1;
        while at > 0 {
            let parent = (at - 1) / 2;
            if self.heap[at].priority <= self.heap[parent].priority {
                break;
            }
            self.heap.swap(at, parent);
            at = parent;
        }
    }

    fn pop(&mut self) {
        if !self.heap.is_empty() {
            self.heap.swap_remove(0);
            self.sift_down(0);
        }
    }

    pub fn active_value(&mut self, mut provider_running: impl FnMut(u64) -> bool) -> Option<&T> {
        while let Some(active) = self.heap.first() {
            if active.expires_at > self.current_tick && provider_running(active.provider) {
                return Some(&self.heap.first().expect("checked above").value);
            }
            self.pop();
        }
        None
    }

    /// RequestHandler intentionally performs no cleanup off the game thread.
    pub fn peek_off_thread(&self) -> Option<&T> {
        self.heap.first().map(|request| &request.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DefaultRotationTarget {
    pub rotation: Rotation,
    pub horizontal_turn_speed: f32,
    pub vertical_turn_speed: f32,
    pub ticks_until_reset: i32,
    pub reset_threshold: f32,
    pub consider_inventory: bool,
    pub movement_correction: MovementCorrection,
}

impl DefaultRotationTarget {
    /// The hidden RotationsValueGroup on Sprint is not a settings tree.
    pub const fn sprint(rotation: Rotation) -> Self {
        Self {
            rotation,
            horizontal_turn_speed: 180.0,
            vertical_turn_speed: 180.0,
            ticks_until_reset: 5,
            reset_threshold: 2.0,
            consider_inventory: false,
            movement_correction: MovementCorrection::Silent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RotationUpdate {
    /// Only set when resetting: player.withFixedYaw synchronizes local yaw.
    /// The caller applies it to yRot, yBob and yBobO together.
    pub reset_player_yaw: Option<f32>,
}

/// Original targets keep references to their processor objects. The numeric
/// identity lets runtime users resolve that same object with live settings.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RotationPlan {
    target: DefaultRotationTarget,
    processor: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RotationManager {
    requests: RequestHandler<RotationPlan>,
    previous_target: Option<RotationPlan>,
    pub current: Option<Rotation>,
    pub last: Option<Rotation>,
    pub last_model: Option<Rotation>,
    pub actual_server: Rotation,
    pub theoretical_server: Rotation,
    /// The requested plan processed this update (none during reset/inventory suspension).
    pub processed_request: Option<u64>,
}

impl RotationManager {
    pub fn previous_rotation_target(&self)->Option<Rotation>{self.previous_target.map(|p|p.target.rotation)}
    pub fn reset_world(&mut self) {
        *self = Self::default();
    }

    pub fn request(
        &mut self,
        provider: u64,
        priority: i32,
        target: DefaultRotationTarget,
        allowed_to_update: bool,
    ) {
        self.request_processed(provider, priority, target, 0, allowed_to_update);
    }

    pub fn request_processed(
        &mut self,
        provider: u64,
        priority: i32,
        target: DefaultRotationTarget,
        processor: u64,
        allowed_to_update: bool,
    ) {
        if allowed_to_update {
            let lifetime = if target.movement_correction == MovementCorrection::ChangeLook {
                1
            } else {
                target.ticks_until_reset
            };
            self.requests.request(
                provider,
                priority,
                lifetime,
                RotationPlan { target, processor },
            );
        }
    }

    pub fn active_target(
        &mut self,
        running: impl FnMut(u64) -> bool,
    ) -> Option<DefaultRotationTarget> {
        self.requests
            .active_value(running)
            .copied()
            .or(self.previous_target)
            .map(|plan| plan.target)
    }

    /// GameTick's FIRST_PRIORITY update after RotationUpdateEvent providers
    /// submitted requests. The clock does not tick if there is no plan at all.
    pub fn update(
        &mut self,
        player: Rotation,
        gcd: f64,
        allowed_to_update: bool,
        inventory_open: bool,
        running: impl FnMut(u64) -> bool,
    ) -> Result<RotationUpdate, RotationError> {
        self.update_with(
            player,
            gcd,
            allowed_to_update,
            inventory_open,
            running,
            |_, target, from, to, _| {
                Ok(from.towards_linear(
                    to,
                    target.horizontal_turn_speed,
                    target.vertical_turn_speed,
                ))
            },
        )
    }

    /// Runtime resolves processor identity on every step, including reset.
    /// The processor sees lastRotation from before the currentRotation setter.
    #[allow(clippy::too_many_arguments)]
    pub fn update_with(
        &mut self,
        player: Rotation,
        gcd: f64,
        allowed_to_update: bool,
        inventory_open: bool,
        running: impl FnMut(u64) -> bool,
        mut process: impl FnMut(
            u64,
            DefaultRotationTarget,
            Rotation,
            Rotation,
            Option<Rotation>,
        ) -> Result<Rotation, RotationError>,
    ) -> Result<RotationUpdate, RotationError> {
        self.processed_request = None;
        self.last_model = self.current;
        let requested = self.requests.active_value(running).copied();
        let Some(plan) = requested.or(self.previous_target) else {
            return Ok(RotationUpdate {
                reset_player_yaw: None,
            });
        };
        let target = plan.target;
        let mut reset_player_yaw = None;
        if allowed_to_update && !(target.consider_inventory && inventory_open) {
            let from = self.current.unwrap_or(player);
            let to = if requested.is_none() {
                player
            } else {
                target.rotation
            };
            let rotation =
                process(plan.processor, target, from, to, self.last)?.normalize(from, gcd)?;
            let diff = rotation.delta_length_squared_to(player);
            if requested.is_none()
                && (target.movement_correction == MovementCorrection::ChangeLook
                    || diff <= target.reset_threshold * target.reset_threshold)
            {
                reset_player_yaw = self
                    .current
                    .map(|current| current.yaw + wrap_degrees(player.yaw - current.yaw));
                self.current = None;
                self.last = None;
                self.previous_target = None;
            } else {
                self.last = Some(self.current.unwrap_or(player));
                self.current = Some(rotation);
                if requested.is_some(){self.processed_request=Some(plan.processor);}
                self.previous_target = Some(plan);
            }
        }
        self.requests.tick(1);
        Ok(RotationUpdate { reset_player_yaw })
    }

    /// GameRenderer.update HEAD, before vanilla camera extraction. The caller
    /// synchronizes old angles and yBob to the actual player before this write.
    pub fn change_look_rotation(
        &mut self,
        partial: f32,
        gcd: f64,
        allowed_to_update: bool,
        inventory_open: bool,
        running: impl FnMut(u64) -> bool,
    ) -> Result<Option<Rotation>, RotationError> {
        let Some(target) = self.active_target(running) else {
            return Ok(None);
        };
        if !allowed_to_update
            || (target.consider_inventory && inventory_open)
            || target.movement_correction != MovementCorrection::ChangeLook
        {
            return Ok(None);
        }
        let Some(current) = self.current else {
            return Ok(None);
        };
        self.last
            .unwrap_or(current)
            .interpolate_to(current, partial)
            .normalize(current, gcd)
            .map(Some)
    }

    /// MouseRotationEvent shifts both interpolation ends and lastModelRotation.
    pub fn mouse_rotation(
        &mut self,
        cursor_x: f64,
        cursor_y: f64,
        allowed_to_update: bool,
        inventory_open: bool,
        running: impl FnMut(u64) -> bool,
    ) {
        let Some(target) = self.active_target(running) else {
            return;
        };
        if !allowed_to_update
            || (target.consider_inventory && inventory_open)
            || target.movement_correction != MovementCorrection::ChangeLook
        {
            return;
        }
        let Some(current) = self.current else {
            return;
        };
        let last = self.last.unwrap_or(current);
        self.current = Some(current.mouse_turn(cursor_x, cursor_y));
        self.last = Some(last.mouse_turn(cursor_x, cursor_y));
        self.last_model = self
            .last_model
            .map(|rotation| rotation.mouse_turn(cursor_x, cursor_y));
    }

    pub fn server_rotation(&self, fake_lagging: bool, freezing: bool) -> Rotation {
        if fake_lagging || freezing {
            self.theoretical_server
        } else {
            self.actual_server
        }
    }

    /// READ_FINAL_STATE of rotation-bearing movement or use-item packets.
    /// Canceled outgoing packets affect only theoretical rotation.
    pub fn outbound_rotation(&mut self, yaw: f32, pitch: f32, canceled: bool) {
        let rotation = Rotation {
            yaw: wrap_degrees(yaw),
            pitch: wrap_degrees(pitch).clamp(-90.0, 90.0),
            is_normalized: true,
        };
        if !canceled {
            self.actual_server = rotation;
        }
        self.theoretical_server = rotation;
    }

    /// Incoming player-position/player-rotation corrections already changed
    /// server state before dispatch, including when the client cancels them.
    pub fn inbound_rotation(
        &mut self,
        yaw: f32,
        pitch: f32,
        relative_yaw: bool,
        relative_pitch: bool,
    ) {
        let rotation = Rotation {
            yaw: yaw
                + if relative_yaw {
                    self.actual_server.yaw
                } else {
                    0.0
                },
            pitch: (pitch
                + if relative_pitch {
                    self.actual_server.pitch
                } else {
                    0.0
                })
            .clamp(-90.0, 90.0),
            is_normalized: true,
        };
        self.actual_server = rotation;
        self.theoretical_server = rotation;
    }

    pub fn model_yaw(&self, partial: f32, vanilla: f32, enabled: bool) -> f32 {
        if !enabled {
            return vanilla;
        }
        let from = self.last_model.map_or(vanilla, |rotation| rotation.yaw);
        let to = self.current.map_or(vanilla, |rotation| rotation.yaw);
        from + partial * wrap_degrees(to - from)
    }

    pub fn model_pitch(&self, partial: f32, vanilla: f32, head_enabled: bool) -> f32 {
        if !head_enabled {
            return vanilla;
        }
        let from = self.last_model.map_or(vanilla, |rotation| rotation.pitch);
        let to = self.current.map_or(vanilla, |rotation| rotation.pitch);
        from + partial * (to - from)
    }
}
