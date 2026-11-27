// SPDX-License-Identifier: GPL-3.0-or-later
//! The original common Turning factories and their identity-preserving pacing.
use super::view_rotation::ViewRotation;
use crate::navigation::walk_jump::wrap_degrees;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookPurpose {
    Movement,
    Aim,
    View,
    Steer,
}
#[derive(Clone, Debug)]
pub enum Turning {
    Instant,
    Linear {
        degrees: f32,
        x_rot_degrees: f32,
    },
    Eased {
        most: f32,
        x_rot_most: f32,
        share: f32,
        least: f32,
    },
    HumanPaced(Box<Turning>),
}
impl Turning {
    pub const CALM_TURN: f32 = 40.0;
    pub const FLICK_TURN: f32 = 90.0;
    pub fn linear(degrees: f32, x_rot_degrees: f32) -> Self {
        Self::Linear {
            degrees,
            x_rot_degrees,
        }
    }
    pub fn eased(most: f32, x_rot_most: f32, share: f32, least: f32) -> Self {
        Self::Eased {
            most,
            x_rot_most,
            share,
            least,
        }
    }
    pub fn calm() -> Self {
        Self::eased(Self::CALM_TURN, Self::CALM_TURN * 0.7, 0.45, 1.5)
    }
    pub fn human_paced(self) -> Self {
        if matches!(self, Self::Instant) {
            self
        } else {
            Self::HumanPaced(Box::new(self))
        }
    }
    pub fn turn(&self, from: ViewRotation, to: ViewRotation, purpose: LookPurpose) -> ViewRotation {
        match *self {
            Self::Instant => to,
            Self::Linear {
                degrees,
                x_rot_degrees,
            } => linear(from, to, degrees, x_rot_degrees),
            Self::Eased {
                most,
                x_rot_most,
                share,
                least,
            } => {
                if purpose == LookPurpose::Steer {
                    return to;
                }
                ViewRotation::new(
                    from.y_rot
                        + eased_step(wrap_degrees(to.y_rot - from.y_rot), most, share, least),
                    from.x_rot + eased_step(to.x_rot - from.x_rot, x_rot_most, share, least),
                )
            }
            Self::HumanPaced(ref inner) => {
                let turned = inner.turn(from, to, purpose);
                let most = if purpose == LookPurpose::Steer {
                    Self::FLICK_TURN
                } else {
                    Self::CALM_TURN
                };
                linear(from, turned, most, most)
            }
        }
    }
}
fn coerce(value: f32, minimum: f32, maximum: f32) -> f32 {
    if minimum > maximum {
        panic!("empty turning range");
    }
    if value < minimum {
        minimum
    } else if value > maximum {
        maximum
    } else {
        value
    }
}
fn linear(from: ViewRotation, to: ViewRotation, degrees: f32, x_rot_degrees: f32) -> ViewRotation {
    ViewRotation::new(
        from.y_rot + coerce(wrap_degrees(to.y_rot - from.y_rot), -degrees, degrees),
        from.x_rot + coerce(to.x_rot - from.x_rot, -x_rot_degrees, x_rot_degrees),
    )
}
fn eased_step(left: f32, most: f32, share: f32, least: f32) -> f32 {
    let size = left.abs();
    let minimum = crate::navigation::geometry::java_min(
        crate::navigation::geometry::java_min(f64::from(least), f64::from(size)),
        f64::from(most),
    ) as f32;
    let step = coerce(size * share, minimum, most);
    if left < 0.0 { -step } else { step }
}
