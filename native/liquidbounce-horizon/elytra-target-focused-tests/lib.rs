#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
pub use behavior_core::{auto_bow,auto_bow_math,rotations,target_renderer};
#[path="../../liquidbounce-behaviors/src/elytra_target.rs"]pub mod elytra_target;
mod behaviors{pub fn current_time_millis()->i64{0}}
#[path="runtime.rs"]mod elytra_target_runtime;
mod config;
