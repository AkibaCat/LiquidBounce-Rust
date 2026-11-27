// SPDX-License-Identifier: GPL-3.0-or-later
//! ScaffoldMovementPrediction history, bootstrap and warmup blending.
use std::collections::VecDeque;
pub type Position = [f64; 3];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub bootstrap_backoff: f32,
    pub cutoff_distance: f32,
    pub warmup_placements: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            bootstrap_backoff: 0.2,
            cutoff_distance: 0.05,
            warmup_placements: 2,
        }
    }
}
pub fn with_length(vector: Position, length: f64) -> Position {
    let squared = vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2];
    // MinecraftVectorExtensions.withLength tests Mth.equal(lengthSq, 0.0),
    // unlike Vec3.normalize's threshold on length itself.
    if squared.abs() < 1.0e-5 {
        [0.0; 3]
    } else {
        let scale = length / squared.sqrt();
        vector.map(|v| v * scale)
    }
}
#[derive(Clone, Debug, Default)]
pub struct Prediction {
    pub history: VecDeque<Position>,
}
impl Prediction {
    pub fn reset(&mut self) {
        self.history.clear();
    }
    pub fn average(&self) -> Option<Position> {
        if self.history.is_empty() {
            return None;
        }
        let mut sum = [0.0; 3];
        for offset in &self.history {
            for i in 0..3 {
                sum[i] += offset[i];
            }
        }
        Some(sum.map(|v| v / self.history.len() as f64))
    }
    pub fn record(
        &mut self,
        enabled: bool,
        player: Position,
        falloff: Option<Position>,
        angle: Option<f32>,
        mut rotate_y: impl FnMut(Position, f32) -> Position,
    ) {
        if !enabled {
            return;
        }
        let (Some(falloff), Some(angle)) = (falloff, angle) else {
            return;
        };
        let offset = std::array::from_fn(|i| player[i] - falloff[i]);
        self.history.push_back(rotate_y(offset, angle));
        if self.history.len() > 4 {
            self.history.pop_front();
        }
    }
    pub fn predict(
        &self,
        player: Position,
        falloff: Position,
        angle: f32,
        offset: Option<[f64; 2]>,
        settings: Settings,
        mut rotate_y: impl FnMut(Position, f32) -> Position,
    ) -> Position {
        let bootstrap = if settings.bootstrap_backoff <= 0.0 {
            falloff
        } else {
            let delta = std::array::from_fn(|i| falloff[i] - player[i]);
            let delta = with_length(delta, f64::from(settings.bootstrap_backoff));
            std::array::from_fn(|i| falloff[i] - delta[i])
        };
        let Some(average) = self.average() else {
            return offset.map_or(bootstrap, |[x, z]| {
                [bootstrap[0] + x, bootstrap[1], bootstrap[2] + z]
            });
        };
        let rotated = rotate_y(average, -angle);
        let predicted: Position = std::array::from_fn(|i| falloff[i] + rotated[i]);
        let blend = if settings.warmup_placements <= 0 {
            1.0
        } else {
            (self.history.len() as f64 / f64::from(settings.warmup_placements)).clamp(0.0, 1.0)
        };
        std::array::from_fn(|i| bootstrap[i] + blend * (predicted[i] - bootstrap[i]))
    }
}
