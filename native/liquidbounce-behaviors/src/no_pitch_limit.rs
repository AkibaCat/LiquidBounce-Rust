// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleNoPitchLimit.kt and MixinEntity.java at LiquidBounce ae0eb396.
use std::sync::atomic::{AtomicU8, Ordering};

const RUNNING: u8 = 1;
const SERVER_SIDE: u8 = 2;

/// The game actor supplies ClientModule.running; the transport reads one
/// coherent atomic setting snapshot and never consults the module registry.
#[derive(Debug)]
pub struct NoPitchLimit {
    state: AtomicU8,
}

impl Default for NoPitchLimit {
    fn default() -> Self {
        Self {
            state: AtomicU8::new(SERVER_SIDE),
        }
    }
}

impl NoPitchLimit {
    pub fn set_running(&self, running: bool) -> bool {
        let before = if running {
            self.state.fetch_or(RUNNING, Ordering::AcqRel)
        } else {
            self.state.fetch_and(!RUNNING, Ordering::AcqRel)
        };
        (before & RUNNING != 0) != running
    }

    pub fn running(&self) -> bool {
        self.state.load(Ordering::Acquire) & RUNNING != 0
    }

    pub fn set_server_side(&self, server_side: bool) -> bool {
        let before = if server_side {
            self.state.fetch_or(SERVER_SIDE, Ordering::AcqRel)
        } else {
            self.state.fetch_and(!SERVER_SIDE, Ordering::AcqRel)
        };
        (before & SERVER_SIDE != 0) != server_side
    }

    pub fn server_side(&self) -> bool {
        self.state.load(Ordering::Acquire) & SERVER_SIDE != 0
    }

    /// Only movement packets carrying rotation have an encoded pitch. Original
    /// Pos/Status packets also have an unused Java xRot field, with no wire effect.
    pub fn outgoing_pitch(&self, pitch: f32) -> f32 {
        if self.state.load(Ordering::Acquire) & (RUNNING | SERVER_SIDE) == RUNNING {
            pitch.clamp(-90.0, 90.0)
        } else {
            pitch
        }
    }

    /// onDisabled clamps the current value, rather than wrapping it modulo 360.
    /// The caller must apply this on the real client actor, even when no packet
    /// has been sent since the last mouse turn.
    pub fn disabled_pitch(pitch: f32) -> f32 {
        pitch.clamp(-90.0, 90.0)
    }
}
